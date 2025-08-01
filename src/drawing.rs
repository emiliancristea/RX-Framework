//! 2D drawing and graphics for the CX Framework.
//! 
//! This module provides a comprehensive 2D graphics API that abstracts
//! over platform-specific drawing operations.

use crate::{Error, Result};
use crate::platform::DrawingContext;

/// RGBA color representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA values (0.0 to 1.0).
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }
    
    /// Create a new color from RGB values (0.0 to 1.0) with full opacity.
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
    
    /// Create a new color from RGBA values (0 to 255).
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }
    
    /// Create a new color from RGB values (0 to 255) with full opacity.
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }
    
    /// Create a grayscale color.
    pub fn gray(value: f32) -> Self {
        Self::rgb(value, value, value)
    }
    
    /// Create a grayscale color with alpha.
    pub fn gray_alpha(value: f32, alpha: f32) -> Self {
        Self::rgba(value, value, value, alpha)
    }
    
    /// Parse a hex color string (e.g., "#FF0000", "FF0000", "#RGB", "RGB").
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');
        
        let (r, g, b, a) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                (r, g, b, 255)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| Error::drawing("Invalid hex color"))?;
                (r, g, b, a)
            }
            _ => return Err(Error::drawing("Invalid hex color length")),
        };
        
        Ok(Self::rgba8(r, g, b, a))
    }
    
    /// Convert to tuple (r, g, b, a).
    pub fn to_tuple(self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
    
    /// Blend this color with another using alpha blending.
    pub fn blend_with(self, other: Color) -> Color {
        let alpha = other.a;
        let inv_alpha = 1.0 - alpha;
        
        Color::rgba(
            self.r * inv_alpha + other.r * alpha,
            self.g * inv_alpha + other.g * alpha,
            self.b * inv_alpha + other.b * alpha,
            self.a.max(other.a),
        )
    }
    
    /// Lighten the color by a factor (0.0 to 1.0).
    pub fn lighten(self, factor: f32) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        Color::rgba(
            (self.r + (1.0 - self.r) * factor).clamp(0.0, 1.0),
            (self.g + (1.0 - self.g) * factor).clamp(0.0, 1.0),
            (self.b + (1.0 - self.b) * factor).clamp(0.0, 1.0),
            self.a,
        )
    }
    
    /// Darken the color by a factor (0.0 to 1.0).
    pub fn darken(self, factor: f32) -> Color {
        let factor = 1.0 - factor.clamp(0.0, 1.0);
        Color::rgba(
            self.r * factor,
            self.g * factor,
            self.b * factor,
            self.a,
        )
    }
    
    /// Set the alpha channel.
    pub fn with_alpha(mut self, alpha: f32) -> Color {
        self.a = alpha.clamp(0.0, 1.0);
        self
    }
    
    // Common colors
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW: Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const CYAN: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA: Color = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const GRAY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    pub const LIGHT_GRAY: Color = Color { r: 0.75, g: 0.75, b: 0.75, a: 1.0 };
    pub const DARK_GRAY: Color = Color { r: 0.25, g: 0.25, b: 0.25, a: 1.0 };
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

/// 2D point representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Create a point at the origin.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
    
    /// Calculate the distance to another point.
    pub fn distance_to(self, other: Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Add another point to this point.
    pub fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
    
    /// Subtract another point from this point.
    pub fn subtract(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
    
    /// Scale the point by a factor.
    pub fn scale(self, factor: f32) -> Point {
        Point::new(self.x * factor, self.y * factor)
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::zero()
    }
}

/// 2D size representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    
    /// Create a zero size.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
    
    /// Create a square size.
    pub fn square(size: f32) -> Self {
        Self::new(size, size)
    }
    
    /// Get the area.
    pub fn area(self) -> f32 {
        self.width * self.height
    }
    
    /// Check if the size is empty (width or height is 0).
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
    
    /// Scale the size by a factor.
    pub fn scale(self, factor: f32) -> Size {
        Size::new(self.width * factor, self.height * factor)
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::zero()
    }
}

