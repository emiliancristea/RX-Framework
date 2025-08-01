//! Test window example for the CX Framework.
//! 
//! This example tests window creation step by step.

use cx::{Application, WindowBuilder, Color, Result};

fn main() -> Result<()> {
    println!("CX Framework - Test Window Example");
    
    // Test 1: Create the application
    println!("Creating application...");
    let app = Application::new()?;
    println!("✓ Application created");
    
    // Test 2: Create a window
    println!("Creating window...");
    let mut window = WindowBuilder::new()
        .title("CX Framework - Test Window")
        .size(400, 300)
        .position(100, 100)
        .build(&app)?;
    println!("✓ Window created");
    
    // Test 3: Show the window
    println!("Showing window...");
    window.show()?;
    println!("✓ Window shown");
    
    // Test 4: Try to get canvas
    println!("Getting canvas...");
    let mut canvas = window.canvas()?;
    println!("✓ Canvas obtained");
    
    // Test 5: Try basic drawing
    println!("Drawing...");
    canvas.clear(Color::rgb(0.8, 0.9, 1.0))?;
    canvas.present()?;
    println!("✓ Drawing completed");
    
    println!("Press Ctrl+C to exit (running for 5 seconds)...");
    
    // Simple loop instead of app.run() to test
    for i in 0..50 {
        println!("Loop iteration {}", i);
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Try to run a single frame
        if let Ok(should_continue) = app.run_frame() {
            if !should_continue {
                println!("Application signaled to exit");
                break;
            }
        } else {
            println!("Error in run_frame");
            break;
        }
    }
    
    println!("Test completed!");
    Ok(())
}