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

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::Color;

    #[test]
    fn test_line_style_default_is_solid() {
        let s: LineStyle = Default::default();
        assert_eq!(s, LineStyle::Solid);
    }

    #[test]
    fn test_line_style_dashed_eq() {
        let a = LineStyle::Dashed { length: 4.0 };
        let b = LineStyle::Dashed { length: 4.0 };
        let c = LineStyle::Dashed { length: 5.0 };
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, LineStyle::Solid);
    }

    #[test]
    fn test_stroke_solid() {
        let color = Color::rgb(255, 0, 0);
        let s = Stroke::solid(color);
        assert_eq!(s.color, color);
        assert!((s.width - 1.5).abs() < 1e-6);
        assert_eq!(s.style, LineStyle::Solid);
    }

    #[test]
    fn test_stroke_dashed() {
        let color = Color::rgb(0, 255, 0);
        let s = Stroke::dashed(color);
        assert_eq!(s.color, color);
        assert!((s.width - 1.5).abs() < 1e-6);
        assert_eq!(s.style, LineStyle::Dashed { length: 4.0 });
    }

    #[test]
    fn test_stroke_with_width() {
        let s = Stroke::solid(Color::rgb(0, 0, 0)).with_width(3.0);
        assert!((s.width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_stroke_with_width_chained_preserves_other_fields() {
        let color = Color::rgb(10, 20, 30);
        let s = Stroke::dashed(color).with_width(2.0);
        assert_eq!(s.color, color);
        assert!((s.width - 2.0).abs() < 1e-6);
        assert_eq!(s.style, LineStyle::Dashed { length: 4.0 });
    }
}
