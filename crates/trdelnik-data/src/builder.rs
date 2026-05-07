//! ChartBuilder - matplotlib-like API for building ChartData
//!
//! ```rust,ignore
//! use trdelnik_data::ChartBuilder;
//! use trdelnik_indicators::{Sma, Rsi, Macd};
//!
//! let chart_data = ChartBuilder::new(series)
//!     .overlay(Sma::new(20))
//!     .panel("rsi", |p| {
//!         p.plot(Rsi::new(14));
//!         p.hline(30.0);
//!         p.hline(70.0);
//!         p.ylim(0.0, 100.0);
//!     })
//!     .panel("macd", |p| {
//!         p.plot(Macd::default());
//!         p.hline(0.0);
//!     })
//!     .build();
//! ```

use trdelnik_core::{
    AxisCoordinate, CandleSeries, IndicatorLine, IndicatorMarker, Timestamp,
};
use trdelnik_render::{Plot, StandardPlot};
use trdelnik_indicators::Plottable;

use crate::chart_data::ChartData;
use crate::panel::{Panel, PanelConfig};
use crate::panel_handle::PanelHandle;

/// Builder for creating ChartData with a matplotlib-like API
pub struct ChartBuilder<X: AxisCoordinate = Timestamp> {
    chart_data: ChartData<X>,
}

impl<X: AxisCoordinate> ChartBuilder<X> {
    /// Create a new builder from a candle series
    pub fn new(series: CandleSeries<X>) -> Self {
        Self {
            chart_data: ChartData::new(series),
        }
    }

    /// Add an overlay indicator to the main price chart
    pub fn overlay(mut self, indicator: impl Plottable) -> Self {
        let plot = indicator.plot(self.chart_data.series());
        self.chart_data.add_overlay(plot);
        self
    }

    /// Add a custom `Plot` impl as an overlay (e.g. heatmap, volume profile).
    pub fn overlay_with(mut self, plot: impl Plot<X>) -> Self {
        self.chart_data.add_overlay(Box::new(plot));
        self
    }

    /// Add a pre-boxed plot as an overlay.
    pub fn overlay_boxed(mut self, plot: Box<dyn Plot<X>>) -> Self {
        self.chart_data.add_overlay(plot);
        self
    }

    /// Add an overlay line from raw x/y values (no indicator).
    ///
    /// `id` is used for theming/grouping (e.g. `"my_signal"`).
    pub fn overlay_line(
        mut self,
        name: impl Into<String>,
        id: impl Into<String>,
        x_values: &[X],
        y_values: &[Option<f64>],
    ) -> Self {
        let id = id.into();
        let mut data = StandardPlot::new(&id);
        data.add_line(IndicatorLine::from_xy(name, &id, x_values, y_values));
        self.chart_data.add_overlay(Box::new(data));
        self
    }

    /// Add multiple boxed plots as overlays.
    pub fn overlay_many(mut self, plots: impl IntoIterator<Item = Box<dyn Plot<X>>>) -> Self {
        for plot in plots {
            self.chart_data.add_overlay(plot);
        }
        self
    }

    /// Add a single marker to the main chart overlay (e.g. trade signal)
    pub fn overlay_marker(mut self, marker: IndicatorMarker<X>) -> Self {
        self.chart_data.add_overlay_marker(marker);
        self
    }

    /// Add multiple markers to the main chart overlay
    pub fn overlay_markers(mut self, markers: impl IntoIterator<Item = IndicatorMarker<X>>) -> Self {
        self.chart_data.add_overlay_markers(markers);
        self
    }

    /// Configure and populate a panel using a closure
    ///
    /// ```rust,ignore
    /// builder.panel("rsi", |p| {
    ///     p.plot(Rsi::new(14));
    ///     p.hline(30.0);
    ///     p.hline(70.0);
    ///     p.ylim(0.0, 100.0);
    /// })
    /// ```
    pub fn panel(mut self, id: &str, f: impl FnOnce(&mut PanelHandle<'_, X>)) -> Self {
        let config = PanelConfig::new(id);
        let mut handle = PanelHandle::new(config, self.chart_data.series());
        f(&mut handle);
        self.chart_data.add_panel_container(handle.build());
        self
    }

    /// Add a pre-built panel directly
    pub fn add_panel(mut self, panel: Panel<X>) -> Self {
        self.chart_data.add_panel_container(panel);
        self
    }

    /// Build the ChartData
    pub fn build(self) -> ChartData<X> {
        self.chart_data
    }
}

/// Extension trait for easily creating ChartData from CandleSeries
pub trait IntoChartData<X: AxisCoordinate> {
    /// Convert to ChartData
    fn into_chart_data(self) -> ChartData<X>;

    /// Start building ChartData with the new API
    fn chart(self) -> ChartBuilder<X>;
}

impl<X: AxisCoordinate> IntoChartData<X> for CandleSeries<X> {
    fn into_chart_data(self) -> ChartData<X> {
        ChartData::new(self)
    }

    fn chart(self) -> ChartBuilder<X> {
        ChartBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{generate_sample_data, Timeframe};
    use trdelnik_indicators::{Ema, Macd, Rsi, Sma, Stochastic};

    #[test]
    fn test_builder_overlay() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartBuilder::new(series)
            .overlay(Sma::new(20))
            .overlay(Ema::new(50))
            .build();

        assert_eq!(chart_data.overlay_plots().len(), 2);
    }

    #[test]
    fn test_builder_panel_with_closure() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartBuilder::new(series)
            .panel("rsi", |p| {
                p.plot(Rsi::new(14));
                p.hline(30.0);
                p.hline(70.0);
                p.ylim(0.0, 100.0);
            })
            .build();

