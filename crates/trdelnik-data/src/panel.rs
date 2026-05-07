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
