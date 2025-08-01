# Getting Started with CX Framework

The CX Framework is a modern, cross-platform GUI framework written in Rust. It provides native window management, 2D graphics, and a comprehensive widget system that works seamlessly across Windows, Linux, and macOS.

## Table of Contents

- [Installation](#installation)
- [Platform Requirements](#platform-requirements)
- [Quick Start](#quick-start)
- [Basic Concepts](#basic-concepts)
- [Creating Your First Application](#creating-your-first-application)
- [Widget System](#widget-system)
- [Event Handling](#event-handling)
- [Layout Management](#layout-management)
- [Drawing and Graphics](#drawing-and-graphics)
- [Examples](#examples)

## Installation

Add CX Framework to your `Cargo.toml`:

```toml
[dependencies]
cx-framework = "0.1.0"
```

## Platform Requirements

### Windows
- Windows 10 or later
- Visual Studio Build Tools (for linking)
- No additional dependencies required

### Linux
- X11 development libraries
- Install on Ubuntu/Debian: `sudo apt install libx11-dev`
- Install on RHEL/CentOS: `sudo yum install libX11-devel`

### macOS
- macOS 10.15 (Catalina) or later
- Xcode Command Line Tools
- No additional dependencies required

## Quick Start

Here's a minimal "Hello World" application:

```rust
use cx::{Application, WindowBuilder, Color, Point, Result};

fn main() -> Result<()> {
    // Create the application
    let app = Application::new()?;
    
    // Create a window
    let mut window = WindowBuilder::new()
        .title("Hello, CX Framework!")
        .size(400, 300)
        .build(&app)?;
    
    // Show the window
    window.show()?;
    
    // Get drawing canvas
    let mut canvas = window.canvas()?;
    
    // Clear with a nice background
    canvas.clear(Color::rgb(0.9, 0.95, 1.0))?;
    
    // Draw some text
    canvas.draw_text(
        "Hello, World!", 
        Point::new(50.0, 100.0), 
        Color::BLACK
    )?;
    
    // Present the frame
    canvas.present()?;
    
    // Run the application
    app.run()
}
```

## Basic Concepts

### Architecture

The CX Framework is built on several core concepts:

- **Application**: The main application instance that manages the event loop
- **Window**: Native windows that can contain widgets and graphics
- **Canvas**: 2D drawing surface for custom graphics
- **Widgets**: Pre-built UI components (buttons, labels, text inputs, etc.)
- **Layout**: System for arranging widgets within containers
- **Events**: User input and system events (mouse, keyboard, window events)

### Cross-Platform Design

CX Framework uses platform-specific backends:
- **Windows**: Win32 API + GDI
- **Linux**: X11 + Xlib
- **macOS**: Cocoa + AppKit

All platform differences are abstracted away, providing a unified API.

## Creating Your First Application

### 1. Basic Window

```rust
use cx::{Application, WindowBuilder, Result};

fn main() -> Result<()> {
    let app = Application::new()?;
    
    let window = WindowBuilder::new()
        .title("My Application")
        .size(800, 600)
        .position(100, 100)
        .build(&app)?;
    
    window.show()?;
    app.run()
}
```

### 2. Adding Widgets

```rust
use cx::{Application, WindowBuilder, Button, Label, Container, Result};

fn main() -> Result<()> {
    let app = Application::new()?;
    
    let window = WindowBuilder::new()
        .title("Widget Demo")
        .size(400, 300)
        .build(&app)?;
    
    // Create widgets
    let button = Button::new(1, "Click Me!".to_string());
    let label = Label::new(2, "Hello, CX!".to_string());
    
    // Create container to hold widgets
    let mut container = Container::new(3);
    container.add_child(Box::new(button));
    container.add_child(Box::new(label));
    
    window.show()?;
    app.run()
}
```

## Widget System

### Available Widgets

- **Button**: Clickable button with text and styling
- **Label**: Static text display
- **TextInput**: Single-line text input field
- **Container**: Layout container for other widgets

### Widget Properties

All widgets support:
- **Position and Size**: `bounds()`, `set_bounds()`
- **Visibility**: `is_visible()`, `set_visible()`
- **Enabled State**: `is_enabled()`, `set_enabled()`
- **Event Handling**: `handle_event()`

### Creating Custom Widgets

```rust
use cx::{Widget, BaseWidget, Event, Canvas, Rect, Point, Size, Result};

pub struct CustomWidget {
    base: BaseWidget,
    custom_data: String,
}

impl CustomWidget {
    pub fn new(id: u64, data: String) -> Self {
        Self {
            base: BaseWidget::new(id),
            custom_data: data,
        }
    }
}

impl Widget for CustomWidget {
    fn bounds(&self) -> Rect {
        self.base.bounds()
    }
    
    fn set_bounds(&mut self, bounds: Rect) {
        self.base.set_bounds(bounds);
    }
    
    fn render(&self, canvas: &mut dyn Canvas) -> Result<()> {
        // Custom rendering logic
        canvas.draw_text(
            &self.custom_data,
            Point::new(10.0, 10.0),
            Color::BLACK
        )
    }
    
    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        // Custom event handling
        Ok(false) // Event not consumed
    }
    
    // ... implement other required methods
}
```

## Event Handling

### Event Types

```rust
use cx::{Event, MouseButton, Key, KeyModifiers};

match event {
    Event::MousePressed { window_id, button, x, y } => {
        println!("Mouse pressed: {:?} at ({}, {})", button, x, y);
    }
    Event::KeyPressed { window_id, key, modifiers } => {
        println!("Key pressed: {:?} with modifiers: {:?}", key, modifiers);
    }
    Event::WindowResized { window_id, width, height } => {
        println!("Window resized: {}x{}", width, height);
    }
    _ => {}
}
```

### Custom Event Handlers

```rust
use cx::{EventHandler, Event, Result};

struct MyEventHandler;

impl EventHandler for MyEventHandler {
    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        match event {
            Event::MousePressed { .. } => {
                println!("Mouse clicked!");
                Ok(true) // Event consumed
            }
            _ => Ok(false) // Event not handled
        }
    }
}
```

## Layout Management

### Flex Layout

```rust
use cx::{Container, FlexLayout, FlexDirection, JustifyContent, AlignItems};

let mut container = Container::new(1);
container.set_layout(
    FlexLayout::new()
        .direction(FlexDirection::Row)
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::Center)
        .gap(10.0)
);
```

### Grid Layout

```rust
use cx::{Container, GridLayout};

let mut container = Container::new(1);
container.set_layout(
    GridLayout::new()
        .columns(3)
        .rows(2)
        .gap(5.0, 5.0)
);
```

## Drawing and Graphics

### Basic Drawing

```rust
use cx::{Color, Point, Size, Rect};

// Get canvas from window
let mut canvas = window.canvas()?;

// Clear background
canvas.clear(Color::WHITE)?;

// Draw shapes
canvas.fill_rect(
    Rect::new(10.0, 10.0, 100.0, 50.0),
    Color::BLUE
)?;

canvas.stroke_rect(
    Rect::new(120.0, 10.0, 100.0, 50.0),
    Color::RED,
    2.0
)?;

// Draw text
canvas.draw_text(
    "Hello, Graphics!",
    Point::new(10.0, 80.0),
    Color::BLACK
)?;

// Present the frame
canvas.present()?;
```

### Colors

```rust
use cx::Color;

// Predefined colors
let red = Color::RED;
let green = Color::GREEN;
let blue = Color::BLUE;

// Custom colors
let purple = Color::rgb(0.5, 0.0, 0.5);
let transparent_blue = Color::rgba(0.0, 0.0, 1.0, 0.5);

// Color from hex
let orange = Color::from_hex("#FF8800");
```

## Examples

The framework includes several example applications:

### Basic Window
```bash
cargo run --example basic_window
```

### Button Demo
```bash
cargo run --example button_demo
```

### Custom Drawing
```bash
cargo run --example custom_drawing
```

## Best Practices

### Performance

1. **Minimize Redraws**: Only call `canvas.present()` when necessary
2. **Batch Operations**: Group drawing operations together
3. **Efficient Layouts**: Use appropriate layout managers for your use case

### Error Handling

```rust
use cx::{Result, Error};

fn create_ui() -> Result<()> {
    let app = Application::new()
        .map_err(|e| Error::framework("Failed to create application"))?;
        
    // ... rest of UI creation
    
    Ok(())
}
```

### Memory Management

- Widgets are automatically managed when added to containers
- Use `Arc<Mutex<T>>` for shared state between widgets
- Avoid circular references between widgets

## Troubleshooting

### Common Issues

**"Failed to create window"**
- Ensure display server is running (Linux X11)
- Check window manager compatibility

**"Missing libraries"**
- Install platform-specific development libraries
- See [Platform Requirements](#platform-requirements)

**"Permission denied"**
- Check file system permissions
- Run with appropriate user privileges

### Debug Mode

Enable debug logging:

```rust
use cx::Application;

let app = Application::new()?
    .with_debug(true);
```

## Next Steps

- Read the [API Documentation](https://docs.rs/cx-framework)
- Check out more [Examples](../examples/)
- Join our [Community](https://github.com/cx-framework/community)
- Report issues on [GitHub](https://github.com/cx-framework/cx-framework/issues)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

CX Framework is licensed under the MIT License. See [LICENSE](../LICENSE) for details.