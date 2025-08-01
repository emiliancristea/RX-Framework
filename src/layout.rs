//! Layout management system for the CX Framework.
//! 
//! This module provides layout managers that automatically arrange widgets
//! according to various layout algorithms.

use crate::{Result, Rect, Size};
use crate::widgets::{Widget, WidgetId};
use std::collections::HashMap;

/// Layout constraint for widgets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConstraint {
    /// Minimum size constraint.
    pub min_size: Size,
    /// Maximum size constraint (None means unlimited).
    pub max_size: Option<Size>,
    /// Whether the widget should expand to fill available space.
    pub expand: bool,
    /// Layout weight for space distribution.
    pub weight: f32,
}

impl LayoutConstraint {
    /// Create a new layout constraint.
    pub fn new() -> Self {
        Self {
            min_size: Size::zero(),
            max_size: None,
            expand: false,
            weight: 1.0,
        }
    }
    
    /// Set the minimum size.
    pub fn min_size(mut self, size: Size) -> Self {
        self.min_size = size;
        self
    }
    
    /// Set the maximum size.
    pub fn max_size(mut self, size: Option<Size>) -> Self {
        self.max_size = size;
        self
    }
    
    /// Set whether the widget should expand.
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }
    
    /// Set the layout weight.
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight.max(0.0);
        self
    }
    
    /// Constrain a size to fit within the constraint limits.
    pub fn constrain_size(&self, size: Size) -> Size {
        let mut constrained = Size::new(
            size.width.max(self.min_size.width),
            size.height.max(self.min_size.height),
        );
        
        if let Some(max_size) = self.max_size {
            constrained.width = constrained.width.min(max_size.width);
            constrained.height = constrained.height.min(max_size.height);
        }
        
        constrained
    }
}

impl Default for LayoutConstraint {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout information for a widget.
#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub widget_id: WidgetId,
    pub constraint: LayoutConstraint,
    pub preferred_size: Size,
    pub allocated_bounds: Rect,
}

/// Base trait for layout managers.
pub trait Layout: Send + Sync {
    /// Get the layout name.
    fn name(&self) -> &'static str;
    
    /// Calculate the preferred size for the given widgets and constraints.
    fn calculate_preferred_size(
        &self,
        widgets: &[&dyn Widget],
        constraints: &[LayoutConstraint],
        available_size: Size,
    ) -> Size;
    
    /// Layout the widgets within the given bounds.
    fn layout_widgets(
        &self,
        widgets: &mut [&mut dyn Widget],
        constraints: &[LayoutConstraint],
        bounds: Rect,
    ) -> Result<Vec<Rect>>;
}

/// Layout manager that manages multiple layout instances.
pub struct LayoutManager {
    layouts: HashMap<String, Box<dyn Layout>>,
    widget_constraints: HashMap<WidgetId, LayoutConstraint>,
}

impl LayoutManager {
    /// Create a new layout manager.
    pub fn new() -> Self {
        let mut manager = Self {
            layouts: HashMap::new(),
            widget_constraints: HashMap::new(),
        };
        
        // Register built-in layouts
        manager.register_layout("flex", Box::new(FlexLayout::new()));
        manager.register_layout("grid", Box::new(GridLayout::new(1, 1)));
        manager.register_layout("absolute", Box::new(AbsoluteLayout::new()));
        
        manager
    }
    
    /// Register a layout manager.
    pub fn register_layout(&mut self, name: &str, layout: Box<dyn Layout>) {
        self.layouts.insert(name.to_string(), layout);
    }
    
    /// Get a layout manager by name.
    pub fn get_layout(&self, name: &str) -> Option<&dyn Layout> {
        self.layouts.get(name).map(|layout| layout.as_ref())
    }
    
    /// Set the layout constraint for a widget.
    pub fn set_constraint(&mut self, widget_id: WidgetId, constraint: LayoutConstraint) {
        self.widget_constraints.insert(widget_id, constraint);
    }
    
    /// Get the layout constraint for a widget.
    pub fn get_constraint(&self, widget_id: WidgetId) -> LayoutConstraint {
        self.widget_constraints.get(&widget_id).copied().unwrap_or_default()
    }
    
    /// Remove the layout constraint for a widget.
    pub fn remove_constraint(&mut self, widget_id: WidgetId) {
        self.widget_constraints.remove(&widget_id);
    }
    
