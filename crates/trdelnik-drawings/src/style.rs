//! Style configuration for drawings

use serde::{Deserialize, Serialize};
use trdelnik_core::Color;

/// Line style configuration
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LineStyle {
    /// Solid line
    Solid,
    /// Dashed line with configurable dash and gap lengths
    Dashed {
        /// Length of each dash in pixels
        dash: f32,
        /// Length of each gap in pixels
        gap: f32,
    },
    /// Dotted line
    Dotted,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self::Solid
    }
}

impl LineStyle {
    /// Create a standard dashed line
    pub fn dashed() -> Self {
        Self::Dashed { dash: 5.0, gap: 3.0 }
    }

    /// Create a dotted line
    pub fn dotted() -> Self {
        Self::Dotted
    }
}

/// Style configuration for a drawing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrawingStyle {
    /// Primary color (lines, borders)
    pub color: Color,
    /// Secondary color (fill, background)
    pub fill_color: Option<Color>,
    /// Line width in pixels
    pub line_width: f32,
    /// Line style
    pub line_style: LineStyle,
    /// Fill opacity (0.0 - 1.0)
    pub fill_opacity: f32,
    /// Font size for labels
    pub font_size: f32,
    /// Show text labels
    pub show_labels: bool,
    /// Show price values
    pub show_prices: bool,
    /// Show percentage values (for Fibonacci, Position tool, etc.)
    pub show_percentages: bool,
}

impl Default for DrawingStyle {
    fn default() -> Self {
        Self {
            color: Color::from_hex("#2962FF").unwrap_or(Color::rgb(41, 98, 255)),
            fill_color: None,
            line_width: 1.0,
            line_style: LineStyle::Solid,
            fill_opacity: 0.1,
            font_size: 11.0,
            show_labels: true,
            show_prices: true,
            show_percentages: true,
        }
    }
}