/// 2D rectangle representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    /// Create a rectangle from position and size.
    pub fn from_point_size(point: Point, size: Size) -> Self {
        Self::new(point.x, point.y, size.width, size.height)
    }
    
    /// Create a rectangle at the origin with the given size.
    pub fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }
    
    /// Create a zero rectangle.
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
    
    /// Get the position as a point.
    pub fn position(self) -> Point {
        Point::new(self.x, self.y)
    }
    
    /// Get the size.
    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }
    
    /// Get the area.
    pub fn area(self) -> f32 {
        self.width * self.height
    }
    
    /// Check if the rectangle is empty.
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
    
    /// Get the right edge x-coordinate.
    pub fn right(self) -> f32 {
        self.x + self.width
    }
    
    /// Get the bottom edge y-coordinate.
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
    
    /// Get the center point.
    pub fn center(self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    
    /// Check if a point is inside the rectangle.
    pub fn contains_point(self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() &&
        point.y >= self.y && point.y < self.bottom()
    }
    
    /// Check if this rectangle intersects with another.
    pub fn intersects_with(self, other: Rect) -> bool {
        self.x < other.right() && self.right() > other.x &&
        self.y < other.bottom() && self.bottom() > other.y
    }
    
    /// Get the intersection of this rectangle with another.
    pub fn intersection(self, other: Rect) -> Option<Rect> {
        let left = self.x.max(other.x);
        let right = self.right().min(other.right());
        let top = self.y.max(other.y);
        let bottom = self.bottom().min(other.bottom());
        
        if left < right && top < bottom {
            Some(Rect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }
    
    /// Get the union of this rectangle with another.
    pub fn union(self, other: Rect) -> Rect {
        let left = self.x.min(other.x);
        let right = self.right().max(other.right());
        let top = self.y.min(other.y);
        let bottom = self.bottom().max(other.bottom());
        
        Rect::new(left, top, right - left, bottom - top)
    }
    
    /// Expand the rectangle by the given amount on all sides.
    pub fn expand(self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.width + 2.0 * amount,
            self.height + 2.0 * amount,
        )
    }
    
    /// Contract the rectangle by the given amount on all sides.
    pub fn contract(self, amount: f32) -> Rect {
        self.expand(-amount)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::zero()
    }
}

/// Drawing canvas that provides 2D graphics operations.
pub struct Canvas {
    context: Box<dyn DrawingContext>,
    transform_stack: Vec<Transform>,
    current_transform: Transform,
}

/// 2D transformation matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub a: f32, pub b: f32,
    pub c: f32, pub d: f32,
    pub tx: f32, pub ty: f32,
}

impl Transform {
    /// Create an identity transform.
    pub fn identity() -> Self {
        Self {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            tx: 0.0, ty: 0.0,
        }
    }
    
