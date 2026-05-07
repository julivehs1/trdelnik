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
