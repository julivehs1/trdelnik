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

use trdelnik_core::{AxisCoordinate, CandleSeries, HLine, IndicatorMarker, PlotData, YAxis};
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

    /// Plot an indicator on the left Y-axis
    pub fn plot(&mut self, indicator: impl Plottable) -> &mut Self {
        let data = indicator.plot(self.series);
        self.panel.add_plot(data);
        self
    }

    /// Plot an indicator on the right Y-axis
    pub fn plot_right(&mut self, indicator: impl Plottable) -> &mut Self {
        let mut data = indicator.plot(self.series);
        for line in &mut data.lines {
            line.axis = YAxis::Right;
        }
        if let Some(histogram) = &mut data.histogram {
            for bar in histogram {
                bar.axis = YAxis::Right;
            }
        }
        self.panel.add_plot(data);
        self
    }

    /// Add a horizontal reference line
    pub fn hline(&mut self, level: f64) -> &mut Self {
        self.panel.add_hline(HLine::new(level));
        self
    }

    /// Add a colored horizontal reference line
    pub fn hline_colored(&mut self, level: f64, color: trdelnik_core::Color) -> &mut Self {
        self.panel.add_hline(HLine::colored(level, color));
        self
    }

    /// Set fixed Y-axis limits
    pub fn ylim(&mut self, min: f64, max: f64) -> &mut Self {
        self.panel.set_y_range(min, max);
        self
    }

    /// Add a marker to the panel
    pub fn marker(&mut self, m: IndicatorMarker<X>) -> &mut Self {
        // Add marker to the last plot, or create a markers-only plot
        if let Some(last) = self.panel.plots.last_mut() {
            last.add_marker(m);
        } else {
            let mut data = PlotData::new("Markers", "markers");
            data.add_marker(m);
            self.panel.add_plot(data);
        }
        self
    }

    /// Set the panel height
    pub fn height(&mut self, h: f32) -> &mut Self {
        self.panel.config.default_height = h;
        self
    }

    /// Add pre-computed plot data directly
    pub fn add_plot_data(&mut self, data: PlotData<X>) -> &mut Self {
        self.panel.add_plot(data);
        self
    }

    /// Consume the handle and return the built panel
    pub(crate) fn build(self) -> Panel<X> {
        self.panel
    }
}
