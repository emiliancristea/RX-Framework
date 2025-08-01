//! Basic tests for the RX Framework.

use rx::{Application, WindowBuilder, Color, Point, Size, Rect, Button, Label, Result};

#[test]
fn test_application_creation() -> Result<()> {
    let _app = Application::new()?;
    Ok(())
}

#[test]
fn test_window_creation() -> Result<()> {
    let app = Application::new()?;
    
    let _window = WindowBuilder::new()
        .title("Test Window")
        .size(400, 300)
        .build(&app)?;
    
    Ok(())
}

#[test]
fn test_basic_widgets() -> Result<()> {
    let _button = Button::new(1, "Test Button".to_string());
    let _label = Label::new(2, "Test Label".to_string());
    Ok(())
}

#[test]
fn test_color_creation() -> Result<()> {
    let _red = Color::rgb(1.0, 0.0, 0.0);
    let _green = Color::GREEN;
    let _blue = Color::rgba(0.0, 0.0, 1.0, 0.5);
    Ok(())
}

#[test]
fn test_geometry_types() -> Result<()> {
    let _point = Point::new(10.0, 20.0);
    let _size = Size::new(100.0, 200.0);
    let _rect = Rect::new(10.0, 20.0, 100.0, 200.0);
    Ok(())
}