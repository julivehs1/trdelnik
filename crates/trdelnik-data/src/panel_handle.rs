//! Matplotlib-style panel handle for configuring panels in builder closures.
//!
//! ```rust,ignore
//! ChartBuilder::new(series)
//!     .panel("rsi", |p| {
//!         p.plot(Rsi::new(14));
//!         p.hline(30.0);
//!         p.hline(70.0);
//!         p.ylim(0.0, 100.0);
//!     })
//! ```

use trdelnik_core::{
    AxisCoordinate, CandleSeries, Color, IndicatorLine, IndicatorMarker, YAxis,
};
use trdelnik_render::{HLine, Plot, StandardPlot};
use trdelnik_indicators::Plottable;

use crate::panel::{Panel, PanelConfig};

/// Handle passed into builder closures for configuring a panel.
///
/// Provides a matplotlib-like API for adding plots, reference lines,
/// markers, and setting axis limits.
pub struct PanelHandle<'a, X: AxisCoordinate> {
    panel: Panel<X>,
    series: &'a CandleSeries<X>,
}

impl<'a, X: AxisCoordinate> PanelHandle<'a, X> {
    /// Create a new panel handle
    pub(crate) fn new(config: PanelConfig, series: &'a CandleSeries<X>) -> Self {
        Self {
            panel: Panel::new(config),
            series,
        }
    }

    /// Plot an indicator on the left Y-axis.
    pub fn plot(&mut self, indicator: impl Plottable) -> &mut Self {
        let plot = indicator.plot(self.series);
        self.panel.add_plot(plot);
        self
    }

    /// Plot a raw line from x/y values (no indicator).
    ///
    /// `id` is used for theming/grouping (e.g. `"my_signal"`).
    pub fn line(
        &mut self,
        name: impl Into<String>,
        id: impl Into<String>,
        x_values: &[X],
        y_values: &[Option<f64>],
    ) -> &mut Self {
        let id = id.into();
        let mut data = StandardPlot::new(&id);
        data.add_line(IndicatorLine::from_xy(name, &id, x_values, y_values));
        self.panel.add_plot(Box::new(data));
        self
    }

    /// Plot an indicator on the right Y-axis.
    ///
    /// Re-targets the resulting plot's lines and bars to the right axis.
    /// Only supported for `StandardPlot`-shaped output (every built-in
    /// indicator). Custom `Plot` impls should be constructed on the right
    /// axis directly and added via [`PanelHandle::add_plot`].
    pub fn plot_right(&mut self, indicator: impl Plottable) -> &mut Self {
        let mut plot = indicator.plot(self.series);
        if let Some(std) = plot.as_any_mut().downcast_mut::<StandardPlot<X>>() {
            std.move_to_axis(YAxis::Right);
        }
        self.panel.add_plot(plot);
        self
    }

    /// Add a horizontal reference line.
    pub fn hline(&mut self, level: f64) -> &mut Self {
        self.panel.add_hline(HLine::new(level));
        self
    }

    /// Add a coloured horizontal reference line.
    pub fn hline_colored(&mut self, level: f64, color: Color) -> &mut Self {
        self.panel.add_hline(HLine::colored(level, color));
        self
    }

    /// Set fixed Y-axis limits.
    pub fn ylim(&mut self, min: f64, max: f64) -> &mut Self {
        self.panel.set_y_range(min, max);
        self
    }

    /// Add a marker to the panel.
    pub fn marker(&mut self, m: IndicatorMarker<X>) -> &mut Self {
        self.panel.add_marker(m);
        self
    }

    /// Add multiple markers to the panel.
    pub fn markers(&mut self, ms: impl IntoIterator<Item = IndicatorMarker<X>>) -> &mut Self {
        self.panel.add_markers(ms);
        self
    }

    /// Set the panel height.
    pub fn height(&mut self, h: f32) -> &mut Self {
        self.panel.config.default_height = h;
        self
    }

    /// Add a custom `Plot` impl directly.
    pub fn add_plot(&mut self, plot: impl Plot<X>) -> &mut Self {
        self.panel.add_plot(Box::new(plot));
        self
    }

    /// Add a pre-boxed plot directly.
    pub fn add_plot_boxed(&mut self, plot: Box<dyn Plot<X>>) -> &mut Self {
        self.panel.add_plot(plot);
        self
    }

    /// Consume the handle and return the built panel
    pub(crate) fn build(self) -> Panel<X> {
        self.panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Candle, Index, IndicatorLine, MarkerShape};
    use trdelnik_indicators::{Rsi, Sma};
    use trdelnik_render::StandardPlot;

    fn series(n: usize) -> CandleSeries<Index> {
        let mut s = CandleSeries::<Index>::new();
        for i in 0..n {
            let p = 100.0 + i as f64;
            s.push(Candle::new(Index(i), p - 0.5, p + 1.0, p - 1.0, p, 1000.0));
        }
        s
    }

    fn red() -> Color {
        Color::rgb(255, 0, 0)
    }

