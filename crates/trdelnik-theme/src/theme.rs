//! Chart theme configuration

use crate::color::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete color scheme for a trading chart
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTheme {
    /// Theme name
    pub name: String,

    // === Background & Layout ===
    /// Background color of the chart
    pub background: Color,
    /// Grid line color
    pub grid: Color,
    /// Border color
    pub border: Color,

    // === Text ===
    /// Text color for labels
    pub text: Color,
    /// Muted text color
    pub text_muted: Color,

    // === Candles ===
    /// Bullish (up) candle color
    pub bullish: Color,
    /// Bearish (down) candle color
    pub bearish: Color,
    /// Bullish candle wick color
    pub bullish_wick: Color,
    /// Bearish candle wick color
    pub bearish_wick: Color,

    // === Volume ===
    /// Volume bar color for bullish candles
    pub volume_bullish: Color,
    /// Volume bar color for bearish candles
    pub volume_bearish: Color,

    // === Crosshair ===
    /// Crosshair line color
    pub crosshair: Color,
    /// Crosshair label background
    pub crosshair_label_bg: Color,

    // === Indicators (default colors) ===
    /// SMA line color
    pub sma: Color,
    /// EMA line color
    pub ema: Color,
    /// RSI line color
    pub rsi: Color,
    /// MACD line color
    pub macd_line: Color,
    /// MACD signal line color
    pub macd_signal: Color,
    /// MACD histogram positive color
    pub macd_hist_positive: Color,
    /// MACD histogram negative color
    pub macd_hist_negative: Color,
    /// RSI overbought/oversold zone color
    pub rsi_zone: Color,

    // === Custom indicator colors ===
    /// Custom colors for indicators by name
    #[serde(default)]
    pub indicator_colors: HashMap<String, Color>,
}

impl ChartTheme {
    /// Get a color for a named indicator, falling back to a default
    pub fn get_indicator_color(&self, name: &str, default: Color) -> Color {
        self.indicator_colors.get(name).copied().unwrap_or(default)
    }

    /// Set a custom color for a named indicator
    pub fn set_indicator_color(&mut self, name: impl Into<String>, color: Color) {
        self.indicator_colors.insert(name.into(), color);
    }

    /// Remove a custom indicator color (will use default)
    pub fn remove_indicator_color(&mut self, name: &str) -> Option<Color> {
        self.indicator_colors.remove(name)
    }

    /// Save the theme to a file as JSON
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), ThemeError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a theme from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, ThemeError> {
        let json = std::fs::read_to_string(path)?;
        let theme = serde_json::from_str(&json)?;
        Ok(theme)
    }

    /// Serialize the theme to a JSON string
    pub fn to_json(&self) -> Result<String, ThemeError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize a theme from a JSON string
    pub fn from_json(json: &str) -> Result<Self, ThemeError> {
        Ok(serde_json::from_str(json)?)
    }
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Error type for theme operations
#[derive(Debug)]
pub enum ThemeError {
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// File I/O error
    Io(std::io::Error),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::Json(e) => write!(f, "JSON error: {}", e),
            ThemeError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for ThemeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThemeError::Json(e) => Some(e),
            ThemeError::Io(e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for ThemeError {
    fn from(e: serde_json::Error) -> Self {
        ThemeError::Json(e)
    }
}

impl From<std::io::Error> for ThemeError {
    fn from(e: std::io::Error) -> Self {
        ThemeError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_serialization() {
        let theme = ChartTheme::dark();
        let json = theme.to_json().unwrap();
        let loaded = ChartTheme::from_json(&json).unwrap();
        assert_eq!(theme, loaded);
    }

    #[test]
    fn test_indicator_colors() {
        let mut theme = ChartTheme::dark();

        // Set custom color
        theme.set_indicator_color("my_sma", Color::rgb(255, 0, 0));
        assert_eq!(
            theme.get_indicator_color("my_sma", Color::black()),
            Color::rgb(255, 0, 0)
        );

        // Get non-existent color returns default
        assert_eq!(
            theme.get_indicator_color("unknown", Color::rgb(0, 255, 0)),
            Color::rgb(0, 255, 0)
        );

        // Remove custom color
        theme.remove_indicator_color("my_sma");
        assert_eq!(
            theme.get_indicator_color("my_sma", Color::black()),
            Color::black()
        );
    }
}
