//! Unix/Linux-specific platform implementation for the CX Framework.
//! 
//! This module provides Unix/Linux-specific implementations using X11.
//! Future versions may include Wayland support.

#[cfg(all(unix, not(target_os = "macos")))]
use x11::{
    xlib::{
        self, Display, Window as XWindow, XEvent, KeyPress, KeyRelease, ButtonPress, ButtonRelease,
        MotionNotify, Expose, ConfigureNotify, ClientMessage, MapNotify, UnmapNotify,
        EnterNotify, LeaveNotify, FocusIn, FocusOut, XOpenDisplay, XDefaultRootWindow,
        XCreateSimpleWindow, XSelectInput, XMapWindow, XUnmapWindow, XDestroyWindow,
        XNextEvent, XPending, XStoreName, XMoveResizeWindow, XGetWindowAttributes,
        XClearWindow, XFillRectangle, XDrawRectangle, XDrawString, XFlush,
        ExposureMask, KeyPressMask, KeyReleaseMask, ButtonPressMask, ButtonReleaseMask,
        PointerMotionMask, StructureNotifyMask, FocusChangeMask, EnterWindowMask, 
        LeaveWindowMask, SubstructureNotifyMask,
    },
    keysym::{
        XK_Escape, XK_Tab, XK_space, XK_Return, XK_BackSpace, XK_Delete, XK_Insert,
        XK_Home, XK_End, XK_Page_Up, XK_Page_Down, XK_Left, XK_Right, XK_Up, XK_Down,
        XK_Shift_L, XK_Shift_R, XK_Control_L, XK_Control_R, XK_Alt_L, XK_Alt_R,
        XK_Super_L, XK_Super_R, XK_F1, XK_F2, XK_F3, XK_F4, XK_F5, XK_F6,
        XK_F7, XK_F8, XK_F9, XK_F10, XK_F11, XK_F12,
    },
};

use crate::{Error, Result};
use super::{
    PlatformBackend, WindowHandle, WindowParams, PlatformEvent, PlatformHandle,
    DrawingContext, MouseButton, Key, KeyModifiers,
};
use std::collections::HashMap;
use std::ptr;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_uint, c_ulong};

/// Unix-specific backend implementation
pub struct UnixBackend {
    display: *mut Display,
    windows: HashMap<u64, UnixWindow>,
    next_window_id: u64,
    event_queue: Vec<PlatformEvent>,
    wm_delete_window: c_ulong,
}

unsafe impl Send for UnixBackend {}
unsafe impl Sync for UnixBackend {}

struct UnixWindow {
    xwindow: XWindow,
    id: u64,
    width: u32,
    height: u32,
    title: String,
}

struct UnixDrawingContext {
    display: *mut Display,
    xwindow: XWindow,
    width: u32,
    height: u32,
    gc: c_ulong, // Graphics Context
}

unsafe impl Send for UnixDrawingContext {}
unsafe impl Sync for UnixDrawingContext {}

impl UnixBackend {
    pub fn new() -> Result<Self> {
        unsafe {
            // Open connection to X server
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err(Error::platform_init(
                    "Failed to open X display. Make sure DISPLAY environment variable is set."
                ));
            }
            
            // Get WM_DELETE_WINDOW atom for proper window closing
            let wm_delete_window_name = CString::new("WM_DELETE_WINDOW").unwrap();
            let wm_delete_window = xlib::XInternAtom(
                display,
                wm_delete_window_name.as_ptr(),
                xlib::False,
            );
            
            Ok(Self {
                display,
                windows: HashMap::new(),
                next_window_id: 1,
                event_queue: Vec::new(),
                wm_delete_window,
            })
        }
    }
    
    fn get_window_by_xwindow(&self, xwindow: XWindow) -> Option<&UnixWindow> {
        self.windows.values().find(|w| w.xwindow == xwindow)
    }
    
    fn get_window_by_xwindow_mut(&mut self, xwindow: XWindow) -> Option<&mut UnixWindow> {
        self.windows.values_mut().find(|w| w.xwindow == xwindow)
    }
}

impl PlatformBackend for UnixBackend {
    fn initialize(&mut self) -> Result<()> {
        // Already initialized in new()
        Ok(())
    }
    
