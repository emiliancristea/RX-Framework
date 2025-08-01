//! Button widget implementation.

use super::{Widget, BaseWidget, WidgetId, WidgetManager};
use crate::{Result, Event, Canvas, Rect, Color, Point, Size};
use crate::platform::MouseButton;
use std::any::Any;
use std::time::Duration;

/// Button widget for user interaction.
pub struct Button {
    base: BaseWidget,
    text: String,
    text_color: Color,
    normal_color: Color,
    hover_color: Color,
    pressed_color: Color,
    disabled_color: Color,
    is_pressed: bool,
    is_hovered: bool,
    on_click: Option<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

impl Button {
    /// Create a new button with the given text.
    pub fn new(id: WidgetId, text: String) -> Self {
        let mut base = BaseWidget::new(id);
        base.set_background_color(Some(Color::LIGHT_GRAY));
        base.set_border(Some(Color::DARK_GRAY), 1.0);
        
        Self {
            base,
            text,
            text_color: Color::BLACK,
            normal_color: Color::LIGHT_GRAY,
            hover_color: Color::WHITE,
            pressed_color: Color::GRAY,
            disabled_color: Color::DARK_GRAY,
            is_pressed: false,
            is_hovered: false,
            on_click: None,
        }
    }
    
    /// Create a button from a base widget and builder.
    pub fn from_base(base: BaseWidget, builder: ButtonBuilder) -> Self {
        Self {
            base,
            text: builder.text,
            text_color: builder.text_color,
            normal_color: builder.normal_color,
            hover_color: builder.hover_color,
            pressed_color: builder.pressed_color,
            disabled_color: builder.disabled_color,
            is_pressed: false,
            is_hovered: false,
            on_click: builder.on_click,
        }
    }
    
    /// Set the button text.
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }
    
    /// Get the button text.
    pub fn text(&self) -> &str {
        &self.text
    }
    
