//! Building blocks for chart-renderable indicator data.
//!
//! These types are the leaf data used by `PlotData` and `Panel`:
//! lines, markers, and histogram bars. The container types
//! (`PlotData`, `Panel`) live in `plot_data.rs` and `trdelnik-data`.

use crate::axis::AxisCoordinate;
use crate::color::Color;

/// Which Y-axis the indicator data should use
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum YAxis {
    /// Left Y-axis (primary)
    #[default]
    Left,
    /// Right Y-axis (secondary, for dual-axis panels)
    Right,
}

/// A single line of indicator data
#[derive(Debug, Clone)]
pub struct IndicatorLine<X: AxisCoordinate> {
    /// Display name of the line (e.g., "SMA 20", "%K")
    pub name: String,
    /// Identifier for theming (e.g., "sma", "stoch_k")
    pub line_id: String,
    /// Points as (x, y) pairs - None values create gaps
    pub points: Vec<(X, Option<f64>)>,
    /// Which Y-axis this line uses
    pub axis: YAxis,
    /// Optional override color (if not set, uses theme color based on line_id)
    pub color: Option<Color>,
}

impl<X: AxisCoordinate> IndicatorLine<X> {
    /// Create a new indicator line
    pub fn new(name: impl Into<String>, line_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            line_id: line_id.into(),
            points: Vec::new(),
            axis: YAxis::Left,
            color: None,
        }
    }

    /// Create a line with specified axis
    pub fn with_axis(mut self, axis: YAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Set a custom color for this line
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Create from x values and y values
    pub fn from_xy(
        name: impl Into<String>,
        line_id: impl Into<String>,
        x_values: &[X],
        y_values: &[Option<f64>],
    ) -> Self {
        let points = x_values
            .iter()
            .zip(y_values.iter())
            .map(|(&x, &y)| (x, y))
            .collect();

        Self {
            name: name.into(),
            line_id: line_id.into(),
            points,
            axis: YAxis::Left,
            color: None,
        }
    }

    /// Create from x values and y values with specified axis
    pub fn from_xy_with_axis(
        name: impl Into<String>,
        line_id: impl Into<String>,
        x_values: &[X],
        y_values: &[Option<f64>],
        axis: YAxis,
    ) -> Self {
        let mut line = Self::from_xy(name, line_id, x_values, y_values);
        line.axis = axis;
        line
    }

    /// Add a point
    pub fn push(&mut self, x: X, y: Option<f64>) {
        self.points.push((x, y));
    }

    /// Get points as plot-ready values
    pub fn plot_points(&self) -> impl Iterator<Item = Option<[f64; 2]>> + '_ {
        self.points.iter().map(|(x, y)| {
            y.map(|y_val| [x.to_plot_value(), y_val])
        })
    }

    /// Get only valid (non-None) points as plot values
    pub fn valid_plot_points(&self) -> Vec<[f64; 2]> {
        self.points
            .iter()
            .filter_map(|(x, y)| y.map(|y_val| [x.to_plot_value(), y_val]))
            .collect()
    }
}

/// Shape of a marker on the chart
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerShape {
    /// Triangle pointing up (buy signal)
    ArrowUp,
    /// Triangle pointing down (sell signal)
    ArrowDown,
    /// X mark (exit signal)
    Cross,
    /// Circle
    Circle,
    /// Square
    Square,
}

/// A marker/point on the chart (for signals, annotations, etc.)
#[derive(Debug, Clone)]
pub struct IndicatorMarker<X: AxisCoordinate> {
    /// X coordinate
    pub x: X,
    /// Y coordinate (price level)
    pub y: f64,
    /// Marker shape
    pub shape: MarkerShape,
    /// Marker color
    pub color: Color,
    /// Optional label
    pub label: Option<String>,
}

impl<X: AxisCoordinate> IndicatorMarker<X> {
    /// Create an upward arrow marker (buy signal)
    pub fn arrow_up(x: X, y: f64, color: Color) -> Self {
        Self {
            x,
            y,
            shape: MarkerShape::ArrowUp,
            color,
            label: None,
        }
    }

