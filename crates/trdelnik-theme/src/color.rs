//! Platform-agnostic color type
//!
//! Re-exported from trdelnik-core for backwards compatibility.

pub use trdelnik_core::Color;

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
}