    fn create_window(&mut self, params: &WindowParams) -> Result<WindowHandle> {
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        
        unsafe {
            let root = XDefaultRootWindow(self.display);
            let screen = xlib::XDefaultScreen(self.display);
            let black_pixel = xlib::XBlackPixel(self.display, screen);
            let white_pixel = xlib::XWhitePixel(self.display, screen);
            
            // Create the window
            let xwindow = XCreateSimpleWindow(
                self.display,
                root,
                params.x.unwrap_or(0) as c_int,
                params.y.unwrap_or(0) as c_int,
                params.width as c_uint,
                params.height as c_uint,
                1, // border width
                black_pixel,
                white_pixel,
            );
            
            if xwindow == 0 {
                return Err(Error::window("Failed to create X11 window"));
            }
            
            // Set window title
            let title_cstr = CString::new(params.title.as_str()).unwrap();
            XStoreName(self.display, xwindow, title_cstr.as_ptr());
            
            // Select input events
            XSelectInput(
                self.display,
                xwindow,
                ExposureMask | KeyPressMask | KeyReleaseMask | ButtonPressMask | 
                ButtonReleaseMask | PointerMotionMask | StructureNotifyMask |
                FocusChangeMask | EnterWindowMask | LeaveWindowMask,
            );
            
            // Setup WM_DELETE_WINDOW protocol
            let mut protocols = [self.wm_delete_window];
            xlib::XSetWMProtocols(
                self.display,
                xwindow,
                protocols.as_mut_ptr(),
                1,
            );
            
            let window = UnixWindow {
                xwindow,
                id: window_id,
                width: params.width,
                height: params.height,
                title: params.title.clone(),
            };
            
            self.windows.insert(window_id, window);
            
            let handle = WindowHandle::new(
                Box::new(UnixWindowHandle { xwindow }),
                window_id,
            );
            
            Ok(handle)
        }
    }
    
    fn destroy_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.remove(&handle.id) {
            unsafe {
                XDestroyWindow(self.display, window.xwindow);
                XFlush(self.display);
            }
        }
        Ok(())
    }
    
    fn show_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                XMapWindow(self.display, window.xwindow);
                XFlush(self.display);
            }
        }
        Ok(())
    }
    
    fn hide_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                XUnmapWindow(self.display, window.xwindow);
                XFlush(self.display);
            }
        }
        Ok(())
    }
    
    fn set_window_title(&mut self, handle: &WindowHandle, title: &str) -> Result<()> {
        if let Some(window) = self.windows.get_mut(&handle.id) {
            window.title = title.to_string();
            unsafe {
                let title_cstr = CString::new(title).unwrap();
                XStoreName(self.display, window.xwindow, title_cstr.as_ptr());
                XFlush(self.display);
            }
        }
        Ok(())
    }
    
    fn set_window_size(&mut self, handle: &WindowHandle, width: u32, height: u32) -> Result<()> {
        if let Some(window) = self.windows.get_mut(&handle.id) {
            window.width = width;
            window.height = height;
            unsafe {
                XMoveResizeWindow(
                    self.display,
                    window.xwindow,
                    0, // keep current position
                    0,
                    width as c_uint,
                    height as c_uint,
                );
                XFlush(self.display);
            }
        }
        Ok(())
    }
    
    fn get_window_size(&self, handle: &WindowHandle) -> Result<(u32, u32)> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
                XGetWindowAttributes(self.display, window.xwindow, &mut attrs);
                Ok((attrs.width as u32, attrs.height as u32))
            }
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn set_window_position(&mut self, handle: &WindowHandle, x: i32, y: i32) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                XMoveResizeWindow(
                    self.display,
                    window.xwindow,
                    x as c_int,
                    y as c_int,
                    window.width as c_uint,
                    window.height as c_uint,
                );
                XFlush(self.display);
            }
        }
        Ok(())
    }
    
    fn get_window_position(&self, handle: &WindowHandle) -> Result<(i32, i32)> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
                XGetWindowAttributes(self.display, window.xwindow, &mut attrs);
                Ok((attrs.x as i32, attrs.y as i32))
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
            unsafe {
                let screen = xlib::XDefaultScreen(self.display);
                let gc = xlib::XDefaultGC(self.display, screen);
                
                Ok(Box::new(UnixDrawingContext {
                    display: self.display,
                    xwindow: window.xwindow,
                    width: window.width,
                    height: window.height,
                    gc: gc as c_ulong,
                }))
            }
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn cleanup(&mut self) -> Result<()> {
        unsafe {
            // Close all windows
            for window in self.windows.values() {
                XDestroyWindow(self.display, window.xwindow);
            }
            self.windows.clear();
            
            // Close X display
            if !self.display.is_null() {
                xlib::XCloseDisplay(self.display);
                self.display = ptr::null_mut();
            }
        }
        Ok(())
    }
}

