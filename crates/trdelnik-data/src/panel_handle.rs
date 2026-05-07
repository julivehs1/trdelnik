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
