//! ChartDataBuilder - fluent API for building ChartData with indicators

use std::collections::HashMap;

use trdelnik_core::{AxisCoordinate, CandleSeries, Placement, Timestamp, YAxis};
use crate::chart_indicator::ChartIndicator;

use crate::chart_data::ChartData;
use crate::panel::{Panel, PanelBuilder, PanelConfig};

/// Builder for creating ChartData with pre-computed indicators
///
/// # Example (New API)
///
/// ```rust,ignore
/// use trdelnik_core::{generate_sample_data, Timeframe};
/// use trdelnik_data::ChartDataBuilder;
/// use trdelnik_indicators::{Sma, BollingerBands, Rsi, Stochastic};
///
/// let series = generate_sample_data(100, Timeframe::H1);
/// let chart_data = ChartDataBuilder::new(series)
///     // Overlays on main chart
///     .add_overlay(Sma::new(20))
///     .add_overlay(BollingerBands::default())
///
///     // Panel with multiple indicators + dual Y-axis
///     .configure_panel("oscillators", |p| p.name("Oscillators").height(120.0))
///     .add_to_panel("oscillators", Rsi::new(14))
///     .add_to_panel_right("oscillators", Stochastic::default())
///
///     // Simple panel (auto-created)
///     .add_to_panel("macd", Macd::default())
///
///     .build();
/// ```
pub struct ChartDataBuilder<X: AxisCoordinate = Timestamp> {
    chart_data: ChartData<X>,
    panel_configs: HashMap<String, PanelConfig>,
    panel_indicators: HashMap<String, Vec<(trdelnik_core::IndicatorOutput<X>, YAxis)>>,
}

impl<X: AxisCoordinate> ChartDataBuilder<X> {
    /// Create a new builder from a candle series
    pub fn new(series: CandleSeries<X>) -> Self {
        Self {
            chart_data: ChartData::new(series),
            panel_configs: HashMap::new(),
            panel_indicators: HashMap::new(),
        }
    }

    // =========================================================================
    // New unified API
    // =========================================================================

    /// Add an indicator using its default placement
    ///
    /// Overlay indicators (like SMA, EMA, Bollinger Bands) will be added to the main chart.
    /// Panel indicators (like RSI, MACD, Stochastic) will create their own panel.
    pub fn add(mut self, indicator: impl ChartIndicator) -> Self {
        let output = indicator.compute(&self.chart_data.series);
        match output.default_placement {
            Placement::Overlay => {
                self.chart_data.add_overlay_indicator(output);
            }
            Placement::Panel => {
                let panel_id = output.indicator_id.clone();
                self.panel_indicators
                    .entry(panel_id)
                    .or_default()
                    .push((output, YAxis::Left));
            }
        }
        self
    }

    /// Add an overlay indicator to the main price chart
    pub fn add_overlay(mut self, indicator: impl ChartIndicator) -> Self {
        let mut output = indicator.compute(&self.chart_data.series);
        output.default_placement = Placement::Overlay;
        self.chart_data.add_overlay_indicator(output);
        self
    }

    /// Add a pre-computed IndicatorOutput directly
    ///
    /// This method allows adding indicator outputs that were computed elsewhere,
    /// such as from script execution results. The output's default_placement
    /// determines whether it's added as an overlay or to a panel.
    ///
    /// # Example
    /// ```rust,ignore
    /// // After executing a script strategy:
    /// for output in strategy.to_overlays(&result, &series) {
    ///     builder = builder.add_indicator_output(output);
    /// }
    /// ```
    pub fn add_indicator_output(mut self, output: trdelnik_core::IndicatorOutput<X>) -> Self {
        match output.default_placement {
            Placement::Overlay => {
                self.chart_data.add_overlay_indicator(output);
            }
            Placement::Panel => {
                let panel_id = output.indicator_id.clone();
                self.panel_indicators
                    .entry(panel_id)
                    .or_default()
                    .push((output, YAxis::Left));
            }
        }
        self
    }

    /// Add multiple pre-computed IndicatorOutputs
    ///
    /// Convenience method for adding multiple outputs at once,
    /// typically from script execution results.
    pub fn add_indicator_outputs(
        mut self,
        outputs: impl IntoIterator<Item = trdelnik_core::IndicatorOutput<X>>,
    ) -> Self {
        for output in outputs {
            self = self.add_indicator_output(output);
        }
        self
    }

