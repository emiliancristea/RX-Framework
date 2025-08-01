//! Basic window example for the CX Framework.
//! 
//! This example demonstrates the fundamental concepts of CX Framework:
//! - Creating an Application instance
//! - Building and configuring a Window
//! - Using the Canvas for 2D drawing
//! - Running the main application loop
//! 
//! ## Features Demonstrated
//! - Window creation with custom title, size, and position
//! - Basic 2D drawing (rectangles, text)
//! - Color management
//! - Application lifecycle management
//! 
//! ## Usage
//! ```bash
//! cargo run --example basic_window
//! ```
//! 
//! This will create a window with a light blue background, a darker blue rectangle,
//! and white text demonstrating the basic drawing capabilities.

use rx::{Application, WindowBuilder, Color, Result};

fn main() -> Result<()> {
    println!("RX Framework - Basic Window Example");
    
    // Create the application
    let app = Application::new()?;
    
    // Create a window
    let mut window = WindowBuilder::new()
        .title("RX Framework - Basic Window")
        .size(800, 600)
        .position(200, 100)
        .build(&app)?;
    
    // Show the window
    window.show()?;
    
    // Get a canvas for drawing
    let mut canvas = window.canvas()?;
    
    // Clear the window with a light blue background
    canvas.clear(Color::rgb(0.9, 0.95, 1.0))?;
    
    // Draw a simple rectangle
    let rect = rx::Rect::new(100.0, 100.0, 200.0, 150.0);
    canvas.fill_rect(rect, Color::rgb(0.2, 0.5, 0.8))?;
    
    // Draw some text
    let text_pos = rx::Point::new(110.0, 180.0);
    canvas.draw_text("Hello, RX Framework!", text_pos, Color::WHITE)?;
    
    // Present the canvas
    canvas.present()?;
    
    println!("Window created successfully!");
    println!("Running application loop...");
    
    // Run the application loop
    app.run()
}