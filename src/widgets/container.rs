//! Container widget implementation for layout management.

use super::{Widget, BaseWidget, WidgetId, WidgetManager};
use crate::{Result, Event, Canvas, Rect, Color, Size};
use std::any::Any;
use std::time::Duration;

/// Layout direction for containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    /// Arrange children horizontally (left to right).
    Horizontal,
    /// Arrange children vertically (top to bottom).
    Vertical,
    /// No automatic layout - children positioned manually.
    None,
}

impl Default for LayoutDirection {
    fn default() -> Self {
        LayoutDirection::None
    }
}

/// Alignment options for container children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Align to start (left/top).
    Start,
    /// Center alignment.
    Center,
    /// Align to end (right/bottom).
    End,
    /// Stretch to fill available space.
    Stretch,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Start
    }
}

/// Padding around container content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Padding {
    /// Create uniform padding.
    pub fn uniform(padding: f32) -> Self {
        Self {
            left: padding,
            top: padding,
            right: padding,
            bottom: padding,
        }
    }
    
    /// Create symmetric padding (horizontal, vertical).
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
    
    /// Create padding with individual values.
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { left, top, right, bottom }
    }
    
    /// Get total horizontal padding.
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    
    /// Get total vertical padding.
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::uniform(0.0)
    }
}

/// Container widget for managing child widgets.
pub struct Container {
    base: BaseWidget,
    children: Vec<Box<dyn Widget>>,
    layout_direction: LayoutDirection,
    main_axis_alignment: Alignment,
    cross_axis_alignment: Alignment,
    padding: Padding,
    spacing: f32,
    clip_children: bool,
}

impl Container {
    /// Create a new container widget.
    pub fn new(id: WidgetId) -> Self {
        let base = BaseWidget::new(id);
        
        Self {
            base,
            children: Vec::new(),
            layout_direction: LayoutDirection::None,
            main_axis_alignment: Alignment::Start,
            cross_axis_alignment: Alignment::Start,
            padding: Padding::default(),
            spacing: 0.0,
            clip_children: false,
        }
    }
    
    /// Create a container from a base widget and builder.
    pub fn from_base(base: BaseWidget, builder: ContainerBuilder) -> Self {
        Self {
            base,
            children: builder.children,
            layout_direction: builder.layout_direction,
            main_axis_alignment: builder.main_axis_alignment,
            cross_axis_alignment: builder.cross_axis_alignment,
            padding: builder.padding,
            spacing: builder.spacing,
            clip_children: builder.clip_children,
        }
    }
    
