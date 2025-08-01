//! Platform abstraction layer for the CX Framework.
//! 
//! This module provides a unified interface for platform-specific operations
//! across Windows, Linux, and macOS. It handles window management, event
//! processing, and system integration in a cross-platform manner.

use crate::Result;
use std::any::Any;
use std::sync::{Arc, Mutex};

// Platform-specific implementations
#[cfg(windows)]
pub mod windows;
#[cfg(unix)]
pub mod unix;
#[cfg(target_os = "macos")]
pub mod macos;

// Common platform abstractions
pub mod common;

// Re-exports based on platform
#[cfg(windows)]
pub use self::windows::*;
#[cfg(all(unix, not(target_os = "macos")))]  
pub use self::unix::*;
#[cfg(target_os = "macos")]
pub use self::macos::*;

/// Platform-specific handle for native resources
pub type PlatformHandle = Box<dyn Any + Send + Sync>;

/// Platform-specific window handle
#[derive(Debug, Clone)]
pub struct WindowHandle {
    pub(crate) inner: Arc<Mutex<PlatformHandle>>,
    pub(crate) id: u64,
}

impl WindowHandle {
    pub fn new(handle: PlatformHandle, id: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(handle)),
            id,
        }
    }
    
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Platform abstraction trait for the application backend
pub trait PlatformBackend: Send + Sync {
    /// Initialize the platform backend
    fn initialize(&mut self) -> Result<()>;
    
    /// Create a new window with the given parameters
    fn create_window(&mut self, params: &WindowParams) -> Result<WindowHandle>;
    
    /// Destroy a window
    fn destroy_window(&mut self, handle: &WindowHandle) -> Result<()>;
    
    /// Show a window
    fn show_window(&mut self, handle: &WindowHandle) -> Result<()>;
    
    /// Hide a window
    fn hide_window(&mut self, handle: &WindowHandle) -> Result<()>;
    
    /// Set window title
    fn set_window_title(&mut self, handle: &WindowHandle, title: &str) -> Result<()>;
    
    /// Set window size
    fn set_window_size(&mut self, handle: &WindowHandle, width: u32, height: u32) -> Result<()>;
    
    /// Get window size
    fn get_window_size(&self, handle: &WindowHandle) -> Result<(u32, u32)>;
    
    /// Set window position
    fn set_window_position(&mut self, handle: &WindowHandle, x: i32, y: i32) -> Result<()>;
    
    /// Get window position
    fn get_window_position(&self, handle: &WindowHandle) -> Result<(i32, i32)>;
    
    /// Process platform events (non-blocking)
    fn poll_events(&mut self) -> Result<Vec<PlatformEvent>>;
    
    /// Wait for platform events (blocking)
    fn wait_events(&mut self) -> Result<Vec<PlatformEvent>>;
    
    /// Get the platform-specific drawing context for a window
    fn get_drawing_context(&self, handle: &WindowHandle) -> Result<Box<dyn DrawingContext>>;
    
    /// Clean up platform resources
    fn cleanup(&mut self) -> Result<()>;
}

/// Window creation parameters
#[derive(Debug, Clone)]
pub struct WindowParams {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub transparent: bool,
    pub fullscreen: bool,
}

impl Default for WindowParams {
    fn default() -> Self {
        Self {
            title: "CX Window".to_string(),
            width: 800,
            height: 600,
            x: None,
            y: None,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            fullscreen: false,
        }
    }
}

/// Platform-agnostic event representation
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// Window was closed
    WindowClosed { window_id: u64 },
    
    /// Window was resized
    WindowResized { window_id: u64, width: u32, height: u32 },
    
    /// Window was moved
    WindowMoved { window_id: u64, x: i32, y: i32 },
    
    /// Window gained focus
    WindowFocused { window_id: u64 },
    
    /// Window lost focus
    WindowUnfocused { window_id: u64 },
    
    /// Mouse button pressed
    MousePressed { window_id: u64, button: MouseButton, x: f64, y: f64 },
    
    /// Mouse button released
    MouseReleased { window_id: u64, button: MouseButton, x: f64, y: f64 },
    
    /// Mouse moved
    MouseMoved { window_id: u64, x: f64, y: f64 },
    
    /// Mouse wheel scrolled
    MouseWheel { window_id: u64, delta_x: f64, delta_y: f64 },
    
    /// Key pressed
    KeyPressed { window_id: u64, key: Key, modifiers: KeyModifiers },
    
    /// Key released
    KeyReleased { window_id: u64, key: Key, modifiers: KeyModifiers },
    
    /// Text input received
    TextInput { window_id: u64, text: String },
    
    /// Application should quit
    Quit,
}

/// Mouse button enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Key enumeration (subset of common keys)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Special keys
    Escape,
    Tab,
    Space,
    Return,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    
    // Arrow keys
    Left,
    Right,
    Up,
    Down,
    
    // Modifiers
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    LeftMeta,
    RightMeta,
    
    // Other
    Unknown(u32),
}

/// Key modifier flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    pub fn new() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }
}

impl Default for KeyModifiers {
    fn default() -> Self {
        Self::new()
    }
}

/// Drawing context trait for platform-specific rendering
pub trait DrawingContext: Send + Sync {
    /// Clear the drawing surface with the given color
    fn clear(&mut self, color: (f32, f32, f32, f32)) -> Result<()>;
    
    /// Draw a filled rectangle
    fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32)) -> Result<()>;
    
    /// Draw a rectangle outline
    fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: (f32, f32, f32, f32), width_: f32) -> Result<()>;
    
    /// Draw text
    fn draw_text(&mut self, text: &str, x: f32, y: f32, color: (f32, f32, f32, f32)) -> Result<()>;
    
    /// Present/flush the drawing operations
    fn present(&mut self) -> Result<()>;
    
    /// Get the size of the drawing surface
    fn size(&self) -> (u32, u32);
}

/// Factory function to create platform-specific backend
pub fn create_platform_backend() -> Result<Box<dyn PlatformBackend>> {
    #[cfg(windows)]
    {
        Ok(Box::new(windows::WindowsBackend::new()?))
    }
    
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Ok(Box::new(unix::UnixBackend::new()?))
    }
    
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacOSBackend::new()?))
    }
}

/// Get platform name as string
pub fn platform_name() -> &'static str {
    #[cfg(windows)]
    return "Windows";
    
    #[cfg(all(unix, not(target_os = "macos")))]
    return "Unix/Linux";
    
    #[cfg(target_os = "macos")]
    return "macOS";
}

/// Platform capabilities query
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub transparent_windows: bool,
    pub window_decorations: bool,
    pub always_on_top: bool,
    pub fullscreen: bool,
    pub multi_window: bool,
    pub opengl_support: bool,
    pub vulkan_support: bool,
    pub metal_support: bool,
}

/// Get platform capabilities
pub fn get_platform_capabilities() -> PlatformCapabilities {
    #[cfg(windows)]
    {
        PlatformCapabilities {
            transparent_windows: true,
            window_decorations: true,
            always_on_top: true,
            fullscreen: true,
            multi_window: true,
            opengl_support: true,
            vulkan_support: true,
            metal_support: false,
        }
    }
    
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        PlatformCapabilities {
            transparent_windows: true,
            window_decorations: true,
            always_on_top: true,
            fullscreen: true,
            multi_window: true,
            opengl_support: true,
            vulkan_support: true,
            metal_support: false,
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        PlatformCapabilities {
            transparent_windows: true,
            window_decorations: true,
            always_on_top: true,
            fullscreen: true,
            multi_window: true,
            opengl_support: true,
            vulkan_support: false,
            metal_support: true,
        }
    }
}