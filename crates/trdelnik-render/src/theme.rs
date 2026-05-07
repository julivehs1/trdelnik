//! Theme abstraction used by `Plot` implementations.
//!
//! `Plot` does not depend on the concrete `trdelnik_theme::ChartTheme` —
//! instead it asks an `IndicatorTheme` for a colour given a `line_id`.
//! Backends adapt their own theme types to this trait.

use trdelnik_core::Color;

/// Source of theme colours for plot rendering.
pub trait IndicatorTheme {
    /// Colour for a line, looked up by its `line_id` (e.g. `"sma"`,
    /// `"bb_upper"`, `"macd_line"`). Backends can fall back to a default
    /// when the id is unknown.
    fn line_color(&self, line_id: &str) -> Color;

    /// Colour for grid lines / reference levels (default for `HLine` when
    /// no override is set).
    fn grid_color(&self) -> Color;
}
