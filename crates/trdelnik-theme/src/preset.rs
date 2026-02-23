//! Pre-built theme presets

use crate::color::Color;
use crate::theme::ChartTheme;
use std::collections::HashMap;

impl ChartTheme {
    /// Dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            background: Color::rgb(19, 23, 34),
            grid: Color::rgb(42, 46, 57),
            text: Color::rgb(209, 212, 220),
            text_muted: Color::rgb(120, 123, 134),
            bullish: Color::rgb(38, 166, 154),
            bearish: Color::rgb(239, 83, 80),
            bullish_wick: Color::rgb(38, 166, 154),
            bearish_wick: Color::rgb(239, 83, 80),
            volume_bullish: Color::rgba(38, 166, 154, 128),
            volume_bearish: Color::rgba(239, 83, 80, 128),
            crosshair: Color::rgb(120, 123, 134),
            crosshair_label_bg: Color::rgb(55, 59, 69),
            border: Color::rgb(42, 46, 57),
            sma: Color::rgb(33, 150, 243),
            ema: Color::rgb(255, 152, 0),
            rsi: Color::rgb(156, 39, 176),
            macd_line: Color::rgb(33, 150, 243),
            macd_signal: Color::rgb(255, 152, 0),
            macd_hist_positive: Color::rgb(38, 166, 154),
            macd_hist_negative: Color::rgb(239, 83, 80),
            rsi_zone: Color::rgba(156, 39, 176, 30),
            indicator_colors: HashMap::new(),
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            background: Color::rgb(255, 255, 255),
            grid: Color::rgb(230, 230, 230),
            text: Color::rgb(19, 23, 34),
            text_muted: Color::rgb(120, 123, 134),
            bullish: Color::rgb(38, 166, 154),
            bearish: Color::rgb(239, 83, 80),
            bullish_wick: Color::rgb(38, 166, 154),
            bearish_wick: Color::rgb(239, 83, 80),
            volume_bullish: Color::rgba(38, 166, 154, 100),
            volume_bearish: Color::rgba(239, 83, 80, 100),
            crosshair: Color::rgb(100, 100, 100),
            crosshair_label_bg: Color::rgb(240, 240, 240),
            border: Color::rgb(200, 200, 200),
            sma: Color::rgb(33, 150, 243),
            ema: Color::rgb(255, 152, 0),
            rsi: Color::rgb(156, 39, 176),
            macd_line: Color::rgb(33, 150, 243),
            macd_signal: Color::rgb(255, 152, 0),
            macd_hist_positive: Color::rgb(38, 166, 154),
            macd_hist_negative: Color::rgb(239, 83, 80),
            rsi_zone: Color::rgba(156, 39, 176, 20),
            indicator_colors: HashMap::new(),
        }
    }

    /// Blue theme (alternative dark)
    pub fn blue() -> Self {
        Self {
            name: "Blue".to_string(),
            background: Color::rgb(16, 24, 40),
            grid: Color::rgb(30, 41, 59),
            text: Color::rgb(226, 232, 240),
            text_muted: Color::rgb(100, 116, 139),
            bullish: Color::rgb(34, 197, 94),
            bearish: Color::rgb(239, 68, 68),
            bullish_wick: Color::rgb(34, 197, 94),
            bearish_wick: Color::rgb(239, 68, 68),
            volume_bullish: Color::rgba(34, 197, 94, 100),
            volume_bearish: Color::rgba(239, 68, 68, 100),
            crosshair: Color::rgb(100, 116, 139),
            crosshair_label_bg: Color::rgb(30, 41, 59),
            border: Color::rgb(30, 41, 59),
            sma: Color::rgb(59, 130, 246),
            ema: Color::rgb(251, 146, 60),
            rsi: Color::rgb(168, 85, 247),
            macd_line: Color::rgb(59, 130, 246),
            macd_signal: Color::rgb(251, 146, 60),
            macd_hist_positive: Color::rgb(34, 197, 94),
            macd_hist_negative: Color::rgb(239, 68, 68),
            rsi_zone: Color::rgba(168, 85, 247, 25),
            indicator_colors: HashMap::new(),
        }
    }

    /// Midnight theme (deep dark)
    pub fn midnight() -> Self {
        Self {
            name: "Midnight".to_string(),
            background: Color::rgb(10, 10, 15),
            grid: Color::rgb(25, 25, 35),
            text: Color::rgb(200, 200, 210),
            text_muted: Color::rgb(90, 90, 100),
            bullish: Color::rgb(0, 200, 150),
            bearish: Color::rgb(255, 80, 100),
            bullish_wick: Color::rgb(0, 200, 150),
            bearish_wick: Color::rgb(255, 80, 100),
            volume_bullish: Color::rgba(0, 200, 150, 80),
            volume_bearish: Color::rgba(255, 80, 100, 80),
            crosshair: Color::rgb(80, 80, 100),
            crosshair_label_bg: Color::rgb(30, 30, 40),
            border: Color::rgb(25, 25, 35),
            sma: Color::rgb(100, 150, 255),
            ema: Color::rgb(255, 180, 50),
            rsi: Color::rgb(180, 100, 255),
            macd_line: Color::rgb(100, 150, 255),
            macd_signal: Color::rgb(255, 180, 50),
            macd_hist_positive: Color::rgb(0, 200, 150),
            macd_hist_negative: Color::rgb(255, 80, 100),
            rsi_zone: Color::rgba(180, 100, 255, 20),
            indicator_colors: HashMap::new(),
        }
    }

    /// Get a list of all built-in theme names
    pub fn preset_names() -> &'static [&'static str] {
        &["Dark", "Light", "Blue", "Midnight"]
    }

    /// Get a preset theme by name
    pub fn preset(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::dark()),
            "light" => Some(Self::light()),
            "blue" => Some(Self::blue()),
            "midnight" => Some(Self::midnight()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets() {
        // All presets should have valid colors
        for name in ChartTheme::preset_names() {
            let theme = ChartTheme::preset(name).unwrap();
            assert_eq!(theme.name.to_lowercase(), name.to_lowercase());
        }
    }

    #[test]
    fn test_preset_lookup() {
        assert!(ChartTheme::preset("dark").is_some());
        assert!(ChartTheme::preset("DARK").is_some());
        assert!(ChartTheme::preset("Dark").is_some());
        assert!(ChartTheme::preset("unknown").is_none());
    }
}
