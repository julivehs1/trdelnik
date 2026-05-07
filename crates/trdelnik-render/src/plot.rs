//! `Plot` trait, `PlotContext`, and built-in plot types.

use std::any::Any;
use std::fmt::Debug;

use trdelnik_core::{AxisCoordinate, Color, HistogramBar, IndicatorLine, YAxis};

use crate::renderer::Renderer;
use crate::style::{LineStyle, Stroke};
use crate::theme::IndicatorTheme;

/// Bundle of `Renderer` + `IndicatorTheme` passed into `Plot::render`.
///
/// Held by reference so plots can borrow it for the duration of one render
/// call. Backends construct this once per panel and hand it to each plot.
pub struct PlotContext<'a, X: AxisCoordinate> {
    /// Drawing primitives.
    pub renderer: &'a mut dyn Renderer<X>,
    /// Theme lookups.
    pub theme: &'a dyn IndicatorTheme,
}

impl<'a, X: AxisCoordinate> PlotContext<'a, X> {
    /// Construct a new context.
    pub fn new(renderer: &'a mut dyn Renderer<X>, theme: &'a dyn IndicatorTheme) -> Self {
        Self { renderer, theme }
    }
}

/// A plottable visualisation that lives inside a `Panel`.
///
/// Implementors describe their data on demand and render themselves
/// through a `PlotContext`. Built-in: [`StandardPlot`] (lines + histogram).
/// To add a new visualisation type (heatmap, volume profile, …) define a
/// struct in any crate and implement this trait — no core changes needed.
pub trait Plot<X: AxisCoordinate>: Send + Sync + Debug + 'static {
    /// Identifier used for theming and grouping (e.g. `"rsi"`, `"sma"`).
    fn indicator_id(&self) -> &str;

    /// Whether this plot has any data on the given axis.
    fn has_axis(&self, axis: YAxis) -> bool;

    /// Numeric range covered on the given axis, or `None` when the plot
    /// has no data on it. The `Panel` aggregates these across plots to
    /// derive the final axis range.
    fn y_range(&self, axis: YAxis) -> Option<(f64, f64)>;

    /// Draw this plot through the given context.
    fn render(&self, ctx: &mut PlotContext<'_, X>);

    /// Lines this plot exposes for built-in tooltips/legends. Default:
    /// empty. Plots that draw lines via `Renderer::draw_polyline` should
    /// also expose them here so tooltips can find them.
    fn lines(&self) -> &[IndicatorLine<X>] {
        &[]
    }

    /// Histogram bars this plot exposes. Default: none.
    fn histogram(&self) -> Option<&[HistogramBar<X>]> {
        None
    }

    /// Escape hatch for renderers that need to downcast to a concrete plot
    /// type (e.g. for backend-specific rendering paths).
    fn as_any(&self) -> &dyn Any;

    /// Mutable escape hatch (used e.g. to retarget a plot to the right axis).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Built-in plot type for the classical "lines + optional histogram" shape.
///
/// Used by every indicator in `trdelnik-indicators`. New visualisation
/// types live in their own structs that implement [`Plot`] directly.
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

    fn render(&self, ctx: &mut PlotContext<'_, X>) {
        // Histogram first so bars sit behind lines.
        if let Some(bars) = self.histogram.as_ref() {
            // Group consecutive bars sharing a colour for a single batched draw.
            // The renderer is free to cache; in practice histograms are usually
            // two colours (pos/neg) so this stays cheap.
            let mut current_color: Option<Color> = None;
            let mut batch: Vec<(X, f64)> = Vec::new();
            let bar_width = bar_spacing(ctx.renderer.transform());
            for bar in bars {
                let value = ctx.renderer.map_axis_value(bar.axis, bar.value);
                if Some(bar.color) != current_color {
                    if let Some(color) = current_color.take() {
                        if !batch.is_empty() {
                            ctx.renderer
                                .draw_bars("histogram", &batch, bar_width, color);
                            batch.clear();
                        }
                    }
                    current_color = Some(bar.color);
                }
                batch.push((bar.x, value));
            }
            if let (Some(color), false) = (current_color, batch.is_empty()) {
                ctx.renderer
                    .draw_bars("histogram", &batch, bar_width, color);
            }
        }

        // Lines.
        for line in &self.lines {
            let color = line
                .color
                .unwrap_or_else(|| ctx.theme.line_color(&line.line_id));
            let stroke = Stroke {
                color,
                width: 1.5,
                style: LineStyle::Solid,
            };
            // Re-key (x, Option<f64>) onto the renderer's axis.
            let mapped: Vec<(X, Option<f64>)> = line
                .points
                .iter()
                .map(|(x, y)| (*x, y.map(|v| ctx.renderer.map_axis_value(line.axis, v))))
                .collect();
            ctx.renderer.draw_polyline(&line.name, &mapped, stroke);
        }
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

/// Default bar width derived from the visible X range, used as a fallback
/// when no explicit width is supplied. Backends may pick something better.
fn bar_spacing(t: crate::transform::Transform) -> f64 {
    // Heuristic: 1/120 of the visible width — tight enough to look like
    // candle width on typical viewports.
    let w = t.bounds.width().max(1.0);
    w / 120.0
}

/// Compute `(min, max)` with 10% padding. Falls back to `(0.0, 100.0)` when
/// the iterator is empty.
pub fn y_range_with_padding<I: IntoIterator<Item = f64>>(values: I) -> (f64, f64) {
    let (min, max) = finite_range(values).unwrap_or((0.0, 100.0));
    let padding = (max - min) * 0.1;
    (min - padding, max + padding)
}

/// Aggregate a list of (min, max) ranges into the outer range. Returns
/// `None` when none are present.
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
