//! # Trdelnik Theme
//!
//! GUI-agnostic theme system for the trdelnik trading chart library.
//! This crate provides platform-independent color definitions and theme presets.
//!
//! ## Features
//!
//! - Platform-agnostic [`Color`] type with hex string support
//! - [`ChartTheme`] with comprehensive color settings
//! - Pre-built theme presets: dark, light, blue, midnight
//! - JSON serialization for theme persistence
//! - Custom indicator color support via HashMap
//!
//! ## Example
//!
//! ```rust
//! use trdelnik_theme::{ChartTheme, Color};
//!
//! // Use a preset theme
//! let theme = ChartTheme::dark();
//!
//! // Customize colors
//! let mut custom = ChartTheme::dark();
//! custom.bullish = Color::rgb(0, 255, 0);
//! custom.set_indicator_color("my_indicator", Color::rgb(255, 0, 255));
//!
//! // Save and load themes
//! # #[cfg(feature = "skip_io_test")]
//! custom.save_to_file(std::path::Path::new("my_theme.json")).unwrap();
//! # #[cfg(feature = "skip_io_test")]
//! let loaded = ChartTheme::load_from_file(std::path::Path::new("my_theme.json")).unwrap();
//! ```

pub mod color;
pub mod preset;
pub mod theme;

// Re-exports
pub use color::Color;
pub use theme::{ChartTheme, ThemeError};
