//! Windows-specific platform implementation for the CX Framework.
//! 
//! This module provides Windows-specific implementations of the platform
//! abstraction layer using the Win32 API through the winapi crate.

#[cfg(windows)]
use winapi::{
    shared::{
        minwindef::{UINT, WPARAM, LPARAM, LRESULT, ATOM},
        windef::{HWND, RECT, HBRUSH},
    },
    um::{
        winuser::{
            RegisterClassW, CreateWindowExW, DestroyWindow, ShowWindow, UpdateWindow,
            GetMessageW, PeekMessageW, TranslateMessage, DispatchMessageW, PostQuitMessage,
            DefWindowProcW, LoadCursorW, GetWindowRect, SetWindowPos, SetWindowTextW,
            GetClientRect, InvalidateRect, BeginPaint, EndPaint, FillRect, GetDC, ReleaseDC,
            SetWindowLongPtrW, GetWindowLongPtrW,
            WNDCLASSW, MSG, WS_OVERLAPPEDWINDOW, SW_SHOW, SW_HIDE,
            WM_DESTROY, WM_CLOSE, WM_SIZE, WM_MOVE, WM_PAINT, WM_KEYDOWN, WM_KEYUP,
            WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_MBUTTONDOWN,
            WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_CHAR, WM_SETFOCUS, WM_KILLFOCUS,
            IDC_ARROW, SWP_NOZORDER, COLOR_WINDOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
            GWLP_USERDATA, PM_REMOVE, VK_ESCAPE, VK_TAB, VK_SPACE, VK_RETURN, VK_BACK,
            VK_DELETE, VK_INSERT, VK_HOME, VK_END, VK_PRIOR, VK_NEXT, VK_LEFT, VK_RIGHT,
            VK_UP, VK_DOWN, VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN,
        },
        wingdi::{CreateSolidBrush, RGB, SetTextColor, SetBkMode, TextOutW, TRANSPARENT},
        libloaderapi::GetModuleHandleW,
    },
};

use crate::{Error, Result};
use super::{
    PlatformBackend, WindowHandle, WindowParams, PlatformEvent,
    DrawingContext, MouseButton, Key, KeyModifiers,
};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::{Arc, Mutex, Once};

static INIT: Once = Once::new();
static mut CLASS_ATOM: ATOM = 0;
static mut GLOBAL_EVENT_QUEUE: Option<Arc<Mutex<Vec<PlatformEvent>>>> = None;

// Thread-safe wrapper for HWND
#[derive(Debug, Clone, Copy)]
struct SafeHWND(usize);

unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}

impl SafeHWND {
    fn new(hwnd: HWND) -> Self {
        SafeHWND(hwnd as usize)
    }
    
    fn as_hwnd(&self) -> HWND {
        self.0 as HWND
    }
}

/// Windows-specific backend implementation
pub struct WindowsBackend {
    windows: HashMap<u64, WindowsWindow>,
    next_window_id: u64,
    event_queue: Arc<Mutex<Vec<PlatformEvent>>>,
}

struct WindowsWindow {
    hwnd: SafeHWND,
    id: u64,
    drawing_context: Option<WindowsDrawingContext>,
}

struct WindowsDrawingContext {
    hwnd: SafeHWND,
    width: u32,
    height: u32,
}

impl WindowsBackend {
    pub fn new() -> Result<Self> {
        Self::initialize_window_class()?;
        
        let event_queue = Arc::new(Mutex::new(Vec::new()));
        
        // Set up global event queue for window procedure
        unsafe {
            GLOBAL_EVENT_QUEUE = Some(Arc::clone(&event_queue));
        }
        
        Ok(Self {
            windows: HashMap::new(),
            next_window_id: 1,
            event_queue,
        })
    }
    
