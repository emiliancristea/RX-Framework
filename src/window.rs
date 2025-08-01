//! Window management for the CX Framework.
//! 
//! This module provides window creation, management, and rendering capabilities
//! in a cross-platform manner.

use crate::{Error, Result, Event, Canvas};
use crate::platform::{PlatformBackend, WindowHandle, WindowParams};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

/// Unique identifier for windows.
pub type WindowId = u64;

/// Main window struct that represents a platform window.
/// 
/// Windows are created through the `WindowBuilder` and managed by the
/// `Application`. Each window has its own rendering context and can
/// handle events independently.
#[derive(Debug, Clone)]
pub struct Window {
    handle: WindowHandle,
    properties: WindowProperties,
    backend: Weak<Mutex<Box<dyn PlatformBackend>>>,
}

/// Window properties that can be queried and modified.
#[derive(Debug, Clone)]
pub struct WindowProperties {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub transparent: bool,
    pub fullscreen: bool,
    pub focused: bool,
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self {
            title: "CX Window".to_string(),
            width: 800,
            height: 600,
            x: 0,
            y: 0,
            visible: false,
            resizable: true,
            decorations: true,
            always_on_top: false,
            transparent: false,
            fullscreen: false,
            focused: false,
        }
    }
}

/// Builder for creating windows with custom properties.
pub struct WindowBuilder {
    params: WindowParams,
}

impl WindowBuilder {
    /// Create a new window builder with default parameters.
    pub fn new() -> Self {
        Self {
            params: WindowParams::default(),
        }
    }
    