impl UnixBackend {
    fn process_events(&mut self, wait: bool) -> Result<Vec<PlatformEvent>> {
        let mut events = Vec::new();
        
        unsafe {
            let mut event: XEvent = std::mem::zeroed();
            
            if wait {
                // Blocking wait for at least one event
                XNextEvent(self.display, &mut event);
                if let Some(platform_event) = self.process_xevent(&event) {
                    events.push(platform_event);
                }
            }
            
            // Process all available events
            while XPending(self.display) > 0 {
                XNextEvent(self.display, &mut event);
                if let Some(platform_event) = self.process_xevent(&event) {
                    events.push(platform_event);
                }
            }
        }
        
        Ok(events)
    }
    
    fn process_xevent(&self, event: &XEvent) -> Option<PlatformEvent> {
        unsafe {
            match event.get_type() {
                KeyPress => {
                    let key_event = xlib::XKeyEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(key_event.window) {
                        let mut buffer = [0u8; 32];
                        let mut keysym = 0;
                        let _count = xlib::XLookupString(
                            &key_event as *const _ as *mut _,
                            buffer.as_mut_ptr() as *mut c_char,
                            buffer.len() as c_int,
                            &mut keysym,
                            ptr::null_mut(),
                        );
                        
                        let key = keysym_to_key(keysym);
                        let modifiers = get_x11_key_modifiers(key_event.state);
                        
                        return Some(PlatformEvent::KeyPressed {
                            window_id: window.id,
                            key,
                            modifiers,
                        });
                    }
                }
                
                KeyRelease => {
                    let key_event = xlib::XKeyEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(key_event.window) {
                        let mut keysym = 0;
                        xlib::XLookupString(
                            &key_event as *const _ as *mut _,
                            ptr::null_mut(),
                            0,
                            &mut keysym,
                            ptr::null_mut(),
                        );
                        
                        let key = keysym_to_key(keysym);
                        let modifiers = get_x11_key_modifiers(key_event.state);
                        
                        return Some(PlatformEvent::KeyReleased {
                            window_id: window.id,
                            key,
                            modifiers,
                        });
                    }
                }
                
                ButtonPress => {
                    let button_event = xlib::XButtonEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(button_event.window) {
                        let button = match button_event.button {
                            1 => MouseButton::Left,
                            2 => MouseButton::Middle,
                            3 => MouseButton::Right,
                            _ => MouseButton::Other(button_event.button as u8),
                        };
                        
                        return Some(PlatformEvent::MousePressed {
                            window_id: window.id,
                            button,
                            x: button_event.x as f64,
                            y: button_event.y as f64,
                        });
                    }
                }
                
                ButtonRelease => {
                    let button_event = xlib::XButtonEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(button_event.window) {
                        let button = match button_event.button {
                            1 => MouseButton::Left,
                            2 => MouseButton::Middle,
                            3 => MouseButton::Right,
                            _ => MouseButton::Other(button_event.button as u8),
                        };
                        
                        return Some(PlatformEvent::MouseReleased {
                            window_id: window.id,
                            button,
                            x: button_event.x as f64,
                            y: button_event.y as f64,
                        });
                    }
                }
                
                MotionNotify => {
                    let motion_event = xlib::XMotionEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(motion_event.window) {
                        return Some(PlatformEvent::MouseMoved {
                            window_id: window.id,
                            x: motion_event.x as f64,
                            y: motion_event.y as f64,
                        });
                    }
                }
                
                ConfigureNotify => {
                    let configure_event = xlib::XConfigureEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(configure_event.window) {
                        return Some(PlatformEvent::WindowResized {
                            window_id: window.id,
                            width: configure_event.width as u32,
                            height: configure_event.height as u32,
                        });
                    }
                }
                
                FocusIn => {
                    let focus_event = xlib::XFocusChangeEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(focus_event.window) {
                        return Some(PlatformEvent::WindowFocused {
                            window_id: window.id,
                        });
                    }
                }
                
                FocusOut => {
                    let focus_event = xlib::XFocusChangeEvent::from(event.clone());
                    if let Some(window) = self.get_window_by_xwindow(focus_event.window) {
                        return Some(PlatformEvent::WindowUnfocused {
                            window_id: window.id,
                        });
                    }
                }
                
                ClientMessage => {
                    let client_event = xlib::XClientMessageEvent::from(event.clone());
                    if client_event.data.get_long(0) as c_ulong == self.wm_delete_window {
                        if let Some(window) = self.get_window_by_xwindow(client_event.window) {
                            return Some(PlatformEvent::WindowClosed {
                                window_id: window.id,
                            });
                        }
                    }
                }
                
                _ => {}
            }
        }
        
        None
    }
}

