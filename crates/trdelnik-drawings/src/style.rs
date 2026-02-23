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
