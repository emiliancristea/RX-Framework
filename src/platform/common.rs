//! Common platform utilities shared across all platforms.
//! 
//! This module contains cross-platform utilities and helper functions
//! that are used by all platform-specific implementations.

use crate::{Error, Result};
use std::time::{Duration, Instant};

/// Platform-agnostic timer for frame rate control and animations
#[derive(Debug, Clone)]
pub struct Timer {
    start_time: Instant,
    last_frame: Instant,
    target_fps: u32,
    frame_duration: Duration,
}

impl Timer {
    /// Create a new timer with the specified target FPS
    pub fn new(target_fps: u32) -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            target_fps,
            frame_duration: Duration::from_secs_f64(1.0 / target_fps as f64),
        }
    }
    
    /// Get the elapsed time since the timer was created
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get the time since the last frame
    pub fn delta_time(&self) -> Duration {
        self.last_frame.elapsed()
    }
    
    /// Mark the start of a new frame and return whether enough time has passed
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame);
        
        if elapsed >= self.frame_duration {
            self.last_frame = now;
            true
        } else {
            false
        }
    }
    
    /// Sleep for the remaining frame time to maintain target FPS
    pub fn sleep_for_frame(&self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_duration {
            std::thread::sleep(self.frame_duration - elapsed);
        }
    }
    
    /// Get the current FPS based on actual frame times
    pub fn current_fps(&self) -> f64 {
        let delta = self.delta_time();
        if delta.as_secs_f64() > 0.0 {
            1.0 / delta.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Set a new target FPS
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
        self.frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
    }
    
    /// Get the target FPS
    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }
}

/// Color utilities for cross-platform color handling
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorUtils;

impl ColorUtils {
    /// Convert RGB values (0-255) to normalized float values (0.0-1.0)
    pub fn rgb_to_float(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
    
    /// Convert RGBA values (0-255) to normalized float values (0.0-1.0)
    pub fn rgba_to_float(r: u8, g: u8, b: u8, a: u8) -> (f32, f32, f32, f32) {
        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }
    
    /// Convert normalized float values (0.0-1.0) to RGB values (0-255)
    pub fn float_to_rgb(r: f32, g: f32, b: f32) -> (u8, u8, u8) {
        (
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
    
    /// Convert normalized float values (0.0-1.0) to RGBA values (0-255)
    pub fn float_to_rgba(r: f32, g: f32, b: f32, a: f32) -> (u8, u8, u8, u8) {
        (
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
            (a.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
    
    /// Parse a hex color string (e.g., "#FF0000" or "FF0000")
    pub fn parse_hex(hex: &str) -> Result<(f32, f32, f32, f32)> {
        let hex = hex.trim_start_matches('#');
        
        let (r, g, b, a) = match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| Error::framework("Invalid hex color"))?;
                (r, g, b, a)
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| Error::framework("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| Error::framework("Invalid hex color"))?;
                (r, g, b, 255)
            }
            _ => return Err(Error::framework("Invalid hex color length")),
        };
        
        Ok(Self::rgba_to_float(r, g, b, a))
    }
}

/// DPI (Dots Per Inch) utilities for handling high-DPI displays
#[derive(Debug, Clone, Copy)]
pub struct DpiUtils;

impl DpiUtils {
    /// Standard DPI value (96 DPI on Windows, 72 DPI on macOS)
    pub const STANDARD_DPI: f32 = 96.0;
    
    /// Calculate DPI scale factor
    pub fn scale_factor(current_dpi: f32) -> f32 {
        current_dpi / Self::STANDARD_DPI
    }
    
    /// Scale a value by DPI
    pub fn scale_value(value: f32, dpi: f32) -> f32 {
        value * Self::scale_factor(dpi)
    }
    
    /// Scale coordinates by DPI
    pub fn scale_point(x: f32, y: f32, dpi: f32) -> (f32, f32) {
        let scale = Self::scale_factor(dpi);
        (x * scale, y * scale)
    }
    
    /// Scale size by DPI
    pub fn scale_size(width: f32, height: f32, dpi: f32) -> (f32, f32) {
        let scale = Self::scale_factor(dpi);
        (width * scale, height * scale)
    }
}

/// Keyboard utilities for cross-platform key handling
#[derive(Debug, Clone, Copy)]
pub struct KeyboardUtils;

impl KeyboardUtils {
    /// Check if a key is a modifier key
    pub fn is_modifier_key(key: &super::Key) -> bool {
        matches!(key, 
            super::Key::LeftShift | super::Key::RightShift |
            super::Key::LeftCtrl | super::Key::RightCtrl |
            super::Key::LeftAlt | super::Key::RightAlt |
            super::Key::LeftMeta | super::Key::RightMeta
        )
    }
    
    /// Check if a key is a function key
    pub fn is_function_key(key: &super::Key) -> bool {
        matches!(key,
            super::Key::F1 | super::Key::F2 | super::Key::F3 | super::Key::F4 |
            super::Key::F5 | super::Key::F6 | super::Key::F7 | super::Key::F8 |
            super::Key::F9 | super::Key::F10 | super::Key::F11 | super::Key::F12
        )
    }
    
    /// Check if a key is an arrow key
    pub fn is_arrow_key(key: &super::Key) -> bool {
        matches!(key, super::Key::Left | super::Key::Right | super::Key::Up | super::Key::Down)
    }
    
    /// Check if a key is a printable character
    pub fn is_printable(key: &super::Key) -> bool {
        use super::Key;
        match key {
            Key::A | Key::B | Key::C | Key::D | Key::E | Key::F | Key::G | Key::H | Key::I | Key::J |
            Key::K | Key::L | Key::M | Key::N | Key::O | Key::P | Key::Q | Key::R | Key::S | Key::T |
            Key::U | Key::V | Key::W | Key::X | Key::Y | Key::Z |
            Key::Key0 | Key::Key1 | Key::Key2 | Key::Key3 | Key::Key4 | Key::Key5 | Key::Key6 | Key::Key7 | Key::Key8 | Key::Key9 |
            Key::Space => true,
            _ => false,
        }
    }
}

/// Performance monitoring utilities
#[derive(Debug)]
pub struct PerformanceMonitor {
    frame_times: Vec<Duration>,
    max_samples: usize,
    current_index: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with the specified sample count
    pub fn new(max_samples: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_samples),
            max_samples,
            current_index: 0,
        }
    }
    
    /// Record a frame time
    pub fn record_frame_time(&mut self, frame_time: Duration) {
        if self.frame_times.len() < self.max_samples {
            self.frame_times.push(frame_time);
        } else {
            self.frame_times[self.current_index] = frame_time;
            self.current_index = (self.current_index + 1) % self.max_samples;
        }
    }
    
    /// Get the average frame time
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::from_secs(0);
        }
        
        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }
    