// Platform handle wrapper
#[derive(Debug)]
struct UnixWindowHandle {
    xwindow: XWindow,
}

impl DrawingContext for UnixDrawingContext {
    fn clear(&mut self, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            // Set background color and clear window
            let screen = xlib::XDefaultScreen(self.display);
            let colormap = xlib::XDefaultColormap(self.display, screen);
            
            let mut xcolor: xlib::XColor = std::mem::zeroed();
            xcolor.red = (color.0 * 65535.0) as u16;
            xcolor.green = (color.1 * 65535.0) as u16;
            xcolor.blue = (color.2 * 65535.0) as u16;
            
            xlib::XAllocColor(self.display, colormap, &mut xcolor);
            xlib::XSetWindowBackground(self.display, self.xwindow, xcolor.pixel);
            XClearWindow(self.display, self.xwindow);
            XFlush(self.display);
        }
        Ok(())
    }
    
    fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let screen = xlib::XDefaultScreen(self.display);
            let colormap = xlib::XDefaultColormap(self.display, screen);
            let gc = self.gc as *mut xlib::_XGC;
            
            let mut xcolor: xlib::XColor = std::mem::zeroed();
            xcolor.red = (color.0 * 65535.0) as u16;
            xcolor.green = (color.1 * 65535.0) as u16;
            xcolor.blue = (color.2 * 65535.0) as u16;
            
            xlib::XAllocColor(self.display, colormap, &mut xcolor);
            xlib::XSetForeground(self.display, gc, xcolor.pixel);
            
            XFillRectangle(
                self.display,
                self.xwindow,
                gc,
                x as c_int,
                y as c_int,
                width as c_uint,
                height as c_uint,
            );
            XFlush(self.display);
        }
        Ok(())
    }
    
    fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32), _stroke_width: f32) -> Result<()> {
        unsafe {
            let screen = xlib::XDefaultScreen(self.display);
            let colormap = xlib::XDefaultColormap(self.display, screen);
            let gc = self.gc as *mut xlib::_XGC;
            
            let mut xcolor: xlib::XColor = std::mem::zeroed();
            xcolor.red = (color.0 * 65535.0) as u16;
            xcolor.green = (color.1 * 65535.0) as u16;
            xcolor.blue = (color.2 * 65535.0) as u16;
            
            xlib::XAllocColor(self.display, colormap, &mut xcolor);
            xlib::XSetForeground(self.display, gc, xcolor.pixel);
            
            XDrawRectangle(
                self.display,
                self.xwindow,
                gc,
                x as c_int,
                y as c_int,
                width as c_uint,
                height as c_uint,
            );
            XFlush(self.display);
        }
        Ok(())
    }
    
    fn draw_text(&mut self, text: &str, x: f32, y: f32, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let screen = xlib::XDefaultScreen(self.display);
            let colormap = xlib::XDefaultColormap(self.display, screen);
            let gc = self.gc as *mut xlib::_XGC;
            
            let mut xcolor: xlib::XColor = std::mem::zeroed();
            xcolor.red = (color.0 * 65535.0) as u16;
            xcolor.green = (color.1 * 65535.0) as u16;
            xcolor.blue = (color.2 * 65535.0) as u16;
            
            xlib::XAllocColor(self.display, colormap, &mut xcolor);
            xlib::XSetForeground(self.display, gc, xcolor.pixel);
            
            let text_cstr = CString::new(text).unwrap();
            XDrawString(
                self.display,
                self.xwindow,
                gc,
                x as c_int,
                y as c_int,
                text_cstr.as_ptr(),
                text.len() as c_int,
            );
            XFlush(self.display);
        }
        Ok(())
    }
    
    fn present(&mut self) -> Result<()> {
        unsafe {
            XFlush(self.display);
        }
        Ok(())
    }
    
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

