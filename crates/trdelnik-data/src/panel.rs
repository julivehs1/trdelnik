//! Panel container for grouping multiple indicators
//!
//! A Panel can contain multiple indicators with support for dual Y-axes.

use trdelnik_core::{AxisCoordinate, IndicatorOutput, YAxis};

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

/// A panel containing one or more indicators
#[derive(Debug, Clone)]
pub struct Panel<X: AxisCoordinate> {
    /// Panel configuration
    pub config: PanelConfig,
    /// Indicators in this panel
    pub indicators: Vec<IndicatorOutput<X>>,
}

impl<X: AxisCoordinate> Panel<X> {
    /// Create a new panel with the given config
    pub fn new(config: PanelConfig) -> Self {
        Self {
            config,
            indicators: Vec::new(),
        }
    }

    /// Create a new panel with auto-generated config from an indicator ID
    pub fn from_indicator_id(id: impl Into<String>) -> Self {
        Self::new(PanelConfig::new(id))
    }

    /// Add an indicator to this panel
    pub fn add_indicator(&mut self, indicator: IndicatorOutput<X>) {
        self.indicators.push(indicator);
    }

    /// Check if this panel has any right-axis data
    pub fn has_right_axis(&self) -> bool {
        self.indicators.iter().any(|i| i.has_right_axis())
    }

    /// Calculate the Y range for the left axis
    pub fn left_y_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for indicator in &self.indicators {
            // Check for fixed range first
            if let Some((y_min, y_max)) = indicator.y_range {
                // Only apply if this indicator has left-axis data
                if indicator.left_lines().count() > 0 {
                    min = min.min(y_min);
                    max = max.max(y_max);
                    continue;
                }
            }

            // Calculate from data
            for line in indicator.left_lines() {
                for (_, y) in &line.points {
                    if let Some(y) = y {
                        min = min.min(*y);
                        max = max.max(*y);
                    }
                }
            }

            // Include left-axis histogram
            if let Some(histogram) = &indicator.histogram {
                for bar in histogram.iter().filter(|b| b.axis == YAxis::Left) {
                    min = min.min(bar.value);
                    max = max.max(bar.value);
                }
            }
        }

        if min.is_infinite() {
            min = 0.0;
        }
        if max.is_infinite() {
            max = 100.0;
        }

        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    /// Calculate the Y range for the right axis
    pub fn right_y_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for indicator in &self.indicators {
            for line in indicator.right_lines() {
                for (_, y) in &line.points {
                    if let Some(y) = y {
                        min = min.min(*y);
                        max = max.max(*y);
                    }
                }
            }

            // Include right-axis histogram
            if let Some(histogram) = &indicator.histogram {
                for bar in histogram.iter().filter(|b| b.axis == YAxis::Right) {
                    min = min.min(bar.value);
                    max = max.max(bar.value);
                }
            }
        }

        if min.is_infinite() {
            min = 0.0;
        }
        if max.is_infinite() {
            max = 100.0;
        }

        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    /// Get all reference lines from all indicators in this panel
    pub fn all_reference_lines(&self) -> Vec<f64> {
        let mut lines: Vec<f64> = Vec::new();
        for indicator in &self.indicators {
            lines.extend(indicator.reference_lines.iter().copied());
        }
        // Remove duplicates
        lines.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        lines.dedup();
        lines
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
        self.indicators.is_empty()
    }
}

/// Builder for configuring a panel
pub struct PanelBuilder {
    config: PanelConfig,
}

impl PanelBuilder {
    /// Create a new panel builder
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            config: PanelConfig::new(id),
        }
    }

    /// Set the display name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the default height
    pub fn height(mut self, height: f32) -> Self {
        self.config.default_height = height;
        self
    }

    /// Set the minimum height
    pub fn min_height(mut self, height: f32) -> Self {
        self.config.min_height = height;
        self
    }

    /// Set the maximum height
    pub fn max_height(mut self, height: f32) -> Self {
        self.config.max_height = height;
        self
    }

    /// Build the panel config
    pub fn build(self) -> PanelConfig {
        self.config
    }
}
