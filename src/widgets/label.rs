//! Label widget implementation for displaying text.

use super::{Widget, BaseWidget, WidgetId, WidgetManager};
use crate::{Result, Event, Canvas, Rect, Color, Point, Size};
use std::any::Any;
use std::time::Duration;

/// Text alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Left
    }
}

/// Vertical alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

impl Default for VerticalAlign {
    fn default() -> Self {
        VerticalAlign::Top
    }
}

/// Label widget for displaying text.
pub struct Label {
    base: BaseWidget,
    text: String,
    text_color: Color,
    font_size: f32,
    text_align: TextAlign,
    vertical_align: VerticalAlign,
    word_wrap: bool,
    multiline: bool,
}

impl Label {
    /// Create a new label with the given text.
    pub fn new(id: WidgetId, text: String) -> Self {
        let base = BaseWidget::new(id);
        
        Self {
            base,
            text,
            text_color: Color::BLACK,
            font_size: 14.0,
            text_align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            word_wrap: false,
            multiline: false,
        }
    }
    
    /// Create a label from a base widget and builder.
    pub fn from_base(base: BaseWidget, builder: LabelBuilder) -> Self {
        Self {
            base,
            text: builder.text,
            text_color: builder.text_color,
            font_size: builder.font_size,
            text_align: builder.text_align,
            vertical_align: builder.vertical_align,
            word_wrap: builder.word_wrap,
            multiline: builder.multiline,
        }
    }
    
    /// Set the label text.
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        self.text = text.into();
    }
    
    /// Get the label text.
    pub fn text(&self) -> &str {
        &self.text
    }
    
    /// Set the text color.
    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = color;
    }
    
    /// Get the text color.
    pub fn text_color(&self) -> Color {
        self.text_color
    }
    
    /// Set the font size.
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.max(1.0);
    }
    
    /// Get the font size.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
    
    /// Set the text alignment.
    pub fn set_text_align(&mut self, align: TextAlign) {
        self.text_align = align;
    }
    
    /// Get the text alignment.
    pub fn text_align(&self) -> TextAlign {
        self.text_align
    }
    
    /// Set the vertical alignment.
    pub fn set_vertical_align(&mut self, align: VerticalAlign) {
        self.vertical_align = align;
    }
    
    /// Get the vertical alignment.
    pub fn vertical_align(&self) -> VerticalAlign {
        self.vertical_align
    }
    
    /// Set word wrap.
    pub fn set_word_wrap(&mut self, wrap: bool) {
        self.word_wrap = wrap;
    }
    
    /// Check if word wrap is enabled.
    pub fn word_wrap(&self) -> bool {
        self.word_wrap
    }
    
    /// Set multiline support.
    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
    }
    
    /// Check if multiline is enabled.
    pub fn multiline(&self) -> bool {
        self.multiline
    }
    
    /// Calculate the text position based on alignment.
    fn calculate_text_position(&self, text_size: Size) -> Point {
        let bounds = self.bounds();
        
        let x = match self.text_align {
            TextAlign::Left => bounds.x + 2.0, // Small padding
            TextAlign::Center => bounds.x + (bounds.width - text_size.width) / 2.0,
            TextAlign::Right => bounds.x + bounds.width - text_size.width - 2.0, // Small padding
        };
        
        let y = match self.vertical_align {
            VerticalAlign::Top => bounds.y + self.font_size + 2.0, // Small padding + baseline
            VerticalAlign::Middle => bounds.y + (bounds.height + self.font_size) / 2.0,
            VerticalAlign::Bottom => bounds.y + bounds.height - 2.0, // Small padding
        };
        
        Point::new(x, y)
    }
    
    /// Estimate text size (rough approximation).
    fn estimate_text_size(&self, text: &str) -> Size {
        let char_width = self.font_size * 0.6; // Rough approximation
        let line_height = self.font_size * 1.2; // Line height with spacing
        
        if self.multiline {
            let lines: Vec<&str> = text.split('\n').collect();
            let max_line_width = lines.iter()
                .map(|line| line.len() as f32 * char_width)
                .fold(0.0, f32::max);
            
            Size::new(max_line_width, lines.len() as f32 * line_height)
        } else {
            Size::new(text.len() as f32 * char_width, line_height)
        }
    }
    
    /// Wrap text to fit within the given width.
    fn wrap_text(&self, text: &str, max_width: f32) -> Vec<String> {
        if !self.word_wrap {
            if self.multiline {
                return text.split('\n').map(|s| s.to_string()).collect();
            } else {
                return vec![text.to_string()];
            }
        }
        
        let char_width = self.font_size * 0.6; // Rough approximation
        let max_chars_per_line = (max_width / char_width) as usize;
        
        if max_chars_per_line == 0 {
            return vec![text.to_string()];
        }
        
        let mut lines = Vec::new();
        let text_lines = if self.multiline {
            text.split('\n').collect()
        } else {
            vec![text]
        };
        
        for line in text_lines {
            if line.len() <= max_chars_per_line {
                lines.push(line.to_string());
            } else {
                // Simple word wrapping
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut current_line = String::new();
                
                for word in words {
                    if current_line.is_empty() {
                        current_line = word.to_string();
                    } else if current_line.len() + 1 + word.len() <= max_chars_per_line {
                        current_line.push(' ');
                        current_line.push_str(word);
                    } else {
                        lines.push(current_line);
                        current_line = word.to_string();
                    }
                }
                
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
            }
        }
        
        lines
    }
}

