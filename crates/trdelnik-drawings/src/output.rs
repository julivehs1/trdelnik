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

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Color, Index};

    fn cp(x: usize, y: f64) -> ChartPoint<Index> {
        ChartPoint::new(Index(x), y)
    }

    fn black() -> Color {
        Color::rgb(0, 0, 0)
    }

    // ---------- DrawingOutput ----------

    #[test]
    fn test_drawing_output_default_is_empty() {
        let o: DrawingOutput<Index> = DrawingOutput::default();
        assert!(o.lines.is_empty());
        assert!(o.horizontal_lines.is_empty());
        assert!(o.vertical_lines.is_empty());
        assert!(o.fills.is_empty());
        assert!(o.labels.is_empty());
        assert!(o.handles.is_empty());
    }

    #[test]
    fn test_drawing_output_new_matches_default() {
        let a: DrawingOutput<Index> = DrawingOutput::new();
        assert_eq!(a.lines.len(), 0);
        assert_eq!(a.handles.len(), 0);
    }

    #[test]
    fn test_drawing_output_add_methods() {
        let mut o: DrawingOutput<Index> = DrawingOutput::new();
        o.add_line(DrawingLine::new(cp(0, 1.0), cp(1, 2.0), black()));
        o.add_horizontal_line(HorizontalLineOutput::new(50.0, black()));
        o.add_vertical_line(VerticalLineOutput::new(Index(3), black()));
        o.add_fill(DrawingFill::new(vec![cp(0, 0.0), cp(1, 1.0)], black(), 0.3));
        o.add_label(DrawingLabel::new(cp(0, 1.0), "hi", black()));
        o.add_handle(DrawingHandle::new(cp(0, 0.0), 0));

        assert_eq!(o.lines.len(), 1);
        assert_eq!(o.horizontal_lines.len(), 1);
        assert_eq!(o.vertical_lines.len(), 1);
        assert_eq!(o.fills.len(), 1);
        assert_eq!(o.labels.len(), 1);
        assert_eq!(o.handles.len(), 1);
    }

    // ---------- DrawingLine ----------

    #[test]
    fn test_drawing_line_new_two_points_solid() {
        let line = DrawingLine::new(cp(0, 1.0), cp(5, 7.0), black());
        assert_eq!(line.points.len(), 2);
        assert!((line.width - 1.0).abs() < 1e-6);
        assert_eq!(line.style, LineStyle::Solid);
        assert!(!line.extend_left);
        assert!(!line.extend_right);
    }

    #[test]
    fn test_drawing_line_polyline() {
        let pts = vec![cp(0, 0.0), cp(1, 1.0), cp(2, 4.0), cp(3, 9.0)];
        let line = DrawingLine::polyline(pts.clone(), black());
        assert_eq!(line.points.len(), pts.len());
    }

    #[test]
    fn test_drawing_line_builder_methods() {
        let line = DrawingLine::new(cp(0, 0.0), cp(1, 1.0), black())
            .with_width(2.5)
            .with_style(LineStyle::Dashed { dash: 5.0, gap: 3.0 });
        assert!((line.width - 2.5).abs() < 1e-6);
        assert!(matches!(line.style, LineStyle::Dashed { .. }));
    }

    #[test]
    fn test_drawing_line_extend_directions() {
        let l = DrawingLine::new(cp(0, 0.0), cp(1, 1.0), black()).extend_right();
        assert!(l.extend_right);
        assert!(!l.extend_left);

        let l = DrawingLine::new(cp(0, 0.0), cp(1, 1.0), black()).extend_left();
        assert!(l.extend_left);
        assert!(!l.extend_right);

        let l = DrawingLine::new(cp(0, 0.0), cp(1, 1.0), black()).extend_both();
        assert!(l.extend_left);
        assert!(l.extend_right);
    }

    // ---------- HorizontalLineOutput ----------

    #[test]
    fn test_horizontal_line_new_defaults() {
        let h = HorizontalLineOutput::new(42.5, black());
        assert!((h.y - 42.5).abs() < 1e-9);
        assert!((h.width - 1.0).abs() < 1e-6);
        assert_eq!(h.style, LineStyle::Solid);
    }

    #[test]
    fn test_horizontal_line_builder() {
        let h = HorizontalLineOutput::new(0.0, black())
            .with_width(3.0)
            .with_style(LineStyle::Dotted);
        assert!((h.width - 3.0).abs() < 1e-6);
        assert_eq!(h.style, LineStyle::Dotted);
    }

    // ---------- VerticalLineOutput ----------

    #[test]
    fn test_vertical_line_new_defaults() {
        let v: VerticalLineOutput<Index> = VerticalLineOutput::new(Index(5), black());
        assert_eq!(v.x, Index(5));
        assert!((v.width - 1.0).abs() < 1e-6);
        assert_eq!(v.style, LineStyle::Solid);
    }

    #[test]
    fn test_vertical_line_builder() {
        let v = VerticalLineOutput::new(Index(0), black())
            .with_width(2.0)
            .with_style(LineStyle::Dashed { dash: 4.0, gap: 2.0 });
        assert!((v.width - 2.0).abs() < 1e-6);
        assert!(matches!(v.style, LineStyle::Dashed { .. }));
    }

    // ---------- DrawingFill ----------

    #[test]
    fn test_drawing_fill_new() {
        let pts = vec![cp(0, 0.0), cp(1, 0.0), cp(1, 1.0), cp(0, 1.0)];
        let f = DrawingFill::new(pts.clone(), black(), 0.5);
        assert_eq!(f.points.len(), 4);
        assert!((f.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_drawing_fill_rect_creates_4_corners() {
        let p1 = cp(0, 0.0);
        let p2 = cp(10, 5.0);
        let f = DrawingFill::rect(p1.clone(), p2.clone(), black(), 0.25);
        assert_eq!(f.points.len(), 4);
        // The 4 corners cover both x and y ranges.
        let xs: Vec<usize> = f.points.iter().map(|p| p.x.0).collect();
        let ys: Vec<f64> = f.points.iter().map(|p| p.y).collect();
        assert!(xs.contains(&0) && xs.contains(&10));
        assert!(ys.iter().any(|y| (*y - 0.0).abs() < 1e-9));
        assert!(ys.iter().any(|y| (*y - 5.0).abs() < 1e-9));
    }

    // ---------- TextAnchor & DrawingLabel ----------

    #[test]
    fn test_text_anchor_default_is_center() {
        let a: TextAnchor = TextAnchor::default();
        assert_eq!(a, TextAnchor::Center);
    }

    #[test]
    fn test_text_anchor_variants_distinct() {
        // Just ensure equality semantics work for all variants
        assert_ne!(TextAnchor::TopLeft, TextAnchor::TopRight);
        assert_ne!(TextAnchor::BottomLeft, TextAnchor::BottomRight);
        assert_eq!(TextAnchor::MiddleLeft, TextAnchor::MiddleLeft);
    }

    #[test]
    fn test_drawing_label_new_defaults() {
        let l = DrawingLabel::new(cp(0, 0.0), "hello", black());
        assert_eq!(l.text, "hello");
        assert!((l.font_size - 11.0).abs() < 1e-6);
        assert_eq!(l.anchor, TextAnchor::Center);
        assert!(l.background.is_none());
        assert!((l.padding - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_drawing_label_builder() {
        let bg = Color::rgb(50, 50, 50);
        let l = DrawingLabel::new(cp(0, 0.0), "x", black())
            .with_font_size(20.0)
            .with_background(bg)
            .with_anchor(TextAnchor::TopLeft);
        assert!((l.font_size - 20.0).abs() < 1e-6);
        assert_eq!(l.background, Some(bg));
        assert_eq!(l.anchor, TextAnchor::TopLeft);
    }

    // ---------- DrawingHandle ----------

    #[test]
    fn test_drawing_handle_new_is_circle() {
        let h: DrawingHandle<Index> = DrawingHandle::new(cp(0, 0.0), 7);
        assert_eq!(h.handle_type, HandleType::Circle);
        assert_eq!(h.anchor_index, 7);
    }

    #[test]
    fn test_drawing_handle_square() {
        let h = DrawingHandle::square(cp(0, 0.0), 1);
        assert_eq!(h.handle_type, HandleType::Square);
        assert_eq!(h.anchor_index, 1);
    }

    #[test]
    fn test_drawing_handle_diamond() {
        let h = DrawingHandle::diamond(cp(0, 0.0), 2);
        assert_eq!(h.handle_type, HandleType::Diamond);
        assert_eq!(h.anchor_index, 2);
    }

    #[test]
    fn test_handle_type_variants_distinct() {
        assert_ne!(HandleType::Circle, HandleType::Square);
        assert_ne!(HandleType::Square, HandleType::Diamond);
    }
}
