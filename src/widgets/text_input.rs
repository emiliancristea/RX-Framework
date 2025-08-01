//! Text input widget implementation for user text entry.

use super::{Widget, BaseWidget, WidgetId, WidgetManager};
use crate::{Result, Event, Canvas, Rect, Color, Point, Size};
use crate::platform::{MouseButton, Key};
use std::any::Any;
use std::time::Duration;

/// Text input widget for user text entry.
pub struct TextInput {
    base: BaseWidget,
    text: String,
    placeholder: String,
    text_color: Color,
    placeholder_color: Color,
    selection_color: Color,
    cursor_color: Color,
    font_size: f32,
    cursor_position: usize,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    is_focused: bool,
    cursor_visible: bool,
    cursor_blink_timer: Duration,
    cursor_blink_interval: Duration,
    max_length: Option<usize>,
    is_password: bool,
    password_char: char,
    read_only: bool,
    on_text_changed: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
    on_enter: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
}

impl TextInput {
    /// Create a new text input widget.
    pub fn new(id: WidgetId) -> Self {
        let mut base = BaseWidget::new(id);
        base.set_background_color(Some(Color::WHITE));
        base.set_border(Some(Color::GRAY), 1.0);
        
        Self {
            base,
            text: String::new(),
            placeholder: String::new(),
            text_color: Color::BLACK,
            placeholder_color: Color::GRAY,
            selection_color: Color::rgba(0.0, 0.5, 1.0, 0.3),
            cursor_color: Color::BLACK,
            font_size: 14.0,
            cursor_position: 0,
            selection_start: None,
            selection_end: None,
            is_focused: false,
            cursor_visible: true,
            cursor_blink_timer: Duration::from_secs(0),
            cursor_blink_interval: Duration::from_millis(500),
            max_length: None,
            is_password: false,
            password_char: '•',
            read_only: false,
            on_text_changed: None,
            on_enter: None,
        }
    }
    
    /// Create a text input from a base widget and builder.
    pub fn from_base(base: BaseWidget, builder: TextInputBuilder) -> Self {
        Self {
            base,
            text: builder.text,
            placeholder: builder.placeholder,
            text_color: builder.text_color,
            placeholder_color: builder.placeholder_color,
            selection_color: builder.selection_color,
            cursor_color: builder.cursor_color,
            font_size: builder.font_size,
            cursor_position: 0,
            selection_start: None,
            selection_end: None,
            is_focused: false,
            cursor_visible: true,
            cursor_blink_timer: Duration::from_secs(0),
            cursor_blink_interval: Duration::from_millis(500),
            max_length: builder.max_length,
            is_password: builder.is_password,
            password_char: builder.password_char,
            read_only: builder.read_only,
            on_text_changed: builder.on_text_changed,
            on_enter: builder.on_enter,
        }
    }
    
    /// Set the text content.
    pub fn set_text<S: Into<String>>(&mut self, text: S) {
        let new_text = text.into();
        
        // Apply max length constraint
        let final_text = if let Some(max_len) = self.max_length {
            if new_text.len() > max_len {
                new_text[..max_len].to_string()
            } else {
                new_text
            }
        } else {
            new_text
        };
        
        self.text = final_text;
        self.cursor_position = self.cursor_position.min(self.text.len());
        self.clear_selection();
        
        // Trigger callback
        if let Some(ref callback) = self.on_text_changed {
            let _ = callback(&self.text);
        }
    }
    
    /// Get the text content.
    pub fn text(&self) -> &str {
        &self.text
    }
    
    /// Set the placeholder text.
    pub fn set_placeholder<S: Into<String>>(&mut self, placeholder: S) {
        self.placeholder = placeholder.into();
    }
    
    /// Get the placeholder text.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }
    
    /// Set the maximum text length.
    pub fn set_max_length(&mut self, max_length: Option<usize>) {
        self.max_length = max_length;
        
        // Truncate existing text if necessary
        if let Some(max_len) = max_length {
            if self.text.len() > max_len {
                self.text.truncate(max_len);
                self.cursor_position = self.cursor_position.min(self.text.len());
            }
        }
    }
    
