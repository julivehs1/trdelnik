//! Platform-agnostic color type

use serde::{Deserialize, Serialize};

/// A platform-agnostic RGBA color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255, where 255 is fully opaque)
    pub a: u8,
}

impl Color {
    /// Create a new fully opaque color from RGB values
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color from RGBA values
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a hex string (e.g., "#FF5733" or "FF5733")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to a hex string (e.g., "#FF5733")
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Create a fully transparent color
    pub const fn transparent() -> Self {
        Self::rgba(0, 0, 0, 0)
    }

    /// Create black
    pub const fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    /// Create white
    pub const fn white() -> Self {
        Self::rgb(255, 255, 255)
    }

    /// Returns true if this color is fully transparent
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }

    /// Returns true if this color is fully opaque
    pub fn is_opaque(&self) -> bool {
        self.a == 255
    }

    /// Set the alpha value, returning a new color
    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Blend this color with another using this color's alpha
    pub fn blend_over(&self, background: Color) -> Color {
        if self.a == 255 {
            return *self;
        }
        if self.a == 0 {
            return background;
        }

        let alpha = self.a as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        Color::rgb(
            (self.r as f32 * alpha + background.r as f32 * inv_alpha) as u8,
            (self.g as f32 * alpha + background.g as f32 * inv_alpha) as u8,
            (self.b as f32 * alpha + background.b as f32 * inv_alpha) as u8,
        )
    }

    /// Lighten the color by a factor (0.0 = no change, 1.0 = white)
    pub fn lighten(&self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self::rgba(
            (self.r as f32 + (255.0 - self.r as f32) * factor) as u8,
            (self.g as f32 + (255.0 - self.g as f32) * factor) as u8,
            (self.b as f32 + (255.0 - self.b as f32) * factor) as u8,
            self.a,
        )
    }

    /// Darken the color by a factor (0.0 = no change, 1.0 = black)
    pub fn darken(&self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self::rgba(
            (self.r as f32 * (1.0 - factor)) as u8,
            (self.g as f32 * (1.0 - factor)) as u8,
            (self.b as f32 * (1.0 - factor)) as u8,
            self.a,
        )
    }

    /// Parse color from name (e.g., "blue", "red", "#FF0000")
    ///
    /// Supports common color names and hex strings. Returns light gray for unknown names.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "blue" => Self::rgb(100, 149, 237),   // Cornflower blue
            "red" => Self::rgb(220, 80, 80),
            "green" => Self::rgb(80, 200, 120),
            "yellow" => Self::rgb(240, 230, 140),
            "orange" => Self::rgb(255, 165, 0),
            "purple" => Self::rgb(147, 112, 219),
            "cyan" => Self::rgb(0, 255, 255),
            "magenta" => Self::rgb(255, 0, 255),
            "white" => Self::rgb(255, 255, 255),
            "gray" | "grey" => Self::rgb(128, 128, 128),
            "black" => Self::rgb(0, 0, 0),
            "pink" => Self::rgb(255, 182, 193),
            "brown" => Self::rgb(139, 90, 43),
            "lime" => Self::rgb(50, 205, 50),
            "teal" => Self::rgb(0, 128, 128),
            "navy" => Self::rgb(0, 0, 128),
            "gold" => Self::rgb(255, 215, 0),
            "silver" => Self::rgb(192, 192, 192),
            s if s.starts_with('#') => Self::from_hex(s).unwrap_or(Self::rgb(200, 200, 200)),
            _ => Self::rgb(200, 200, 200), // Default: light gray
        }
    }

    /// Convert to an array [r, g, b, a]
    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Create from an array [r, g, b, a]
    pub fn from_array(arr: [u8; 4]) -> Self {
        Self::rgba(arr[0], arr[1], arr[2], arr[3])
    }

    /// Convert to normalized floats [0.0, 1.0]
    pub fn to_normalized(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::black()
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_hex() {
        let c = Color::from_hex("#FF8040").unwrap();
        assert_eq!(c, Color::rgb(255, 128, 64));
        assert_eq!(c.to_hex(), "#FF8040");
    }

    #[test]
    fn test_color_hex_with_alpha() {
        let c = Color::from_hex("#FF804080").unwrap();
        assert_eq!(c, Color::rgba(255, 128, 64, 128));
    }

    #[test]
    fn test_color_lighten_darken() {
        let c = Color::rgb(100, 100, 100);
        let lighter = c.lighten(0.5);
        let darker = c.darken(0.5);

        assert!(lighter.r > c.r);
        assert!(darker.r < c.r);
    }

    #[test]
    fn test_color_from_name() {
        // Named colors
        assert_eq!(Color::from_name("blue"), Color::rgb(100, 149, 237));
        assert_eq!(Color::from_name("BLUE"), Color::rgb(100, 149, 237)); // Case insensitive
        assert_eq!(Color::from_name("red"), Color::rgb(220, 80, 80));
        assert_eq!(Color::from_name("green"), Color::rgb(80, 200, 120));

        // Gray/grey variants
        assert_eq!(Color::from_name("gray"), Color::rgb(128, 128, 128));
        assert_eq!(Color::from_name("grey"), Color::rgb(128, 128, 128));

        // Hex colors
        assert_eq!(Color::from_name("#FF0000"), Color::rgb(255, 0, 0));
        assert_eq!(Color::from_name("#00ff00"), Color::rgb(0, 255, 0));

        // Unknown color defaults to light gray
        assert_eq!(Color::from_name("unknown"), Color::rgb(200, 200, 200));
    }

    // ---------- Constructors / constants ----------

    #[test]
    fn test_rgba_constructor() {
        let c = Color::rgba(10, 20, 30, 40);
        assert_eq!(c, Color { r: 10, g: 20, b: 30, a: 40 });
    }

    #[test]
    fn test_transparent_black_white_constants() {
        assert_eq!(Color::transparent(), Color::rgba(0, 0, 0, 0));
        assert_eq!(Color::black(), Color::rgb(0, 0, 0));
        assert_eq!(Color::white(), Color::rgb(255, 255, 255));
    }

    #[test]
    fn test_default_is_black() {
        assert_eq!(Color::default(), Color::black());
    }

    // ---------- Hex parsing edge cases ----------

    #[test]
    fn test_from_hex_strips_leading_hash() {
        assert_eq!(Color::from_hex("FF8040"), Color::from_hex("#FF8040"));
    }

    #[test]
    fn test_from_hex_invalid_length_returns_none() {
        assert!(Color::from_hex("F").is_none());
        assert!(Color::from_hex("FF").is_none());
        assert!(Color::from_hex("FFAA").is_none());
        assert!(Color::from_hex("FF8040AABB").is_none()); // 10 chars
    }

    #[test]
    fn test_from_hex_invalid_chars_returns_none() {
        assert!(Color::from_hex("ZZZZZZ").is_none());
        assert!(Color::from_hex("FF80GG").is_none());
    }

    // ---------- Display / to_hex with alpha ----------

    #[test]
    fn test_to_hex_includes_alpha_when_not_opaque() {
        let c = Color::rgba(255, 128, 64, 128);
        assert_eq!(c.to_hex(), "#FF804080");
    }

    #[test]
    fn test_display_uses_to_hex() {
        let c = Color::rgb(255, 0, 0);
        assert_eq!(format!("{}", c), c.to_hex());
    }

    // ---------- Transparency / opacity helpers ----------

    #[test]
    fn test_is_transparent_and_is_opaque() {
        assert!(Color::transparent().is_transparent());
        assert!(!Color::transparent().is_opaque());
        assert!(Color::black().is_opaque());
        assert!(!Color::black().is_transparent());
        let mid = Color::rgba(0, 0, 0, 100);
        assert!(!mid.is_transparent());
        assert!(!mid.is_opaque());
    }

    #[test]
    fn test_with_alpha_changes_only_alpha() {
        let c = Color::rgb(10, 20, 30);
        let faded = c.with_alpha(50);
        assert_eq!(faded.r, 10);
        assert_eq!(faded.g, 20);
        assert_eq!(faded.b, 30);
        assert_eq!(faded.a, 50);
    }

    // ---------- blend_over ----------

    #[test]
    fn test_blend_over_opaque_returns_self() {
        let red = Color::rgb(255, 0, 0);
        let blue = Color::rgb(0, 0, 255);
        assert_eq!(red.blend_over(blue), red);
    }

    #[test]
    fn test_blend_over_transparent_returns_background() {
        let bg = Color::rgb(0, 0, 255);
        assert_eq!(Color::transparent().blend_over(bg), bg);
    }

    #[test]
    fn test_blend_over_50pct_alpha_mixes_evenly() {
        let red50 = Color::rgba(200, 0, 0, 128);
        let bg = Color::rgb(0, 200, 0);
        let mixed = red50.blend_over(bg);
        // Mid-alpha mix sits between the two colours.
        assert!(mixed.r > 50 && mixed.r < 150);
        assert!(mixed.g > 50 && mixed.g < 150);
    }

    // ---------- Array / normalized ----------

    #[test]
    fn test_to_array_and_from_array_round_trip() {
        let c = Color::rgba(1, 2, 3, 4);
        let arr = c.to_array();
        assert_eq!(arr, [1, 2, 3, 4]);
        assert_eq!(Color::from_array(arr), c);
    }

    #[test]
    fn test_to_normalized_divides_by_255() {
        let c = Color::rgba(255, 0, 128, 64);
        let n = c.to_normalized();
        assert!((n[0] - 1.0).abs() < 1e-6);
        assert!((n[1] - 0.0).abs() < 1e-6);
        assert!((n[2] - 128.0 / 255.0).abs() < 1e-6);
        assert!((n[3] - 64.0 / 255.0).abs() < 1e-6);
    }

    // ---------- from_name remaining variants ----------

    #[test]
    fn test_from_name_all_remaining_named_colors() {
        assert_eq!(Color::from_name("yellow"), Color::rgb(240, 230, 140));
        assert_eq!(Color::from_name("orange"), Color::rgb(255, 165, 0));
        assert_eq!(Color::from_name("purple"), Color::rgb(147, 112, 219));
        assert_eq!(Color::from_name("cyan"), Color::rgb(0, 255, 255));
        assert_eq!(Color::from_name("magenta"), Color::rgb(255, 0, 255));
        assert_eq!(Color::from_name("white"), Color::rgb(255, 255, 255));
        assert_eq!(Color::from_name("black"), Color::rgb(0, 0, 0));
        assert_eq!(Color::from_name("pink"), Color::rgb(255, 182, 193));
        assert_eq!(Color::from_name("brown"), Color::rgb(139, 90, 43));
        assert_eq!(Color::from_name("lime"), Color::rgb(50, 205, 50));
        assert_eq!(Color::from_name("teal"), Color::rgb(0, 128, 128));
        assert_eq!(Color::from_name("navy"), Color::rgb(0, 0, 128));
        assert_eq!(Color::from_name("gold"), Color::rgb(255, 215, 0));
        assert_eq!(Color::from_name("silver"), Color::rgb(192, 192, 192));
    }

    #[test]
    fn test_from_name_invalid_hex_falls_back_to_light_gray() {
        // Looks like a hex string but isn't parseable
        assert_eq!(Color::from_name("#GG"), Color::rgb(200, 200, 200));
    }

    // ---------- Lighten / darken edge cases ----------

    #[test]
    fn test_lighten_zero_factor_is_noop() {
        let c = Color::rgb(80, 90, 100);
        assert_eq!(c.lighten(0.0), c);
    }

    #[test]
    fn test_lighten_one_is_white() {
        let c = Color::rgb(80, 90, 100);
        let white = c.lighten(1.0);
        assert_eq!(white.r, 255);
        assert_eq!(white.g, 255);
        assert_eq!(white.b, 255);
    }

    #[test]
    fn test_darken_one_is_black() {
        let c = Color::rgb(80, 90, 100);
        let black = c.darken(1.0);
        assert_eq!(black.r, 0);
        assert_eq!(black.g, 0);
        assert_eq!(black.b, 0);
    }

    #[test]
    fn test_lighten_clamps_factor_above_one() {
        let c = Color::rgb(50, 50, 50);
        // factor > 1 must be clamped to 1 (doesn't overflow / produce invalid u8)
        let _ = c.lighten(5.0);
    }
}