    /// Configure a panel before adding indicators to it
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.configure_panel("oscillators", |p| {
    ///     p.name("Oscillators").height(120.0).min_height(60.0)
    /// })
    /// ```
    pub fn configure_panel(
        mut self,
        panel_id: impl Into<String>,
        configure: impl FnOnce(PanelBuilder) -> PanelBuilder,
    ) -> Self {
        let id = panel_id.into();
        let builder = PanelBuilder::new(&id);
        let config = configure(builder).build();
        self.panel_configs.insert(id, config);
        self
    }

    /// Add an indicator to a panel (left Y-axis)
    ///
    /// If the panel doesn't exist, it will be auto-created with default settings.
    pub fn add_to_panel(mut self, panel_id: impl Into<String>, indicator: impl ChartIndicator) -> Self {
        let id = panel_id.into();
        let output = indicator.compute(&self.chart_data.series);
        self.panel_indicators
            .entry(id)
            .or_default()
            .push((output, YAxis::Left));
        self
    }

    /// Add an indicator to a panel's right Y-axis
    ///
    /// Use this when you want to show an indicator with a different scale
    /// alongside other indicators in the same panel.
    pub fn add_to_panel_right(
        mut self,
        panel_id: impl Into<String>,
        indicator: impl ChartIndicator,
    ) -> Self {
        let id = panel_id.into();
        let mut output = indicator.compute(&self.chart_data.series);
        // Mark all lines as right-axis
        for line in &mut output.lines {
            line.axis = YAxis::Right;
        }
        // Mark histogram as right-axis if present
        if let Some(histogram) = &mut output.histogram {
            for bar in histogram {
                bar.axis = YAxis::Right;
            }
        }
        self.panel_indicators
            .entry(id)
            .or_default()
            .push((output, YAxis::Right));
        self
    }

    /// Build the ChartData
    pub fn build(mut self) -> ChartData<X> {
        // Build panels from collected indicators
        for (panel_id, indicators) in self.panel_indicators {
            let config = self
                .panel_configs
                .remove(&panel_id)
                .unwrap_or_else(|| PanelConfig::new(&panel_id));

            let mut panel = Panel::new(config);
            for (output, _axis) in indicators {
                // The axis is already set on the indicator's lines
                panel.add_indicator(output);
            }
            self.chart_data.add_panel_container(panel);
        }

        self.chart_data
    }
}

/// Extension trait for easily creating ChartData from CandleSeries
pub trait IntoChartData<X: AxisCoordinate> {
    /// Convert to ChartData
    fn into_chart_data(self) -> ChartData<X>;

    /// Start building ChartData with indicators
    fn build_chart_data(self) -> ChartDataBuilder<X>;
}

impl<X: AxisCoordinate> IntoChartData<X> for CandleSeries<X> {
    fn into_chart_data(self) -> ChartData<X> {
        ChartData::new(self)
    }

    fn build_chart_data(self) -> ChartDataBuilder<X> {
        ChartDataBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{generate_sample_data, Timeframe};
    use trdelnik_indicators::{Ema, Macd, Rsi, Sma, Stochastic};

    #[test]
    fn test_builder_new_api() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartDataBuilder::new(series)
            .add_overlay(Sma::new(20))
            .add_overlay(Ema::new(50))
            .build();

        assert_eq!(chart_data.overlay_indicators().len(), 2);
    }

    #[test]
    fn test_builder_add_uses_default_placement() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartDataBuilder::new(series)
            .add(Sma::new(20)) // Overlay by default
            .add(Rsi::new(14)) // Panel by default
            .build();

        assert_eq!(chart_data.overlay_indicators().len(), 1);
        assert_eq!(chart_data.panel_containers().len(), 1);
    }

    #[test]
    fn test_builder_panel_grouping() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartDataBuilder::new(series)
            .configure_panel("oscillators", |p| p.name("Oscillators").height(120.0))
            .add_to_panel("oscillators", Rsi::new(14))
            .add_to_panel_right("oscillators", Stochastic::default())
            .add_to_panel("macd", Macd::default())
            .build();

        assert_eq!(chart_data.panel_containers().len(), 2);

        // Find the oscillators panel
        let oscillators = chart_data
            .panel_containers()
            .iter()
            .find(|p| p.id() == "oscillators")
            .unwrap();
        assert_eq!(oscillators.indicators.len(), 2);
        assert!(oscillators.has_right_axis());
    }

    #[test]
    fn test_extension_trait() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = series.build_chart_data().add_overlay(Sma::new(20)).build();

        assert_eq!(chart_data.overlay_indicators().len(), 1);
    }
}