    /// Clear all constraints.
    pub fn clear_constraints(&mut self) {
        self.widget_constraints.clear();
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Flexible box layout (similar to CSS Flexbox).
#[derive(Debug, Clone)]
pub struct FlexLayout {
    direction: FlexDirection,
    wrap: FlexWrap,
    justify_content: JustifyContent,
    align_items: AlignItems,
    align_content: AlignContent,
    gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

impl FlexLayout {
    /// Create a new flex layout.
    pub fn new() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            gap: 0.0,
        }
    }
    
    /// Set the flex direction.
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }
    
    /// Set the flex wrap.
    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = wrap;
        self
    }
    
    /// Set the justify content.
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }
    
    /// Set the align items.
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }
    
    /// Set the align content.
    pub fn align_content(mut self, align: AlignContent) -> Self {
        self.align_content = align;
        self
    }
    
    /// Set the gap between items.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }
    
    /// Check if the direction is horizontal.
    fn is_horizontal(&self) -> bool {
        matches!(self.direction, FlexDirection::Row | FlexDirection::RowReverse)
    }
    
    /// Get the main axis size from a size.
    fn main_axis_size(&self, size: Size) -> f32 {
        if self.is_horizontal() { size.width } else { size.height }
    }
    
    /// Get the cross axis size from a size.
    fn cross_axis_size(&self, size: Size) -> f32 {
        if self.is_horizontal() { size.height } else { size.width }
    }
    
    /// Create a size from main and cross axis values.
    fn create_size(&self, main: f32, cross: f32) -> Size {
        if self.is_horizontal() {
            Size::new(main, cross)
        } else {
            Size::new(cross, main)
        }
    }
}

impl Default for FlexLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for FlexLayout {
    fn name(&self) -> &'static str {
        "flex"
    }
    
    fn calculate_preferred_size(
        &self,
        widgets: &[&dyn Widget],
        constraints: &[LayoutConstraint],
        _available_size: Size,
    ) -> Size {
        if widgets.is_empty() {
            return Size::zero();
        }
        
        let mut total_main = 0.0;
        let mut max_cross: f32 = 0.0;
        let gap_total = self.gap * (widgets.len() - 1).max(0usize) as f32;
        
        for (widget, constraint) in widgets.iter().zip(constraints.iter()) {
            let preferred = widget.preferred_size();
            let constrained = constraint.constrain_size(preferred);
            
            total_main += self.main_axis_size(constrained);
            max_cross = max_cross.max(self.cross_axis_size(constrained));
        }
        
        self.create_size(total_main + gap_total, max_cross)
    }
    
    fn layout_widgets(
        &self,
        widgets: &mut [&mut dyn Widget],
        constraints: &[LayoutConstraint],
        bounds: Rect,
    ) -> Result<Vec<Rect>> {
        if widgets.is_empty() {
            return Ok(Vec::new());
        }
        
        let available_main = self.main_axis_size(bounds.size());
        let available_cross = self.cross_axis_size(bounds.size());
        let gap_total = self.gap * (widgets.len() - 1).max(0usize) as f32;
        
        // Calculate sizes
        let mut widget_sizes = Vec::new();
        let mut total_preferred_main = 0.0;
        let mut total_weight = 0.0;
        
        for (widget, constraint) in widgets.iter().zip(constraints.iter()) {
            let preferred = widget.preferred_size();
            let constrained = constraint.constrain_size(preferred);
            let main_size = self.main_axis_size(constrained);
            
            widget_sizes.push(constrained);
            total_preferred_main += main_size;
            
            if constraint.expand {
                total_weight += constraint.weight;
            }
        }
        
        // Distribute extra space
        let available_for_content = available_main - gap_total;
        let extra_space = (available_for_content - total_preferred_main).max(0.0);
        
        let mut final_sizes = Vec::new();
        for (_i, (size, constraint)) in widget_sizes.iter().zip(constraints.iter()).enumerate() {
            let mut final_size = *size;
            
            if constraint.expand && total_weight > 0.0 {
                let extra_main = extra_space * (constraint.weight / total_weight);
                let new_main = self.main_axis_size(final_size) + extra_main;
                final_size = self.create_size(new_main, self.cross_axis_size(final_size));
            }
            
            // Handle cross-axis alignment
            match self.align_items {
                AlignItems::Stretch => {
                    final_size = self.create_size(self.main_axis_size(final_size), available_cross);
                }
                _ => {
                    // Size already set
                }
            }
            
            final_sizes.push(final_size);
        }
        
        // Position widgets
        let mut widget_bounds = Vec::new();
        let mut current_main = 0.0;
        
        // Handle justify-content
        match self.justify_content {
            JustifyContent::FlexStart => {
                // Already at 0
            }
            JustifyContent::FlexEnd => {
                let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum::<f32>() + gap_total;
                current_main = available_for_content - used_space;
            }
            JustifyContent::Center => {
                let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum::<f32>() + gap_total;
                current_main = (available_for_content - used_space) / 2.0;
            }
            JustifyContent::SpaceBetween => {
                if widgets.len() > 1 {
                    let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum();
                    let _extra_gap = (available_for_content - used_space) / (widgets.len() - 1) as f32;
                    // We'll handle this in the positioning loop
                }
            }
            JustifyContent::SpaceAround => {
                let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum();
                let extra_gap = (available_for_content - used_space) / widgets.len() as f32;
                current_main = extra_gap / 2.0;
            }
            JustifyContent::SpaceEvenly => {
                let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum();
                let extra_gap = (available_for_content - used_space) / (widgets.len() + 1) as f32;
                current_main = extra_gap;
            }
        }
        
        for (i, size) in final_sizes.iter().enumerate() {
            let main_size = self.main_axis_size(*size);
            let cross_size = self.cross_axis_size(*size);
            
            // Handle cross-axis alignment
            let cross_position = match self.align_items {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => available_cross - cross_size,
                AlignItems::Center => (available_cross - cross_size) / 2.0,
                AlignItems::Stretch => 0.0, // Already stretched
                AlignItems::Baseline => 0.0, // TODO: Implement baseline alignment
            };
            
            let widget_bounds_rect = if self.is_horizontal() {
                Rect::new(
                    bounds.x + current_main,
                    bounds.y + cross_position,
                    main_size,
                    cross_size,
                )
            } else {
                Rect::new(
                    bounds.x + cross_position,
                    bounds.y + current_main,
                    cross_size,
                    main_size,
                )
            };
            
            widget_bounds.push(widget_bounds_rect);
            
            // Advance to next position
            current_main += main_size;
            
            // Add gap
            if i < final_sizes.len() - 1 {
                match self.justify_content {
                    JustifyContent::SpaceBetween if widgets.len() > 1 => {
                        let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum();
                        let extra_gap = (available_for_content - used_space) / (widgets.len() - 1) as f32;
                        current_main += extra_gap;
                    }
                    JustifyContent::SpaceAround => {
                        let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum();
                        let extra_gap = (available_for_content - used_space) / widgets.len() as f32;
                        current_main += extra_gap;
                    }
                    JustifyContent::SpaceEvenly => {
                        let used_space: f32 = final_sizes.iter().map(|s| self.main_axis_size(*s)).sum();
                        let extra_gap = (available_for_content - used_space) / (widgets.len() + 1) as f32;
                        current_main += extra_gap;
                    }
                    _ => {
                        current_main += self.gap;
                    }
                }
            }
        }
        
        // Apply bounds to widgets
        for (widget, bounds) in widgets.iter_mut().zip(widget_bounds.iter()) {
            widget.set_bounds(*bounds);
        }
        
        Ok(widget_bounds)
    }
}