impl Widget for Label {
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
        let text_size = self.estimate_text_size(&self.text);
        Size::new(text_size.width + 4.0, text_size.height + 4.0) // Add padding
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
    
    fn handle_event(&mut self, _event: &Event) -> Result<bool> {
        // Labels don't handle events by default
        Ok(false)
    }
    
    fn update(&mut self, _delta_time: Duration) -> Result<()> {
        // Labels don't need frame-by-frame updates
        Ok(())
    }
    
    fn render(&self, canvas: &mut Canvas) -> Result<()> {
        if !self.is_visible() {
            return Ok(());
        }
        
        // Render base (background and border)
        self.base.render_base(canvas)?;
        
        // Render text
        if !self.text.is_empty() {
            let bounds = self.bounds();
            let wrapped_lines = self.wrap_text(&self.text, bounds.width - 4.0); // Account for padding
            let line_height = self.font_size * 1.2;
            
            for (i, line) in wrapped_lines.iter().enumerate() {
                if line.is_empty() {
                    continue;
                }
                
                let line_size = self.estimate_text_size(line);
                let mut line_pos = self.calculate_text_position(line_size);
                
                // Adjust for multiple lines
                if wrapped_lines.len() > 1 {
                    match self.vertical_align {
                        VerticalAlign::Top => {
                            line_pos.y += i as f32 * line_height;
                        }
                        VerticalAlign::Middle => {
                            let total_height = wrapped_lines.len() as f32 * line_height;
                            line_pos.y = bounds.y + (bounds.height - total_height) / 2.0 + (i as f32 + 1.0) * line_height;
                        }
                        VerticalAlign::Bottom => {
                            let total_height = wrapped_lines.len() as f32 * line_height;
                            line_pos.y = bounds.y + bounds.height - total_height + (i as f32 + 1.0) * line_height;
                        }
                    }
                }
                
                canvas.draw_text(line, line_pos, self.text_color)?;
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

/// Builder for creating labels.
pub struct LabelBuilder {
    id: Option<WidgetId>,
    bounds: Option<Rect>,
    visible: bool,
    enabled: bool,
    background_color: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
    text: String,
    text_color: Color,
    font_size: f32,
    text_align: TextAlign,
    vertical_align: VerticalAlign,
    word_wrap: bool,
    multiline: bool,
}

impl LabelBuilder {
    /// Create a new label builder.
    pub fn new() -> Self {
        Self {
            id: None,
            bounds: None,
            visible: true,
            enabled: true,
            background_color: None,
            border_color: None,
            border_width: 0.0,
            text: "Label".to_string(),
            text_color: Color::BLACK,
            font_size: 14.0,
            text_align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            word_wrap: false,
            multiline: false,
        }
    }
    
    /// Set the label text.
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = text.into();
        self
    }
    
    /// Set the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }
    
    /// Set the font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(1.0);
        self
    }
    
    /// Set the text alignment.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }
    
    /// Set the vertical alignment.
    pub fn vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }
    
    /// Enable/disable word wrap.
    pub fn word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }
    
    /// Enable/disable multiline support.
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }
}

impl Default for LabelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Apply the widget_builder macro
crate::widget_builder!(LabelBuilder, Label);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::WidgetManager;

    #[test]
    fn test_label_creation() {
        let label = Label::new(1, "Test Label".to_string());
        assert_eq!(label.id(), 1);
        assert_eq!(label.text(), "Test Label");
        assert_eq!(label.text_color(), Color::BLACK);
        assert_eq!(label.font_size(), 14.0);
        assert_eq!(label.text_align(), TextAlign::Left);
        assert_eq!(label.vertical_align(), VerticalAlign::Top);
        assert!(!label.word_wrap());
        assert!(!label.multiline());
    }

    #[test]
    fn test_label_builder() {
        let mut manager = WidgetManager::new();
        
        let _label = LabelBuilder::new()
            .text("Hello, World!")
            .text_color(Color::BLUE)
            .font_size(16.0)
            .text_align(TextAlign::Center)
            .vertical_align(VerticalAlign::Middle)
            .word_wrap(true)
            .multiline(true)
            .size(200.0, 100.0)
            .build(&mut manager);
        
        // Test would continue if we had a proper widget system
    }

    #[test]
    fn test_text_size_estimation() {
        let label = Label::new(1, "Test".to_string());
        let size = label.estimate_text_size("Hello");
        
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
        
        // Longer text should be wider
        let longer_size = label.estimate_text_size("Hello, World!");
        assert!(longer_size.width > size.width);
    }

    #[test]
    fn test_text_wrapping() {
        let mut label = Label::new(1, "This is a long text that should wrap".to_string());
        label.set_word_wrap(true);
        
        let lines = label.wrap_text("This is a long text that should wrap", 100.0);
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_multiline_text() {
        let label = Label::new(1, "Line 1\nLine 2\nLine 3".to_string());
        let size = label.estimate_text_size("Line 1\nLine 2\nLine 3");
        
        // Should be taller for multiline text
        let single_line_size = label.estimate_text_size("Line 1");
        assert!(size.height > single_line_size.height);
    }
}