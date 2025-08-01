//! macOS-specific platform implementation for the CX Framework.
//! 
//! This module provides macOS-specific implementations using Cocoa/AppKit.

#[cfg(target_os = "macos")]
use cocoa::{
    appkit::{
        NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSBackingStoreBuffered,
        NSWindow, NSWindowStyleMask, NSEventType, NSEvent, NSEventModifierFlags,
        NSRunningApplication, NSApplicationActivateIgnoringOtherApps,
        NSView, NSColor, NSGraphicsContext, NSBezierPath, NSString, NSFont,
        NSMakePoint, NSMakeSize, NSMakeRect, NSRect, NSPoint, NSSize,
    },
    base::{id, nil, YES, NO, BOOL},
    foundation::{NSAutoreleasePool, NSString as NSStringFoundation, NSArray, NSDictionary},
};

#[cfg(target_os = "macos")]
use objc::{
    runtime::{Object, Class, Sel, BOOL as objc_BOOL},
    declare::ClassDecl,
    msg_send, sel, sel_impl,
    class,
};

use crate::{Error, Result};
use super::{
    PlatformBackend, WindowHandle, WindowParams, PlatformEvent, PlatformHandle,
    DrawingContext, MouseButton, Key, KeyModifiers,
};
use std::collections::HashMap;
use std::ptr;
use std::sync::{Arc, Mutex, Once};
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_int, c_uint, c_ulong};

static INIT: Once = Once::new();
static mut NSAPP_INITIALIZED: bool = false;

// Thread-safe wrapper for NSWindow
#[derive(Debug, Clone)]
struct SafeNSWindow(usize);

unsafe impl Send for SafeNSWindow {}
unsafe impl Sync for SafeNSWindow {}

impl SafeNSWindow {
    fn new(nswindow: id) -> Self {
        SafeNSWindow(nswindow as usize)
    }
    
    fn as_id(&self) -> id {
        self.0 as id
    }
}

/// macOS-specific backend implementation
pub struct MacOSBackend {
    windows: HashMap<u64, MacOSWindow>,
    next_window_id: u64,
    event_queue: Arc<Mutex<Vec<PlatformEvent>>>,
    nsapp: id,
    autorelease_pool: id,
}

unsafe impl Send for MacOSBackend {}
unsafe impl Sync for MacOSBackend {}

struct MacOSWindow {
    nswindow: SafeNSWindow,
    id: u64,
    width: u32,
    height: u32,
    title: String,
}

struct MacOSDrawingContext {
    nswindow: SafeNSWindow,
    width: u32,
    height: u32,
    graphics_context: Option<id>,
}

unsafe impl Send for MacOSDrawingContext {}
unsafe impl Sync for MacOSDrawingContext {}

impl MacOSBackend {
    pub fn new() -> Result<Self> {
        unsafe {
            let autorelease_pool = NSAutoreleasePool::new(nil);
            
            // Initialize NSApplication singleton
            INIT.call_once(|| {
                let nsapp = NSApp();
                if nsapp == nil {
                    panic!("Failed to get NSApplication instance");
                }
                
                let _: () = msg_send![nsapp, setActivationPolicy: NSApplicationActivationPolicyRegular];
                let _: () = msg_send![nsapp, activateIgnoringOtherApps: YES];
                
                NSAPP_INITIALIZED = true;
            });
            
            let nsapp = NSApp();
            if nsapp == nil {
                return Err(Error::platform_init(
                    "Failed to initialize NSApplication"
                ));
            }
            
            let event_queue = Arc::new(Mutex::new(Vec::new()));
            
            Ok(Self {
                windows: HashMap::new(),
                next_window_id: 1,
                event_queue,
                nsapp,
                autorelease_pool,
            })
        }
    }
    
    fn get_window_by_nswindow(&self, nswindow: id) -> Option<&MacOSWindow> {
        let safe_nswindow = SafeNSWindow::new(nswindow);
        self.windows.values().find(|w| w.nswindow.0 == safe_nswindow.0)
    }
    
