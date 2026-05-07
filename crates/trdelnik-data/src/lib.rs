//! # Trdelnik Data
//!
//! Data layer for the trdelnik trading chart library.
//! This crate provides the bridge between raw data/indicators and the UI.
//!
//! ## Key Components
//!
//! - [`ChartData`] - The main data structure that the UI receives
//! - [`ChartBuilder`] - Matplotlib-like API for building chart data with indicators
//! - [`Panel`] - Container for grouping plots with hlines, y_range, dual Y-axis
//! - [`PanelHandle`] - Handle for configuring panels in builder closures
//!
//! ## Example
//!
//! ```rust,ignore
//! use trdelnik_data::ChartBuilder;
//! use trdelnik_indicators::{Sma, Rsi, Macd};
//!
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
//! ```

pub mod builder;
pub mod chart_data;
pub mod computed;
pub mod panel;
pub mod panel_handle;

// Re-exports
pub use builder::{ChartBuilder, IntoChartData};
pub use chart_data::ChartData;
pub use computed::{CacheError, ComputedIndicators, IndicatorKey};
pub use panel::{Panel, PanelConfig};
pub use panel_handle::PanelHandle;

// Re-export types from core for convenience
pub use trdelnik_core::{HistogramBar, IndicatorLine, YAxis};
pub use trdelnik_render::{HLine, HLineStyle, Plot, StandardPlot};
