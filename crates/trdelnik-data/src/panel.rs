//! Panel container for grouping multiple plots
//!
//! A `Panel` holds one or more `Plot` trait objects, plus panel-level
//! configuration: horizontal reference lines, markers, optional fixed
//! Y-range, and a config struct (id, name, height bounds).

use trdelnik_core::{AxisCoordinate, IndicatorMarker, YAxis};
use trdelnik_render::{aggregate_ranges, HLine, Plot};

/// Configuration for a panel
#[derive(Debug, Clone)]
pub struct PanelConfig {
    /// Unique identifier for this panel
    pub id: String,
    /// Display name for the panel
    pub name: String,
    /// Default height in pixels
    pub default_height: f32,
    /// Minimum height in pixels
    pub min_height: f32,
    /// Maximum height in pixels
    pub max_height: f32,
}

impl PanelConfig {
    /// Create a new panel config with default settings
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            default_height: 100.0,
            min_height: 50.0,
            max_height: 300.0,
        }
    }

    /// Set the display name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the default height
    pub fn height(mut self, height: f32) -> Self {
        self.default_height = height;
        self
    }

    /// Set the minimum height
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }

    /// Set the maximum height
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self::new("panel")
    }
}

/// A panel containing one or more plots.
///
/// Plots are stored as `Box<dyn Plot<X>>` so any visualisation type can live
/// in any panel. The renderer iterates `plots()` and uses trait methods
/// (`lines()`, `histogram()`, `as_any()`) to draw.
#[derive(Debug)]
pub struct Panel<X: AxisCoordinate> {
    /// Panel configuration
    pub config: PanelConfig,
    plots: Vec<Box<dyn Plot<X>>>,
    hlines: Vec<HLine>,
    markers: Vec<IndicatorMarker<X>>,
    y_range: Option<(f64, f64)>,
}

impl<X: AxisCoordinate> Panel<X> {
    /// Create a new panel with the given config
    pub fn new(config: PanelConfig) -> Self {
        Self {
            config,
            plots: Vec::new(),
            hlines: Vec::new(),
            markers: Vec::new(),
            y_range: None,
        }
    }

    /// Add a plot to this panel.
    pub fn add_plot(&mut self, plot: Box<dyn Plot<X>>) {
        self.plots.push(plot);
    }

    /// Add a horizontal reference line.
    pub fn add_hline(&mut self, hline: HLine) {
        self.hlines.push(hline);
    }

    /// Add a marker to this panel.
    pub fn add_marker(&mut self, marker: IndicatorMarker<X>) {
        self.markers.push(marker);
    }

    /// Add multiple markers to this panel.
    pub fn add_markers(&mut self, markers: impl IntoIterator<Item = IndicatorMarker<X>>) {
        self.markers.extend(markers);
    }

    /// Set the fixed Y-axis range.
    pub fn set_y_range(&mut self, min: f64, max: f64) {
        self.y_range = Some((min, max));
    }

    /// All plots in this panel.
    pub fn plots(&self) -> &[Box<dyn Plot<X>>] {
        &self.plots
    }

    /// All horizontal reference lines.
    pub fn hlines(&self) -> &[HLine] {
        &self.hlines
    }

    /// All markers attached to this panel.
    pub fn markers(&self) -> &[IndicatorMarker<X>] {
        &self.markers
    }

    /// Fixed Y-range override, if any.
    pub fn fixed_y_range(&self) -> Option<(f64, f64)> {
        self.y_range
    }

    /// Whether any plot in this panel uses the right axis.
    pub fn has_right_axis(&self) -> bool {
        self.plots.iter().any(|p| p.has_axis(YAxis::Right))
    }

    /// Y-range for the left axis (uses fixed range if set).
    pub fn left_y_range(&self) -> (f64, f64) {
        if let Some(range) = self.y_range {
            return range;
        }
        let ranges = self.plots.iter().filter_map(|p| p.y_range(YAxis::Left));
        match aggregate_ranges(ranges) {
            Some((lo, hi)) => pad_range(lo, hi),
            None => (0.0, 100.0),
        }
    }

    /// Y-range for the right axis.
    pub fn right_y_range(&self) -> (f64, f64) {
        let ranges = self.plots.iter().filter_map(|p| p.y_range(YAxis::Right));
        match aggregate_ranges(ranges) {
            Some((lo, hi)) => pad_range(lo, hi),
            None => (0.0, 100.0),
        }
    }

    /// Get the panel ID.
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get the panel name.
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Whether the panel has no plots.
    pub fn is_empty(&self) -> bool {
        self.plots.is_empty()
    }

    /// Number of plots in this panel.
    pub fn plot_count(&self) -> usize {
        self.plots.len()
    }
}

