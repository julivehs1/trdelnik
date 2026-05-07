//! # Trdelnik UI
//!
//! egui-based rendering for the trdelnik trading chart library.
//!
//! This crate provides the visualization layer that renders pre-computed
//! chart data. **No indicator calculations happen here** - all computations
//! are done in the `trdelnik-data` crate before rendering.
//!
//! ## Key Components
//!
//! - [`TradingChart`] - The main chart widget
//! - [`ChartConfig`] - Configuration options
//! - [`ChartResponse`] - Response from chart interaction
//! - [`PanelHeightState`] - State management for resizable panels
//!
//! ## Example
//!
//! ```rust,ignore
//! use trdelnik_core::{generate_sample_data, Timeframe};
//! use trdelnik_data::ChartBuilder;
//! use trdelnik_theme::ChartTheme;
//! use trdelnik_ui::{TradingChart, ChartConfig};
//! use trdelnik_indicators::{Sma, Rsi, Macd};
//!
//! let series = generate_sample_data(100, Timeframe::H1);
//! let chart_data = ChartBuilder::new(series)
//!     .overlay(Sma::new(20))
//!     .panel("rsi", |p| {
//!         p.plot(Rsi::new(14));
//!         p.hline(30.0);
//!         p.hline(70.0);
//!         p.ylim(0.0, 100.0);
//!     })
//!     .panel("macd", |p| {
//!         p.plot(Macd::default());
//!         p.hline(0.0);
//!     })
//!     .build();
//!
//! let theme = ChartTheme::dark();
//! // TradingChart::new(&chart_data, &theme).show(ui);
//! ```

pub mod axis_interaction;
pub mod axis_scale;
pub mod chart;
pub mod config;
pub mod drawing_layer;
pub mod egui_theme;
pub mod panel_state;
pub mod panels;
pub mod resize_handle;

// Re-exports
pub use axis_scale::{SharedXState, PanelYState, CursorState, ViewState};
pub use chart::{ChartResponse, PanelRenderInfo, TradingChart};
pub use config::ChartConfig;
pub use drawing_layer::{
    handle_drawing_input, render_drawings, render_drawings_for_panel, screen_to_data,
    DrawingRenderTheme, ViewBounds,
};
pub use egui_theme::{from_egui_color, to_egui_color, FromEguiColor, ToEguiColor};
pub use panel_state::PanelHeightState;
pub use resize_handle::{render_resize_handle, simple_resize_handle};