    /// Set the text color.
    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = color;
    }
    
    /// Set the button colors for different states.
    pub fn set_colors(&mut self, normal: Color, hover: Color, pressed: Color, disabled: Color) {
        self.normal_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self.disabled_color = disabled;
    }
    
    /// Set the click callback.
    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
    }
    
    /// Check if the button is currently pressed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
    
    /// Check if the button is currently hovered.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }
    
    /// Get the current background color based on state.
    fn current_background_color(&self) -> Color {
        if !self.base.is_enabled() {
            self.disabled_color
        } else if self.is_pressed {
            self.pressed_color
        } else if self.is_hovered {
            self.hover_color
        } else {
            self.normal_color
        }
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.base.id()
    }
    
    fn bounds(&self) -> Rect {
        self.base.bounds()
    }
    
    fn set_bounds(&mut self, bounds: Rect) {
        self.base.set_bounds(bounds);
    }
    
    fn preferred_size(&self) -> Size {
        // Calculate preferred size based on text
        // For now, use a simple heuristic
        let text_width = self.text.len() as f32 * 8.0; // Assume ~8 pixels per character
        let text_height = 16.0; // Assume 16 pixel font height
        
        Size::new(
            text_width + 20.0, // Add padding
            text_height + 10.0, // Add padding
        )
    }
    
    fn is_visible(&self) -> bool {
        self.base.is_visible()
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.base.set_visible(visible);
    }
    
    fn is_enabled(&self) -> bool {
        self.base.is_enabled()
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.base.set_enabled(enabled);
    }
    
    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        if !self.is_visible() || !self.is_enabled() {
            return Ok(false);
        }
        
        match event {
            Event::MousePressed { window_id: _, button, x, y } => {
                let point = Point::new(*x as f32, *y as f32);
                if *button == MouseButton::Left && self.base.contains_point(point) {
                    self.is_pressed = true;
                    return Ok(true);
                }
            }
            
            Event::MouseReleased { window_id: _, button, x, y } => {
                let point = Point::new(*x as f32, *y as f32);
                if *button == MouseButton::Left {
                    if self.is_pressed && self.base.contains_point(point) {
                        // Button was clicked
                        if let Some(ref callback) = self.on_click {
                            callback()?;
                        }
                    }
                    self.is_pressed = false;
                    return Ok(true);
                }
            }
            
            Event::MouseMoved { window_id: _, x, y } => {
                let point = Point::new(*x as f32, *y as f32);
                let was_hovered = self.is_hovered;
                self.is_hovered = self.base.contains_point(point);
                
                // If hover state changed, we should redraw
                if was_hovered != self.is_hovered {
                    return Ok(true);
                }
            }
            
            Event::MouseLeft { window_id: _ } => {
                self.is_hovered = false;
                self.is_pressed = false;
            }
            
            _ => {}
        }
        
        Ok(false)
    }
    
    fn update(&mut self, _delta_time: Duration) -> Result<()> {
        // Button doesn't need frame-by-frame updates
        Ok(())
    }
    
    fn render(&self, canvas: &mut Canvas) -> Result<()> {
        if !self.is_visible() {
            return Ok(());
        }
        
        // Draw background with current state color
        canvas.fill_rect(self.bounds(), self.current_background_color())?;
        
        // Draw border
        if let Some(border_color) = self.base.border_color() {
            canvas.stroke_rect(self.bounds(), border_color, self.base.border_width())?;
        }
        
        // Draw text centered
        let bounds = self.bounds();
        let text_x = bounds.x + bounds.width / 2.0 - (self.text.len() as f32 * 4.0); // Rough centering
        let text_y = bounds.y + bounds.height / 2.0 + 8.0; // Rough vertical centering
        
        canvas.draw_text(&self.text, Point::new(text_x, text_y), self.text_color)?;
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Builder for creating buttons.
pub struct ButtonBuilder {
    id: Option<WidgetId>,
    bounds: Option<Rect>,
    visible: bool,
    enabled: bool,
    background_color: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
    text: String,
    text_color: Color,
    normal_color: Color,
    hover_color: Color,
    pressed_color: Color,
    disabled_color: Color,
    on_click: Option<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

impl ButtonBuilder {
    /// Create a new button builder.
    pub fn new() -> Self {
        Self {
            id: None,
            bounds: None,
            visible: true,
            enabled: true,
            background_color: None,
            border_color: None,
            border_width: 0.0,
            text: "Button".to_string(),
            text_color: Color::BLACK,
            normal_color: Color::LIGHT_GRAY,
            hover_color: Color::WHITE,
            pressed_color: Color::GRAY,
            disabled_color: Color::DARK_GRAY,
            on_click: None,
        }
    }
    
    /// Set the button text.
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = text.into();
        self
    }
    
    /// Set the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }
    
    /// Set the button colors for different states.
    pub fn colors(mut self, normal: Color, hover: Color, pressed: Color, disabled: Color) -> Self {
        self.normal_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self.disabled_color = disabled;
        self
    }
    
    /// Set the normal (default) color.
    pub fn normal_color(mut self, color: Color) -> Self {
        self.normal_color = color;
        self
    }
    
    /// Set the hover color.
    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }
    
    /// Set the pressed color.
    pub fn pressed_color(mut self, color: Color) -> Self {
        self.pressed_color = color;
        self
    }
    
    /// Set the disabled color.
    pub fn disabled_color(mut self, color: Color) -> Self {
        self.disabled_color = color;
        self
    }
    
    /// Set the click callback.
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Apply the widget_builder macro
crate::widget_builder!(ButtonBuilder, Button);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::WidgetManager;

    #[test]
    fn test_button_creation() {
        let button = Button::new(1, "Test Button".to_string());
        assert_eq!(button.id(), 1);
        assert_eq!(button.text(), "Test Button");
        assert!(button.is_enabled());
        assert!(button.is_visible());
        assert!(!button.is_pressed());
        assert!(!button.is_hovered());
    }

    #[test]
    fn test_button_builder() {
        let mut manager = WidgetManager::new();
        
        let _button = ButtonBuilder::new()
            .text("Click Me")
            .text_color(Color::BLUE)
            .normal_color(Color::WHITE)
            .hover_color(Color::LIGHT_GRAY)
            .size(100.0, 30.0)
            .build(&mut manager);
        
        // Test would continue if we had a proper widget system
    }

    #[test]
    fn test_button_preferred_size() {
        let button = Button::new(1, "Test".to_string());
        let size = button.preferred_size();
        
        // Should be text width + padding
        assert!(size.width > 20.0); // At least padding
        assert!(size.height > 10.0); // At least padding
    }
}