    /// Create a downward arrow marker (sell signal)
    pub fn arrow_down(x: X, y: f64, color: Color) -> Self {
        Self {
            x,
            y,
            shape: MarkerShape::ArrowDown,
            color,
            label: None,
        }
    }

    /// Create a cross marker (exit signal)
    pub fn cross(x: X, y: f64, color: Color) -> Self {
        Self {
            x,
            y,
            shape: MarkerShape::Cross,
            color,
            label: None,
        }
    }

    /// Create a circle marker
    pub fn circle(x: X, y: f64, color: Color) -> Self {
        Self {
            x,
            y,
            shape: MarkerShape::Circle,
            color,
            label: None,
        }
    }

    /// Add a label to the marker
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A histogram bar
#[derive(Debug, Clone)]
pub struct HistogramBar<X: AxisCoordinate> {
    /// X coordinate
    pub x: X,
    /// Value (height of bar, can be negative)
    pub value: f64,
    /// Which Y-axis this bar uses
    pub axis: YAxis,
    /// Color of this bar - set by the indicator
    pub color: Color,
}

impl<X: AxisCoordinate> HistogramBar<X> {
    /// Create a new histogram bar with a color
    pub fn new(x: X, value: f64, color: Color) -> Self {
        Self {
            x,
            value,
            axis: YAxis::Left,
            color,
        }
    }

