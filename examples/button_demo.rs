//! Button demo example for the RX Framework.
//! 
//! This example demonstrates the widget system with buttons and event handling.

use rx::{
    Application, WindowBuilder, Color, Result, Rect, Size,
    widgets::{WidgetManager, Button, Label, Container, LayoutDirection, Alignment, Padding},
    layout::{FlexLayout, FlexDirection, JustifyContent, AlignItems},
};

fn main() -> Result<()> {
    println!("RX Framework - Button Demo Example");
    
    // Create the application
    let app = Application::new()?;
    
    // Create a window
    let mut window = WindowBuilder::new()
        .title("RX Framework - Button Demo")
        .size(600, 400)
        .position(300, 200)
        .build(&app)?;
    
    // Show the window
    window.show()?;
    
    // Create widget manager
    let mut widget_manager = WidgetManager::new();
    
    // Create main container
    let main_container = Container::new(widget_manager.next_id());
    let mut main_container = Box::new(main_container);
    main_container.set_bounds(Rect::new(0.0, 0.0, 600.0, 400.0));
    main_container.set_layout_direction(LayoutDirection::Vertical);
    main_container.set_main_axis_alignment(Alignment::Center);
    main_container.set_cross_axis_alignment(Alignment::Center);
    main_container.set_padding(Padding::uniform(20.0));
    main_container.set_spacing(10.0);
    main_container.set_background_color(Some(Color::rgb(0.95, 0.95, 0.95)));
    
    // Create title label
    let mut title_label = Label::new(widget_manager.next_id(), "CX Framework Button Demo".to_string());
    title_label.set_bounds(Rect::new(0.0, 0.0, 400.0, 40.0));
    title_label.set_text_color(Color::rgb(0.2, 0.2, 0.2));
    title_label.set_font_size(24.0);
    title_label.set_text_align(cx::widgets::label::TextAlign::Center);
    
    // Create buttons
    let mut button1 = Button::new(widget_manager.next_id(), "Click Me".to_string());
    button1.set_bounds(Rect::new(0.0, 0.0, 120.0, 40.0));
    button1.set_colors(
        Color::rgb(0.3, 0.6, 0.9),  // normal
        Color::rgb(0.4, 0.7, 1.0),  // hover
        Color::rgb(0.2, 0.5, 0.8),  // pressed
        Color::rgb(0.5, 0.5, 0.5),  // disabled
    );
    button1.set_on_click(|| {
        println!("Button 1 clicked!");
        Ok(())
    });
    
    let mut button2 = Button::new(widget_manager.next_id(), "Another Button".to_string());
    button2.set_bounds(Rect::new(0.0, 0.0, 150.0, 40.0));
    button2.set_colors(
        Color::rgb(0.9, 0.4, 0.3),  // normal
        Color::rgb(1.0, 0.5, 0.4),  // hover
        Color::rgb(0.8, 0.3, 0.2),  // pressed
        Color::rgb(0.5, 0.5, 0.5),  // disabled
    );
    button2.set_on_click(|| {
        println!("Button 2 clicked!");
        Ok(())
    });
    
    let mut button3 = Button::new(widget_manager.next_id(), "Disabled".to_string());
    button3.set_bounds(Rect::new(0.0, 0.0, 100.0, 40.0));
    button3.set_enabled(false);
    
    // Create status label
    let mut status_label = Label::new(widget_manager.next_id(), "Ready - Click buttons to test interaction".to_string());
    status_label.set_bounds(Rect::new(0.0, 0.0, 400.0, 30.0));
    status_label.set_text_color(Color::rgb(0.4, 0.4, 0.4));
    status_label.set_font_size(14.0);
    status_label.set_text_align(cx::widgets::label::TextAlign::Center);
    
    // Add widgets to container
    main_container.add_child(Box::new(title_label));
    main_container.add_child(Box::new(button1));
    main_container.add_child(Box::new(button2));
    main_container.add_child(Box::new(button3));
    main_container.add_child(Box::new(status_label));
    
    // Add main container to widget manager
    widget_manager.add_widget(main_container);
    
    println!("Widgets created successfully!");
    println!("Running application loop...");
    
    // Main application loop
    let mut running = true;
    while running {
        // Process events
        let events = app.event_loop.poll_events()?;
        
        for event in &events {
            match event {
                cx::Event::Quit => {
                    running = false;
                    break;
                }
                cx::Event::WindowClosed { .. } => {
                    running = false;
                    break;
                }
                _ => {
                    // Forward event to widget manager
                    widget_manager.handle_event(event)?;
                }
            }
        }
        
        // Update widgets
        let delta_time = std::time::Duration::from_millis(16); // ~60 FPS
        widget_manager.update(delta_time)?;
        
        // Render
        let mut canvas = window.canvas()?;
        canvas.clear(Color::WHITE)?;
        widget_manager.render(&mut canvas)?;
        canvas.present()?;
        
        // Simple frame rate limiting
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    
    println!("Application closing...");
    Ok(())
}