    /// Set the window title.
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.params.title = title.into();
        self
    }
    
    /// Set the window size.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.params.width = width;
        self.params.height = height;
        self
    }
    
    /// Set the window width.
    pub fn width(mut self, width: u32) -> Self {
        self.params.width = width;
        self
    }
    
    /// Set the window height.
    pub fn height(mut self, height: u32) -> Self {
        self.params.height = height;
        self
    }
    
    /// Set the window position.
    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.params.x = Some(x);
        self.params.y = Some(y);
        self
    }
    
    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.params.resizable = resizable;
        self
    }
    
    /// Set whether the window has decorations (title bar, borders).
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.params.decorations = decorations;
        self
    }
    
    /// Set whether the window is always on top.
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.params.always_on_top = always_on_top;
        self
    }
    
    /// Set whether the window is transparent.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.params.transparent = transparent;
        self
    }
    
    /// Set whether the window starts in fullscreen mode.
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.params.fullscreen = fullscreen;
        self
    }
    
    /// Build the window using the provided application.
    pub fn build(self, app: &crate::Application) -> Result<Window> {
        let backend = app.backend();
        let handle = {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.create_window(&self.params)?
        };
        
        let properties = WindowProperties {
            title: self.params.title.clone(),
            width: self.params.width,
            height: self.params.height,
            x: self.params.x.unwrap_or(0),
            y: self.params.y.unwrap_or(0),
            visible: false,
            resizable: self.params.resizable,
            decorations: self.params.decorations,
            always_on_top: self.params.always_on_top,
            transparent: self.params.transparent,
            fullscreen: self.params.fullscreen,
            focused: false,
        };
        
        let window = Window {
            handle,
            properties,
            backend: Arc::downgrade(&backend),
        };
        
        // Register the window with the application
        app.register_window(window.clone())?;
        
        Ok(window)
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Window {
    /// Get the window ID.
    pub fn id(&self) -> WindowId {
        self.handle.id()
    }
    
    /// Get the window properties.
    pub fn properties(&self) -> &WindowProperties {
        &self.properties
    }
    
    /// Show the window.
    pub fn show(&mut self) -> Result<()> {
        if let Some(backend) = self.backend.upgrade() {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.show_window(&self.handle)?;
            self.properties.visible = true;
        }
        Ok(())
    }
    
    /// Hide the window.
    pub fn hide(&mut self) -> Result<()> {
        if let Some(backend) = self.backend.upgrade() {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.hide_window(&self.handle)?;
            self.properties.visible = false;
        }
        Ok(())
    }
    
    /// Set the window title.
    pub fn set_title<S: Into<String>>(&mut self, title: S) -> Result<()> {
        let title = title.into();
        if let Some(backend) = self.backend.upgrade() {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.set_window_title(&self.handle, &title)?;
            self.properties.title = title;
        }
        Ok(())
    }
    
    /// Get the window title.
    pub fn title(&self) -> &str {
        &self.properties.title
    }
    
    /// Set the window size.
    pub fn set_size(&mut self, width: u32, height: u32) -> Result<()> {
        if let Some(backend) = self.backend.upgrade() {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.set_window_size(&self.handle, width, height)?;
            self.properties.width = width;
            self.properties.height = height;
        }
        Ok(())
    }
    
    /// Get the window size.
    pub fn size(&self) -> (u32, u32) {
        (self.properties.width, self.properties.height)
    }
    
    /// Get the window width.
    pub fn width(&self) -> u32 {
        self.properties.width
    }
    
    /// Get the window height.
    pub fn height(&self) -> u32 {
        self.properties.height
    }
    
    /// Set the window position.
    pub fn set_position(&mut self, x: i32, y: i32) -> Result<()> {
        if let Some(backend) = self.backend.upgrade() {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.set_window_position(&self.handle, x, y)?;
            self.properties.x = x;
            self.properties.y = y;
        }
        Ok(())
    }
    
    /// Get the window position.
    pub fn position(&self) -> (i32, i32) {
        (self.properties.x, self.properties.y)
    }
    
    /// Check if the window is visible.
    pub fn is_visible(&self) -> bool {
        self.properties.visible
    }
    
    /// Check if the window is resizable.
    pub fn is_resizable(&self) -> bool {
        self.properties.resizable
    }
    
    /// Check if the window has decorations.
    pub fn has_decorations(&self) -> bool {
        self.properties.decorations
    }
    
    /// Check if the window is always on top.
    pub fn is_always_on_top(&self) -> bool {
        self.properties.always_on_top
    }
    
    /// Check if the window is transparent.
    pub fn is_transparent(&self) -> bool {
        self.properties.transparent
    }
    
    /// Check if the window is in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.properties.fullscreen
    }
    
    /// Check if the window is focused.
    pub fn is_focused(&self) -> bool {
        self.properties.focused
    }
    
    /// Get a drawing canvas for this window.
    pub fn canvas(&self) -> Result<Canvas> {
        if let Some(backend) = self.backend.upgrade() {
            let backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            let context = backend_lock.get_drawing_context(&self.handle)?;
            Ok(Canvas::new(context))
        } else {
            Err(Error::window("Backend no longer available"))
        }
    }
    
    /// Handle an event for this window.
    pub(crate) fn handle_event(&self, event: &Event) -> Result<()> {
        // Update window properties based on events
        match event {
            Event::WindowResized { window_id, width: _width, height: _height } if *window_id == self.id() => {
                // Note: We can't modify self here because this method takes &self
                // In a real implementation, we'd need to use interior mutability
                // or handle this differently
            }
            Event::WindowMoved { window_id, x: _x, y: _y } if *window_id == self.id() => {
                // Same issue as above
            }
            Event::WindowFocused { window_id } if *window_id == self.id() => {
                // Same issue as above
            }
            Event::WindowUnfocused { window_id } if *window_id == self.id() => {
                // Same issue as above
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Update the window (called every frame).
    pub(crate) fn update(&self, _delta_time: Duration) -> Result<()> {
        // Window-specific update logic would go here
        Ok(())
    }
    
    /// Render the window (called every frame).
    pub(crate) fn render(&self) -> Result<()> {
        // Basic rendering - just present the current frame
        if let Some(backend) = self.backend.upgrade() {
            let backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            let mut context = backend_lock.get_drawing_context(&self.handle)?;
            context.present()?;
        }
        Ok(())
    }
    
    /// Close the window.
    pub fn close(&self) -> Result<()> {
        if let Some(backend) = self.backend.upgrade() {
            let mut backend_lock = backend.lock().map_err(|_| Error::window("Failed to lock backend"))?;
            backend_lock.destroy_window(&self.handle)?;
        }
        Ok(())
    }
}

/// Window manager for handling multiple windows.
pub struct WindowManager {
    windows: Vec<Window>,
}

impl WindowManager {
    /// Create a new window manager.
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }
    
    /// Add a window to the manager.
    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }
    
    /// Remove a window from the manager.
    pub fn remove_window(&mut self, window_id: WindowId) -> Option<Window> {
        if let Some(index) = self.windows.iter().position(|w| w.id() == window_id) {
            Some(self.windows.remove(index))
        } else {
            None
        }
    }
    
    /// Get a window by ID.
    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id() == window_id)
    }
    
    /// Get a mutable reference to a window by ID.
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id() == window_id)
    }
    
    /// Get all windows.
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }
    
    /// Get the number of windows.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
    
    /// Update all windows.
    pub fn update_all(&self, delta_time: Duration) -> Result<()> {
        for window in &self.windows {
            window.update(delta_time)?;
        }
        Ok(())
    }
    
    /// Render all windows.
    pub fn render_all(&self) -> Result<()> {
        for window in &self.windows {
            window.render()?;
        }
        Ok(())
    }
    
    /// Close all windows.
    pub fn close_all(&self) -> Result<()> {
        for window in &self.windows {
            window.close()?;
        }
        Ok(())
    }
    
    /// Clear all windows.
    pub fn clear(&mut self) {
        self.windows.clear();
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Window event handler trait.
pub trait WindowEventHandler {
    /// Called when the window is resized.
    fn on_resize(&mut self, _width: u32, _height: u32) -> Result<()> {
        Ok(())
    }
    
    /// Called when the window is moved.
    fn on_move(&mut self, _x: i32, _y: i32) -> Result<()> {
        Ok(())
    }
    
    /// Called when the window gains focus.
    fn on_focus(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the window loses focus.
    fn on_unfocus(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the window is about to close.
    fn on_close(&mut self) -> Result<bool> {
        Ok(true) // Allow close by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_builder() {
        let builder = WindowBuilder::new()
            .title("Test Window")
            .size(1024, 768)
            .position(100, 100)
            .resizable(false);
        
        assert_eq!(builder.params.title, "Test Window");
        assert_eq!(builder.params.width, 1024);
        assert_eq!(builder.params.height, 768);
        assert_eq!(builder.params.x, Some(100));
        assert_eq!(builder.params.y, Some(100));
        assert_eq!(builder.params.resizable, false);
    }

    #[test]
    fn test_window_properties() {
        let props = WindowProperties::default();
        assert_eq!(props.title, "CX Window");
        assert_eq!(props.width, 800);
        assert_eq!(props.height, 600);
        assert_eq!(props.resizable, true);
        assert_eq!(props.decorations, true);
    }

    #[test]
    fn test_window_manager() {
        let mut manager = WindowManager::new();
        assert_eq!(manager.window_count(), 0);
        
        // We can't actually create windows in tests without a real backend
        // so we just test the basic functionality
        manager.clear();
        assert_eq!(manager.window_count(), 0);
    }
}