    /// Create a histogram bar with specified axis
    pub fn with_axis(mut self, axis: YAxis) -> Self {
        self.axis = axis;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::Index;

    fn red() -> Color {
        Color::rgb(255, 0, 0)
    }

    // ---------- YAxis ----------

    #[test]
    fn test_yaxis_default_is_left() {
        let a: YAxis = Default::default();
        assert_eq!(a, YAxis::Left);
        assert_ne!(a, YAxis::Right);
    }

    // ---------- IndicatorLine ----------

    #[test]
    fn test_indicator_line_new_defaults() {
        let line: IndicatorLine<Index> = IndicatorLine::new("SMA 20", "sma");
        assert_eq!(line.name, "SMA 20");
        assert_eq!(line.line_id, "sma");
        assert!(line.points.is_empty());
        assert_eq!(line.axis, YAxis::Left);
        assert!(line.color.is_none());
    }

    #[test]
    fn test_indicator_line_builders() {
        let line: IndicatorLine<Index> = IndicatorLine::new("X", "x")
            .with_axis(YAxis::Right)
            .with_color(red());
        assert_eq!(line.axis, YAxis::Right);
        assert_eq!(line.color, Some(red()));
    }

    #[test]
    fn test_indicator_line_from_xy_zips_pairs() {
        let xs = [Index(0), Index(1), Index(2)];
        let ys = [Some(10.0), None, Some(30.0)];
        let line = IndicatorLine::from_xy("RSI", "rsi", &xs, &ys);
        assert_eq!(line.points.len(), 3);
        assert_eq!(line.points[0], (Index(0), Some(10.0)));
        assert_eq!(line.points[1], (Index(1), None));
        assert_eq!(line.points[2], (Index(2), Some(30.0)));
        // from_xy uses default Left axis
        assert_eq!(line.axis, YAxis::Left);
    }

    #[test]
    fn test_indicator_line_from_xy_truncates_to_shorter_input() {
        let xs = [Index(0), Index(1), Index(2)];
        let ys = [Some(1.0), Some(2.0)];
        let line = IndicatorLine::from_xy("L", "l", &xs, &ys);
        assert_eq!(line.points.len(), 2);
    }

    #[test]
    fn test_indicator_line_from_xy_with_axis_sets_axis() {
        let xs = [Index(0)];
        let ys = [Some(5.0)];
        let line = IndicatorLine::from_xy_with_axis("X", "x", &xs, &ys, YAxis::Right);
        assert_eq!(line.axis, YAxis::Right);
    }

    #[test]
    fn test_indicator_line_push_appends() {
        let mut line: IndicatorLine<Index> = IndicatorLine::new("L", "l");
        line.push(Index(0), Some(1.0));
        line.push(Index(1), None);
        line.push(Index(2), Some(3.0));
        assert_eq!(line.points.len(), 3);
        assert_eq!(line.points[2], (Index(2), Some(3.0)));
    }

    #[test]
    fn test_indicator_line_plot_points_yields_options() {
        let xs = [Index(0), Index(1), Index(2)];
        let ys = [Some(10.0), None, Some(30.0)];
        let line = IndicatorLine::from_xy("L", "l", &xs, &ys);
        let p: Vec<Option<[f64; 2]>> = line.plot_points().collect();
        assert_eq!(p.len(), 3);
        assert_eq!(p[0], Some([0.0, 10.0]));
        assert_eq!(p[1], None);
        assert_eq!(p[2], Some([2.0, 30.0]));
    }

    #[test]
    fn test_indicator_line_valid_plot_points_drops_none() {
        let xs = [Index(0), Index(1), Index(2)];
        let ys = [Some(10.0), None, Some(30.0)];
        let line = IndicatorLine::from_xy("L", "l", &xs, &ys);
        let p = line.valid_plot_points();
        assert_eq!(p, vec![[0.0, 10.0], [2.0, 30.0]]);
    }

    #[test]
    fn test_indicator_line_valid_plot_points_empty_when_all_none() {
        let xs = [Index(0)];
        let ys = [None];
        let line: IndicatorLine<Index> = IndicatorLine::from_xy("L", "l", &xs, &ys);
        assert!(line.valid_plot_points().is_empty());
    }

    // ---------- IndicatorMarker ----------

    #[test]
    fn test_marker_arrow_up() {
        let m = IndicatorMarker::arrow_up(Index(5), 100.0, red());
        assert_eq!(m.x, Index(5));
        assert!((m.y - 100.0).abs() < 1e-9);
        assert_eq!(m.shape, MarkerShape::ArrowUp);
        assert_eq!(m.color, red());
        assert!(m.label.is_none());
    }

    #[test]
    fn test_marker_arrow_down() {
        let m = IndicatorMarker::arrow_down(Index(0), 1.0, red());
        assert_eq!(m.shape, MarkerShape::ArrowDown);
    }

    #[test]
    fn test_marker_cross() {
        let m = IndicatorMarker::cross(Index(0), 1.0, red());
        assert_eq!(m.shape, MarkerShape::Cross);
    }

    #[test]
    fn test_marker_circle() {
        let m = IndicatorMarker::circle(Index(0), 1.0, red());
        assert_eq!(m.shape, MarkerShape::Circle);
    }

    #[test]
    fn test_marker_with_label() {
        let m = IndicatorMarker::arrow_up(Index(0), 1.0, red()).with_label("Buy");
        assert_eq!(m.label.as_deref(), Some("Buy"));
    }

    #[test]
    fn test_marker_shape_variants_distinct() {
        assert_ne!(MarkerShape::ArrowUp, MarkerShape::ArrowDown);
        assert_ne!(MarkerShape::Circle, MarkerShape::Square);
        assert_ne!(MarkerShape::Cross, MarkerShape::Square);
    }

    // ---------- HistogramBar ----------

    #[test]
    fn test_histogram_bar_new_defaults() {
        let bar = HistogramBar::new(Index(3), -2.5, red());
        assert_eq!(bar.x, Index(3));
        assert!((bar.value - (-2.5)).abs() < 1e-9);
        assert_eq!(bar.axis, YAxis::Left);
        assert_eq!(bar.color, red());
    }

    #[test]
    fn test_histogram_bar_with_axis() {
        let bar = HistogramBar::new(Index(0), 1.0, red()).with_axis(YAxis::Right);
        assert_eq!(bar.axis, YAxis::Right);
    }
}