    fn get_window_by_nswindow_mut(&mut self, nswindow: id) -> Option<&mut MacOSWindow> {
        let safe_nswindow = SafeNSWindow::new(nswindow);
        self.windows.values_mut().find(|w| w.nswindow.0 == safe_nswindow.0)
    }
}

impl PlatformBackend for MacOSBackend {
    fn initialize(&mut self) -> Result<()> {
        // Already initialized in new()
        Ok(())
    }
    
    fn create_window(&mut self, params: &WindowParams) -> Result<WindowHandle> {
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        
        unsafe {
            // Create NSWindow with specified parameters
            let frame = NSMakeRect(
                params.x.unwrap_or(0) as f64,
                params.y.unwrap_or(0) as f64,
                params.width as f64,
                params.height as f64,
            );
            
            let style_mask = NSWindowStyleMask::NSTitledWindowMask
                | NSWindowStyleMask::NSClosableWindowMask
                | NSWindowStyleMask::NSMiniaturizableWindowMask
                | NSWindowStyleMask::NSResizableWindowMask;
            
            let nswindow = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                frame,
                style_mask,
                NSBackingStoreBuffered,
                NO,
            );
            
            if nswindow == nil {
                return Err(Error::window("Failed to create NSWindow"));
            }
            
            // Set window title
            let title_string = NSString::alloc(nil).init_str(&params.title);
            let _: () = msg_send![nswindow, setTitle: title_string];
            
            // Configure window behavior
            let _: () = msg_send![nswindow, setReleasedWhenClosed: NO];
            let _: () = msg_send![nswindow, makeKeyAndOrderFront: nil];
            
            let safe_nswindow = SafeNSWindow::new(nswindow);
            let window = MacOSWindow {
                nswindow: safe_nswindow.clone(),
                id: window_id,
                width: params.width,
                height: params.height,
                title: params.title.clone(),
            };
            
            self.windows.insert(window_id, window);
            
            let handle = WindowHandle::new(
                Box::new(MacOSWindowHandle { nswindow: safe_nswindow }),
                window_id,
            );
            
            Ok(handle)
        }
    }
    
    fn destroy_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.remove(&handle.id) {
            unsafe {
                let nswindow = window.nswindow.as_id();
                let _: () = msg_send![nswindow, close];
                let _: () = msg_send![nswindow, release];
            }
        }
        Ok(())
    }
    
    fn show_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let nswindow = window.nswindow.as_id();
                let _: () = msg_send![nswindow, makeKeyAndOrderFront: nil];
            }
        }
        Ok(())
    }
    
    fn hide_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let nswindow = window.nswindow.as_id();
                let _: () = msg_send![nswindow, orderOut: nil];
            }
        }
        Ok(())
    }
    
    fn set_window_title(&mut self, handle: &WindowHandle, title: &str) -> Result<()> {
        if let Some(window) = self.windows.get_mut(&handle.id) {
            window.title = title.to_string();
            unsafe {
                let nswindow = window.nswindow.as_id();
                let title_string = NSString::alloc(nil).init_str(title);
                let _: () = msg_send![nswindow, setTitle: title_string];
            }
        }
        Ok(())
    }
    
    fn set_window_size(&mut self, handle: &WindowHandle, width: u32, height: u32) -> Result<()> {
        if let Some(window) = self.windows.get_mut(&handle.id) {
            window.width = width;
            window.height = height;
            unsafe {
                let nswindow = window.nswindow.as_id();
                let frame: NSRect = msg_send![nswindow, frame];
                let new_frame = NSMakeRect(frame.origin.x, frame.origin.y, width as f64, height as f64);
                let _: () = msg_send![nswindow, setFrame: new_frame display: YES];
            }
        }
        Ok(())
    }
    
    fn get_window_size(&self, handle: &WindowHandle) -> Result<(u32, u32)> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let nswindow = window.nswindow.as_id();
                let frame: NSRect = msg_send![nswindow, frame];
                Ok((frame.size.width as u32, frame.size.height as u32))
            }
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn set_window_position(&mut self, handle: &WindowHandle, x: i32, y: i32) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let nswindow = window.nswindow.as_id();
                let frame: NSRect = msg_send![nswindow, frame];
                let new_frame = NSMakeRect(x as f64, y as f64, frame.size.width, frame.size.height);
                let _: () = msg_send![nswindow, setFrame: new_frame display: YES];
            }
        }
        Ok(())
    }
    
    fn get_window_position(&self, handle: &WindowHandle) -> Result<(i32, i32)> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let nswindow = window.nswindow.as_id();
                let frame: NSRect = msg_send![nswindow, frame];
                Ok((frame.origin.x as i32, frame.origin.y as i32))
            }
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn poll_events(&mut self) -> Result<Vec<PlatformEvent>> {
        self.process_events(false)
    }
    
    fn wait_events(&mut self) -> Result<Vec<PlatformEvent>> {
        self.process_events(true)
    }
    
    fn get_drawing_context(&self, handle: &WindowHandle) -> Result<Box<dyn DrawingContext>> {
        if let Some(window) = self.windows.get(&handle.id) {
            Ok(Box::new(MacOSDrawingContext {
                nswindow: window.nswindow.clone(),
                width: window.width,
                height: window.height,
                graphics_context: None,
            }))
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn cleanup(&mut self) -> Result<()> {
        unsafe {
            // Close all windows
            for window in self.windows.values() {
                let nswindow = window.nswindow.as_id();
                let _: () = msg_send![nswindow, close];
                let _: () = msg_send![nswindow, release];
            }
            self.windows.clear();
            
            // Clean up autorelease pool
            if self.autorelease_pool != nil {
                let _: () = msg_send![self.autorelease_pool, release];
            }
        }
        Ok(())
    }
}

impl MacOSBackend {
    fn process_events(&mut self, wait: bool) -> Result<Vec<PlatformEvent>> {
        let mut events = Vec::new();
        
        unsafe {
            let mask = NSEventType::NSAnyEventType.0;
            let until_date = if wait {
                // Distant future for blocking wait
                msg_send![class!(NSDate), distantFuture]
            } else {
                // Immediate return for non-blocking poll
                msg_send![class!(NSDate), distantPast]
            };
            
            loop {
                let event: id = msg_send![self.nsapp,
                    nextEventMatchingMask: mask
                    untilDate: until_date
                    inMode: NSDefaultRunLoopMode
                    dequeue: YES
                ];
                
                if event == nil {
                    break;
                }
                
                // Process the event
                if let Some(platform_event) = self.process_nsevent(event) {
                    events.push(platform_event);
                }
                
                // Send event to application for default handling
                let _: () = msg_send![self.nsapp, sendEvent: event];
                
                // For polling, only process one batch
                if !wait {
                    break;
                }
            }
        }
        
        Ok(events)
    }
    
    fn process_nsevent(&self, event: id) -> Option<PlatformEvent> {
        unsafe {
            let event_type: NSEventType = msg_send![event, type];
            let window: id = msg_send![event, window];
            
            if window == nil {
                return None;
            }
            
            let window_data = self.get_window_by_nswindow(window)?;
            let window_id = window_data.id;
            
            match event_type {
                NSEventType::NSLeftMouseDown => {
                    let location: NSPoint = msg_send![event, locationInWindow];
                    Some(PlatformEvent::MousePressed {
                        window_id,
                        button: MouseButton::Left,
                        x: location.x,
                        y: location.y,
                    })
                }
                
                NSEventType::NSLeftMouseUp => {
                    let location: NSPoint = msg_send![event, locationInWindow];
                    Some(PlatformEvent::MouseReleased {
                        window_id,
                        button: MouseButton::Left,
                        x: location.x,
                        y: location.y,
                    })
                }
                
                NSEventType::NSRightMouseDown => {
                    let location: NSPoint = msg_send![event, locationInWindow];
                    Some(PlatformEvent::MousePressed {
                        window_id,
                        button: MouseButton::Right,
                        x: location.x,
                        y: location.y,
                    })
                }
                
                NSEventType::NSRightMouseUp => {
                    let location: NSPoint = msg_send![event, locationInWindow];
                    Some(PlatformEvent::MouseReleased {
                        window_id,
                        button: MouseButton::Right,
                        x: location.x,
                        y: location.y,
                    })
                }
                
                NSEventType::NSMouseMoved | NSEventType::NSLeftMouseDragged | NSEventType::NSRightMouseDragged => {
                    let location: NSPoint = msg_send![event, locationInWindow];
                    Some(PlatformEvent::MouseMoved {
                        window_id,
                        x: location.x,
                        y: location.y,
                    })
                }
                
                NSEventType::NSScrollWheel => {
                    let delta_y: f64 = msg_send![event, scrollingDeltaY];
                    let delta_x: f64 = msg_send![event, scrollingDeltaX];
                    Some(PlatformEvent::MouseWheel {
                        window_id,
                        delta_x,
                        delta_y,
                    })
                }
                
                NSEventType::NSKeyDown => {
                    let key_code: u16 = msg_send![event, keyCode];
                    let modifiers: NSEventModifierFlags = msg_send![event, modifierFlags];
                    Some(PlatformEvent::KeyPressed {
                        window_id,
                        key: keycode_to_key(key_code),
                        modifiers: nsmodifiers_to_key_modifiers(modifiers),
                    })
                }
                
                NSEventType::NSKeyUp => {
                    let key_code: u16 = msg_send![event, keyCode];
                    let modifiers: NSEventModifierFlags = msg_send![event, modifierFlags];
                    Some(PlatformEvent::KeyReleased {
                        window_id,
                        key: keycode_to_key(key_code),
                        modifiers: nsmodifiers_to_key_modifiers(modifiers),
                    })
                }
                
                _ => None,
            }
        }
    }
}

// Platform handle wrapper
#[derive(Debug)]
struct MacOSWindowHandle {
    nswindow: SafeNSWindow,
}

impl DrawingContext for MacOSDrawingContext {
    fn clear(&mut self, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let nswindow = self.nswindow.as_id();
            let content_view: id = msg_send![nswindow, contentView];
            
            if content_view != nil {
                let ns_color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                    nil,
                    color.0 as f64,
                    color.1 as f64,
                    color.2 as f64,
                    color.3 as f64,
                );
                
                let _: () = msg_send![content_view, setBackgroundColor: ns_color];
                let _: () = msg_send![content_view, setNeedsDisplay: YES];
            }
        }
        Ok(())
    }
    
    fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let nswindow = self.nswindow.as_id();
            let content_view: id = msg_send![nswindow, contentView];
            
            if content_view != nil {
                let _: () = msg_send![content_view, lockFocus];
                
                let ns_color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                    nil,
                    color.0 as f64,
                    color.1 as f64,
                    color.2 as f64,
                    color.3 as f64,
                );
                let _: () = msg_send![ns_color, set];
                
                let rect = NSMakeRect(x as f64, y as f64, width as f64, height as f64);
                let path = NSBezierPath::bezierPathWithRect_(nil, rect);
                let _: () = msg_send![path, fill];
                
                let _: () = msg_send![content_view, unlockFocus];
            }
        }
        Ok(())
    }
    
    fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32), stroke_width: f32) -> Result<()> {
        unsafe {
            let nswindow = self.nswindow.as_id();
            let content_view: id = msg_send![nswindow, contentView];
            
            if content_view != nil {
                let _: () = msg_send![content_view, lockFocus];
                
                let ns_color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                    nil,
                    color.0 as f64,
                    color.1 as f64,
                    color.2 as f64,
                    color.3 as f64,
                );
                let _: () = msg_send![ns_color, set];
                
                let rect = NSMakeRect(x as f64, y as f64, width as f64, height as f64);
                let path = NSBezierPath::bezierPathWithRect_(nil, rect);
                let _: () = msg_send![path, setLineWidth: stroke_width as f64];
                let _: () = msg_send![path, stroke];
                
                let _: () = msg_send![content_view, unlockFocus];
            }
        }
        Ok(())
    }
    
    fn draw_text(&mut self, text: &str, x: f32, y: f32, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let nswindow = self.nswindow.as_id();
            let content_view: id = msg_send![nswindow, contentView];
            
            if content_view != nil {
                let _: () = msg_send![content_view, lockFocus];
                
                let ns_string = NSString::alloc(nil).init_str(text);
                let font = NSFont::systemFontOfSize_(nil, 14.0);
                let ns_color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                    nil,
                    color.0 as f64,
                    color.1 as f64,
                    color.2 as f64,
                    color.3 as f64,
                );
                
                let point = NSMakePoint(x as f64, y as f64);
                let _: () = msg_send![ns_string,
                    drawAtPoint: point
                    withAttributes: NSDictionary::dictionaryWithObjects_forKeys_(
                        nil,
                        NSArray::arrayWithObjects_(nil, &[font, ns_color]),
                        NSArray::arrayWithObjects_(nil, &[
                            NSString::alloc(nil).init_str("NSFont"),
                            NSString::alloc(nil).init_str("NSForegroundColor")
                        ])
                    )
                ];
                
                let _: () = msg_send![content_view, unlockFocus];
            }
        }
        Ok(())
    }
    
    fn present(&mut self) -> Result<()> {
        unsafe {
            let nswindow = self.nswindow.as_id();
            let content_view: id = msg_send![nswindow, contentView];
            
            if content_view != nil {
                let _: () = msg_send![content_view, setNeedsDisplay: YES];
                let _: () = msg_send![nswindow, flushWindow];
            }
        }
        Ok(())
    }
    
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