/// Grid layout manager.
#[derive(Debug, Clone)]
pub struct GridLayout {
    rows: usize,
    columns: usize,
    row_gap: f32,
    column_gap: f32,
}

impl GridLayout {
    /// Create a new grid layout.
    pub fn new(rows: usize, columns: usize) -> Self {
        Self {
            rows: rows.max(1),
            columns: columns.max(1),
            row_gap: 0.0,
            column_gap: 0.0,
        }
    }
    
    /// Set the number of rows and columns.
    pub fn size(mut self, rows: usize, columns: usize) -> Self {
        self.rows = rows.max(1);
        self.columns = columns.max(1);
        self
    }
    
    /// Set the gap between rows and columns.
    pub fn gap(mut self, row_gap: f32, column_gap: f32) -> Self {
        self.row_gap = row_gap.max(0.0);
        self.column_gap = column_gap.max(0.0);
        self
    }
}

impl Layout for GridLayout {
    fn name(&self) -> &'static str {
        "grid"
    }
    
    fn calculate_preferred_size(
        &self,
        widgets: &[&dyn Widget],
        constraints: &[LayoutConstraint],
        _available_size: Size,
    ) -> Size {
        if widgets.is_empty() {
            return Size::zero();
        }
        
        let mut max_cell_width: f32 = 0.0;
        let mut max_cell_height: f32 = 0.0;
        
        for (widget, constraint) in widgets.iter().zip(constraints.iter()) {
            let preferred = widget.preferred_size();
            let constrained = constraint.constrain_size(preferred);
            
            max_cell_width = max_cell_width.max(constrained.width);
            max_cell_height = max_cell_height.max(constrained.height);
        }
        
        let total_width = max_cell_width * self.columns as f32 + self.column_gap * (self.columns - 1) as f32;
        let total_height = max_cell_height * self.rows as f32 + self.row_gap * (self.rows - 1) as f32;
        
        Size::new(total_width, total_height)
    }
    
    fn layout_widgets(
        &self,
        widgets: &mut [&mut dyn Widget],
        _constraints: &[LayoutConstraint],
        bounds: Rect,
    ) -> Result<Vec<Rect>> {
        let cell_width = (bounds.width - self.column_gap * (self.columns - 1) as f32) / self.columns as f32;
        let cell_height = (bounds.height - self.row_gap * (self.rows - 1) as f32) / self.rows as f32;
        
        let mut widget_bounds = Vec::new();
        
        for (i, widget) in widgets.iter_mut().enumerate() {
            let row = i / self.columns;
            let col = i % self.columns;
            
            if row >= self.rows {
                break; // No more space in grid
            }
            
            let x = bounds.x + col as f32 * (cell_width + self.column_gap);
            let y = bounds.y + row as f32 * (cell_height + self.row_gap);
            
            let widget_rect = Rect::new(x, y, cell_width, cell_height);
            widget.set_bounds(widget_rect);
            widget_bounds.push(widget_rect);
        }
        
        Ok(widget_bounds)
    }
}

