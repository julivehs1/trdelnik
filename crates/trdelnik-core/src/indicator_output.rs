//! Unified indicator output types
//!
//! These types represent the output of any indicator, regardless of where
//! it will be rendered (overlay on main chart or in a separate panel).

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

/// Default placement for an indicator
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Placement {
    /// Render as overlay on the main price chart
    Overlay,
    /// Render in a separate panel
    #[default]
    Panel,
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

/// Unified output from any indicator
///
/// This can represent both overlay indicators (like SMA, Bollinger Bands)
/// and panel indicators (like RSI, MACD).
#[derive(Debug, Clone)]
pub struct IndicatorOutput<X: AxisCoordinate> {
    /// Display name of the indicator (e.g., "RSI 14", "SMA 20")
    pub name: String,
    /// Identifier for the indicator (e.g., "rsi", "sma")
    pub indicator_id: String,
    /// Lines to draw
    pub lines: Vec<IndicatorLine<X>>,
    /// Histogram bars (e.g., MACD histogram)
    pub histogram: Option<Vec<HistogramBar<X>>>,
    /// Markers (e.g., buy/sell signals)
    pub markers: Vec<IndicatorMarker<X>>,
    /// Reference lines (e.g., 30/70 for RSI, 0 for MACD)
    pub reference_lines: Vec<f64>,
    /// Fixed Y-axis range, if any (e.g., 0-100 for RSI)
    pub y_range: Option<(f64, f64)>,
    /// Default placement for this indicator
    pub default_placement: Placement,
}

impl<X: AxisCoordinate> IndicatorOutput<X> {
    /// Create a new indicator output
    pub fn new(
        name: impl Into<String>,
        indicator_id: impl Into<String>,
        placement: Placement,
    ) -> Self {
        Self {
            name: name.into(),
            indicator_id: indicator_id.into(),
            lines: Vec::new(),
            histogram: None,
            markers: Vec::new(),
            reference_lines: Vec::new(),
            y_range: None,
            default_placement: placement,
        }
    }

    /// Create an overlay indicator output
    pub fn overlay(name: impl Into<String>, indicator_id: impl Into<String>) -> Self {
        Self::new(name, indicator_id, Placement::Overlay)
    }

    /// Create a panel indicator output
    pub fn panel(name: impl Into<String>, indicator_id: impl Into<String>) -> Self {
        Self::new(name, indicator_id, Placement::Panel)
    }

    /// Create a single-line overlay indicator
    pub fn single_overlay(
        name: impl Into<String>,
        line_id: impl Into<String>,
        x_values: &[X],
        y_values: &[Option<f64>],
    ) -> Self {
        let name_str = name.into();
        let mut output = Self::overlay(name_str.clone(), line_id.into());
        output.add_line(IndicatorLine::from_xy(&name_str, &output.indicator_id, x_values, y_values));
        output
    }

    /// Add a line to the output
    pub fn add_line(&mut self, line: IndicatorLine<X>) {
        self.lines.push(line);
    }

    /// Add a marker to the output
    pub fn add_marker(&mut self, marker: IndicatorMarker<X>) {
        self.markers.push(marker);
    }

    /// Add multiple markers to the output
    pub fn add_markers(&mut self, markers: impl IntoIterator<Item = IndicatorMarker<X>>) {
        self.markers.extend(markers);
    }

    /// Set histogram data with per-bar colors
    pub fn set_histogram(&mut self, x_values: &[X], values: &[Option<f64>], colors: &[Color]) {
        self.histogram = Some(
            x_values
                .iter()
                .zip(values.iter())
                .zip(colors.iter())
                .filter_map(|((&x, &v), &color)| {
                    v.map(|value| HistogramBar::new(x, value, color))
                })
                .collect(),
        );
    }

    /// Set histogram data with a single color for all bars
    pub fn set_histogram_uniform(&mut self, x_values: &[X], values: &[Option<f64>], color: Color) {
        self.histogram = Some(
            x_values
                .iter()
                .zip(values.iter())
                .filter_map(|(&x, &v)| {
                    v.map(|value| HistogramBar::new(x, value, color))
                })
                .collect(),
        );
    }

    /// Set histogram bars directly
    pub fn set_histogram_bars(&mut self, bars: Vec<HistogramBar<X>>) {
        self.histogram = Some(bars);
    }

    /// Set histogram data with positive/negative coloring (e.g., for MACD)
    pub fn set_histogram_pos_neg(
        &mut self,
        x_values: &[X],
        values: &[Option<f64>],
        positive_color: Color,
        negative_color: Color,
    ) {
        self.histogram = Some(
            x_values
                .iter()
                .zip(values.iter())
                .filter_map(|(&x, &v)| {
                    v.map(|value| {
                        let color = if value >= 0.0 {
                            positive_color
                        } else {
                            negative_color
                        };
                        HistogramBar::new(x, value, color)
                    })
                })
                .collect(),
        );
    }

    /// Set reference lines
    pub fn set_reference_lines(&mut self, lines: Vec<f64>) {
        self.reference_lines = lines;
    }

    /// Set fixed Y range
    pub fn set_y_range(&mut self, range: (f64, f64)) {
        self.y_range = Some(range);
    }

    /// Check if this output has any right-axis data
    pub fn has_right_axis(&self) -> bool {
        self.lines.iter().any(|l| l.axis == YAxis::Right)
            || self
                .histogram
                .as_ref()
                .map(|h| h.iter().any(|b| b.axis == YAxis::Right))
                .unwrap_or(false)
    }

    /// Get lines for the left axis
    pub fn left_lines(&self) -> impl Iterator<Item = &IndicatorLine<X>> {
        self.lines.iter().filter(|l| l.axis == YAxis::Left)
    }

    /// Get lines for the right axis
    pub fn right_lines(&self) -> impl Iterator<Item = &IndicatorLine<X>> {
        self.lines.iter().filter(|l| l.axis == YAxis::Right)
    }

    /// Calculate Y range from left-axis data
    pub fn left_y_range(&self) -> (f64, f64) {
        if let Some(range) = self.y_range {
            return range;
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for line in self.left_lines() {
            for (_, y) in &line.points {
                if let Some(y) = y {
                    min = min.min(*y);
                    max = max.max(*y);
                }
            }
        }

        if let Some(histogram) = &self.histogram {
            for bar in histogram.iter().filter(|b| b.axis == YAxis::Left) {
                min = min.min(bar.value);
                max = max.max(bar.value);
            }
        }

        if min.is_infinite() {
            min = 0.0;
        }
        if max.is_infinite() {
            max = 100.0;
        }

        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    /// Calculate Y range from right-axis data
    pub fn right_y_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for line in self.right_lines() {
            for (_, y) in &line.points {
                if let Some(y) = y {
                    min = min.min(*y);
                    max = max.max(*y);
                }
            }
        }

        if let Some(histogram) = &self.histogram {
            for bar in histogram.iter().filter(|b| b.axis == YAxis::Right) {
                min = min.min(bar.value);
                max = max.max(bar.value);
            }
        }

        if min.is_infinite() {
            min = 0.0;
        }
        if max.is_infinite() {
            max = 100.0;
        }

        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    /// Calculate combined Y range (for single-axis mode)
    pub fn calculate_y_range(&self) -> (f64, f64) {
        if let Some(range) = self.y_range {
            return range;
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for line in &self.lines {
            for (_, y) in &line.points {
                if let Some(y) = y {
                    min = min.min(*y);
                    max = max.max(*y);
                }
            }
        }

        if let Some(histogram) = &self.histogram {
            for bar in histogram {
                min = min.min(bar.value);
                max = max.max(bar.value);
            }
        }

        if min.is_infinite() {
            min = 0.0;
        }
        if max.is_infinite() {
            max = 100.0;
        }

        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

}