    /// Add a child widget.
    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.children.push(child);
        self.layout_children();
    }
    
    /// Remove a child widget by ID.
    pub fn remove_child(&mut self, child_id: WidgetId) -> Option<Box<dyn Widget>> {
        if let Some(index) = self.children.iter().position(|child| child.id() == child_id) {
            let child = self.children.remove(index);
            self.layout_children();
            Some(child)
        } else {
            None
        }
    }
    
    /// Get a child widget by ID.
    pub fn get_child(&self, child_id: WidgetId) -> Option<&dyn Widget> {
        self.children.iter().find(|child| child.id() == child_id).map(|child| child.as_ref())
    }
    
    /// Get a mutable child widget by ID.
    pub fn get_child_mut(&mut self, child_id: WidgetId) -> Option<&mut Box<dyn Widget>> {
        self.children.iter_mut().find(|child| child.id() == child_id)
    }

    /// Execute a closure with a mutable reference to a child widget.
    pub fn with_child_mut<F, R>(&mut self, child_id: WidgetId, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Widget) -> R,
    {
        self.children
            .iter_mut()
            .find(|child| child.id() == child_id)
            .map(|child| f(child.as_mut()))
    }
    
    /// Get all children.
    pub fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }
    
    /// Get the number of children.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
    
    /// Clear all children.
    pub fn clear_children(&mut self) {
        self.children.clear();
    }
    
    /// Set the layout direction.
    pub fn set_layout_direction(&mut self, direction: LayoutDirection) {
        self.layout_direction = direction;
        self.layout_children();
    }
    
    /// Get the layout direction.
    pub fn layout_direction(&self) -> LayoutDirection {
        self.layout_direction
    }
    
    /// Set the main axis alignment.
    pub fn set_main_axis_alignment(&mut self, alignment: Alignment) {
        self.main_axis_alignment = alignment;
        self.layout_children();
    }
    
    /// Set the cross axis alignment.
    pub fn set_cross_axis_alignment(&mut self, alignment: Alignment) {
        self.cross_axis_alignment = alignment;
        self.layout_children();
    }
    
    /// Set the padding.
    pub fn set_padding(&mut self, padding: Padding) {
        self.padding = padding;
        self.layout_children();
    }
    
    /// Get the padding.
    pub fn padding(&self) -> Padding {
        self.padding
    }
    
    /// Set the spacing between children.
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing.max(0.0);
        self.layout_children();
    }
    
    /// Get the spacing.
    pub fn spacing(&self) -> f32 {
        self.spacing
    }
    
    /// Set whether to clip children to container bounds.
    pub fn set_clip_children(&mut self, clip: bool) {
        self.clip_children = clip;
    }
    
    /// Check if children are clipped.
    pub fn clip_children(&self) -> bool {
        self.clip_children
    }
    
    /// Get the content area (bounds minus padding).
    pub fn content_area(&self) -> Rect {
        let bounds = self.bounds();
        Rect::new(
            bounds.x + self.padding.left,
            bounds.y + self.padding.top,
            bounds.width - self.padding.horizontal(),
            bounds.height - self.padding.vertical(),
        )
    }
    
    /// Layout all children according to the current layout settings.
    fn layout_children(&mut self) {
        if self.children.is_empty() {
            return;
        }
        
        match self.layout_direction {
            LayoutDirection::None => {
                // No automatic layout - children positioned manually
            }
            LayoutDirection::Horizontal => {
                self.layout_horizontal();
            }
            LayoutDirection::Vertical => {
                self.layout_vertical();
            }
        }
    }
    
    /// Layout children horizontally.
    fn layout_horizontal(&mut self) {
        let content_area = self.content_area();
        let total_spacing = self.spacing * (self.children.len() - 1).max(0usize) as f32;
        
        // Calculate preferred sizes
        let mut child_sizes = Vec::new();
        let mut total_preferred_width = 0.0;
        let mut max_preferred_height: f32 = 0.0;
        
        for child in &self.children {
            let preferred = child.preferred_size();
            child_sizes.push(preferred);
            total_preferred_width += preferred.width;
            max_preferred_height = max_preferred_height.max(preferred.height);
        }
        
        // Calculate available space
        let available_width = content_area.width - total_spacing;
        let available_height = content_area.height;
        
        // Distribute space among children
        let child_widths: Vec<f32> = if total_preferred_width <= available_width {
            // Use preferred sizes
            child_sizes.iter().map(|size| size.width).collect()
        } else {
            // Scale down proportionally
            let scale = available_width / total_preferred_width;
            child_sizes.iter().map(|size| size.width * scale).collect()
        };
        
        // Position children
        let mut current_x = content_area.x;
        match self.main_axis_alignment {
            Alignment::Start => {
                // Already positioned at start
            }
            Alignment::Center => {
                let used_width: f32 = child_widths.iter().sum::<f32>() + total_spacing;
                current_x += (available_width - used_width) / 2.0;
            }
            Alignment::End => {
                let used_width: f32 = child_widths.iter().sum::<f32>() + total_spacing;
                current_x += available_width - used_width;
            }
            Alignment::Stretch => {
                // Children already sized to fill available space
            }
        }
        
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_width = child_widths[i];
            let child_height = match self.cross_axis_alignment {
                Alignment::Stretch => available_height,
                _ => child_sizes[i].height.min(available_height),
            };
            
            let child_y = match self.cross_axis_alignment {
                Alignment::Start => content_area.y,
                Alignment::Center => content_area.y + (available_height - child_height) / 2.0,
                Alignment::End => content_area.y + available_height - child_height,
                Alignment::Stretch => content_area.y,
            };
            
            child.set_bounds(Rect::new(current_x, child_y, child_width, child_height));
            current_x += child_width + self.spacing;
        }
    }
    
    /// Layout children vertically.
    fn layout_vertical(&mut self) {
        let content_area = self.content_area();
        let total_spacing = self.spacing * (self.children.len() - 1).max(0usize) as f32;
        
        // Calculate preferred sizes
        let mut child_sizes = Vec::new();
        let mut total_preferred_height = 0.0;
        let mut max_preferred_width: f32 = 0.0;
        
        for child in &self.children {
            let preferred = child.preferred_size();
            child_sizes.push(preferred);
            total_preferred_height += preferred.height;
            max_preferred_width = max_preferred_width.max(preferred.width);
        }
        
        // Calculate available space
        let available_width = content_area.width;
        let available_height = content_area.height - total_spacing;
        
        // Distribute space among children
        let child_heights: Vec<f32> = if total_preferred_height <= available_height {
            // Use preferred sizes
            child_sizes.iter().map(|size| size.height).collect()
        } else {
            // Scale down proportionally
            let scale = available_height / total_preferred_height;
            child_sizes.iter().map(|size| size.height * scale).collect()
        };
        
        // Position children
        let mut current_y = content_area.y;
        match self.main_axis_alignment {
            Alignment::Start => {
                // Already positioned at start
            }
            Alignment::Center => {
                let used_height: f32 = child_heights.iter().sum::<f32>() + total_spacing;
                current_y += (available_height - used_height) / 2.0;
            }
            Alignment::End => {
                let used_height: f32 = child_heights.iter().sum::<f32>() + total_spacing;
                current_y += available_height - used_height;
            }
            Alignment::Stretch => {
                // Children already sized to fill available space
            }
        }
        
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_height = child_heights[i];
            let child_width = match self.cross_axis_alignment {
                Alignment::Stretch => available_width,
                _ => child_sizes[i].width.min(available_width),
            };
            
            let child_x = match self.cross_axis_alignment {
                Alignment::Start => content_area.x,
                Alignment::Center => content_area.x + (available_width - child_width) / 2.0,
                Alignment::End => content_area.x + available_width - child_width,
                Alignment::Stretch => content_area.x,
            };
            
            child.set_bounds(Rect::new(child_x, current_y, child_width, child_height));
            current_y += child_height + self.spacing;
        }
    }
}

