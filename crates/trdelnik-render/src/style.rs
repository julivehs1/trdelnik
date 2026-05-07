//! Visual style primitives shared by all renderers.

use trdelnik_core::Color;

/// Style of a stroked line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    /// Solid stroke.
    Solid,
    /// Dashed stroke. `length` is the length of each dash segment in plot units.
    Dashed { length: f32 },
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle::Solid
    }
}

/// A stroke style with width + color + dash pattern.
#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    /// Line color.
    pub color: Color,
    /// Line width in pixels.
    pub width: f32,
    /// Solid or dashed.
    pub style: LineStyle,
}

impl Stroke {
    /// Solid stroke with the given colour and 1.5px width.
    pub fn solid(color: Color) -> Self {
        Self {
            color,
            width: 1.5,
            style: LineStyle::Solid,
        }
    }

    /// Dashed stroke with the given colour, 1.5px width, dash length 4.
    pub fn dashed(color: Color) -> Self {
        Self {
            color,
            width: 1.5,
            style: LineStyle::Dashed { length: 4.0 },
        }
    }

    /// Override the width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}
