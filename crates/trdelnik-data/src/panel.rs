//! Panel container for grouping multiple indicators
//!
//! A Panel can contain multiple plot data sets with support for dual Y-axes,
//! horizontal reference lines, and fixed Y-range.

use trdelnik_core::{
    y_range_with_padding, AxisCoordinate, HLine, IndicatorMarker, PlotData, YAxis,
};

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

/// A panel containing one or more plot data sets
#[derive(Debug, Clone)]
pub struct Panel<X: AxisCoordinate> {
    /// Panel configuration
    pub config: PanelConfig,
    /// Plot data sets in this panel
    pub plots: Vec<PlotData<X>>,
    /// Horizontal reference lines (e.g. 30/70 for RSI, 0 for MACD)
    pub hlines: Vec<HLine>,
    /// Markers on this panel (e.g. signals)
    pub markers: Vec<IndicatorMarker<X>>,
    /// Fixed Y-axis range (e.g. 0-100 for RSI)
    pub y_range: Option<(f64, f64)>,
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

    /// Add plot data to this panel
    pub fn add_plot(&mut self, plot: PlotData<X>) {
        self.plots.push(plot);
    }

    /// Add a horizontal reference line
    pub fn add_hline(&mut self, hline: HLine) {
        self.hlines.push(hline);
    }

    /// Add a marker to this panel
    pub fn add_marker(&mut self, marker: IndicatorMarker<X>) {
        self.markers.push(marker);
    }

    /// Add multiple markers to this panel
    pub fn add_markers(&mut self, markers: impl IntoIterator<Item = IndicatorMarker<X>>) {
        self.markers.extend(markers);
    }

    /// Set the fixed Y-axis range
    pub fn set_y_range(&mut self, min: f64, max: f64) {
        self.y_range = Some((min, max));
    }

    /// Check if this panel has any right-axis data
    pub fn has_right_axis(&self) -> bool {
        self.plots.iter().any(|p| p.has_right_axis())
    }

    /// Calculate the Y range for the left axis
    pub fn left_y_range(&self) -> (f64, f64) {
        if let Some(range) = self.y_range {
            return range;
        }
        y_range_with_padding(self.axis_values(YAxis::Left))
    }

    /// Calculate the Y range for the right axis
    pub fn right_y_range(&self) -> (f64, f64) {
        y_range_with_padding(self.axis_values(YAxis::Right))
    }

    /// All numeric values (line points + histogram bars) on a given axis.
    fn axis_values(&self, axis: YAxis) -> impl Iterator<Item = f64> + '_ {
        self.plots.iter().flat_map(move |plot| {
            let line_values = plot
                .lines
                .iter()
                .filter(move |l| l.axis == axis)
                .flat_map(|l| l.points.iter().filter_map(|(_, y)| *y));
            let hist_values = plot
                .histogram
                .iter()
                .flatten()
                .filter(move |b| b.axis == axis)
                .map(|b| b.value);
            line_values.chain(hist_values)
        })
    }

    /// Get the panel ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get the panel name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Check if the panel is empty
    pub fn is_empty(&self) -> bool {
        self.plots.is_empty()
    }
}
