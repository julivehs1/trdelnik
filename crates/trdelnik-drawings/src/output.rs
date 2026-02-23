//! Output types for drawing rendering
//!
//! These types represent the rendered output of a drawing,
//! which can then be converted to actual graphics primitives.

use crate::{ChartPoint, LineStyle};
use trdelnik_core::{AxisCoordinate, Color};

/// The complete render output of a drawing
#[derive(Clone, Debug)]
pub struct DrawingOutput<X: AxisCoordinate> {
    /// Lines to draw
    pub lines: Vec<DrawingLine<X>>,
    /// Horizontal lines (span full width at fixed Y)
    pub horizontal_lines: Vec<HorizontalLineOutput>,
    /// Vertical lines (span full height at fixed X)
    pub vertical_lines: Vec<VerticalLineOutput<X>>,
    /// Filled areas
    pub fills: Vec<DrawingFill<X>>,
    /// Text labels
    pub labels: Vec<DrawingLabel<X>>,
    /// Interactive handles (shown when selected)
    pub handles: Vec<DrawingHandle<X>>,
}

impl<X: AxisCoordinate> Default for DrawingOutput<X> {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            horizontal_lines: Vec::new(),
            vertical_lines: Vec::new(),
            fills: Vec::new(),
            labels: Vec::new(),
            handles: Vec::new(),
        }
    }
}

impl<X: AxisCoordinate> DrawingOutput<X> {
    /// Create an empty output
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a line
    pub fn add_line(&mut self, line: DrawingLine<X>) {
        self.lines.push(line);
    }

    /// Add a horizontal line (spans full width)
    pub fn add_horizontal_line(&mut self, line: HorizontalLineOutput) {
        self.horizontal_lines.push(line);
    }

    /// Add a vertical line (spans full height)
    pub fn add_vertical_line(&mut self, line: VerticalLineOutput<X>) {
        self.vertical_lines.push(line);
    }

    /// Add a fill
    pub fn add_fill(&mut self, fill: DrawingFill<X>) {
        self.fills.push(fill);
    }

    /// Add a label
    pub fn add_label(&mut self, label: DrawingLabel<X>) {
        self.labels.push(label);
    }

    /// Add a handle
    pub fn add_handle(&mut self, handle: DrawingHandle<X>) {
        self.handles.push(handle);
    }
}

/// A line or polyline to draw
#[derive(Clone, Debug)]
pub struct DrawingLine<X: AxisCoordinate> {
    /// Points defining the line
    pub points: Vec<ChartPoint<X>>,
    /// Line color
    pub color: Color,
    /// Line width in pixels
    pub width: f32,
    /// Line style
    pub style: LineStyle,
    /// Extend the line infinitely to the left
    pub extend_left: bool,
    /// Extend the line infinitely to the right
    pub extend_right: bool,
}

impl<X: AxisCoordinate> DrawingLine<X> {
    /// Create a new line between two points
    pub fn new(start: ChartPoint<X>, end: ChartPoint<X>, color: Color) -> Self {
        Self {
            points: vec![start, end],
            color,
            width: 1.0,
            style: LineStyle::Solid,
            extend_left: false,
            extend_right: false,
        }
    }

    /// Create a polyline from multiple points
    pub fn polyline(points: Vec<ChartPoint<X>>, color: Color) -> Self {
        Self {
            points,
            color,
            width: 1.0,
            style: LineStyle::Solid,
            extend_left: false,
            extend_right: false,
        }
    }

    /// Set the line width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the line style
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Extend to the right
    pub fn extend_right(mut self) -> Self {
        self.extend_right = true;
        self
    }

    /// Extend to the left
    pub fn extend_left(mut self) -> Self {
        self.extend_left = true;
        self
    }

    /// Extend in both directions
    pub fn extend_both(mut self) -> Self {
        self.extend_left = true;
        self.extend_right = true;
        self
    }
}

/// A horizontal line at a fixed Y value (spans full plot width)
#[derive(Clone, Debug)]
pub struct HorizontalLineOutput {
    /// Y value in data coordinates
    pub y: f64,
    /// Line color
    pub color: Color,
    /// Line width in pixels
    pub width: f32,
    /// Line style
    pub style: LineStyle,
}