    /// Get the maximum text length.
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }
    
    /// Set password mode.
    pub fn set_password(&mut self, is_password: bool) {
        self.is_password = is_password;
    }
    
    /// Check if in password mode.
    pub fn is_password(&self) -> bool {
        self.is_password
    }
    
    /// Set the password character.
    pub fn set_password_char(&mut self, char: char) {
        self.password_char = char;
    }
    
    /// Set read-only mode.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }
    
    /// Check if read-only.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
    
    /// Set the text changed callback.
    pub fn set_on_text_changed<F>(&mut self, callback: F)
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.on_text_changed = Some(Box::new(callback));
    }
    
    /// Set the enter key callback.
    pub fn set_on_enter<F>(&mut self, callback: F)
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.on_enter = Some(Box::new(callback));
    }
    
    /// Focus the text input.
    pub fn focus(&mut self) {
        self.is_focused = true;
        self.cursor_visible = true;
        self.cursor_blink_timer = Duration::from_secs(0);
    }
    
    /// Unfocus the text input.
    pub fn unfocus(&mut self) {
        self.is_focused = false;
        self.cursor_visible = false;
        self.clear_selection();
    }
    
    /// Check if the text input is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
    
    /// Clear the current selection.
    fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }
    
    /// Insert text at the cursor position.
    fn insert_text(&mut self, text: &str) {
        if self.read_only {
            return;
        }
        
        // Remove selected text first
        if self.has_selection() {
            self.delete_selection();
        }
        
        // Check max length
        let new_text = if let Some(max_len) = self.max_length {
            let available_space = max_len.saturating_sub(self.text.len());
            if text.len() > available_space {
                &text[..available_space]
            } else {
                text
            }
        } else {
            text
        };
        
        // Insert the text
        self.text.insert_str(self.cursor_position, new_text);
        self.cursor_position += new_text.len();
        
        // Trigger callback
        if let Some(ref callback) = self.on_text_changed {
            let _ = callback(&self.text);
        }
    }
    
    /// Delete character at cursor position.
    fn delete_char(&mut self) {
        if self.read_only {
            return;
        }
        
        if self.has_selection() {
            self.delete_selection();
        } else if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
            
            // Trigger callback
            if let Some(ref callback) = self.on_text_changed {
                let _ = callback(&self.text);
            }
        }
    }
    
    /// Delete character after cursor position.
    fn delete_char_forward(&mut self) {
        if self.read_only {
            return;
        }
        
        if self.has_selection() {
            self.delete_selection();
        } else if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
            
            // Trigger callback
            if let Some(ref callback) = self.on_text_changed {
                let _ = callback(&self.text);
            }
        }
    }
    
    /// Delete the current selection.
    fn delete_selection(&mut self) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start, end) = if start <= end { (start, end) } else { (end, start) };
            self.text.replace_range(start..end, "");
            self.cursor_position = start;
            self.clear_selection();
            
            // Trigger callback
            if let Some(ref callback) = self.on_text_changed {
                let _ = callback(&self.text);
            }
        }
    }
    
    /// Check if there's a text selection.
    fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_end.is_some()
    }
    
    /// Move cursor left.
    fn move_cursor_left(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
        } else {
            self.clear_selection();
        }
        
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
        
        if extend_selection {
            self.selection_end = Some(self.cursor_position);
        }
    }
    
    /// Move cursor right.
    fn move_cursor_right(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
        } else {
            self.clear_selection();
        }
        
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
        
        if extend_selection {
            self.selection_end = Some(self.cursor_position);
        }
    }
    
    /// Move cursor to the beginning.
    fn move_cursor_home(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
            self.cursor_position = 0;
            self.selection_end = Some(self.cursor_position);
        } else {
            self.clear_selection();
            self.cursor_position = 0;
        }
    }
    
    /// Move cursor to the end.
    fn move_cursor_end(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_position);
            }
            self.cursor_position = self.text.len();
            self.selection_end = Some(self.cursor_position);
        } else {
            self.clear_selection();
            self.cursor_position = self.text.len();
        }
    }
    
    /// Get the display text (with password masking if enabled).
    fn display_text(&self) -> String {
        if self.is_password && !self.text.is_empty() {
            self.password_char.to_string().repeat(self.text.len())
        } else {
            self.text.clone()
        }
    }
    
    /// Calculate cursor x position for rendering.
    fn cursor_x_position(&self) -> f32 {
        let bounds = self.bounds();
        let char_width = self.font_size * 0.6; // Rough approximation
        bounds.x + 5.0 + (self.cursor_position as f32 * char_width) // 5px padding
    }
}

