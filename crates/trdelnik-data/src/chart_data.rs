//! ChartData - the main interface between data and UI

use trdelnik_core::{
    AxisCoordinate, CandleSeries, Color, HistogramBar, IndicatorOutput, Placement, SignalSeries,
    Timeframe, Timestamp,
};

use crate::computed::ComputedIndicators;
use crate::panel::Panel;

/// Complete chart data ready for rendering
///
/// This is the main interface between the data layer and the UI.
/// It contains pre-computed indicator values so the UI only needs to render.
#[derive(Debug)]
pub struct ChartData<X: AxisCoordinate = Timestamp> {
    /// The candle series
    pub(crate) series: CandleSeries<X>,
    /// Computed indicator cache
    pub(crate) computed: ComputedIndicators,
    /// Trading signals
    pub(crate) signals: SignalSeries<X>,
    /// Pre-computed overlay indicators
    pub(crate) overlays: Vec<IndicatorOutput<X>>,
    /// Pre-computed panel containers
    pub(crate) panels: Vec<Panel<X>>,
}

impl<X: AxisCoordinate> ChartData<X> {
    /// Create new chart data from a candle series
    pub fn new(series: CandleSeries<X>) -> Self {
        Self {
            series,
            computed: ComputedIndicators::new(),
            signals: SignalSeries::new(),
            overlays: Vec::new(),
            panels: Vec::new(),
        }
    }

    /// Get the candle series
    pub fn series(&self) -> &CandleSeries<X> {
        &self.series
    }

    /// Get mutable candle series
    pub fn series_mut(&mut self) -> &mut CandleSeries<X> {
        &mut self.series
    }

    /// Get the trading signals
    pub fn signals(&self) -> &SignalSeries<X> {
        &self.signals
    }

    /// Get mutable signals
    pub fn signals_mut(&mut self) -> &mut SignalSeries<X> {
        &mut self.signals
    }

    /// Set trading signals
    pub fn set_signals(&mut self, signals: SignalSeries<X>) {
        self.signals = signals;
    }

    /// Get the computed indicators cache
    pub fn computed(&self) -> &ComputedIndicators {
        &self.computed
    }

    /// Get mutable computed indicators cache
    pub fn computed_mut(&mut self) -> &mut ComputedIndicators {
        &mut self.computed
    }

    /// Get overlay indicators
    pub fn overlay_indicators(&self) -> &[IndicatorOutput<X>] {
        &self.overlays
    }

    /// Get panel containers
    pub fn panel_containers(&self) -> &[Panel<X>] {
        &self.panels
    }

    /// Add an overlay indicator
    pub fn add_overlay_indicator(&mut self, overlay: IndicatorOutput<X>) {
        self.overlays.push(overlay);
    }

    /// Add a panel container
    pub fn add_panel_container(&mut self, panel: Panel<X>) {
        self.panels.push(panel);
    }

    /// Clear all overlays
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
    }

    /// Clear all panels
    pub fn clear_panels(&mut self) {
        self.panels.clear();
    }

    /// Invalidate computed indicators (call when data changes)
    pub fn invalidate(&mut self) {
        self.computed.invalidate();
        self.overlays.clear();
        self.panels.clear();
    }

    /// Get the timeframe
    pub fn timeframe(&self) -> Option<Timeframe> {
        self.series.timeframe()
    }

    /// Get the number of candles
    pub fn len(&self) -> usize {
        self.series.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// Get X values for all candles
    pub fn x_values(&self) -> Vec<X> {
        self.series.candles().iter().map(|c| c.x).collect()
    }

    /// Get the price range
    pub fn price_range(&self) -> Option<(f64, f64)> {
        self.series.price_range()
    }

    /// Get the volume range
    pub fn volume_range(&self) -> Option<(f64, f64)> {
        self.series.volume_range()
    }

    /// Get the X range as plot values
    pub fn x_range(&self) -> Option<(f64, f64)> {
        self.series.x_range()
    }

    /// Get the spacing between candles
    pub fn x_spacing(&self) -> f64 {
        self.series.x_spacing()
    }

    /// Create a volume panel from the candle data
    ///
    /// The indicator sets the color for each bar based on whether the candle is bullish or bearish.
    pub fn create_volume_panel(&self, bullish_color: Color, bearish_color: Color) -> Panel<X> {
        let candles = self.series.candles();

        // Create histogram bars with colors based on candle direction
        let bars: Vec<HistogramBar<X>> = candles
            .iter()
            .map(|c| {
                let color = if c.is_bullish() {
                    bullish_color
                } else {
                    bearish_color
                };
                HistogramBar::new(c.x, c.volume, color)
            })
            .collect();

        let mut indicator = IndicatorOutput::new("Volume", "volume", Placement::Panel);
        indicator.set_histogram_bars(bars);
        indicator.set_y_range((
            0.0,
            self.volume_range().map(|(_, max)| max * 1.1).unwrap_or(100.0),
        ));

        let config = crate::panel::PanelConfig::new("volume")
            .name("Volume")
            .height(100.0)
            .min_height(50.0)
            .max_height(300.0);

        let mut panel = Panel::new(config);
        panel.add_indicator(indicator);
        panel
    }
}

impl ChartData<Timestamp> {
    /// Get the time range in milliseconds
    pub fn time_range(&self) -> Option<(i64, i64)> {
        self.series.time_range()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{generate_sample_data, IndicatorOutput, Placement, Timeframe};

    #[test]
    fn test_chart_data_creation() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartData::new(series);

        assert_eq!(chart_data.len(), 100);
        assert!(!chart_data.is_empty());
        assert!(chart_data.overlay_indicators().is_empty());
        assert!(chart_data.panel_containers().is_empty());
    }

    #[test]
    fn test_chart_data_invalidation() {
        let series = generate_sample_data(100, Timeframe::H1);
        let mut chart_data = ChartData::new(series);

        // Add some data
        chart_data.add_overlay_indicator(IndicatorOutput::new("test", "test", Placement::Overlay));
        assert_eq!(chart_data.overlay_indicators().len(), 1);

        // Invalidate
        chart_data.invalidate();
        assert!(chart_data.overlay_indicators().is_empty());
    }
}