impl Widget for Container {
    fn id(&self) -> WidgetId {
        self.base.id()
    }
    
    fn bounds(&self) -> Rect {
        self.base.bounds()
    }
    
    fn set_bounds(&mut self, bounds: Rect) {
        self.base.set_bounds(bounds);
        self.layout_children();
    }
    
    fn preferred_size(&self) -> Size {
        if self.children.is_empty() {
            return Size::new(100.0, 100.0); // Default size
        }
        
        match self.layout_direction {
            LayoutDirection::None => {
                // Calculate bounds of all children
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                
                for child in &self.children {
                    let bounds = child.bounds();
                    min_x = min_x.min(bounds.x);
                    min_y = min_y.min(bounds.y);
                    max_x = max_x.max(bounds.right());
                    max_y = max_y.max(bounds.bottom());
                }
                
                Size::new(
                    (max_x - min_x) + self.padding.horizontal(),
                    (max_y - min_y) + self.padding.vertical(),
                )
            }
            LayoutDirection::Horizontal => {
                let mut total_width = 0.0;
                let mut max_height: f32 = 0.0;
                
                for child in &self.children {
                    let preferred = child.preferred_size();
                    total_width += preferred.width;
                    max_height = max_height.max(preferred.height);
                }
                
                let spacing = self.spacing * (self.children.len() - 1).max(0usize) as f32;
                Size::new(
                    total_width + spacing + self.padding.horizontal(),
                    max_height + self.padding.vertical(),
                )
            }
            LayoutDirection::Vertical => {
                let mut max_width: f32 = 0.0;
                let mut total_height = 0.0;
                
                for child in &self.children {
                    let preferred = child.preferred_size();
                    max_width = max_width.max(preferred.width);
                    total_height += preferred.height;
                }
                
                let spacing = self.spacing * (self.children.len() - 1).max(0usize) as f32;
                Size::new(
                    max_width + self.padding.horizontal(),
                    total_height + spacing + self.padding.vertical(),
                )
            }
        }
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
        
        // Forward events to children (in reverse order for proper hit testing)
        for child in self.children.iter_mut().rev() {
            if child.handle_event(event)? {
                return Ok(true); // Event was consumed by child
            }
        }
        
        Ok(false)
    }
    
    fn update(&mut self, delta_time: Duration) -> Result<()> {
        // Update all children
        for child in &mut self.children {
            child.update(delta_time)?;
        }
        
        Ok(())
    }
    
