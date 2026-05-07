//! Plot data types for the matplotlib-like chart API.
//!
//! `PlotData` is the pure-data return value of `Plottable::plot()`.
//! It contains only renderable data — no panel configuration like
//! reference lines, y-range or placement hints.

use crate::axis::AxisCoordinate;
use crate::color::Color;
use crate::indicator_output::{HistogramBar, IndicatorLine};

/// Pure data output from a plottable indicator.
///
/// Contains only lines and histogram bars — no panel configuration.
/// Markers, hlines, y_range, height live on the `Panel` (or
/// `ChartData` for the main chart).
///
/// Display names live on each `IndicatorLine`. The `indicator_id`
/// is a group-hint for future theming/legend use.
#[derive(Debug, Clone)]
pub struct PlotData<X: AxisCoordinate> {
    /// Identifier for theming and grouping (e.g. "rsi", "sma", "bollinger")
    pub indicator_id: String,
    /// Lines to draw
    pub lines: Vec<IndicatorLine<X>>,
    /// Histogram bars (e.g. MACD histogram)
    pub histogram: Option<Vec<HistogramBar<X>>>,
}

impl<X: AxisCoordinate> PlotData<X> {
    /// Create a new empty PlotData
    pub fn new(indicator_id: impl Into<String>) -> Self {
        Self {
            indicator_id: indicator_id.into(),
            lines: Vec::new(),
            histogram: None,
        }
    }

    /// Add a line
    pub fn add_line(&mut self, line: IndicatorLine<X>) {
        self.lines.push(line);
    }

    /// Set histogram bars directly
    pub fn set_histogram_bars(&mut self, bars: Vec<HistogramBar<X>>) {
        self.histogram = Some(bars);
    }

    /// Set histogram data with positive/negative coloring
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

    /// Check if this output has any right-axis data
    pub fn has_right_axis(&self) -> bool {
        use crate::indicator_output::YAxis;
        self.lines.iter().any(|l| l.axis == YAxis::Right)
            || self
                .histogram
                .as_ref()
                .map(|h| h.iter().any(|b| b.axis == YAxis::Right))
                .unwrap_or(false)
    }

    /// Get lines for the left axis
    pub fn left_lines(&self) -> impl Iterator<Item = &IndicatorLine<X>> {
        use crate::indicator_output::YAxis;
        self.lines.iter().filter(|l| l.axis == YAxis::Left)
    }

    /// Get lines for the right axis
    pub fn right_lines(&self) -> impl Iterator<Item = &IndicatorLine<X>> {
        use crate::indicator_output::YAxis;
        self.lines.iter().filter(|l| l.axis == YAxis::Right)
    }

    /// Calculate Y range from all data
    pub fn calculate_y_range(&self) -> (f64, f64) {
        let line_values = self.lines.iter().flat_map(|l| {
            l.points.iter().filter_map(|(_, y)| *y)
        });
        let hist_values = self.histogram.iter().flatten().map(|b| b.value);
        y_range_with_padding(line_values.chain(hist_values))
    }
}

/// Compute (min, max) over an iterator of f64 values, with 10% padding.
///
/// Falls back to (0.0, 100.0) when the iterator is empty.
pub fn y_range_with_padding<I: IntoIterator<Item = f64>>(values: I) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in values {
        min = min.min(v);
        max = max.max(v);
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

/// Horizontal reference line.
///
/// Lives on the `Panel`, not inside indicator data.
#[derive(Debug, Clone, Copy)]
pub struct HLine {
    /// Y-axis level
    pub level: f64,
    /// Optional color override (if None, uses theme default)
    pub color: Option<Color>,
    /// Line style
    pub style: HLineStyle,
}

/// Style for horizontal reference lines
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HLineStyle {
    /// Solid line (used for zero lines)
    Solid,
    /// Dashed line (default for reference levels)
    #[default]
    Dashed,
}

impl HLine {
    /// Create a new dashed hline at the given level
    pub fn new(level: f64) -> Self {
        Self {
            level,
            color: None,
            style: if level == 0.0 {
                HLineStyle::Solid
            } else {
                HLineStyle::Dashed
            },
        }
    }

    /// Create an hline with a custom color
    pub fn colored(level: f64, color: Color) -> Self {
        Self {
            level,
            color: Some(color),
            style: if level == 0.0 {
                HLineStyle::Solid
            } else {
                HLineStyle::Dashed
            },
        }
    }

    /// Set the style
    pub fn with_style(mut self, style: HLineStyle) -> Self {
        self.style = style;
        self
    }
}

impl From<f64> for HLine {
    fn from(level: f64) -> Self {
        Self::new(level)
    }
}
