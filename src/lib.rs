//! # CX Framework
//! 
//! A lightweight, modern cross-platform GUI framework for Rust.
//! 
//! ## Features
//! 
//! - Cross-platform support (Windows, Linux, macOS)
//! - Minimal external dependencies
//! - Safe, concurrent design using Rust idioms
//! - Modular architecture with pluggable backends
//! - Modern 2D graphics and layout system
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use cx::{Application, Window, WindowBuilder};
//! 
//! fn main() -> cx::Result<()> {
//!     let app = Application::new()?;
//!     let window = WindowBuilder::new()
//!         .title("Hello CX Framework")
//!         .size(800, 600)
//!         .build(&app)?;
//!     
//!     app.run()
//! }
//! ```

// Core modules
pub mod application;
pub mod error;
pub mod events;
pub mod window;
pub mod platform;
pub mod drawing;
pub mod widgets;
pub mod layout;

// Public API re-exports
pub use application::{Application, ApplicationBuilder};
pub use error::{Error, Result};
pub use events::{Event, EventHandler, EventLoop};
pub use window::{Window, WindowBuilder, WindowId};
pub use drawing::{Canvas, Color, Point, Size, Rect};
pub use widgets::{Widget, Button, TextInput, Label};
pub use layout::{Layout, LayoutManager, FlexLayout};
pub use platform::{MouseButton, Key, KeyModifiers};

// Platform-specific exports (for advanced users)
pub mod platform_exports {
    pub use crate::platform::*;
}

// Version and build information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}