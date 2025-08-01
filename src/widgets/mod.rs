//! Widget system for the CX Framework.
//! 
//! This module provides a comprehensive widget system with common UI controls
//! and a flexible architecture for creating custom widgets.

use crate::{Result, Event, Canvas, Rect, Color, Point, Size};
use std::any::Any;

pub mod button;
pub mod text_input;
pub mod label;
pub mod container;

pub use button::Button;
pub use text_input::TextInput;
pub use label::Label;
pub use container::Container;

/// Unique identifier for widgets.
pub type WidgetId = u64;

/// Widget trait that all UI controls must implement.
pub trait Widget: Send + Sync {
    /// Get the widget's unique ID.
    fn id(&self) -> WidgetId;
    
    /// Get the widget's bounding rectangle.
    fn bounds(&self) -> Rect;
    
    /// Set the widget's position and size.
    fn set_bounds(&mut self, bounds: Rect);
    
    /// Get the widget's preferred size.
    fn preferred_size(&self) -> Size;
    
    /// Check if the widget is visible.
    fn is_visible(&self) -> bool;
    
    /// Set the widget's visibility.
    fn set_visible(&mut self, visible: bool);
    
    /// Check if the widget is enabled.
    fn is_enabled(&self) -> bool;
    
    /// Set the widget's enabled state.
    fn set_enabled(&mut self, enabled: bool);
    
    /// Handle an event and return whether it was consumed.
    fn handle_event(&mut self, event: &Event) -> Result<bool>;
    
    /// Update the widget (called every frame).
    fn update(&mut self, delta_time: std::time::Duration) -> Result<()>;
    
    /// Render the widget to the canvas.
    fn render(&self, canvas: &mut Canvas) -> Result<()>;
    
    /// Get the widget as Any for downcasting.
    fn as_any(&self) -> &dyn Any;
    
    /// Get the widget as mutable Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// Get the widget's type name.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Base widget implementation that provides common functionality.
#[derive(Debug, Clone)]
pub struct BaseWidget {
    id: WidgetId,
    bounds: Rect,
    visible: bool,
    enabled: bool,
    focused: bool,
    background_color: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
}

impl BaseWidget {
    /// Create a new base widget with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::zero(),
            visible: true,
            enabled: true,
            focused: false,
            background_color: None,
            border_color: None,
            border_width: 0.0,
        }
    }
    
    /// Get the widget ID.
    pub fn id(&self) -> WidgetId {
        self.id
    }
    
    /// Get the bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }
    
    /// Set the bounds.
    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }
    
    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }
    
    /// Set focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    
    /// Set the background color.
    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }
    
    /// Get the background color.
    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }
    
    /// Set the border color and width.
    pub fn set_border(&mut self, color: Option<Color>, width: f32) {
        self.border_color = color;
        self.border_width = width;
    }
    
    /// Get the border color.
    pub fn border_color(&self) -> Option<Color> {
        self.border_color
    }
    
    /// Get the border width.
    pub fn border_width(&self) -> f32 {
        self.border_width
    }
    
    /// Check if a point is inside the widget.
    pub fn contains_point(&self, point: Point) -> bool {
        self.bounds.contains_point(point)
    }
    
    /// Render the base widget (background and border).
    pub fn render_base(&self, canvas: &mut Canvas) -> Result<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Draw background
        if let Some(bg_color) = self.background_color {
            canvas.fill_rect(self.bounds, bg_color)?;
        }
        
        // Draw border
        if let Some(border_color) = self.border_color {
            if self.border_width > 0.0 {
                canvas.stroke_rect(self.bounds, border_color, self.border_width)?;
            }
        }
        
        Ok(())
    }
}

/// Widget manager for handling collections of widgets.
pub struct WidgetManager {
    widgets: Vec<Box<dyn Widget>>,
    next_id: WidgetId,
    focused_widget: Option<WidgetId>,
    hovered_widget: Option<WidgetId>,
}