/// Absolute layout manager (no automatic positioning).
#[derive(Debug, Clone)]
pub struct AbsoluteLayout {
    // No configuration needed for absolute layout
}

impl AbsoluteLayout {
    /// Create a new absolute layout.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AbsoluteLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for AbsoluteLayout {
    fn name(&self) -> &'static str {
        "absolute"
    }
    
    fn calculate_preferred_size(
        &self,
        widgets: &[&dyn Widget],
        _constraints: &[LayoutConstraint],
        _available_size: Size,
    ) -> Size {
        if widgets.is_empty() {
            return Size::zero();
        }
        
        // Find the bounding box of all widgets
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        for widget in widgets {
            let bounds = widget.bounds();
            min_x = min_x.min(bounds.x);
            min_y = min_y.min(bounds.y);
            max_x = max_x.max(bounds.right());
            max_y = max_y.max(bounds.bottom());
        }
        
        Size::new(max_x - min_x, max_y - min_y)
    }
    
    fn layout_widgets(
        &self,
        widgets: &mut [&mut dyn Widget],
        _constraints: &[LayoutConstraint],
        _bounds: Rect,
    ) -> Result<Vec<Rect>> {
        // Absolute layout doesn't change widget positions
        let widget_bounds = widgets.iter().map(|widget| widget.bounds()).collect();
        Ok(widget_bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constraint() {
        let constraint = LayoutConstraint::new()
            .min_size(Size::new(50.0, 30.0))
            .max_size(Some(Size::new(200.0, 100.0)))
            .expand(true)
            .weight(2.0);
        
        assert_eq!(constraint.min_size, Size::new(50.0, 30.0));
        assert_eq!(constraint.max_size, Some(Size::new(200.0, 100.0)));
        assert!(constraint.expand);
        assert_eq!(constraint.weight, 2.0);
        
        let constrained = constraint.constrain_size(Size::new(25.0, 15.0));
        assert_eq!(constrained, Size::new(50.0, 30.0)); // Applied minimum
        
        let constrained = constraint.constrain_size(Size::new(300.0, 150.0));
        assert_eq!(constrained, Size::new(200.0, 100.0)); // Applied maximum
    }

    #[test]
    fn test_flex_layout() {
        let layout = FlexLayout::new()
            .direction(FlexDirection::Row)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Stretch);
        
        assert_eq!(layout.name(), "flex");
        assert!(layout.is_horizontal());
        
        let size = Size::new(100.0, 50.0);
        assert_eq!(layout.main_axis_size(size), 100.0);
        assert_eq!(layout.cross_axis_size(size), 50.0);
    }

    #[test]
    fn test_grid_layout() {
        let layout = GridLayout::new(2, 3).gap(5.0, 10.0);
        
        assert_eq!(layout.name(), "grid");
        assert_eq!(layout.rows, 2);
        assert_eq!(layout.columns, 3);
        assert_eq!(layout.row_gap, 5.0);
        assert_eq!(layout.column_gap, 10.0);
    }

    #[test]
    fn test_layout_manager() {
        let mut manager = LayoutManager::new();
        
        assert!(manager.get_layout("flex").is_some());
        assert!(manager.get_layout("grid").is_some());
        assert!(manager.get_layout("absolute").is_some());
        
        let constraint = LayoutConstraint::new().expand(true);
        manager.set_constraint(1, constraint);
        
        let retrieved = manager.get_constraint(1);
        assert!(retrieved.expand);
    }
}