impl DrawingStyle {
    /// Create a new style with the given color
    pub fn with_color(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Set the line width
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Set the line style
    pub fn line_style(mut self, style: LineStyle) -> Self {
        self.line_style = style;
        self
    }

    /// Set the fill color and opacity
    pub fn fill(mut self, color: Color, opacity: f32) -> Self {
        self.fill_color = Some(color);
        self.fill_opacity = opacity;
        self
    }

    /// Disable labels
    pub fn no_labels(mut self) -> Self {
        self.show_labels = false;
        self
    }

    /// Get the fill color with opacity applied
    pub fn effective_fill_color(&self) -> Option<Color> {
        self.fill_color.as_ref().or(Some(&self.color)).map(|c| {
            Color::rgba(c.r, c.g, c.b, (self.fill_opacity * 255.0) as u8)
        })
    }
}

/// Preset colors for the drawing toolbar
pub const COLOR_PRESETS: &[&str] = &[
    "#2962FF", // Blue
    "#F23645", // Red
    "#089981", // Green
    "#FF9800", // Orange
    "#9C27B0", // Purple
    "#00BCD4", // Cyan
    "#FFEB3B", // Yellow
    "#787B86", // Gray
];

/// Get preset colors as Color values
pub fn preset_colors() -> Vec<Color> {
    COLOR_PRESETS
        .iter()
        .filter_map(|hex| Color::from_hex(hex))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- LineStyle ----------

    #[test]
    fn test_line_style_default_is_solid() {
        assert_eq!(LineStyle::default(), LineStyle::Solid);
    }

    #[test]
    fn test_line_style_dashed_constructor() {
        let s = LineStyle::dashed();
        assert_eq!(s, LineStyle::Dashed { dash: 5.0, gap: 3.0 });
    }

    #[test]
    fn test_line_style_dotted_constructor() {
        assert_eq!(LineStyle::dotted(), LineStyle::Dotted);
    }

    #[test]
    fn test_line_style_variants_distinct() {
        assert_ne!(LineStyle::Solid, LineStyle::Dotted);
        assert_ne!(LineStyle::Solid, LineStyle::dashed());
        assert_ne!(LineStyle::Dotted, LineStyle::dashed());
    }

    #[test]
    fn test_line_style_dashed_inequality_on_lengths() {
        let a = LineStyle::Dashed { dash: 5.0, gap: 3.0 };
        let b = LineStyle::Dashed { dash: 4.0, gap: 3.0 };
        assert_ne!(a, b);
    }

    // ---------- DrawingStyle ----------

    #[test]
    fn test_drawing_style_default_values() {
        let s = DrawingStyle::default();
        assert!((s.line_width - 1.0).abs() < 1e-6);
        assert_eq!(s.line_style, LineStyle::Solid);
        assert!((s.fill_opacity - 0.1).abs() < 1e-6);
        assert!((s.font_size - 11.0).abs() < 1e-6);
        assert!(s.show_labels);
        assert!(s.show_prices);
        assert!(s.show_percentages);
        assert!(s.fill_color.is_none());
    }

    #[test]
    fn test_drawing_style_with_color() {
        let red = Color::rgb(255, 0, 0);
        let s = DrawingStyle::with_color(red);
        assert_eq!(s.color, red);
        // Other fields should remain at defaults
        assert!((s.line_width - 1.0).abs() < 1e-6);
        assert!(s.show_labels);
    }

    #[test]
    fn test_drawing_style_line_width_builder() {
        let s = DrawingStyle::default().line_width(3.0);
        assert!((s.line_width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_drawing_style_line_style_builder() {
        let s = DrawingStyle::default().line_style(LineStyle::Dotted);
        assert_eq!(s.line_style, LineStyle::Dotted);
    }

    #[test]
    fn test_drawing_style_fill_builder() {
        let blue = Color::rgb(0, 0, 255);
        let s = DrawingStyle::default().fill(blue, 0.5);
        assert_eq!(s.fill_color, Some(blue));
        assert!((s.fill_opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_drawing_style_no_labels_disables_labels() {
        let s = DrawingStyle::default().no_labels();
        assert!(!s.show_labels);
    }

    #[test]
    fn test_drawing_style_builders_chain() {
        let red = Color::rgb(255, 0, 0);
        let blue = Color::rgb(0, 0, 255);
        let s = DrawingStyle::with_color(red)
            .line_width(2.5)
            .line_style(LineStyle::dashed())
            .fill(blue, 0.3)
            .no_labels();
        assert_eq!(s.color, red);
        assert!((s.line_width - 2.5).abs() < 1e-6);
        assert_eq!(s.line_style, LineStyle::Dashed { dash: 5.0, gap: 3.0 });
        assert_eq!(s.fill_color, Some(blue));
        assert!((s.fill_opacity - 0.3).abs() < 1e-6);
        assert!(!s.show_labels);
    }

    #[test]
    fn test_effective_fill_color_uses_fill_color_when_set() {
        let red = Color::rgb(255, 0, 0);
        let blue = Color::rgb(0, 0, 255);
        let s = DrawingStyle::with_color(red).fill(blue, 0.5);
        let eff = s.effective_fill_color().unwrap();
        // RGB taken from fill_color, alpha derived from opacity (0.5*255 ≈ 127)
        assert_eq!(eff.r, 0);
        assert_eq!(eff.g, 0);
        assert_eq!(eff.b, 255);
        assert!((eff.a as i32 - 127).abs() <= 1);
    }

    #[test]
    fn test_effective_fill_color_falls_back_to_main_color() {
        let red = Color::rgb(255, 0, 0);
        // Default sets fill_color = None, fill_opacity = 0.1
        let s = DrawingStyle::with_color(red);
        let eff = s.effective_fill_color().unwrap();
        assert_eq!(eff.r, 255);
        assert_eq!(eff.g, 0);
        assert_eq!(eff.b, 0);
    }

    #[test]
    fn test_effective_fill_color_zero_opacity() {
        let s = DrawingStyle::default().fill(Color::rgb(0, 255, 0), 0.0);
        let eff = s.effective_fill_color().unwrap();
        assert_eq!(eff.a, 0);
    }

    // ---------- Presets ----------

    #[test]
    fn test_color_presets_count() {
        assert_eq!(COLOR_PRESETS.len(), 8);
    }

    #[test]
    fn test_preset_colors_returns_valid_colors() {
        let colors = preset_colors();
        // All 8 hex strings parse successfully
        assert_eq!(colors.len(), 8);
    }

    #[test]
    fn test_preset_colors_first_is_blue_2962ff() {
        let colors = preset_colors();
        let first = colors[0];
        assert_eq!(first.r, 0x29);
        assert_eq!(first.g, 0x62);
        assert_eq!(first.b, 0xFF);
    }
}
