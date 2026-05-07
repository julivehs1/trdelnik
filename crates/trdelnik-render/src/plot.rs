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

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Index, MarkerShape};

    use crate::transform::{Bounds2D, Transform};

    // ---------- Test fixtures: mock Renderer + Theme ----------

    /// Recording renderer — every draw call is appended to `events` so we
    /// can assert exactly what `Plot::render` produced.
    struct MockRenderer {
        transform: Transform,
        right_axis: bool,
        scale_right: f64,
        events: Vec<MockEvent>,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                transform: Transform::new(Bounds2D::new((0.0, 0.0), (12.0, 100.0))),
                right_axis: false,
                scale_right: 1.0,
                events: Vec::new(),
            }
        }

        fn with_right_axis(mut self, scale: f64) -> Self {
            self.right_axis = true;
            self.scale_right = scale;
            self
        }
    }

    #[derive(Debug, PartialEq)]
    enum MockEvent {
        Polyline {
            name: String,
            // (x_idx, Option<y>) — we drop the X type generic by stashing the index
            points: Vec<(usize, Option<f64>)>,
            color: Color,
        },
        Bars {
            name: String,
            count: usize,
            color: Color,
        },
    }

    impl Renderer<Index> for MockRenderer {
        fn transform(&self) -> Transform {
            self.transform
        }
        fn draw_polyline(
            &mut self,
            name: &str,
            points: &[(Index, Option<f64>)],
            stroke: Stroke,
        ) {
            self.events.push(MockEvent::Polyline {
                name: name.to_string(),
                points: points.iter().map(|(x, y)| (x.0, *y)).collect(),
                color: stroke.color,
            });
        }
        fn draw_bar(&mut self, _x: Index, _base: f64, _v: f64, _w: f64, _c: Color) {}
        fn draw_bars(&mut self, name: &str, bars: &[(Index, f64)], _w: f64, color: Color) {
            self.events.push(MockEvent::Bars {
                name: name.to_string(),
                count: bars.len(),
                color,
            });
        }
        fn draw_hline(&mut self, _l: f64, _s: Stroke) {}
        fn draw_marker(&mut self, _x: Index, _y: f64, _s: MarkerShape, _c: Color) {}
        fn draw_polygon(&mut self, _points: &[(Index, f64)], _fill: Color) {}
        fn draw_text(&mut self, _x: Index, _y: f64, _t: &str, _c: Color) {}
        fn has_right_axis(&self) -> bool {
            self.right_axis
        }
        fn map_axis_value(&self, axis: YAxis, value: f64) -> f64 {
            // Apply a non-identity transform on the right axis so we can
            // verify the value reaches the renderer.
            match axis {
                YAxis::Right => value * self.scale_right,
                YAxis::Left => value,
            }
        }
    }

    /// Theme that returns a deterministic colour per `line_id`.
    struct MockTheme;
    impl IndicatorTheme for MockTheme {
        fn line_color(&self, line_id: &str) -> Color {
            match line_id {
                "sma" => Color::rgb(10, 20, 30),
                "macd_line" => Color::rgb(40, 50, 60),
                "macd_signal" => Color::rgb(70, 80, 90),
                _ => Color::rgb(0, 0, 0),
            }
        }
        fn grid_color(&self) -> Color {
            Color::rgb(200, 200, 200)
        }
    }

    fn idx_line(name: &str, id: &str, ys: &[Option<f64>]) -> IndicatorLine<Index> {
        let xs: Vec<Index> = (0..ys.len()).map(Index).collect();
        IndicatorLine::from_xy(name, id, &xs, ys)
    }

    // ---------- Plot trait helpers (StandardPlot) ----------

    #[test]
    fn test_new_returns_empty_plot() {
        let p: StandardPlot<Index> = StandardPlot::new("test");
        assert_eq!(p.indicator_id(), "test");
        assert!(p.lines().is_empty());
        assert!(p.histogram().is_none());
    }

    #[test]
    fn test_add_line_appends() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        p.add_line(idx_line("L", "x", &[Some(1.0)]));
        p.add_line(idx_line("M", "y", &[Some(2.0)]));
        assert_eq!(p.lines().len(), 2);
    }

    #[test]
    fn test_set_histogram_bars_replaces() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        let bars = vec![HistogramBar::new(Index(0), 1.0, Color::rgb(255, 0, 0))];
        p.set_histogram_bars(bars);
        let h = p.histogram().unwrap();
        assert_eq!(h.len(), 1);

        let bars = vec![
            HistogramBar::new(Index(0), 1.0, Color::rgb(255, 0, 0)),
            HistogramBar::new(Index(1), -1.0, Color::rgb(0, 255, 0)),
        ];
        p.set_histogram_bars(bars);
        assert_eq!(p.histogram().unwrap().len(), 2);
    }

    #[test]
    fn test_set_histogram_pos_neg_skips_none() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        let xs = vec![Index(0), Index(1), Index(2), Index(3)];
        let ys = vec![Some(1.0), None, Some(-2.0), Some(0.0)];
        let pos = Color::rgb(0, 200, 0);
        let neg = Color::rgb(200, 0, 0);
        p.set_histogram_pos_neg(&xs, &ys, pos, neg);
        let h = p.histogram().unwrap();
        // Only 3 entries (None is filtered)
        assert_eq!(h.len(), 3);
        // First positive → green
        assert_eq!(h[0].color, pos);
        // Negative → red
        assert_eq!(h[1].color, neg);
        // Zero is treated as positive (>= 0)
        assert_eq!(h[2].color, pos);
    }

    #[test]
    fn test_left_lines_and_right_lines_filter_by_axis() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        let mut a = idx_line("A", "a", &[Some(1.0)]);
        a.axis = YAxis::Left;
        let mut b = idx_line("B", "b", &[Some(2.0)]);
        b.axis = YAxis::Right;
        p.add_line(a);
        p.add_line(b);
        assert_eq!(p.left_lines().count(), 1);
        assert_eq!(p.right_lines().count(), 1);
    }

    #[test]
    fn test_move_to_axis_relocates_lines_and_bars() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        p.add_line(idx_line("A", "a", &[Some(1.0)]));
        p.set_histogram_bars(vec![HistogramBar::new(Index(0), 1.0, Color::rgb(0, 0, 0))]);
        p.move_to_axis(YAxis::Right);
        assert!(p.lines.iter().all(|l| l.axis == YAxis::Right));
        assert!(p
            .histogram
            .as_ref()
            .unwrap()
            .iter()
            .all(|b| b.axis == YAxis::Right));
    }

    // ---------- Plot::has_axis / y_range ----------

    #[test]
    fn test_has_axis_via_lines() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        p.add_line(idx_line("A", "a", &[Some(1.0)]));
        assert!(p.has_axis(YAxis::Left));
        assert!(!p.has_axis(YAxis::Right));
    }

    #[test]
    fn test_has_axis_via_histogram_only() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        let mut bar = HistogramBar::new(Index(0), 1.0, Color::rgb(0, 0, 0));
        bar.axis = YAxis::Right;
        p.set_histogram_bars(vec![bar]);
        assert!(!p.has_axis(YAxis::Left));
        assert!(p.has_axis(YAxis::Right));
    }

    #[test]
    fn test_has_axis_false_when_empty() {
        let p: StandardPlot<Index> = StandardPlot::new("x");
        assert!(!p.has_axis(YAxis::Left));
        assert!(!p.has_axis(YAxis::Right));
    }

    #[test]
    fn test_y_range_from_lines_and_histogram() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        // Lines: 1, None, 5
        p.add_line(idx_line("A", "a", &[Some(1.0), None, Some(5.0)]));
        // Histogram: -2 (extends the lower bound)
        p.set_histogram_bars(vec![HistogramBar::new(Index(0), -2.0, Color::rgb(0, 0, 0))]);
        let (lo, hi) = p.y_range(YAxis::Left).unwrap();
        assert!((lo - (-2.0)).abs() < 1e-9);
        assert!((hi - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_y_range_returns_none_for_axis_with_no_data() {
        let p: StandardPlot<Index> = StandardPlot::new("x");
        assert!(p.y_range(YAxis::Right).is_none());
    }

    #[test]
    fn test_y_range_skips_none_values() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        p.add_line(idx_line("A", "a", &[None, None, Some(7.0)]));
        let (lo, hi) = p.y_range(YAxis::Left).unwrap();
        assert!((lo - 7.0).abs() < 1e-9);
        assert!((hi - 7.0).abs() < 1e-9);
    }

    // ---------- Plot::render ----------

    #[test]
    fn test_render_emits_polyline_for_each_line() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        p.add_line(idx_line("SMA", "sma", &[Some(1.0), Some(2.0)]));
        p.add_line(idx_line("EMA", "ema", &[Some(3.0), Some(4.0)]));

        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));

        let polyline_count = renderer
            .events
            .iter()
            .filter(|e| matches!(e, MockEvent::Polyline { .. }))
            .count();
        assert_eq!(polyline_count, 2);
    }

    #[test]
    fn test_render_uses_theme_color_when_line_has_no_override() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        // No `with_color` → render must use the theme.
        p.add_line(idx_line("SMA", "sma", &[Some(1.0)]));
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        let MockEvent::Polyline { color, .. } = &renderer.events[0] else {
            panic!("expected polyline");
        };
        assert_eq!(*color, Color::rgb(10, 20, 30));
    }

    #[test]
    fn test_render_respects_explicit_line_color() {
        let red = Color::rgb(255, 0, 0);
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        let line = idx_line("SMA", "sma", &[Some(1.0)]).with_color(red);
        p.add_line(line);
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        let MockEvent::Polyline { color, .. } = &renderer.events[0] else {
            panic!("expected polyline");
        };
        assert_eq!(*color, red);
    }

    #[test]
    fn test_render_passes_none_y_through_for_gaps() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        p.add_line(idx_line("L", "sma", &[Some(1.0), None, Some(3.0)]));
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        let MockEvent::Polyline { points, .. } = &renderer.events[0] else {
            panic!("expected polyline");
        };
        assert_eq!(points.len(), 3);
        assert_eq!(points[1].1, None);
    }

    #[test]
    fn test_render_applies_axis_value_mapping() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        // Right-axis line with value 100 — renderer scales by 0.5
        let mut line = idx_line("R", "sma", &[Some(100.0)]);
        line.axis = YAxis::Right;
        p.add_line(line);
        let mut renderer = MockRenderer::new().with_right_axis(0.5);
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        let MockEvent::Polyline { points, .. } = &renderer.events[0] else {
            panic!("expected polyline");
        };
        assert_eq!(points[0].1, Some(50.0));
    }

    #[test]
    fn test_render_emits_histogram_bars_grouped_by_color() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        // Three pos-neg-pos transitions — that's 3 batches.
        let pos = Color::rgb(0, 200, 0);
        let neg = Color::rgb(200, 0, 0);
        p.set_histogram_bars(vec![
            HistogramBar::new(Index(0), 1.0, pos),
            HistogramBar::new(Index(1), 2.0, pos),
            HistogramBar::new(Index(2), -1.0, neg),
            HistogramBar::new(Index(3), 3.0, pos),
        ]);
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        let bar_events: Vec<&MockEvent> = renderer
            .events
            .iter()
            .filter(|e| matches!(e, MockEvent::Bars { .. }))
            .collect();
        // 3 batches: [pos, pos], [neg], [pos]
        assert_eq!(bar_events.len(), 3);
        if let MockEvent::Bars { count, color, .. } = bar_events[0] {
            assert_eq!(*count, 2);
            assert_eq!(*color, pos);
        }
        if let MockEvent::Bars { count, color, .. } = bar_events[1] {
            assert_eq!(*count, 1);
            assert_eq!(*color, neg);
        }
        if let MockEvent::Bars { count, color, .. } = bar_events[2] {
            assert_eq!(*count, 1);
            assert_eq!(*color, pos);
        }
    }

    #[test]
    fn test_render_no_histogram_when_none() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        p.add_line(idx_line("L", "sma", &[Some(1.0)]));
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        let bar_events = renderer
            .events
            .iter()
            .filter(|e| matches!(e, MockEvent::Bars { .. }))
            .count();
        assert_eq!(bar_events, 0);
    }

    #[test]
    fn test_render_histogram_before_lines() {
        let mut p: StandardPlot<Index> = StandardPlot::new("test");
        p.add_line(idx_line("L", "sma", &[Some(1.0)]));
        p.set_histogram_bars(vec![HistogramBar::new(Index(0), 1.0, Color::rgb(0, 200, 0))]);
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        // Histogram should be drawn first (z-order: bars behind lines)
        match &renderer.events[0] {
            MockEvent::Bars { .. } => {}
            _ => panic!("expected histogram before line"),
        }
        match &renderer.events[1] {
            MockEvent::Polyline { .. } => {}
            _ => panic!("expected polyline after histogram"),
        }
    }

    #[test]
    fn test_render_empty_plot_produces_no_events() {
        let p: StandardPlot<Index> = StandardPlot::new("empty");
        let mut renderer = MockRenderer::new();
        let theme = MockTheme;
        p.render(&mut PlotContext::new(&mut renderer, &theme));
        assert!(renderer.events.is_empty());
    }

    // ---------- as_any / as_any_mut ----------

    #[test]
    fn test_as_any_downcast_works() {
        let p: StandardPlot<Index> = StandardPlot::new("x");
        let any_ref = (&p as &dyn Plot<Index>).as_any();
        assert!(any_ref.downcast_ref::<StandardPlot<Index>>().is_some());
    }

    #[test]
    fn test_as_any_mut_downcast_works() {
        let mut p: StandardPlot<Index> = StandardPlot::new("x");
        let any_mut = (&mut p as &mut dyn Plot<Index>).as_any_mut();
        assert!(any_mut.downcast_mut::<StandardPlot<Index>>().is_some());
    }

    // ---------- Free helpers ----------

    #[test]
    fn test_y_range_with_padding_pads_by_10pct() {
        let (lo, hi) = y_range_with_padding([0.0, 100.0]);
        assert!((lo - (-10.0)).abs() < 1e-9);
        assert!((hi - 110.0).abs() < 1e-9);
    }

    #[test]
    fn test_y_range_with_padding_empty_falls_back() {
        let (lo, hi) = y_range_with_padding(std::iter::empty::<f64>());
        // finite_range returns None → fallback (0,100), then pad by 10
        assert!((lo - (-10.0)).abs() < 1e-9);
        assert!((hi - 110.0).abs() < 1e-9);
    }

    #[test]
    fn test_aggregate_ranges_empty_is_none() {
        let v: Vec<(f64, f64)> = vec![];
        assert!(aggregate_ranges(v).is_none());
    }

    #[test]
    fn test_aggregate_ranges_picks_outer_bounds() {
        let v = vec![(0.0, 10.0), (-5.0, 3.0), (2.0, 20.0)];
        let (lo, hi) = aggregate_ranges(v).unwrap();
        assert!((lo - (-5.0)).abs() < 1e-9);
        assert!((hi - 20.0).abs() < 1e-9);
    }

    // ---------- HLine ----------

    #[test]
    fn test_hline_default_style_is_dashed() {
        let s: HLineStyle = Default::default();
        assert_eq!(s, HLineStyle::Dashed);
    }

    #[test]
    fn test_hline_new_dashed_for_nonzero_level() {
        let h = HLine::new(50.0);
        assert!((h.level - 50.0).abs() < 1e-9);
        assert!(h.color.is_none());
        assert_eq!(h.style, HLineStyle::Dashed);
    }

    #[test]
    fn test_hline_new_solid_for_zero_level() {
        let h = HLine::new(0.0);
        assert_eq!(h.style, HLineStyle::Solid);
    }

    #[test]
    fn test_hline_colored_carries_color_and_style_rule() {
        let red = Color::rgb(255, 0, 0);
        let h_nonzero = HLine::colored(50.0, red);
        assert_eq!(h_nonzero.color, Some(red));
        assert_eq!(h_nonzero.style, HLineStyle::Dashed);

        let h_zero = HLine::colored(0.0, red);
        assert_eq!(h_zero.style, HLineStyle::Solid);
    }

    #[test]
    fn test_hline_with_style_override() {
        let h = HLine::new(50.0).with_style(HLineStyle::Solid);
        assert_eq!(h.style, HLineStyle::Solid);
    }

    #[test]
    fn test_hline_from_f64() {
        let h: HLine = 30.0_f64.into();
        assert!((h.level - 30.0).abs() < 1e-9);
        assert_eq!(h.style, HLineStyle::Dashed);
    }
}