    fn handle(series_ref: &CandleSeries<Index>) -> PanelHandle<'_, Index> {
        PanelHandle::new(crate::panel::PanelConfig::new("p"), series_ref)
    }

    // ---------- plot / line / plot_right ----------

    #[test]
    fn test_plot_adds_indicator_plot() {
        let s = series(20);
        let mut h = handle(&s);
        h.plot(Sma::new(5));
        let panel = h.build();
        assert_eq!(panel.plot_count(), 1);
    }

    #[test]
    fn test_plot_returns_self_for_chaining() {
        let s = series(20);
        let mut h = handle(&s);
        h.plot(Sma::new(5)).plot(Sma::new(10));
        let panel = h.build();
        assert_eq!(panel.plot_count(), 2);
    }

    #[test]
    fn test_line_adds_raw_line_plot() {
        let s = series(5);
        let mut h = handle(&s);
        let xs: Vec<Index> = (0..5).map(Index).collect();
        let ys = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0)];
        h.line("Custom", "custom_id", &xs, &ys);
        let panel = h.build();
        assert_eq!(panel.plot_count(), 1);
        assert_eq!(panel.plots()[0].lines().len(), 1);
    }

    #[test]
    fn test_plot_right_re_targets_standard_plot_to_right_axis() {
        let s = series(20);
        let mut h = handle(&s);
        h.plot_right(Rsi::new(5));
        let panel = h.build();
        assert!(panel.has_right_axis(), "expected at least one right-axis plot");
    }

    // ---------- hlines ----------

    #[test]
    fn test_hline_default_color() {
        let s = series(5);
        let mut h = handle(&s);
        h.hline(50.0);
        let panel = h.build();
        assert_eq!(panel.hlines().len(), 1);
    }

    #[test]
    fn test_hline_colored() {
        let s = series(5);
        let mut h = handle(&s);
        h.hline_colored(70.0, red());
        let panel = h.build();
        assert_eq!(panel.hlines().len(), 1);
    }

    // ---------- ylim / height ----------

    #[test]
    fn test_ylim_sets_fixed_range() {
        let s = series(5);
        let mut h = handle(&s);
        h.ylim(0.0, 100.0);
        let panel = h.build();
        assert_eq!(panel.fixed_y_range(), Some((0.0, 100.0)));
    }

    #[test]
    fn test_height_sets_default_height_on_config() {
        let s = series(5);
        let mut h = handle(&s);
        h.height(250.0);
        let panel = h.build();
        assert!((panel.config.default_height - 250.0).abs() < 1e-6);
    }

    // ---------- markers ----------

    #[test]
    fn test_marker_one() {
        let s = series(5);
        let mut h = handle(&s);
        h.marker(IndicatorMarker {
            x: Index(0),
            y: 1.0,
            shape: MarkerShape::Circle,
            color: red(),
            label: None,
        });
        let panel = h.build();
        assert_eq!(panel.markers().len(), 1);
    }

    #[test]
    fn test_markers_iter() {
        let s = series(5);
        let mut h = handle(&s);
        h.markers(vec![
            IndicatorMarker {
                x: Index(0),
                y: 1.0,
                shape: MarkerShape::Circle,
                color: red(),
                label: None,
            },
            IndicatorMarker {
                x: Index(1),
                y: 2.0,
                shape: MarkerShape::Square,
                color: red(),
                label: None,
            },
        ]);
        let panel = h.build();
        assert_eq!(panel.markers().len(), 2);
    }

    // ---------- add_plot / add_plot_boxed ----------

    #[test]
    fn test_add_plot_with_custom_impl() {
        let s = series(5);
        let mut h = handle(&s);
        let custom: StandardPlot<Index> = StandardPlot::new("custom");
        h.add_plot(custom);
        let panel = h.build();
        assert_eq!(panel.plot_count(), 1);
    }

    #[test]
    fn test_add_plot_boxed() {
        let s = series(5);
        let mut h = handle(&s);
        let boxed: Box<dyn Plot<Index>> = Box::new(StandardPlot::<Index>::new("boxed"));
        h.add_plot_boxed(boxed);
        let panel = h.build();
        assert_eq!(panel.plot_count(), 1);
    }

    // ---------- chained mixed builders ----------

    #[test]
    fn test_full_chain_builds_complete_panel() {
        let s = series(20);
        let mut h = handle(&s);
        h.plot(Sma::new(5))
            .plot(Sma::new(10))
            .hline(0.0)
            .hline_colored(50.0, red())
            .ylim(-100.0, 200.0)
            .height(180.0);
        let panel = h.build();
        assert_eq!(panel.plot_count(), 2);
        assert_eq!(panel.hlines().len(), 2);
        assert_eq!(panel.fixed_y_range(), Some((-100.0, 200.0)));
        assert!((panel.config.default_height - 180.0).abs() < 1e-6);
    }

    // Suppress unused-import warnings for IndicatorLine when we only use
    // it via the public StandardPlot::add_line above.
    #[allow(dead_code)]
    fn _silence_unused() {
        let _: Option<IndicatorLine<Index>> = None;
    }
}