fn pad_range(lo: f64, hi: f64) -> (f64, f64) {
    let padding = (hi - lo) * 0.1;
    (lo - padding, hi + padding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Color, Index, IndicatorLine, MarkerShape};
    use trdelnik_render::StandardPlot;

    fn red() -> Color {
        Color::rgb(255, 0, 0)
    }

    fn left_axis_plot(id: &str, ys: &[f64]) -> StandardPlot<Index> {
        let xs: Vec<Index> = (0..ys.len()).map(Index).collect();
        let opts: Vec<Option<f64>> = ys.iter().map(|&y| Some(y)).collect();
        let mut p = StandardPlot::new(id);
        p.add_line(IndicatorLine::from_xy("L", id, &xs, &opts));
        p
    }

    fn right_axis_plot(id: &str, ys: &[f64]) -> StandardPlot<Index> {
        let mut p = left_axis_plot(id, ys);
        p.move_to_axis(YAxis::Right);
        p
    }

    // ---------- PanelConfig ----------

    #[test]
    fn test_panel_config_new_defaults() {
        let c = PanelConfig::new("rsi");
        assert_eq!(c.id, "rsi");
        assert_eq!(c.name, "rsi");
        assert!((c.default_height - 100.0).abs() < 1e-6);
        assert!((c.min_height - 50.0).abs() < 1e-6);
        assert!((c.max_height - 300.0).abs() < 1e-6);
    }

    #[test]
    fn test_panel_config_default_uses_panel_id() {
        let c = PanelConfig::default();
        assert_eq!(c.id, "panel");
        assert_eq!(c.name, "panel");
    }

    #[test]
    fn test_panel_config_builders_chain() {
        let c = PanelConfig::new("x")
            .name("X axis")
            .height(200.0)
            .min_height(80.0)
            .max_height(500.0);
        assert_eq!(c.name, "X axis");
        assert!((c.default_height - 200.0).abs() < 1e-6);
        assert!((c.min_height - 80.0).abs() < 1e-6);
        assert!((c.max_height - 500.0).abs() < 1e-6);
    }

    // ---------- Panel basics ----------

    #[test]
    fn test_panel_new_is_empty() {
        let p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        assert!(p.is_empty());
        assert_eq!(p.plot_count(), 0);
        assert!(p.plots().is_empty());
        assert!(p.hlines().is_empty());
        assert!(p.markers().is_empty());
        assert!(p.fixed_y_range().is_none());
    }

    #[test]
    fn test_panel_id_and_name() {
        let p: Panel<Index> = Panel::new(PanelConfig::new("rsi").name("RSI Panel"));
        assert_eq!(p.id(), "rsi");
        assert_eq!(p.name(), "RSI Panel");
    }

    #[test]
    fn test_add_plot_increases_count() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_plot(Box::new(left_axis_plot("a", &[1.0, 2.0])));
        p.add_plot(Box::new(left_axis_plot("b", &[3.0])));
        assert_eq!(p.plot_count(), 2);
        assert!(!p.is_empty());
    }

    // ---------- HLines / markers / y-range ----------

    #[test]
    fn test_add_hline() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_hline(HLine::new(50.0));
        p.add_hline(HLine::colored(70.0, red()));
        assert_eq!(p.hlines().len(), 2);
    }

    #[test]
    fn test_add_marker_one() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_marker(IndicatorMarker {
            x: Index(0),
            y: 1.0,
            shape: MarkerShape::Circle,
            color: red(),
            label: None,
        });
        assert_eq!(p.markers().len(), 1);
    }

    #[test]
    fn test_add_markers_iter() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        let ms = vec![
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
        ];
        p.add_markers(ms);
        assert_eq!(p.markers().len(), 2);
    }

    #[test]
    fn test_set_y_range_records_fixed_range() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.set_y_range(0.0, 100.0);
        assert_eq!(p.fixed_y_range(), Some((0.0, 100.0)));
    }

    // ---------- Axis detection ----------

    #[test]
    fn test_has_right_axis_false_when_only_left() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_plot(Box::new(left_axis_plot("a", &[1.0])));
        assert!(!p.has_right_axis());
    }

    #[test]
    fn test_has_right_axis_true_when_any_plot_uses_right() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_plot(Box::new(left_axis_plot("a", &[1.0])));
        p.add_plot(Box::new(right_axis_plot("b", &[2.0])));
        assert!(p.has_right_axis());
    }

    // ---------- Y-range derivation ----------

    #[test]
    fn test_left_y_range_uses_fixed_when_set() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.set_y_range(-10.0, 10.0);
        // Even with a plot present, the fixed range wins
        p.add_plot(Box::new(left_axis_plot("a", &[1000.0])));
        assert_eq!(p.left_y_range(), (-10.0, 10.0));
    }

    #[test]
    fn test_left_y_range_pads_data_range_by_10pct() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        // Single line spanning [0, 100]
        p.add_plot(Box::new(left_axis_plot("a", &[0.0, 50.0, 100.0])));
        let (lo, hi) = p.left_y_range();
        // Expected pad = (100-0)*0.1 = 10
        assert!((lo - (-10.0)).abs() < 1e-6);
        assert!((hi - 110.0).abs() < 1e-6);
    }

    #[test]
    fn test_left_y_range_fallback_when_no_plots() {
        let p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        assert_eq!(p.left_y_range(), (0.0, 100.0));
    }

    #[test]
    fn test_right_y_range_pads_data_range() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_plot(Box::new(right_axis_plot("r", &[10.0, 20.0])));
        let (lo, hi) = p.right_y_range();
        // pad = (20-10)*0.1 = 1
        assert!((lo - 9.0).abs() < 1e-6);
        assert!((hi - 21.0).abs() < 1e-6);
    }

    #[test]
    fn test_right_y_range_fallback_when_no_right_axis_plots() {
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        // Only a left-axis plot
        p.add_plot(Box::new(left_axis_plot("a", &[1.0, 2.0])));
        assert_eq!(p.right_y_range(), (0.0, 100.0));
    }

    // ---------- pad_range (private helper, exercised through ranges above) ----------

    #[test]
    fn test_pad_range_zero_extent() {
        // Constant single value → range collapses to (v, v) → padded by 0
        let mut p: Panel<Index> = Panel::new(PanelConfig::new("p"));
        p.add_plot(Box::new(left_axis_plot("a", &[5.0])));
        let (lo, hi) = p.left_y_range();
        // pad = (5-5)*0.1 = 0, so lo == hi == 5
        assert!((lo - 5.0).abs() < 1e-6);
        assert!((hi - 5.0).abs() < 1e-6);
    }
}