impl HorizontalLineOutput {
    /// Create a new horizontal line
    pub fn new(y: f64, color: Color) -> Self {
        Self {
            y,
            color,
            width: 1.0,
            style: LineStyle::Solid,
        }
    }

    /// Set line width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set line style
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }
}

/// A vertical line at a fixed X value (spans full plot height)
#[derive(Clone, Debug)]
pub struct VerticalLineOutput<X: AxisCoordinate> {
    /// X value in data coordinates
    pub x: X,
    /// Line color
    pub color: Color,
    /// Line width in pixels
    pub width: f32,
    /// Line style
    pub style: LineStyle,
}

impl<X: AxisCoordinate> VerticalLineOutput<X> {
    /// Create a new vertical line
    pub fn new(x: X, color: Color) -> Self {
        Self {
            x,
            color,
            width: 1.0,
            style: LineStyle::Solid,
        }
    }

    /// Set line width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set line style
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }
}

/// A filled polygon
#[derive(Clone, Debug)]
pub struct DrawingFill<X: AxisCoordinate> {
    /// Points defining the polygon (will be closed automatically)
    pub points: Vec<ChartPoint<X>>,
    /// Fill color
    pub color: Color,
    /// Fill opacity (0.0 - 1.0)
    pub opacity: f32,
}

impl<X: AxisCoordinate> DrawingFill<X> {
    /// Create a new fill from points
    pub fn new(points: Vec<ChartPoint<X>>, color: Color, opacity: f32) -> Self {
        Self {
            points,
            color,
            opacity,
        }
    }

    /// Create a rectangle fill from two corner points
    pub fn rect(p1: ChartPoint<X>, p2: ChartPoint<X>, color: Color, opacity: f32) -> Self {
        Self {
            points: vec![
                p1.clone(),
                ChartPoint::new(p2.x, p1.y),
                p2.clone(),
                ChartPoint::new(p1.x, p2.y),
            ],
            color,
            opacity,
        }
    }
}

/// Text anchor position
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    #[default]
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// A text label
#[derive(Clone, Debug)]
pub struct DrawingLabel<X: AxisCoordinate> {
    /// Position in data coordinates
    pub position: ChartPoint<X>,
    /// Text content
    pub text: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Text color
    pub color: Color,
    /// Background color (None = transparent)
    pub background: Option<Color>,
    /// Text anchor point
    pub anchor: TextAnchor,
    /// Padding around text (for background)
    pub padding: f32,
}

impl<X: AxisCoordinate> DrawingLabel<X> {
    /// Create a new label
    pub fn new(position: ChartPoint<X>, text: impl Into<String>, color: Color) -> Self {
        Self {
            position,
            text: text.into(),
            font_size: 11.0,
            color,
            background: None,
            anchor: TextAnchor::Center,
            padding: 2.0,
        }
    }

    /// Set the font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the background color
    pub fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set the anchor position
    pub fn with_anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }
}

/// Handle type for interactive editing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleType {
    /// Circular handle (default)
    Circle,
    /// Square handle (for corners)
    Square,
    /// Diamond handle (for midpoints)
    Diamond,
}

/// An interactive handle for editing
#[derive(Clone, Debug)]
pub struct DrawingHandle<X: AxisCoordinate> {
    /// Position in data coordinates
    pub point: ChartPoint<X>,
    /// Handle visual type
    pub handle_type: HandleType,
    /// Index of the anchor point this handle controls
    pub anchor_index: usize,
}

impl<X: AxisCoordinate> DrawingHandle<X> {
    /// Create a new handle
    pub fn new(point: ChartPoint<X>, anchor_index: usize) -> Self {
        Self {
            point,
            handle_type: HandleType::Circle,
            anchor_index,
        }
    }

    /// Create a square handle
    pub fn square(point: ChartPoint<X>, anchor_index: usize) -> Self {
        Self {
            point,
            handle_type: HandleType::Square,
            anchor_index,
        }
    }

    /// Create a diamond handle
    pub fn diamond(point: ChartPoint<X>, anchor_index: usize) -> Self {
        Self {
            point,
            handle_type: HandleType::Diamond,
            anchor_index,
        }
    }
}