    /// Create a translation transform.
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            tx: x, ty: y,
        }
    }
    
    /// Create a scale transform.
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx, b: 0.0,
            c: 0.0, d: sy,
            tx: 0.0, ty: 0.0,
        }
    }
    
    /// Create a rotation transform (angle in radians).
    pub fn rotate(angle: f32) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self {
            a: cos_a, b: sin_a,
            c: -sin_a, d: cos_a,
            tx: 0.0, ty: 0.0,
        }
    }
    
    /// Transform a point.
    pub fn transform_point(self, point: Point) -> Point {
        Point::new(
            self.a * point.x + self.c * point.y + self.tx,
            self.b * point.x + self.d * point.y + self.ty,
        )
    }
    
    /// Combine with another transform.
    pub fn combine(self, other: Transform) -> Transform {
        Transform {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            tx: self.a * other.tx + self.c * other.ty + self.tx,
            ty: self.b * other.tx + self.d * other.ty + self.ty,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Canvas {
    /// Create a new canvas with the given drawing context.
    pub fn new(context: Box<dyn DrawingContext>) -> Self {
        Self {
            context,
            transform_stack: Vec::new(),
            current_transform: Transform::identity(),
        }
    }
    
    /// Get the canvas size.
    pub fn size(&self) -> Size {
        let (width, height) = self.context.size();
        Size::new(width as f32, height as f32)
    }
    
    /// Clear the canvas with a color.
    pub fn clear(&mut self, color: Color) -> Result<()> {
        self.context.clear(color.to_tuple())
    }
    
    /// Fill a rectangle.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        let transformed = self.transform_rect(rect);
        self.context.fill_rect(
            transformed.x,
            transformed.y,
            transformed.width,
            transformed.height,
            color.to_tuple(),
        )
    }
    
    /// Stroke a rectangle outline.
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, stroke_width: f32) -> Result<()> {
        let transformed = self.transform_rect(rect);
        self.context.stroke_rect(
            transformed.x,
            transformed.y,
            transformed.width,
            transformed.height,
            color.to_tuple(),
            stroke_width,
        )
    }
    
    /// Draw text.
    pub fn draw_text(&mut self, text: &str, position: Point, color: Color) -> Result<()> {
        let transformed = self.current_transform.transform_point(position);
        self.context.draw_text(text, transformed.x, transformed.y, color.to_tuple())
    }
    
    /// Present/flush the drawing operations.
    pub fn present(&mut self) -> Result<()> {
        self.context.present()
    }
    
    /// Save the current transform state.
    pub fn save(&mut self) {
        self.transform_stack.push(self.current_transform);
    }
    
    /// Restore the previous transform state.
    pub fn restore(&mut self) {
        if let Some(transform) = self.transform_stack.pop() {
            self.current_transform = transform;
        }
    }
    
    /// Translate the coordinate system.
    pub fn translate(&mut self, x: f32, y: f32) {
        self.current_transform = self.current_transform.combine(Transform::translate(x, y));
    }
    
    /// Scale the coordinate system.
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.current_transform = self.current_transform.combine(Transform::scale(sx, sy));
    }
    
    /// Rotate the coordinate system (angle in radians).
    pub fn rotate(&mut self, angle: f32) {
        self.current_transform = self.current_transform.combine(Transform::rotate(angle));
    }
    
    /// Set the transform directly.
    pub fn set_transform(&mut self, transform: Transform) {
        self.current_transform = transform;
    }
    
    /// Get the current transform.
    pub fn transform(&self) -> Transform {
        self.current_transform
    }
    
    /// Transform a rectangle by the current transform.
    fn transform_rect(&self, rect: Rect) -> Rect {
        let top_left = self.current_transform.transform_point(Point::new(rect.x, rect.y));
        let bottom_right = self.current_transform.transform_point(Point::new(rect.right(), rect.bottom()));
        
        Rect::new(
            top_left.x,
            top_left.y,
            bottom_right.x - top_left.x,
            bottom_right.y - top_left.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(1.0, 0.5, 0.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
        
        let color = Color::rgba8(255, 128, 0, 128);
        assert_eq!(color.r, 1.0);
        assert!((color.g - 128.0/255.0).abs() < 0.01);
        assert_eq!(color.b, 0.0);
        assert!((color.a - 128.0/255.0).abs() < 0.01);
    }

    #[test]
    fn test_color_hex_parsing() {
        let color = Color::from_hex("#FF8000").unwrap();
        assert_eq!(color.r, 1.0);
        assert!((color.g - 128.0/255.0).abs() < 0.01);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
        
        let color = Color::from_hex("F80").unwrap();
        assert_eq!(color.r, 1.0);
        assert!((color.g - 136.0/255.0).abs() < 0.01);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_point_operations() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(3.0, 4.0);
        
        let sum = p1.add(p2);
        assert_eq!(sum.x, 4.0);
        assert_eq!(sum.y, 6.0);
        
        let distance = p1.distance_to(p2);
        assert!((distance - 2.828).abs() < 0.01);
    }

    #[test]
    fn test_rect_operations() {
        let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let rect2 = Rect::new(5.0, 5.0, 10.0, 10.0);
        
        assert!(rect1.intersects_with(rect2));
        
        let intersection = rect1.intersection(rect2).unwrap();
        assert_eq!(intersection.x, 5.0);
        assert_eq!(intersection.y, 5.0);
        assert_eq!(intersection.width, 5.0);
        assert_eq!(intersection.height, 5.0);
        
        assert!(rect1.contains_point(Point::new(5.0, 5.0)));
        assert!(!rect1.contains_point(Point::new(15.0, 15.0)));
    }

    #[test]
    fn test_transform() {
        let transform = Transform::translate(10.0, 20.0);
        let point = Point::new(5.0, 5.0);
        let transformed = transform.transform_point(point);
        
        assert_eq!(transformed.x, 15.0);
        assert_eq!(transformed.y, 25.0);
        
        let scale = Transform::scale(2.0, 3.0);
        let combined = transform.combine(scale);
        let final_point = combined.transform_point(point);
        
        assert_eq!(final_point.x, 20.0);
        assert_eq!(final_point.y, 35.0);
    }
}