    fn render(&self, canvas: &mut Canvas) -> Result<()> {
        if !self.is_visible() {
            return Ok(());
        }
        
        // Render base (background and border)
        self.base.render_base(canvas)?;
        
        // Set up clipping if enabled
        if self.clip_children {
            // TODO: Implement clipping in canvas
            // For now, we'll just render without clipping
        }
        
        // Render all children
        for child in &self.children {
            if child.is_visible() {
                child.render(canvas)?;
            }
        }
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Builder for creating container widgets.
pub struct ContainerBuilder {
    id: Option<WidgetId>,
    bounds: Option<Rect>,
    visible: bool,
    enabled: bool,
    background_color: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
    children: Vec<Box<dyn Widget>>,
    layout_direction: LayoutDirection,
    main_axis_alignment: Alignment,
    cross_axis_alignment: Alignment,
    padding: Padding,
    spacing: f32,
    clip_children: bool,
}

impl ContainerBuilder {
    /// Create a new container builder.
    pub fn new() -> Self {
        Self {
            id: None,
            bounds: None,
            visible: true,
            enabled: true,
            background_color: None,
            border_color: None,
            border_width: 0.0,
            children: Vec::new(),
            layout_direction: LayoutDirection::None,
            main_axis_alignment: Alignment::Start,
            cross_axis_alignment: Alignment::Start,
            padding: Padding::default(),
            spacing: 0.0,
            clip_children: false,
        }
    }
    
    /// Add a child widget.
    pub fn child(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }
    
    /// Add multiple child widgets.
    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children.extend(children);
        self
    }
    
    /// Set the layout direction.
    pub fn layout_direction(mut self, direction: LayoutDirection) -> Self {
        self.layout_direction = direction;
        self
    }
    
    /// Set main axis alignment.
    pub fn main_axis_alignment(mut self, alignment: Alignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }
    
    /// Set cross axis alignment.
    pub fn cross_axis_alignment(mut self, alignment: Alignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }
    
    /// Set padding.
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
    
    /// Set uniform padding.
    pub fn padding_uniform(mut self, padding: f32) -> Self {
        self.padding = Padding::uniform(padding);
        self
    }
    
    /// Set spacing between children.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }
    
    /// Enable/disable child clipping.
    pub fn clip_children(mut self, clip: bool) -> Self {
        self.clip_children = clip;
        self
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Apply the widget_builder macro
crate::widget_builder!(ContainerBuilder, Container);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{WidgetManager, Button};

    #[test]
    fn test_container_creation() {
        let container = Container::new(1);
        assert_eq!(container.id(), 1);
        assert_eq!(container.child_count(), 0);
        assert_eq!(container.layout_direction(), LayoutDirection::None);
        assert_eq!(container.spacing(), 0.0);
        assert!(!container.clip_children());
    }

    #[test]
    fn test_padding() {
        let padding = Padding::uniform(10.0);
        assert_eq!(padding.left, 10.0);
        assert_eq!(padding.horizontal(), 20.0);
        assert_eq!(padding.vertical(), 20.0);
        
        let padding = Padding::symmetric(5.0, 10.0);
        assert_eq!(padding.left, 5.0);
        assert_eq!(padding.top, 10.0);
        assert_eq!(padding.horizontal(), 10.0);
        assert_eq!(padding.vertical(), 20.0);
    }

    #[test]
    fn test_container_content_area() {
        let mut container = Container::new(1);
        container.set_bounds(Rect::new(10.0, 20.0, 100.0, 80.0));
        container.set_padding(Padding::uniform(5.0));
        
        let content_area = container.content_area();
        assert_eq!(content_area.x, 15.0);
        assert_eq!(content_area.y, 25.0);
        assert_eq!(content_area.width, 90.0);
        assert_eq!(content_area.height, 70.0);
    }

    #[test]
    fn test_container_builder() {
        let mut manager = WidgetManager::new();
        
        let button1 = Box::new(Button::new(manager.next_id(), "Button 1".to_string()));
        let button2 = Box::new(Button::new(manager.next_id(), "Button 2".to_string()));
        
        let _container = ContainerBuilder::new()
            .layout_direction(LayoutDirection::Horizontal)
            .main_axis_alignment(Alignment::Center)
            .cross_axis_alignment(Alignment::Stretch)
            .padding_uniform(10.0)
            .spacing(5.0)
            .child(button1)
            .child(button2)
            .build(&mut manager);
        
        // Test would continue if we had a proper widget system
    }
}