impl Widget for TextInput {
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
        let char_width = self.font_size * 0.6;
        let min_width = char_width * 10.0; // Minimum 10 characters wide
        let height = self.font_size * 1.5; // Font size + padding
        
        Size::new(min_width + 10.0, height + 10.0) // Add padding
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
                if *button == MouseButton::Left {
                    if self.base.contains_point(point) {
                        self.focus();
                        
                        // Calculate cursor position from mouse click
                        let char_width = self.font_size * 0.6;
                        let relative_x = point.x - self.bounds().x - 5.0; // Account for padding
                        let char_pos = (relative_x / char_width).round() as usize;
                        self.cursor_position = char_pos.min(self.text.len());
                        self.clear_selection();
                        
                        return Ok(true);
                    } else {
                        self.unfocus();
                    }
                }
            }
            
            Event::KeyPressed { window_id: _, key, modifiers, repeat: _ } => {
                if !self.is_focused {
                    return Ok(false);
                }
                
                match key {
                    Key::Backspace => {
                        self.delete_char();
                        return Ok(true);
                    }
                    Key::Delete => {
                        self.delete_char_forward();
                        return Ok(true);
                    }
                    Key::Left => {
                        self.move_cursor_left(modifiers.shift);
                        return Ok(true);
                    }
                    Key::Right => {
                        self.move_cursor_right(modifiers.shift);
                        return Ok(true);
                    }
                    Key::Home => {
                        self.move_cursor_home(modifiers.shift);
                        return Ok(true);
                    }
                    Key::End => {
                        self.move_cursor_end(modifiers.shift);
                        return Ok(true);
                    }
                    Key::Return => {
                        if let Some(ref callback) = self.on_enter {
                            callback(&self.text)?;
                        }
                        return Ok(true);
                    }
                    Key::A if modifiers.ctrl => {
                        // Select all
                        self.selection_start = Some(0);
                        self.selection_end = Some(self.text.len());
                        self.cursor_position = self.text.len();
                        return Ok(true);
                    }
                    _ => {}
                }
            }
            
            Event::TextInput { window_id: _, text } => {
                if self.is_focused {
                    self.insert_text(text);
                    return Ok(true);
                }
            }
            
            _ => {}
        }
        
        Ok(false)
    }
    
    fn update(&mut self, delta_time: Duration) -> Result<()> {
        if self.is_focused {
            self.cursor_blink_timer += delta_time;
            if self.cursor_blink_timer >= self.cursor_blink_interval {
                self.cursor_visible = !self.cursor_visible;
                self.cursor_blink_timer = Duration::from_secs(0);
            }
        }
        
        Ok(())
    }
    
    fn render(&self, canvas: &mut Canvas) -> Result<()> {
        if !self.is_visible() {
            return Ok(());
        }
        
        let bounds = self.bounds();
        
        // Render base (background and border)
        self.base.render_base(canvas)?;
        
        // Render focus indicator
        if self.is_focused {
            let focus_color = Color::rgba(0.0, 0.5, 1.0, 0.5);
            canvas.stroke_rect(bounds, focus_color, 2.0)?;
        }
        
        // Render text or placeholder
        let display_text = self.display_text();
        let text_to_show = if display_text.is_empty() && !self.placeholder.is_empty() {
            &self.placeholder
        } else {
            &display_text
        };
        
        let text_color = if display_text.is_empty() && !self.placeholder.is_empty() {
            self.placeholder_color
        } else {
            self.text_color
        };
        
        if !text_to_show.is_empty() {
            let text_pos = Point::new(bounds.x + 5.0, bounds.y + bounds.height / 2.0 + self.font_size / 2.0);
            canvas.draw_text(text_to_show, text_pos, text_color)?;
        }
        
        // Render selection
        if self.has_selection() {
            if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                let (start, end) = if start <= end { (start, end) } else { (end, start) };
                let char_width = self.font_size * 0.6;
                let selection_x = bounds.x + 5.0 + (start as f32 * char_width);
                let selection_width = (end - start) as f32 * char_width;
                let selection_rect = Rect::new(selection_x, bounds.y + 2.0, selection_width, bounds.height - 4.0);
                canvas.fill_rect(selection_rect, self.selection_color)?;
            }
        }
        
        // Render cursor
        if self.is_focused && self.cursor_visible && !self.has_selection() {
            let cursor_x = self.cursor_x_position();
            let cursor_rect = Rect::new(cursor_x, bounds.y + 2.0, 1.0, bounds.height - 4.0);
            canvas.fill_rect(cursor_rect, self.cursor_color)?;
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

/// Builder for creating text input widgets.
pub struct TextInputBuilder {
    id: Option<WidgetId>,
    bounds: Option<Rect>,
    visible: bool,
    enabled: bool,
    background_color: Option<Color>,
    border_color: Option<Color>,
    border_width: f32,
    text: String,
    placeholder: String,
    text_color: Color,
    placeholder_color: Color,
    selection_color: Color,
    cursor_color: Color,
    font_size: f32,
    max_length: Option<usize>,
    is_password: bool,
    password_char: char,
    read_only: bool,
    on_text_changed: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
    on_enter: Option<Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
}

impl TextInputBuilder {
    /// Create a new text input builder.
    pub fn new() -> Self {
        Self {
            id: None,
            bounds: None,
            visible: true,
            enabled: true,
            background_color: Some(Color::WHITE),
            border_color: Some(Color::GRAY),
            border_width: 1.0,
            text: String::new(),
            placeholder: String::new(),
            text_color: Color::BLACK,
            placeholder_color: Color::GRAY,
            selection_color: Color::rgba(0.0, 0.5, 1.0, 0.3),
            cursor_color: Color::BLACK,
            font_size: 14.0,
            max_length: None,
            is_password: false,
            password_char: '•',
            read_only: false,
            on_text_changed: None,
            on_enter: None,
        }
    }
    
    /// Set the initial text.
    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.text = text.into();
        self
    }
    
    /// Set the placeholder text.
    pub fn placeholder<S: Into<String>>(mut self, placeholder: S) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    
    /// Set the text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }
    
    /// Set the placeholder color.
    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = color;
        self
    }
    
    /// Set the font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(1.0);
        self
    }
    
    /// Set the maximum text length.
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
    
    /// Enable password mode.
    pub fn password(mut self, password_char: Option<char>) -> Self {
        self.is_password = true;
        if let Some(char) = password_char {
            self.password_char = char;
        }
        self
    }
    
    /// Set read-only mode.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    
    /// Set the text changed callback.
    pub fn on_text_changed<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.on_text_changed = Some(Box::new(callback));
        self
    }
    
    /// Set the enter key callback.
    pub fn on_enter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.on_enter = Some(Box::new(callback));
        self
    }
}

