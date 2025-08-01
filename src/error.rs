//! Error handling for the CX Framework.
//! 
//! This module provides a comprehensive error system that handles both
//! framework-level errors and platform-specific errors in a unified way.

use std::fmt;

/// Result type alias for CX Framework operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error type for the CX Framework.
#[derive(Debug)]
pub enum Error {
    /// Platform-specific initialization failed
    PlatformInit(String),
    
    /// Window creation or manipulation failed
    WindowError(String),
    
    /// Event system error
    EventError(String),
    
    /// Drawing/rendering error
    DrawingError(String),
    
    /// Widget-related error
    WidgetError(String),
    
    /// Layout computation error
    LayoutError(String),
    
    /// Resource loading error (fonts, images, etc.)
    ResourceError(String),
    
    /// Invalid operation or state
    InvalidOperation(String),
    
    /// Generic framework error
    Framework(String),
    
    /// IO-related errors
    Io(std::io::Error),
    
    /// Platform-specific error codes
    PlatformSpecific {
        platform: String,
        code: i32,
        message: String,
    },
}

impl Error {
    /// Create a new platform initialization error
    pub fn platform_init<S: Into<String>>(msg: S) -> Self {
        Error::PlatformInit(msg.into())
    }
    
    /// Create a new window error
    pub fn window<S: Into<String>>(msg: S) -> Self {
        Error::WindowError(msg.into())
    }
    
    /// Create a new event error
    pub fn event<S: Into<String>>(msg: S) -> Self {
        Error::EventError(msg.into())
    }
    
    /// Create a new drawing error
    pub fn drawing<S: Into<String>>(msg: S) -> Self {
        Error::DrawingError(msg.into())
    }
    
    /// Create a new widget error
    pub fn widget<S: Into<String>>(msg: S) -> Self {
        Error::WidgetError(msg.into())
    }
    
    /// Create a new layout error
    pub fn layout<S: Into<String>>(msg: S) -> Self {
        Error::LayoutError(msg.into())
    }
    
    /// Create a new resource error
    pub fn resource<S: Into<String>>(msg: S) -> Self {
        Error::ResourceError(msg.into())
    }
    
    /// Create a new invalid operation error
    pub fn invalid_operation<S: Into<String>>(msg: S) -> Self {
        Error::InvalidOperation(msg.into())
    }
    
    /// Create a new framework error
    pub fn framework<S: Into<String>>(msg: S) -> Self {
        Error::Framework(msg.into())
    }
    
    /// Create a new platform-specific error
    pub fn platform_specific<S1: Into<String>, S2: Into<String>>(platform: S1, code: i32, message: S2) -> Self {
        Error::PlatformSpecific {
            platform: platform.into(),
            code,
            message: message.into(),
        }
    }
    
    /// Get the error category as a string
    pub fn category(&self) -> &'static str {
        match self {
            Error::PlatformInit(_) => "Platform Initialization",
            Error::WindowError(_) => "Window",
            Error::EventError(_) => "Event System",
            Error::DrawingError(_) => "Drawing",
            Error::WidgetError(_) => "Widget",
            Error::LayoutError(_) => "Layout",
            Error::ResourceError(_) => "Resource",
            Error::InvalidOperation(_) => "Invalid Operation",
            Error::Framework(_) => "Framework",
            Error::Io(_) => "I/O",
            Error::PlatformSpecific { .. } => "Platform Specific",
        }
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::PlatformInit(_) => false,
            Error::WindowError(_) => true,
            Error::EventError(_) => true,
            Error::DrawingError(_) => true,
            Error::WidgetError(_) => true,
            Error::LayoutError(_) => true,
            Error::ResourceError(_) => true,
            Error::InvalidOperation(_) => true,
            Error::Framework(_) => false,
            Error::Io(_) => true,
            Error::PlatformSpecific { code, .. } => *code >= 0, // Negative codes typically fatal
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PlatformInit(msg) => write!(f, "Platform initialization failed: {}", msg),
            Error::WindowError(msg) => write!(f, "Window error: {}", msg),
            Error::EventError(msg) => write!(f, "Event system error: {}", msg),
            Error::DrawingError(msg) => write!(f, "Drawing error: {}", msg),
            Error::WidgetError(msg) => write!(f, "Widget error: {}", msg),
            Error::LayoutError(msg) => write!(f, "Layout error: {}", msg),
            Error::ResourceError(msg) => write!(f, "Resource error: {}", msg),
            Error::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            Error::Framework(msg) => write!(f, "Framework error: {}", msg),
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::PlatformSpecific { platform, code, message } => {
                write!(f, "Platform error ({}): {} (code: {})", platform, message, code)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(windows)]
impl From<windows_errors::WindowsError> for Error {
    fn from(err: windows_errors::WindowsError) -> Self {
        Error::platform_specific("Windows", err.code() as i32, err.to_string())
    }
}

// Utility macros for error creation
#[macro_export]
macro_rules! cx_error {
    ($kind:ident, $($arg:tt)*) => {
        $crate::Error::$kind(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! cx_bail {
    ($kind:ident, $($arg:tt)*) => {
        return Err(cx_error!($kind, $($arg)*))
    };
}

// Custom error types for specific Windows errors
#[cfg(windows)]
pub mod windows_errors {
    use super::*;
    
    #[derive(Debug, Clone)]
    pub struct WindowsError {
        code: u32,
        message: String,
    }
    
    impl WindowsError {
        pub fn new(code: u32) -> Self {
            let message = get_windows_error_message(code);
            Self { code, message }
        }
        
        pub fn code(&self) -> u32 {
            self.code
        }
    }
    
    impl fmt::Display for WindowsError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} (code: {})", self.message, self.code)
        }
    }
    
    impl std::error::Error for WindowsError {}
    
    fn get_windows_error_message(code: u32) -> String {
        // This would typically use FormatMessage on Windows
        // For now, return a generic message
        format!("Windows error code: {}", code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::window("Test window error");
        assert_eq!(err.category(), "Window");
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_display() {
        let err = Error::platform_init("Failed to initialize");
        assert_eq!(err.to_string(), "Platform initialization failed: Failed to initialize");
    }

    #[test]
    fn test_error_macro() {
        let err = cx_error!(framework, "Test error with {}", "formatting");
        assert!(matches!(err, Error::Framework(_)));
    }
}