// Helper functions for key mapping
fn keycode_to_key(keycode: u16) -> Key {
    match keycode {
        // Letters (a-z)
        0 => Key::A, 11 => Key::B, 8 => Key::C, 2 => Key::D, 14 => Key::E,
        3 => Key::F, 5 => Key::G, 4 => Key::H, 34 => Key::I, 38 => Key::J,
        40 => Key::K, 37 => Key::L, 46 => Key::M, 45 => Key::N, 31 => Key::O,
        35 => Key::P, 12 => Key::Q, 15 => Key::R, 1 => Key::S, 17 => Key::T,
        32 => Key::U, 9 => Key::V, 13 => Key::W, 7 => Key::X, 16 => Key::Y,
        6 => Key::Z,
        
        // Numbers (0-9)
        29 => Key::Key0, 18 => Key::Key1, 19 => Key::Key2, 20 => Key::Key3,
        21 => Key::Key4, 23 => Key::Key5, 22 => Key::Key6, 26 => Key::Key7,
        28 => Key::Key8, 25 => Key::Key9,
        
        // Special keys
        53 => Key::Escape,
        48 => Key::Tab,
        49 => Key::Space,
        36 => Key::Return,
        51 => Key::Backspace,
        117 => Key::Delete,
        114 => Key::Insert,
        115 => Key::Home,
        119 => Key::End,
        116 => Key::PageUp,
        121 => Key::PageDown,
        123 => Key::Left,
        124 => Key::Right,
        126 => Key::Up,
        125 => Key::Down,
        
        // Function keys
        122 => Key::F1, 120 => Key::F2, 99 => Key::F3, 118 => Key::F4,
        96 => Key::F5, 97 => Key::F6, 98 => Key::F7, 100 => Key::F8,
        101 => Key::F9, 109 => Key::F10, 103 => Key::F11, 111 => Key::F12,
        
        // Modifier keys
        56 => Key::LeftShift,
        60 => Key::RightShift,
        59 => Key::LeftCtrl,
        62 => Key::RightCtrl,
        58 => Key::LeftAlt,
        61 => Key::RightAlt,
        55 => Key::LeftMeta,
        54 => Key::RightMeta,
        
        _ => Key::Unknown(keycode as u32),
    }
}

fn nsmodifiers_to_key_modifiers(modifiers: NSEventModifierFlags) -> KeyModifiers {
    KeyModifiers {
        shift: modifiers.contains(NSEventModifierFlags::NSShiftKeyMask),
        ctrl: modifiers.contains(NSEventModifierFlags::NSControlKeyMask),
        alt: modifiers.contains(NSEventModifierFlags::NSAlternateKeyMask),
        meta: modifiers.contains(NSEventModifierFlags::NSCommandKeyMask),
    }
}