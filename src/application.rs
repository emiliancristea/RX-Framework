//! Application management for the CX Framework.
//! 
//! This module provides the main Application struct that manages the framework
//! lifecycle, event handling, and window management.

use crate::{Error, Result, Event, EventLoop, Window, WindowBuilder};
use crate::platform::{create_platform_backend, PlatformBackend};
use crate::platform::common::{Timer, PerformanceMonitor};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Main application struct that manages the framework lifecycle.
/// 
/// The Application is the entry point for any CX Framework program.
/// It handles platform initialization, window management, and the main event loop.
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use cx::{Application, WindowBuilder};
/// 
/// fn main() -> cx::Result<()> {
///     let app = Application::new()?;
///     let window = WindowBuilder::new()
///         .title("My App")
///         .size(800, 600)
///         .build(&app)?;
///     
///     app.run()
/// }
/// ```
pub struct Application {
    backend: Arc<Mutex<Box<dyn PlatformBackend>>>,
    windows: Arc<Mutex<HashMap<u64, Window>>>,
    event_loop: Arc<Mutex<EventLoop>>,
    is_running: Arc<Mutex<bool>>,
    config: ApplicationConfig,
}

/// Configuration for the application.
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    /// Target frames per second for the main loop
    pub target_fps: u32,
    
    /// Whether to enable performance monitoring
    pub enable_performance_monitoring: bool,
    
    /// Maximum number of performance samples to keep
    pub performance_sample_count: usize,
    
    /// Whether to enable vsync (if supported by platform)
    pub vsync: bool,
    
    /// Application name for platform-specific features
    pub app_name: String,
    
    /// Application version
    pub app_version: String,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            enable_performance_monitoring: false,
            performance_sample_count: 100,
            vsync: true,
            app_name: "CX Application".to_string(),
            app_version: "1.0.0".to_string(),
        }
    }
}

/// Builder for creating applications with custom configuration.
pub struct ApplicationBuilder {
    config: ApplicationConfig,
}

