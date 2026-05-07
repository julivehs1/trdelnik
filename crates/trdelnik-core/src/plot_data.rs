//! `Plot` trait and built-in plot types for the matplotlib-like chart API.
//!
//! `Plot<X>` is the extension point for visualisations: anything implementing
//! it can be added to a `Panel`. The built-in [`StandardPlot`] covers the
//! "lines + optional histogram" shape used by every classical indicator. To
//! add a new visualisation type (heatmap, volume profile, footprint chart,
//! etc.), define a struct in any crate and implement [`Plot`] for it — no
//! changes to core are needed.

use std::any::Any;
use std::fmt::Debug;

use crate::axis::AxisCoordinate;
use crate::color::Color;
use crate::indicator_output::{HistogramBar, IndicatorLine, YAxis};

/// A plottable visualisation that lives inside a `Panel`.
///
/// Implementors describe their data on demand: which Y-axes they use, the
/// numeric range on each axis, and (for the built-in renderer) any lines or
/// histogram bars they want drawn. Custom visualisations that need their own
/// rendering path use [`Plot::as_any`] for downcasting in the renderer.
pub trait Plot<X: AxisCoordinate>: Send + Sync + Debug + 'static {
    /// Identifier used for theming and grouping (e.g. `"rsi"`, `"sma"`).
    fn indicator_id(&self) -> &str;

    /// Whether this plot has any data on the given axis.
    fn has_axis(&self, axis: YAxis) -> bool;

    /// Numeric range covered on the given axis, or `None` if this plot has
    /// no data on it. The `Panel` aggregates across plots to derive the
    /// final axis range.
    fn y_range(&self, axis: YAxis) -> Option<(f64, f64)>;

    /// Lines this plot wants the built-in renderer to draw. Default: empty.
    fn lines(&self) -> &[IndicatorLine<X>] {
        &[]
    }

    /// Histogram bars this plot wants the built-in renderer to draw.
    /// Default: none.
    fn histogram(&self) -> Option<&[HistogramBar<X>]> {
        None
    }

    /// Escape hatch for renderers that need to downcast to a concrete plot
    /// type (custom visualisations beyond lines + histogram).
    fn as_any(&self) -> &dyn Any;

    /// Mutable escape hatch (used e.g. to retarget a plot to the right axis).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Built-in plot type for the classical "lines + optional histogram" shape.
///
/// Used by every indicator in `trdelnik-indicators`. New visualisation types
/// (heatmaps, volume profiles, etc.) live in their own structs that
/// implement [`Plot`] directly.
#[derive(Debug, Clone)]
pub struct StandardPlot<X: AxisCoordinate> {
    indicator_id: String,
    lines: Vec<IndicatorLine<X>>,
    histogram: Option<Vec<HistogramBar<X>>>,
}

impl<X: AxisCoordinate> StandardPlot<X> {
    /// Create an empty `StandardPlot` with the given indicator id.
    pub fn new(indicator_id: impl Into<String>) -> Self {
        Self {
            indicator_id: indicator_id.into(),
            lines: Vec::new(),
            histogram: None,
        }
    }

    /// Append a line.
    pub fn add_line(&mut self, line: IndicatorLine<X>) {
        self.lines.push(line);
    }

    /// Replace all histogram bars.
    pub fn set_histogram_bars(&mut self, bars: Vec<HistogramBar<X>>) {
        self.histogram = Some(bars);
    }

    /// Build histogram bars from a values slice, colouring positives and
    /// negatives differently. Convenience for indicators like MACD/PPO.
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

    /// Lines for the left axis.
    pub fn left_lines(&self) -> impl Iterator<Item = &IndicatorLine<X>> {
        self.lines.iter().filter(|l| l.axis == YAxis::Left)
    }

    /// Lines for the right axis.
    pub fn right_lines(&self) -> impl Iterator<Item = &IndicatorLine<X>> {
        self.lines.iter().filter(|l| l.axis == YAxis::Right)
    }

    /// Move every line and histogram bar onto the given axis.
    pub fn move_to_axis(&mut self, axis: YAxis) {
        for line in &mut self.lines {
            line.axis = axis;
        }
        if let Some(bars) = self.histogram.as_mut() {
            for bar in bars {
                bar.axis = axis;
            }
        }
    }
}

impl<X: AxisCoordinate> Plot<X> for StandardPlot<X> {
    fn indicator_id(&self) -> &str {
        &self.indicator_id
    }

    fn has_axis(&self, axis: YAxis) -> bool {
        self.lines.iter().any(|l| l.axis == axis)
            || self
                .histogram
                .as_ref()
                .map(|h| h.iter().any(|b| b.axis == axis))
                .unwrap_or(false)
    }

    fn y_range(&self, axis: YAxis) -> Option<(f64, f64)> {
        let line_values = self
            .lines
            .iter()
            .filter(|l| l.axis == axis)
            .flat_map(|l| l.points.iter().filter_map(|(_, y)| *y));
        let hist_values = self
            .histogram
            .iter()
            .flatten()
            .filter(|b| b.axis == axis)
            .map(|b| b.value);
        finite_range(line_values.chain(hist_values))
    }

    fn lines(&self) -> &[IndicatorLine<X>] {
        &self.lines
    }

    fn histogram(&self) -> Option<&[HistogramBar<X>]> {
        self.histogram.as_deref()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Compute `(min, max)` over an iterator. Returns `None` when empty.
fn finite_range<I: IntoIterator<Item = f64>>(values: I) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut any = false;
    for v in values {
        any = true;
        min = min.min(v);
        max = max.max(v);
    }
    if any {
        Some((min, max))
    } else {
        None
    }
}

/// Compute `(min, max)` with 10% padding. Falls back to `(0.0, 100.0)` when
/// the iterator is empty.
pub fn y_range_with_padding<I: IntoIterator<Item = f64>>(values: I) -> (f64, f64) {
    let (min, max) = finite_range(values).unwrap_or((0.0, 100.0));
    let padding = (max - min) * 0.1;
    (min - padding, max + padding)
}

/// Aggregate a list of optional ranges into the outer (min, max). Returns
/// `None` when none of the inputs are present.
pub fn aggregate_ranges<I: IntoIterator<Item = (f64, f64)>>(ranges: I) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut any = false;
    for (lo, hi) in ranges {
        any = true;
        min = min.min(lo);
        max = max.max(hi);
    }
    if any {
        Some((min, max))
    } else {
        None
    }
}

/// Horizontal reference line. Lives on the `Panel`, not inside plot data.
#[derive(Debug, Clone, Copy)]
pub struct HLine {
    /// Y-axis level.
    pub level: f64,
    /// Optional color override (theme default when `None`).
    pub color: Option<Color>,
    /// Line style.
    pub style: HLineStyle,
}

/// Style for horizontal reference lines.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HLineStyle {
    /// Solid line (used for zero lines).
    Solid,
    /// Dashed line (default for reference levels).
    #[default]
    Dashed,
}

impl HLine {
    /// New dashed hline at the given level (solid for `0.0`).
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

    /// New hline with a custom colour.
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

    /// Override the style.
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
