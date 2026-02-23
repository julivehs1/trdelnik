//! egui theme conversion utilities

use egui::Color32;
use trdelnik_theme::Color;

/// Convert a platform-agnostic Color to egui Color32
pub fn to_egui_color(color: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}

/// Convert egui Color32 to platform-agnostic Color
pub fn from_egui_color(color: Color32) -> Color {
    Color::rgba(color.r(), color.g(), color.b(), color.a())
}

/// Extension trait for Color to easily convert to Color32
pub trait ToEguiColor {
    fn to_egui(&self) -> Color32;
}

impl ToEguiColor for Color {
    fn to_egui(&self) -> Color32 {
        to_egui_color(*self)
    }
}

/// Extension trait for Color32 to convert to platform-agnostic Color
pub trait FromEguiColor {
    fn to_color(&self) -> Color;
}

impl FromEguiColor for Color32 {
    fn to_color(&self) -> Color {
        from_egui_color(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversion() {
        // Test with fully opaque color (no premultiplication issues)
        let color = Color::rgb(255, 128, 64);
        let egui_color = to_egui_color(color);
        let back = from_egui_color(egui_color);

        assert_eq!(color, back);
    }

    #[test]
    fn test_extension_traits() {
        let color = Color::rgb(100, 150, 200);
        let egui_color = color.to_egui();

        assert_eq!(egui_color.r(), 100);
        assert_eq!(egui_color.g(), 150);
        assert_eq!(egui_color.b(), 200);
        assert_eq!(egui_color.a(), 255);
    }
}