    fn initialize_window_class() -> Result<()> {
        unsafe {
            INIT.call_once(|| {
                let class_name = wide_string("CXFrameworkWindow");
                let hinstance = GetModuleHandleW(ptr::null());
                
                let wc = WNDCLASSW {
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: hinstance,
                    hIcon: ptr::null_mut(),
                    hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                    hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                    lpszMenuName: ptr::null(),
                    lpszClassName: class_name.as_ptr(),
                };
                
                CLASS_ATOM = RegisterClassW(&wc);
            });
            
            if CLASS_ATOM == 0 {
                return Err(Error::platform_init("Failed to register window class"));
            }
        }
        
        Ok(())
    }
    
    fn get_window_by_hwnd(&self, hwnd: HWND) -> Option<&WindowsWindow> {
        let safe_hwnd = SafeHWND::new(hwnd);
        self.windows.values().find(|w| w.hwnd.0 == safe_hwnd.0)
    }
    
    fn get_window_by_hwnd_mut(&mut self, hwnd: HWND) -> Option<&mut WindowsWindow> {
        let safe_hwnd = SafeHWND::new(hwnd);
        self.windows.values_mut().find(|w| w.hwnd.0 == safe_hwnd.0)
    }
}

impl PlatformBackend for WindowsBackend {
    fn initialize(&mut self) -> Result<()> {
        // Already initialized in new()
        Ok(())
    }
    