impl ApplicationBuilder {
    /// Create a new application builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: ApplicationConfig::default(),
        }
    }
    
    /// Set the target FPS for the application.
    pub fn target_fps(mut self, fps: u32) -> Self {
        self.config.target_fps = fps;
        self
    }
    
    /// Enable or disable performance monitoring.
    pub fn performance_monitoring(mut self, enable: bool) -> Self {
        self.config.enable_performance_monitoring = enable;
        self
    }
    
    /// Set the number of performance samples to keep.
    pub fn performance_sample_count(mut self, count: usize) -> Self {
        self.config.performance_sample_count = count;
        self
    }
    
    /// Enable or disable vsync.
    pub fn vsync(mut self, enable: bool) -> Self {
        self.config.vsync = enable;
        self
    }
    
    /// Set the application name.
    pub fn app_name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.app_name = name.into();
        self
    }
    
    /// Set the application version.
    pub fn app_version<S: Into<String>>(mut self, version: S) -> Self {
        self.config.app_version = version.into();
        self
    }
    
    /// Build the application with the specified configuration.
    pub fn build(self) -> Result<Application> {
        Application::with_config(self.config)
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    /// Create a new application with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(ApplicationConfig::default())
    }
    
    /// Create a new application with custom configuration.
    pub fn with_config(config: ApplicationConfig) -> Result<Self> {
        let backend = create_platform_backend()?;
        let backend = Arc::new(Mutex::new(backend));
        
        // Initialize the platform backend
        {
            let mut backend_lock = backend.lock().map_err(|_| Error::framework("Failed to lock backend"))?;
            backend_lock.initialize()?;
        }
        
        let event_loop = EventLoop::new(Arc::clone(&backend))?;
        
        Ok(Self {
            backend,
            windows: Arc::new(Mutex::new(HashMap::new())),
            event_loop: Arc::new(Mutex::new(event_loop)),
            is_running: Arc::new(Mutex::new(false)),
            config,
        })
    }
    
    /// Get the application configuration.
    pub fn config(&self) -> &ApplicationConfig {
        &self.config
    }
    
    /// Create a new window builder associated with this application.
    pub fn window_builder(&self) -> WindowBuilder {
        WindowBuilder::new()
    }
    
    /// Get a window by its ID.
    pub fn get_window(&self, window_id: u64) -> Option<Window> {
        let windows = self.windows.lock().ok()?;
        windows.get(&window_id).cloned()
    }
    
    /// Get all windows managed by this application.
    pub fn windows(&self) -> Vec<Window> {
        if let Ok(windows) = self.windows.lock() {
            windows.values().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get the number of windows managed by this application.
    pub fn window_count(&self) -> usize {
        if let Ok(windows) = self.windows.lock() {
            windows.len()
        } else {
            0
        }
    }
    
    /// Register a window with the application.
    pub(crate) fn register_window(&self, window: Window) -> Result<()> {
        let mut windows = self.windows.lock().map_err(|_| Error::framework("Failed to lock windows"))?;
        windows.insert(window.id(), window);
        Ok(())
    }
    
    /// Unregister a window from the application.
    pub(crate) fn unregister_window(&self, window_id: u64) -> Result<()> {
        let mut windows = self.windows.lock().map_err(|_| Error::framework("Failed to lock windows"))?;
        windows.remove(&window_id);
        Ok(())
    }
    
    /// Get a reference to the platform backend.
    pub(crate) fn backend(&self) -> Arc<Mutex<Box<dyn PlatformBackend>>> {
        Arc::clone(&self.backend)
    }
    
    /// Check if the application is currently running.
    pub fn is_running(&self) -> bool {
        if let Ok(running) = self.is_running.lock() {
            *running
        } else {
            false
        }
    }
    
    /// Request the application to quit.
    pub fn quit(&self) -> Result<()> {
        let mut running = self.is_running.lock().map_err(|_| Error::framework("Failed to lock running state"))?;
        *running = false;
        Ok(())
    }
    
    /// Run the main application loop.
    /// 
    /// This method blocks until the application receives a quit signal.
    /// It handles all platform events, window updates, and frame timing.
    pub fn run(self) -> Result<()> {
        // Set running state
        {
            let mut running = self.is_running.lock().map_err(|_| Error::framework("Failed to lock running state"))?;
            *running = true;
        }
        
        let mut timer = Timer::new(self.config.target_fps);
        let mut performance_monitor = if self.config.enable_performance_monitoring {
            Some(PerformanceMonitor::new(self.config.performance_sample_count))
        } else {
            None
        };
        
        while self.is_running() {
            let frame_start = std::time::Instant::now();
            
            // Process events
            let events = {
                let mut event_loop = self.event_loop.lock().map_err(|_| Error::framework("Failed to lock event loop"))?;
                event_loop.poll_events()?
            };
            
            // Handle application-level events
            for event in &events {
                match event {
                    Event::Quit => {
                        self.quit()?;
                        break;
                    }
                    Event::WindowClosed { window_id } => {
                        self.unregister_window(*window_id)?;
                        // If no windows left, quit the application
                        if self.window_count() == 0 {
                            self.quit()?;
                            break;
                        }
                    }
                    _ => {
                        // Forward other events to windows
                        self.dispatch_event_to_windows(event)?;
                    }
                }
            }
            
            // Update and render all windows
            self.update_windows(timer.delta_time())?;
            
            // Record performance data
            if let Some(ref mut monitor) = performance_monitor {
                monitor.record_frame_time(frame_start.elapsed());
            }
            
            // Frame rate limiting
            if timer.tick() {
                timer.sleep_for_frame();
            }
        }
        
        // Cleanup
        self.cleanup()?;
        
        Ok(())
    }
    
    /// Run a single frame of the application loop.
    /// 
    /// This is useful for integrating CX Framework with other event loops
    /// or for implementing custom application logic.
    pub fn run_frame(&self) -> Result<bool> {
        if !self.is_running() {
            return Ok(false);
        }
        
        // Process events
        let events = {
            let mut event_loop = self.event_loop.lock().map_err(|_| Error::framework("Failed to lock event loop"))?;
            event_loop.poll_events()?
        };
        
        // Handle application-level events
        for event in &events {
            match event {
                Event::Quit => {
                    self.quit()?;
                    return Ok(false);
                }
                Event::WindowClosed { window_id } => {
                    self.unregister_window(*window_id)?;
                    // If no windows left, quit the application
                    if self.window_count() == 0 {
                        self.quit()?;
                        return Ok(false);
                    }
                }
                _ => {
                    // Forward other events to windows
                    self.dispatch_event_to_windows(event)?;
                }
            }
        }
        
        // Update and render all windows
        self.update_windows(std::time::Duration::from_millis(16))?; // Assume ~60 FPS
        
        Ok(true)
    }
    
    fn dispatch_event_to_windows(&self, event: &Event) -> Result<()> {
        let windows = self.windows.lock().map_err(|_| Error::framework("Failed to lock windows"))?;
        
        for window in windows.values() {
            window.handle_event(event)?;
        }
        
        Ok(())
    }
    
    fn update_windows(&self, delta_time: std::time::Duration) -> Result<()> {
        let windows = self.windows.lock().map_err(|_| Error::framework("Failed to lock windows"))?;
        
        for window in windows.values() {
            window.update(delta_time)?;
            window.render()?;
        }
        
        Ok(())
    }
    
    fn cleanup(&self) -> Result<()> {
        // Close all windows
        {
            let mut windows = self.windows.lock().map_err(|_| Error::framework("Failed to lock windows"))?;
            windows.clear();
        }
        
        // Cleanup platform backend
        {
            let mut backend = self.backend.lock().map_err(|_| Error::framework("Failed to lock backend"))?;
            backend.cleanup()?;
        }
        
        Ok(())
    }
    
    /// Get performance statistics (if monitoring is enabled).
    pub fn performance_stats(&self) -> Option<PerformanceStats> {
        // This would return performance data if monitoring is enabled
        // For now, return None as we don't store the monitor in the Application
        None::<PerformanceStats>
    }
}

/// Performance statistics for the application.
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub average_fps: f64,
    pub average_frame_time_ms: f64,
    pub min_frame_time_ms: f64,
    pub max_frame_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_builder() {
        let builder = ApplicationBuilder::new()
            .target_fps(120)
            .app_name("Test App")
            .vsync(false);
        
        // We can't actually build and test the application without a proper platform
        // so we just test the builder configuration
        assert_eq!(builder.config.target_fps, 120);
        assert_eq!(builder.config.app_name, "Test App");
        assert_eq!(builder.config.vsync, false);
    }

    #[test]
    fn test_application_config_default() {
        let config = ApplicationConfig::default();
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.enable_performance_monitoring, false);
        assert_eq!(config.vsync, true);
    }
}