        assert_eq!(chart_data.panel_containers().len(), 1);
        let panel = &chart_data.panel_containers()[0];
        assert_eq!(panel.id(), "rsi");
        assert_eq!(panel.plot_count(), 1);
        assert_eq!(panel.hlines().len(), 2);
        assert_eq!(panel.fixed_y_range(), Some((0.0, 100.0)));
    }

    #[test]
    fn test_builder_multiple_panels() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartBuilder::new(series)
            .overlay(Sma::new(20))
            .panel("rsi", |p| {
                p.plot(Rsi::new(14));
                p.hline(30.0);
                p.hline(70.0);
                p.ylim(0.0, 100.0);
            })
            .panel("macd", |p| {
                p.plot(Macd::default());
                p.hline(0.0);
            })
            .build();

        assert_eq!(chart_data.overlay_plots().len(), 1);
        assert_eq!(chart_data.panel_containers().len(), 2);
    }

    #[test]
    fn test_builder_dual_axis_panel() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartBuilder::new(series)
            .panel("oscillators", |p| {
                p.plot(Rsi::new(14));
                p.plot_right(Stochastic::default());
                p.hline(30.0);
                p.hline(70.0);
                p.ylim(0.0, 100.0);
            })
            .build();

        assert_eq!(chart_data.panel_containers().len(), 1);
        let panel = &chart_data.panel_containers()[0];
        assert_eq!(panel.plot_count(), 2);
        assert!(panel.has_right_axis());
    }

    #[test]
    fn test_extension_trait() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = series.chart().overlay(Sma::new(20)).build();

        assert_eq!(chart_data.overlay_plots().len(), 1);
    }

    use crate::panel::PanelConfig;
    use trdelnik_core::Color;

    fn red() -> Color {
        Color::rgb(255, 0, 0)
    }

    // ---------- overlay_with / overlay_boxed / overlay_line / overlay_many ----------

    #[test]
    fn test_overlay_with_custom_plot() {
        let s = generate_sample_data(20, Timeframe::H1);
        let custom: StandardPlot<Timestamp> = StandardPlot::new("custom");
        let cd = ChartBuilder::new(s).overlay_with(custom).build();
        assert_eq!(cd.overlay_plots().len(), 1);
    }

    #[test]
    fn test_overlay_boxed() {
        let s = generate_sample_data(20, Timeframe::H1);
        let boxed: Box<dyn Plot<Timestamp>> = Box::new(StandardPlot::<Timestamp>::new("boxed"));
        let cd = ChartBuilder::new(s).overlay_boxed(boxed).build();
        assert_eq!(cd.overlay_plots().len(), 1);
    }

    #[test]
    fn test_overlay_line_with_raw_xy() {
        let s = generate_sample_data(5, Timeframe::H1);
        let xs: Vec<Timestamp> = s.candles().iter().map(|c| c.x).collect();
        let ys = vec![Some(1.0), None, Some(3.0), Some(4.0), Some(5.0)];
        let cd = ChartBuilder::new(s).overlay_line("Custom", "custom_id", &xs, &ys).build();
        assert_eq!(cd.overlay_plots().len(), 1);
        assert_eq!(cd.overlay_plots()[0].lines().len(), 1);
    }

    #[test]
    fn test_overlay_many_appends_all() {
        let s = generate_sample_data(10, Timeframe::H1);
        let plots: Vec<Box<dyn Plot<Timestamp>>> = vec![
            Box::new(StandardPlot::<Timestamp>::new("a")),
            Box::new(StandardPlot::<Timestamp>::new("b")),
            Box::new(StandardPlot::<Timestamp>::new("c")),
        ];
        let cd = ChartBuilder::new(s).overlay_many(plots).build();
        assert_eq!(cd.overlay_plots().len(), 3);
    }

    // ---------- overlay_marker / overlay_markers ----------

    #[test]
    fn test_overlay_marker_one() {
        let s = generate_sample_data(5, Timeframe::H1);
        let cd = ChartBuilder::new(s)
            .overlay_marker(IndicatorMarker::arrow_up(Timestamp(0), 100.0, red()))
            .build();
        assert_eq!(cd.overlay_markers().len(), 1);
    }

    #[test]
    fn test_overlay_markers_iter() {
        let s = generate_sample_data(5, Timeframe::H1);
        let markers = vec![
            IndicatorMarker::arrow_up(Timestamp(0), 100.0, red()),
            IndicatorMarker::arrow_down(Timestamp(60_000), 110.0, red()),
        ];
        let cd = ChartBuilder::new(s).overlay_markers(markers).build();
        assert_eq!(cd.overlay_markers().len(), 2);
    }

    // ---------- add_panel ----------

    #[test]
    fn test_add_panel_pre_built() {
        let s = generate_sample_data(5, Timeframe::H1);
        let panel: Panel<Timestamp> = Panel::new(PanelConfig::new("custom"));
        let cd = ChartBuilder::new(s).add_panel(panel).build();
        assert_eq!(cd.panel_containers().len(), 1);
        assert_eq!(cd.panel_containers()[0].id(), "custom");
    }

    // ---------- IntoChartData ----------

    #[test]
    fn test_into_chart_data_creates_chart_with_no_overlays() {
        let s = generate_sample_data(5, Timeframe::H1);
        let cd: ChartData<Timestamp> = s.into_chart_data();
        assert_eq!(cd.len(), 5);
        assert!(cd.overlay_plots().is_empty());
    }
}
