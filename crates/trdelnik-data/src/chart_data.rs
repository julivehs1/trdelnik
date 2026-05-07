//! ChartData - the main interface between data and UI

use trdelnik_core::{
    AxisCoordinate, CandleSeries, Color, HistogramBar, IndicatorMarker, SignalSeries, Timeframe,
    Timestamp,
};
use trdelnik_render::{Plot, StandardPlot};

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
    /// Pre-computed overlay plots (boxed for trait-object dispatch)
    pub(crate) overlays: Vec<Box<dyn Plot<X>>>,
    /// Markers drawn on the main chart (e.g. trade signals)
    pub(crate) overlay_markers: Vec<IndicatorMarker<X>>,
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
            overlay_markers: Vec::new(),
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

    /// Get overlay plots
    pub fn overlay_plots(&self) -> &[Box<dyn Plot<X>>] {
        &self.overlays
    }

    /// Get overlay markers (drawn on the main chart)
    pub fn overlay_markers(&self) -> &[IndicatorMarker<X>] {
        &self.overlay_markers
    }

    /// Get panel containers
    pub fn panel_containers(&self) -> &[Panel<X>] {
        &self.panels
    }

    /// Add an overlay plot
    pub fn add_overlay(&mut self, overlay: Box<dyn Plot<X>>) {
        self.overlays.push(overlay);
    }

    /// Add a marker to the main chart overlay
    pub fn add_overlay_marker(&mut self, marker: IndicatorMarker<X>) {
        self.overlay_markers.push(marker);
    }

    /// Add multiple markers to the main chart overlay
    pub fn add_overlay_markers(&mut self, markers: impl IntoIterator<Item = IndicatorMarker<X>>) {
        self.overlay_markers.extend(markers);
    }

    /// Add a panel container
    pub fn add_panel_container(&mut self, panel: Panel<X>) {
        self.panels.push(panel);
    }

    /// Clear all overlays
    pub fn clear_overlays(&mut self) {
        self.overlays.clear();
        self.overlay_markers.clear();
    }

    /// Clear all panels
    pub fn clear_panels(&mut self) {
        self.panels.clear();
    }

    /// Invalidate computed indicators (call when data changes)
    pub fn invalidate(&mut self) {
        self.computed.invalidate();
        self.overlays.clear();
        self.overlay_markers.clear();
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
    pub fn create_volume_panel(&self, bullish_color: Color, bearish_color: Color) -> Panel<X> {
        let candles = self.series.candles();

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

        let mut plot = StandardPlot::new("volume");
        plot.set_histogram_bars(bars);

        let config = crate::panel::PanelConfig::new("volume")
            .name("Volume")
            .height(100.0)
            .min_height(50.0)
            .max_height(300.0);

        let mut panel = Panel::new(config);
        panel.add_plot(Box::new(plot));
        panel.set_y_range(
            0.0,
            self.volume_range().map(|(_, max)| max * 1.1).unwrap_or(100.0),
        );

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
    use trdelnik_core::{generate_sample_data, Timeframe};

    #[test]
    fn test_chart_data_creation() {
        let series = generate_sample_data(100, Timeframe::H1);
        let chart_data = ChartData::new(series);

        assert_eq!(chart_data.len(), 100);
        assert!(!chart_data.is_empty());
        assert!(chart_data.overlay_plots().is_empty());
        assert!(chart_data.panel_containers().is_empty());
    }

    #[test]
    fn test_chart_data_invalidation() {
        let series = generate_sample_data(100, Timeframe::H1);
        let mut chart_data = ChartData::new(series);

        chart_data.add_overlay(Box::new(StandardPlot::new("test")));
        assert_eq!(chart_data.overlay_plots().len(), 1);

        chart_data.invalidate();
        assert!(chart_data.overlay_plots().is_empty());
    }

    use crate::panel::PanelConfig;
    use trdelnik_core::{Candle, Signal, SignalDirection, SignalSeries};

    fn small_series() -> CandleSeries<Timestamp> {
        let mut s = CandleSeries::with_timeframe(Timeframe::H1);
        for i in 0..3 {
            let p = 100.0 + i as f64;
            // Alternate bullish/bearish so the volume-panel coloring branches
            // both get exercised.
            let (open, close) = if i % 2 == 0 { (p, p + 1.0) } else { (p + 1.0, p) };
            s.push(Candle::new(
                Timestamp(i as i64 * 60_000),
                open,
                p + 2.0,
                p - 1.0,
                close,
                100.0 * (i as f64 + 1.0),
            ));
        }
        s
    }

    fn red() -> Color {
        Color::rgb(255, 0, 0)
    }
    fn green() -> Color {
        Color::rgb(0, 255, 0)
    }

    // ---------- Series accessors ----------

    #[test]
    fn test_series_accessors() {
        let mut data = ChartData::new(small_series());
        assert_eq!(data.series().len(), 3);
        // Mutate via series_mut
        data.series_mut().push(Candle::new(
            Timestamp(999_000),
            1.0,
            1.5,
            0.5,
            1.2,
            10.0,
        ));
        assert_eq!(data.len(), 4);
    }

    #[test]
    fn test_x_values_returns_timestamps() {
        let data = ChartData::new(small_series());
        let xs = data.x_values();
        assert_eq!(xs.len(), 3);
        assert_eq!(xs[0], Timestamp(0));
        assert_eq!(xs[2], Timestamp(120_000));
    }

    // ---------- Signals ----------

    #[test]
    fn test_signals_default_empty() {
        let data = ChartData::new(small_series());
        assert!(data.signals().is_empty());
    }

    #[test]
    fn test_set_signals_overrides() {
        let mut data = ChartData::new(small_series());
        let mut s = SignalSeries::<Timestamp>::new();
        s.push(Signal::new(Timestamp(0), 100.0, SignalDirection::Buy, "test"));
        data.set_signals(s);
        assert_eq!(data.signals().len(), 1);
    }

    #[test]
    fn test_signals_mut_can_modify() {
        let mut data = ChartData::new(small_series());
        data.signals_mut()
            .push(Signal::new(Timestamp(0), 100.0, SignalDirection::Buy, "x"));
        assert_eq!(data.signals().len(), 1);
    }

    // ---------- Computed ----------

    #[test]
    fn test_computed_accessors() {
        let mut data = ChartData::new(small_series());
        assert_eq!(data.computed().version(), 0);
        // Touch via mut accessor
        data.computed_mut().invalidate();
        assert_eq!(data.computed().version(), 1);
    }

    // ---------- Overlay markers ----------

    #[test]
    fn test_add_overlay_marker_one() {
        let mut data = ChartData::new(small_series());
        data.add_overlay_marker(IndicatorMarker::arrow_up(Timestamp(0), 100.0, red()));
        assert_eq!(data.overlay_markers().len(), 1);
    }

    #[test]
    fn test_add_overlay_markers_iter() {
        let mut data = ChartData::new(small_series());
        let markers = vec![
            IndicatorMarker::arrow_up(Timestamp(0), 100.0, red()),
            IndicatorMarker::arrow_down(Timestamp(60_000), 110.0, red()),
        ];
        data.add_overlay_markers(markers);
        assert_eq!(data.overlay_markers().len(), 2);
    }

    // ---------- Panels ----------

    #[test]
    fn test_add_panel_container() {
        let mut data = ChartData::new(small_series());
        data.add_panel_container(crate::panel::Panel::new(PanelConfig::new("custom")));
        assert_eq!(data.panel_containers().len(), 1);
        assert_eq!(data.panel_containers()[0].id(), "custom");
    }

    // ---------- Clear methods ----------

    #[test]
    fn test_clear_overlays_removes_plots_and_markers() {
        let mut data = ChartData::new(small_series());
        data.add_overlay(Box::new(StandardPlot::new("x")));
        data.add_overlay_marker(IndicatorMarker::arrow_up(Timestamp(0), 100.0, red()));
        data.clear_overlays();
        assert!(data.overlay_plots().is_empty());
        assert!(data.overlay_markers().is_empty());
    }

    #[test]
    fn test_clear_panels_only_clears_panels() {
        let mut data = ChartData::new(small_series());
        data.add_overlay(Box::new(StandardPlot::new("x")));
        data.add_panel_container(crate::panel::Panel::new(PanelConfig::new("p")));
        data.clear_panels();
        assert!(data.panel_containers().is_empty());
        // Overlays untouched
        assert_eq!(data.overlay_plots().len(), 1);
    }

    #[test]
    fn test_invalidate_also_clears_markers_and_panels() {
        let mut data = ChartData::new(small_series());
        data.add_overlay(Box::new(StandardPlot::new("x")));
        data.add_overlay_marker(IndicatorMarker::arrow_up(Timestamp(0), 100.0, red()));
        data.add_panel_container(crate::panel::Panel::new(PanelConfig::new("p")));
        let prev_version = data.computed().version();

        data.invalidate();
        assert_eq!(data.computed().version(), prev_version + 1);
        assert!(data.overlay_plots().is_empty());
        assert!(data.overlay_markers().is_empty());
        assert!(data.panel_containers().is_empty());
    }

    // ---------- Range / spacing helpers ----------

    #[test]
    fn test_timeframe_propagated_from_series() {
        let data = ChartData::new(small_series());
        assert_eq!(data.timeframe(), Some(Timeframe::H1));
    }

    #[test]
    fn test_price_volume_x_range_propagated() {
        let data = ChartData::new(small_series());
        let (lo, hi) = data.price_range().unwrap();
        assert!(lo < hi);
        let (vlo, vhi) = data.volume_range().unwrap();
        assert_eq!(vlo, 0.0);
        assert!(vhi > 0.0);
        let (xlo, xhi) = data.x_range().unwrap();
        assert!(xhi > xlo);
    }

    #[test]
    fn test_x_spacing_uses_timeframe() {
        let data = ChartData::new(small_series());
        // H1 = 3_600_000 ms
        assert_eq!(data.x_spacing(), 3_600_000.0);
    }

    // ---------- Volume panel ----------

    #[test]
    fn test_create_volume_panel_uses_bullish_and_bearish_colors() {
        let data = ChartData::new(small_series());
        let panel = data.create_volume_panel(green(), red());
        assert_eq!(panel.id(), "volume");
        assert_eq!(panel.plot_count(), 1);
        // The single plot should expose histogram bars
        let bars = panel.plots()[0].histogram().unwrap();
        assert_eq!(bars.len(), 3);
        // small_series alternates bullish/bearish: i=0 bullish, i=1 bearish, i=2 bullish
        assert_eq!(bars[0].color, green());
        assert_eq!(bars[1].color, red());
        assert_eq!(bars[2].color, green());
        // y_range is forced to (0, 1.1*max_volume)
        let (ymin, ymax) = panel.fixed_y_range().unwrap();
        assert_eq!(ymin, 0.0);
        // max volume = 300, padded to 330
        assert!((ymax - 330.0).abs() < 1e-6);
    }

    #[test]
    fn test_create_volume_panel_empty_series_uses_fallback_max() {
        let data: ChartData<Timestamp> = ChartData::new(CandleSeries::new());
        let panel = data.create_volume_panel(green(), red());
        let (ymin, ymax) = panel.fixed_y_range().unwrap();
        assert_eq!(ymin, 0.0);
        assert!((ymax - 100.0).abs() < 1e-6);
    }

    // ---------- Timestamp-only helper ----------

    #[test]
    fn test_time_range_propagates() {
        let data = ChartData::new(small_series());
        assert_eq!(data.time_range(), Some((0, 120_000)));
    }
}