// Helper functions for X11 key mapping
fn keysym_to_key(keysym: c_ulong) -> Key {
    match keysym {
        XK_Escape => Key::Escape,
        XK_Tab => Key::Tab,
        XK_space => Key::Space,
        XK_Return => Key::Return,
        XK_BackSpace => Key::Backspace,
        XK_Delete => Key::Delete,
        XK_Insert => Key::Insert,
        XK_Home => Key::Home,
        XK_End => Key::End,
        XK_Page_Up => Key::PageUp,
        XK_Page_Down => Key::PageDown,
        XK_Left => Key::Left,
        XK_Right => Key::Right,
        XK_Up => Key::Up,
        XK_Down => Key::Down,
        XK_F1 => Key::F1,
        XK_F2 => Key::F2,
        XK_F3 => Key::F3,
        XK_F4 => Key::F4,
        XK_F5 => Key::F5,
        XK_F6 => Key::F6,
        XK_F7 => Key::F7,
        XK_F8 => Key::F8,
        XK_F9 => Key::F9,
        XK_F10 => Key::F10,
        XK_F11 => Key::F11,
        XK_F12 => Key::F12,
        XK_Shift_L => Key::LeftShift,
        XK_Shift_R => Key::RightShift,
        XK_Control_L => Key::LeftCtrl,
        XK_Control_R => Key::RightCtrl,
        XK_Alt_L => Key::LeftAlt,
        XK_Alt_R => Key::RightAlt,
        XK_Super_L => Key::LeftMeta,
        XK_Super_R => Key::RightMeta,
        // Letters
        b'a' as c_ulong..=b'z' as c_ulong => {
            match keysym as u8 {
                b'a' => Key::A, b'b' => Key::B, b'c' => Key::C, b'd' => Key::D,
                b'e' => Key::E, b'f' => Key::F, b'g' => Key::G, b'h' => Key::H,
                b'i' => Key::I, b'j' => Key::J, b'k' => Key::K, b'l' => Key::L,
                b'm' => Key::M, b'n' => Key::N, b'o' => Key::O, b'p' => Key::P,
                b'q' => Key::Q, b'r' => Key::R, b's' => Key::S, b't' => Key::T,
                b'u' => Key::U, b'v' => Key::V, b'w' => Key::W, b'x' => Key::X,
                b'y' => Key::Y, b'z' => Key::Z,
                _ => Key::Unknown(keysym as u32),
            }
        }
        // Numbers
        b'0' as c_ulong..=b'9' as c_ulong => {
            match keysym as u8 {
                b'0' => Key::Key0, b'1' => Key::Key1, b'2' => Key::Key2,
                b'3' => Key::Key3, b'4' => Key::Key4, b'5' => Key::Key5,
                b'6' => Key::Key6, b'7' => Key::Key7, b'8' => Key::Key8,
                b'9' => Key::Key9,
                _ => Key::Unknown(keysym as u32),
            }
        }
        _ => Key::Unknown(keysym as u32),
    }
}

fn get_x11_key_modifiers(state: c_uint) -> KeyModifiers {
    KeyModifiers {
        shift: (state & xlib::ShiftMask) != 0,
        ctrl: (state & xlib::ControlMask) != 0,
        alt: (state & xlib::Mod1Mask) != 0,
        meta: (state & xlib::Mod4Mask) != 0,
    }
}