    /// Get the average FPS
    pub fn average_fps(&self) -> f64 {
        let avg_time = self.average_frame_time();
        if avg_time.as_secs_f64() > 0.0 {
            1.0 / avg_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get the minimum frame time (best performance)
    pub fn min_frame_time(&self) -> Option<Duration> {
        self.frame_times.iter().min().copied()
    }
    
    /// Get the maximum frame time (worst performance)
    pub fn max_frame_time(&self) -> Option<Duration> {
        self.frame_times.iter().max().copied()
    }
    
    /// Clear all recorded samples
    pub fn clear(&mut self) {
        self.frame_times.clear();
        self.current_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let mut timer = Timer::new(60);
        assert_eq!(timer.target_fps(), 60);
        
        // Test tick (might not tick immediately due to timing)
        let _ = timer.tick();
    }

    #[test]
    fn test_color_utils() {
        let (r, g, b) = ColorUtils::rgb_to_float(255, 128, 0);
        assert_eq!(r, 1.0);
        assert_eq!(g, 128.0 / 255.0);
        assert_eq!(b, 0.0);
        
        let (r2, g2, b2) = ColorUtils::float_to_rgb(r, g, b);
        assert_eq!(r2, 255);
        assert_eq!(g2, 128);
        assert_eq!(b2, 0);
    }

    #[test]
    fn test_hex_color_parsing() {
        let color = ColorUtils::parse_hex("#FF0000").unwrap();
        assert_eq!(color, (1.0, 0.0, 0.0, 1.0));
        
        let color = ColorUtils::parse_hex("00FF00").unwrap();
        assert_eq!(color, (0.0, 1.0, 0.0, 1.0));
        
        let color = ColorUtils::parse_hex("#F00").unwrap();
        assert_eq!(color, (1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_dpi_utils() {
        let scale = DpiUtils::scale_factor(192.0);
        assert_eq!(scale, 2.0);
        
        let scaled = DpiUtils::scale_value(10.0, 192.0);
        assert_eq!(scaled, 20.0);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new(3);
        
        monitor.record_frame_time(Duration::from_millis(16));
        monitor.record_frame_time(Duration::from_millis(17));
        monitor.record_frame_time(Duration::from_millis(15));
        
        let avg = monitor.average_frame_time();
        assert!(avg.as_millis() >= 15 && avg.as_millis() <= 17);
    }
}