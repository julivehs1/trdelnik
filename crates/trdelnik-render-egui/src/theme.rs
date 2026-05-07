//! Adapter wrapping `trdelnik_theme::ChartTheme` as an `IndicatorTheme`.

use trdelnik_core::Color;
use trdelnik_render::IndicatorTheme;
use trdelnik_theme::ChartTheme;

/// Adapter that turns a `ChartTheme` into the backend-agnostic
/// `IndicatorTheme`. Cheap to construct (holds a borrow).
pub struct ChartThemeAdapter<'a> {
    theme: &'a ChartTheme,
}

impl<'a> ChartThemeAdapter<'a> {
    /// Wrap a borrowed `ChartTheme`.
    pub fn new(theme: &'a ChartTheme) -> Self {
        Self { theme }
    }
}

impl<'a> IndicatorTheme for ChartThemeAdapter<'a> {
    fn line_color(&self, line_id: &str) -> Color {
        let default = match line_id {
            "sma" | "wma" | "bb_upper" | "bb_lower" | "kc_upper" | "kc_lower" => self.theme.sma,
            "ema" | "bb_middle" | "kc_middle" => self.theme.ema,
            "rsi" => self.theme.rsi,
            "macd_line" => self.theme.macd_line,
            "macd_signal" | "ppo_signal" => self.theme.macd_signal,
            _ => self.theme.sma,
        };
        self.theme.get_indicator_color(line_id, default)
    }

    fn grid_color(&self) -> Color {
        self.theme.grid
    }
}