impl WidgetManager {
    /// Create a new widget manager.
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            next_id: 1,
            focused_widget: None,
            hovered_widget: None,
        }
    }
    
    /// Generate a new unique widget ID.
    pub fn next_id(&mut self) -> WidgetId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Add a widget to the manager.
    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }
    
    /// Remove a widget by ID.
    pub fn remove_widget(&mut self, widget_id: WidgetId) -> Option<Box<dyn Widget>> {
        if let Some(index) = self.widgets.iter().position(|w| w.id() == widget_id) {
            Some(self.widgets.remove(index))
        } else {
            None
        }
    }
    
    /// Get a widget by ID.
    pub fn get_widget(&self, widget_id: WidgetId) -> Option<&dyn Widget> {
        self.widgets.iter().find(|w| w.id() == widget_id).map(|w| w.as_ref())
    }
    
    /// Get a mutable widget by ID.
    pub fn get_widget_mut(&mut self, widget_id: WidgetId) -> Option<&mut Box<dyn Widget>> {
        self.widgets.iter_mut().find(|w| w.id() == widget_id)
    }

    /// Execute a closure with a mutable reference to a widget.
    pub fn with_widget_mut<F, R>(&mut self, widget_id: WidgetId, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget) -> R,
    {
        self.widgets
            .iter_mut()
            .find(|w| w.id() == widget_id)
            .map(|widget| f(widget.as_mut()))
    }
    
    /// Get all widgets.
    pub fn widgets(&self) -> &[Box<dyn Widget>] {
        &self.widgets
    }
    
    /// Get the number of widgets.
    pub fn widget_count(&self) -> usize {
        self.widgets.len()
    }
    
    /// Handle an event and dispatch it to the appropriate widgets.
    pub fn handle_event(&mut self, event: &Event) -> Result<bool> {
        // Handle mouse events for focus and hover
        match event {
            Event::MousePressed { window_id: _, button: _, x, y } => {
                let point = Point::new(*x as f32, *y as f32);
                self.update_focus_and_hover(point)?;
            }
            Event::MouseMoved { window_id: _, x, y } => {
                let point = Point::new(*x as f32, *y as f32);
                self.update_hover(point)?;
            }
            _ => {}
        }
        
        // First, try to handle the event with the focused widget
        if let Some(focused_id) = self.focused_widget {
            if let Some(result) = self.with_widget_mut(focused_id, |widget| {
                widget.handle_event(event)
            }) {
                if let Ok(consumed) = result {
                    if consumed {
                        return Ok(true); // Event consumed
                    }
                }
            }
        }
        
        // Then, try all other widgets in reverse order (top to bottom)
        for widget in self.widgets.iter_mut().rev() {
            if Some(widget.id()) != self.focused_widget {
                if widget.handle_event(event)? {
                    return Ok(true); // Event consumed
                }
            }
        }
        
        Ok(false) // Event not consumed
    }
    
    /// Update all widgets.
    pub fn update(&mut self, delta_time: std::time::Duration) -> Result<()> {
        for widget in &mut self.widgets {
            widget.update(delta_time)?;
        }
        Ok(())
    }
    
    /// Render all widgets.
    pub fn render(&self, canvas: &mut Canvas) -> Result<()> {
        for widget in &self.widgets {
            if widget.is_visible() {
                widget.render(canvas)?;
            }
        }
        Ok(())
    }
    
    /// Clear all widgets.
    pub fn clear(&mut self) {
        self.widgets.clear();
        self.focused_widget = None;
        self.hovered_widget = None;
    }
    
    /// Set the focused widget.
    pub fn set_focused_widget(&mut self, widget_id: Option<WidgetId>) {
        // Unfocus the previously focused widget
        if let Some(old_focused) = self.focused_widget {
            self.with_widget_mut(old_focused, |_widget| {
                // Set focus state if the widget supports it
                // This would require extending the Widget trait or using downcasting
                // For now, we just acknowledge the focus change
            });
        }
        
        self.focused_widget = widget_id;
        
        // Focus the new widget
        if let Some(new_focused) = widget_id {
            self.with_widget_mut(new_focused, |_widget| {
                // Set focus state if the widget supports it
                // This would require extending the Widget trait or using downcasting
                // For now, we just acknowledge the focus change
            });
        }
    }
    
    /// Get the focused widget ID.
    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.focused_widget
    }
    
    /// Get the hovered widget ID.
    pub fn hovered_widget(&self) -> Option<WidgetId> {
        self.hovered_widget
    }
    
    /// Find the widget at the given point (topmost widget).
    pub fn widget_at_point(&self, point: Point) -> Option<WidgetId> {
        // Search in reverse order to get the topmost widget
        for widget in self.widgets.iter().rev() {
            if widget.is_visible() && widget.bounds().contains_point(point) {
                return Some(widget.id());
            }
        }
        None
    }
    
    fn update_focus_and_hover(&mut self, point: Point) -> Result<()> {
        let widget_at_point = self.widget_at_point(point);
        
        // Update focus
        self.set_focused_widget(widget_at_point);
        
        // Update hover
        self.hovered_widget = widget_at_point;
        
        Ok(())
    }
    
    fn update_hover(&mut self, point: Point) -> Result<()> {
        self.hovered_widget = self.widget_at_point(point);
        Ok(())
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget builder trait for creating widgets with fluent API.
pub trait WidgetBuilder<T: Widget> {
    /// Build the widget.
    fn build(self, manager: &mut WidgetManager) -> T;
}

/// Macro for creating widget builders.
#[macro_export]
macro_rules! widget_builder {
    ($builder:ident, $widget:ident) => {
        impl $builder {
            pub fn id(mut self, id: WidgetId) -> Self {
                self.id = Some(id);
                self
            }
            
            pub fn bounds(mut self, bounds: Rect) -> Self {
                self.bounds = Some(bounds);
                self
            }
            
            pub fn position(mut self, x: f32, y: f32) -> Self {
                if let Some(ref mut bounds) = self.bounds {
                    bounds.x = x;
                    bounds.y = y;
                } else {
                    self.bounds = Some(Rect::new(x, y, 100.0, 30.0)); // Default size
                }
                self
            }
            
            pub fn size(mut self, width: f32, height: f32) -> Self {
                if let Some(ref mut bounds) = self.bounds {
                    bounds.width = width;
                    bounds.height = height;
                } else {
                    self.bounds = Some(Rect::new(0.0, 0.0, width, height));
                }
                self
            }
            
            pub fn visible(mut self, visible: bool) -> Self {
                self.visible = visible;
                self
            }
            
            pub fn enabled(mut self, enabled: bool) -> Self {
                self.enabled = enabled;
                self
            }
            
            pub fn background_color(mut self, color: Color) -> Self {
                self.background_color = Some(color);
                self
            }
            
            pub fn border(mut self, color: Color, width: f32) -> Self {
                self.border_color = Some(color);
                self.border_width = width;
                self
            }
        }
        
        impl $builder {
            pub fn build(self, manager: &mut WidgetManager) -> $widget {
                let id = self.id.unwrap_or_else(|| manager.next_id());
                let bounds = self.bounds.unwrap_or_else(|| Rect::new(0.0, 0.0, 100.0, 30.0));
                
                let mut base = BaseWidget::new(id);
                base.set_bounds(bounds);
                base.set_visible(self.visible);
                base.set_enabled(self.enabled);
                base.set_background_color(self.background_color);
                base.set_border(self.border_color, self.border_width);
                
                $widget::from_base(base, self)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_widget() {
        let mut widget = BaseWidget::new(1);
        assert_eq!(widget.id(), 1);
        assert!(widget.is_visible());
        assert!(widget.is_enabled());
        assert!(!widget.is_focused());
        
        widget.set_visible(false);
        assert!(!widget.is_visible());
        
        widget.set_bounds(Rect::new(10.0, 20.0, 100.0, 50.0));
        assert_eq!(widget.bounds(), Rect::new(10.0, 20.0, 100.0, 50.0));
    }

    #[test]
    fn test_widget_manager() {
        let mut manager = WidgetManager::new();
        assert_eq!(manager.widget_count(), 0);
        
        let id = manager.next_id();
        assert_eq!(id, 1);
        
        let next_id = manager.next_id();
        assert_eq!(next_id, 2);
    }
}