impl Default for TextInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Apply the widget_builder macro
crate::widget_builder!(TextInputBuilder, TextInput);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::WidgetManager;

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new(1);
        assert_eq!(input.id(), 1);
        assert_eq!(input.text(), "");
        assert!(!input.is_focused());
        assert!(!input.is_password());
        assert!(!input.is_read_only());
    }

    #[test]
    fn test_text_input_builder() {
        let mut manager = WidgetManager::new();
        
        let _input = TextInputBuilder::new()
            .text("Initial text")
            .placeholder("Enter text...")
            .max_length(50)
            .password(Some('*'))
            .size(200.0, 30.0)
            .build(&mut manager);
        
        // Test would continue if we had a proper widget system
    }

    #[test]
    fn test_text_insertion() {
        let mut input = TextInput::new(1);
        input.focus();
        
        input.insert_text("Hello");
        assert_eq!(input.text(), "Hello");
        assert_eq!(input.cursor_position, 5);
        
        input.cursor_position = 0;
        input.insert_text("Hi ");
        assert_eq!(input.text(), "Hi Hello");
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = TextInput::new(1);
        input.set_text("Hello");
        input.cursor_position = 2;
        
        input.move_cursor_left(false);
        assert_eq!(input.cursor_position, 1);
        
        input.move_cursor_right(false);
        assert_eq!(input.cursor_position, 2);
        
        input.move_cursor_home(false);
        assert_eq!(input.cursor_position, 0);
        
        input.move_cursor_end(false);
        assert_eq!(input.cursor_position, 5);
    }

    #[test]
    fn test_password_mode() {
        let mut input = TextInput::new(1);
        input.set_password(true);
        input.set_text("secret");
        
        let display = input.display_text();
        assert_eq!(display, "••••••");
        assert_eq!(input.text(), "secret");
    }

    #[test]
    fn test_max_length() {
        let mut input = TextInput::new(1);
        input.set_max_length(Some(5));
        input.set_text("Hello World");
        
        assert_eq!(input.text(), "Hello");
    }
}