    fn create_window(&mut self, params: &WindowParams) -> Result<WindowHandle> {
        unsafe {
            let class_name = wide_string("CXFrameworkWindow");
            let window_title = wide_string(&params.title);
            
            let mut style = WS_OVERLAPPEDWINDOW;
            if !params.decorations {
                style = winapi::um::winuser::WS_POPUP;
            }
            if !params.resizable {
                style &= !(winapi::um::winuser::WS_THICKFRAME | winapi::um::winuser::WS_MAXIMIZEBOX);
            }
            
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                style,
                params.x.unwrap_or(CW_USEDEFAULT),
                params.y.unwrap_or(CW_USEDEFAULT),
                params.width as i32,
                params.height as i32,
                ptr::null_mut(),
                ptr::null_mut(),
                GetModuleHandleW(ptr::null()),
                ptr::null_mut(),
            );
            
            if hwnd.is_null() {
                return Err(Error::window("Failed to create window"));
            }
            
            let window_id = self.next_window_id;
            self.next_window_id += 1;
            
            let safe_hwnd = SafeHWND::new(hwnd);
            let window = WindowsWindow {
                hwnd: safe_hwnd.clone(),
                id: window_id,
                drawing_context: None,
            };
            
            self.windows.insert(window_id, window);
            
            // Set the window ID as user data for the window proc
            SetWindowLongPtrW(
                hwnd, 
                GWLP_USERDATA, 
                window_id as isize
            );
            
            let handle = WindowHandle::new(
                Box::new(WindowsWindowHandle { hwnd: safe_hwnd }),
                window_id,
            );
            
            Ok(handle)
        }
    }
    
    fn destroy_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.remove(&handle.id) {
            unsafe {
                DestroyWindow(window.hwnd.as_hwnd());
            }
        }
        Ok(())
    }
    
    fn show_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                ShowWindow(window.hwnd.as_hwnd(), SW_SHOW);
                UpdateWindow(window.hwnd.as_hwnd());
            }
        }
        Ok(())
    }
    
    fn hide_window(&mut self, handle: &WindowHandle) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                ShowWindow(window.hwnd.as_hwnd(), SW_HIDE);
            }
        }
        Ok(())
    }
    
    fn set_window_title(&mut self, handle: &WindowHandle, title: &str) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            let title_wide = wide_string(title);
            unsafe {
                SetWindowTextW(window.hwnd.as_hwnd(), title_wide.as_ptr());
            }
        }
        Ok(())
    }
    
    fn set_window_size(&mut self, handle: &WindowHandle, width: u32, height: u32) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                SetWindowPos(
                    window.hwnd.as_hwnd(),
                    ptr::null_mut(),
                    0, 0,
                    width as i32, height as i32,
                    SWP_NOZORDER | winapi::um::winuser::SWP_NOMOVE,
                );
            }
        }
        Ok(())
    }
    
    fn get_window_size(&self, handle: &WindowHandle) -> Result<(u32, u32)> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(window.hwnd.as_hwnd(), &mut rect);
                Ok(((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32))
            }
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn set_window_position(&mut self, handle: &WindowHandle, x: i32, y: i32) -> Result<()> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                SetWindowPos(
                    window.hwnd.as_hwnd(),
                    ptr::null_mut(),
                    x, y,
                    0, 0,
                    SWP_NOZORDER | winapi::um::winuser::SWP_NOSIZE,
                );
            }
        }
        Ok(())
    }
    
    fn get_window_position(&self, handle: &WindowHandle) -> Result<(i32, i32)> {
        if let Some(window) = self.windows.get(&handle.id) {
            unsafe {
                let mut rect: RECT = std::mem::zeroed();
                GetWindowRect(window.hwnd.as_hwnd(), &mut rect);
                Ok((rect.left, rect.top))
            }
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn poll_events(&mut self) -> Result<Vec<PlatformEvent>> {
        self.process_messages(false)
    }
    
    fn wait_events(&mut self) -> Result<Vec<PlatformEvent>> {
        self.process_messages(true)
    }
    
    fn get_drawing_context(&self, handle: &WindowHandle) -> Result<Box<dyn DrawingContext>> {
        if let Some(window) = self.windows.get(&handle.id) {
            let (width, height) = self.get_window_size(handle)?;
            Ok(Box::new(WindowsDrawingContext {
                hwnd: window.hwnd,
                width,
                height,
            }))
        } else {
            Err(Error::window("Window not found"))
        }
    }
    
    fn cleanup(&mut self) -> Result<()> {
        // Windows will clean up resources automatically
        self.windows.clear();
        Ok(())
    }
}

impl WindowsBackend {
    fn process_messages(&mut self, wait: bool) -> Result<Vec<PlatformEvent>> {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            
            if wait {
                // Blocking wait for at least one message
                if GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                
                // Process any additional messages in the queue
                while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            } else {
                // Non-blocking: process all available messages
                while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        
        // Return accumulated events from the event queue and clear it
        let mut events = Vec::new();
        if let Ok(mut queue) = self.event_queue.lock() {
            events.append(&mut *queue);
        }
        
        Ok(events)
    }
}

// Platform handle wrapper
#[derive(Debug)]
struct WindowsWindowHandle {
    hwnd: SafeHWND,
}

// Window procedure for handling messages
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Get window ID from user data
    let window_id = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as u64;
    
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            push_event(PlatformEvent::Quit);
            0
        }
        WM_CLOSE => {
            push_event(PlatformEvent::WindowClosed { window_id });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_SIZE => {
            let width = (lparam & 0xFFFF) as u32;
            let height = ((lparam >> 16) & 0xFFFF) as u32;
            push_event(PlatformEvent::WindowResized { window_id, width, height });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MOVE => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            push_event(PlatformEvent::WindowMoved { window_id, x, y });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_PAINT => {
            let mut ps: winapi::um::winuser::PAINTSTRUCT = std::mem::zeroed();
            let _hdc = BeginPaint(hwnd, &mut ps);
            EndPaint(hwnd, &ps);
            0
        }
        WM_LBUTTONDOWN => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MousePressed { 
                window_id, 
                button: MouseButton::Left, 
                x, 
                y 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_LBUTTONUP => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MouseReleased { 
                window_id, 
                button: MouseButton::Left, 
                x, 
                y 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_RBUTTONDOWN => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MousePressed { 
                window_id, 
                button: MouseButton::Right, 
                x, 
                y 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_RBUTTONUP => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MouseReleased { 
                window_id, 
                button: MouseButton::Right, 
                x, 
                y 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MBUTTONDOWN => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MousePressed { 
                window_id, 
                button: MouseButton::Middle, 
                x, 
                y 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MBUTTONUP => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MouseReleased { 
                window_id, 
                button: MouseButton::Middle, 
                x, 
                y 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MOUSEMOVE => {
            let x = (lparam & 0xFFFF) as i16 as f64;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as f64;
            push_event(PlatformEvent::MouseMoved { window_id, x, y });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) & 0xFFFF) as i16 as f64 / 120.0; // WHEEL_DELTA is 120
            push_event(PlatformEvent::MouseWheel { 
                window_id, 
                delta_x: 0.0, 
                delta_y: delta 
            });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_KEYDOWN => {
            let key = virtual_key_to_key(wparam);
            let modifiers = get_key_modifiers();
            push_event(PlatformEvent::KeyPressed { window_id, key, modifiers });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_KEYUP => {
            let key = virtual_key_to_key(wparam);
            let modifiers = get_key_modifiers();
            push_event(PlatformEvent::KeyReleased { window_id, key, modifiers });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CHAR => {
            if let Ok(text) = String::from_utf16(&[(wparam as u16)]) {
                if !text.chars().any(|c| c.is_control()) {
                    push_event(PlatformEvent::TextInput { window_id, text });
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_SETFOCUS => {
            push_event(PlatformEvent::WindowFocused { window_id });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_KILLFOCUS => {
            push_event(PlatformEvent::WindowUnfocused { window_id });
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// Helper function to push events to the global queue
unsafe fn push_event(event: PlatformEvent) {
    if let Some(ref queue) = GLOBAL_EVENT_QUEUE {
        if let Ok(mut events) = queue.lock() {
            events.push(event);
        }
    }
}

impl DrawingContext for WindowsDrawingContext {
    fn clear(&mut self, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let hdc = GetDC(self.hwnd.as_hwnd());
            if hdc.is_null() {
                return Err(Error::drawing("Failed to get device context"));
            }
            
            let brush = CreateSolidBrush(RGB(
                (color.0 * 255.0) as u8,
                (color.1 * 255.0) as u8,
                (color.2 * 255.0) as u8,
            ));
            
            let rect = RECT {
                left: 0,
                top: 0,
                right: self.width as i32,
                bottom: self.height as i32,
            };
            
            FillRect(hdc, &rect, brush);
            winapi::um::wingdi::DeleteObject(brush as *mut _);
            ReleaseDC(self.hwnd.as_hwnd(), hdc);
        }
        Ok(())
    }
    
    fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let hdc = GetDC(self.hwnd.as_hwnd());
            if hdc.is_null() {
                return Err(Error::drawing("Failed to get device context"));
            }
            
            let brush = CreateSolidBrush(RGB(
                (color.0 * 255.0) as u8,
                (color.1 * 255.0) as u8,
                (color.2 * 255.0) as u8,
            ));
            
            let rect = RECT {
                left: x as i32,
                top: y as i32,
                right: (x + width) as i32,
                bottom: (y + height) as i32,
            };
            
            FillRect(hdc, &rect, brush);
            winapi::um::wingdi::DeleteObject(brush as *mut _);
            ReleaseDC(self.hwnd.as_hwnd(), hdc);
        }
        Ok(())
    }
    
    fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32), stroke_width: f32) -> Result<()> {
        // Simple implementation using fill_rect for now
        let sw = stroke_width;
        self.fill_rect(x, y, width, sw, color)?; // Top
        self.fill_rect(x, y + height - sw, width, sw, color)?; // Bottom
        self.fill_rect(x, y, sw, height, color)?; // Left
        self.fill_rect(x + width - sw, y, sw, height, color)?; // Right
        Ok(())
    }
    
    fn draw_text(&mut self, text: &str, x: f32, y: f32, color: (f32, f32, f32, f32)) -> Result<()> {
        unsafe {
            let hdc = GetDC(self.hwnd.as_hwnd());
            if hdc.is_null() {
                return Err(Error::drawing("Failed to get device context"));
            }
            
            let text_wide = wide_string(text);
            SetTextColor(hdc, RGB(
                (color.0 * 255.0) as u8,
                (color.1 * 255.0) as u8,
                (color.2 * 255.0) as u8,
            ));
            SetBkMode(hdc, TRANSPARENT as i32);
            
            TextOutW(
                hdc,
                x as i32,
                y as i32,
                text_wide.as_ptr(),
                text_wide.len() as i32 - 1, // -1 for null terminator
            );
            
            ReleaseDC(self.hwnd.as_hwnd(), hdc);
        }
        Ok(())
    }
    
    fn present(&mut self) -> Result<()> {
        unsafe {
            InvalidateRect(self.hwnd.as_hwnd(), ptr::null(), 0);
            UpdateWindow(self.hwnd.as_hwnd());
        }
        Ok(())
    }
    
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

// Helper function to convert Rust strings to wide strings for Windows API
fn wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

// Key mapping functions
fn virtual_key_to_key(vkey: WPARAM) -> Key {
    match vkey as i32 {
        VK_ESCAPE => Key::Escape,
        VK_TAB => Key::Tab,
        VK_SPACE => Key::Space,
        VK_RETURN => Key::Return,
        VK_BACK => Key::Backspace,
        VK_DELETE => Key::Delete,
        VK_INSERT => Key::Insert,
        VK_HOME => Key::Home,
        VK_END => Key::End,
        VK_PRIOR => Key::PageUp,
        VK_NEXT => Key::PageDown,
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        // Letters (A-Z)
        0x41..=0x5A => {
            match vkey as u8 {
                b'A' => Key::A, b'B' => Key::B, b'C' => Key::C, b'D' => Key::D, 
                b'E' => Key::E, b'F' => Key::F, b'G' => Key::G, b'H' => Key::H,
                b'I' => Key::I, b'J' => Key::J, b'K' => Key::K, b'L' => Key::L,
                b'M' => Key::M, b'N' => Key::N, b'O' => Key::O, b'P' => Key::P,
                b'Q' => Key::Q, b'R' => Key::R, b'S' => Key::S, b'T' => Key::T,
                b'U' => Key::U, b'V' => Key::V, b'W' => Key::W, b'X' => Key::X,
                b'Y' => Key::Y, b'Z' => Key::Z,
                _ => Key::Unknown(vkey as u32),
            }
        }
        // Numbers (0-9)
        0x30..=0x39 => {
            match vkey as u8 {
                b'0' => Key::Key0, b'1' => Key::Key1, b'2' => Key::Key2,
                b'3' => Key::Key3, b'4' => Key::Key4, b'5' => Key::Key5,
                b'6' => Key::Key6, b'7' => Key::Key7, b'8' => Key::Key8,
                b'9' => Key::Key9,
                _ => Key::Unknown(vkey as u32),
            }
        }
        // Function keys
        0x70..=0x7B => {
            match vkey as i32 {
                0x70 => Key::F1, 0x71 => Key::F2, 0x72 => Key::F3, 0x73 => Key::F4,
                0x74 => Key::F5, 0x75 => Key::F6, 0x76 => Key::F7, 0x77 => Key::F8,
                0x78 => Key::F9, 0x79 => Key::F10, 0x7A => Key::F11, 0x7B => Key::F12,
                _ => Key::Unknown(vkey as u32),
            }
        }
        // Modifier keys
        VK_SHIFT => Key::LeftShift,
        VK_CONTROL => Key::LeftCtrl,
        VK_MENU => Key::LeftAlt,
        VK_LWIN => Key::LeftMeta,
        VK_RWIN => Key::RightMeta,
        _ => Key::Unknown(vkey as u32),
    }
}

fn get_key_modifiers() -> KeyModifiers {
    unsafe {
        KeyModifiers {
            shift: winapi::um::winuser::GetKeyState(VK_SHIFT) < 0,
            ctrl: winapi::um::winuser::GetKeyState(VK_CONTROL) < 0,
            alt: winapi::um::winuser::GetKeyState(VK_MENU) < 0,
            meta: winapi::um::winuser::GetKeyState(VK_LWIN) < 0 || 
                  winapi::um::winuser::GetKeyState(VK_RWIN) < 0,
        }
    }
}