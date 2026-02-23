//! Chart configuration

/// Configuration for the trading chart
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// Show volume panel
    pub show_volume: bool,
    /// Volume panel height ratio (0.0 to 1.0)
    pub volume_height_ratio: f32,
    /// Show crosshair
    pub show_crosshair: bool,
    /// Show grid
    pub show_grid: bool,
    /// Show legend
    pub show_legend: bool,
    /// Candle width as fraction of spacing (0.0 to 1.0)
    pub candle_width_ratio: f64,
    /// Minimum visible candles
    pub min_visible_candles: usize,
    /// Maximum visible candles
    pub max_visible_candles: usize,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            show_volume: true,
            volume_height_ratio: 0.2,
            show_crosshair: true,
            show_grid: true,
            show_legend: true,
            candle_width_ratio: 0.8,
            min_visible_candles: 10,
            max_visible_candles: 500,
        }
    }
}

impl ChartConfig {
    /// Create a new config with volume panel disabled
    pub fn without_volume() -> Self {
        Self {
            show_volume: false,
            ..Default::default()
        }
    }

    /// Builder: set show_volume
    pub fn with_volume(mut self, show: bool) -> Self {
        self.show_volume = show;
        self
    }

    /// Builder: set show_grid
    pub fn with_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Builder: set show_legend
    pub fn with_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Builder: set candle width ratio
    pub fn with_candle_width(mut self, ratio: f64) -> Self {
        self.candle_width_ratio = ratio.clamp(0.1, 1.0);
        self
    }
}
