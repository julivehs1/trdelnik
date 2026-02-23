//! # Trdelnik Data
//!
//! Data layer with caching for the trdelnik trading chart library.
//! This crate provides the bridge between raw data/indicators and the UI.
//!
//! ## Key Components
//!
//! - [`ChartData`] - The main data structure that the UI receives
//! - [`ChartDataBuilder`] - Fluent API for building chart data with indicators
//! - [`Panel`] - Container for grouping multiple indicators with dual Y-axis support
//! - [`ComputedIndicators`] - Cache for computed indicator values
//!
//! ## Example (New API)
//!
//! ```rust,ignore
//! use trdelnik_core::{generate_sample_data, Timeframe};
//! use trdelnik_data::{ChartDataBuilder, IntoChartData};
//! use trdelnik_indicators::{Sma, BollingerBands, Rsi, Stochastic, Macd};
//!
//! // Create sample data
//! let series = generate_sample_data(100, Timeframe::H1);
//!
//! // Build chart data with indicators - all calculations happen here
//! let chart_data = ChartDataBuilder::new(series)
//!     // Overlays on main chart
//!     .add_overlay(Sma::new(20))
//!     .add_overlay(BollingerBands::default())
//!
//!     // Panel with multiple indicators + dual Y-axis
//!     .configure_panel("oscillators", |p| p.name("Oscillators").height(120.0))
//!     .add_to_panel("oscillators", Rsi::new(14))
//!     .add_to_panel_right("oscillators", Stochastic::default())
//!
//!     // Simple panel (auto-created)
//!     .add_to_panel("macd", Macd::default())
//!
//!     .build();
//!
//! // The UI only needs to render - no calculations!
//! assert_eq!(chart_data.overlay_indicators().len(), 2);
//! assert_eq!(chart_data.panel_containers().len(), 2);
//! ```
//!
//! ## Using the Extension Trait
//!
//! ```rust
//! use trdelnik_core::{generate_sample_data, Timeframe};
//! use trdelnik_data::IntoChartData;
//! use trdelnik_indicators::Sma;
//!
//! let chart_data = generate_sample_data(100, Timeframe::H1)
//!     .build_chart_data()
//!     .add_overlay(Sma::new(20))
//!     .build();
//! ```

pub mod builder;
pub mod chart_data;
pub mod chart_indicator;
pub mod computed;
pub mod panel;
pub mod script_bridge;

// Re-exports
pub use builder::{ChartDataBuilder, IntoChartData};
pub use chart_indicator::ChartIndicator;
pub use chart_data::ChartData;
pub use computed::{CacheError, ComputedIndicators, IndicatorKey};
pub use panel::{Panel, PanelBuilder, PanelConfig};
pub use script_bridge::{
    create_signals_overlay, plots_to_overlays, plots_to_overlays_with_config, signals_to_markers,
    ScriptPlotConfig, SignalStats,
};

// Re-export indicator types from core for convenience
pub use trdelnik_core::{HistogramBar, IndicatorLine, IndicatorOutput, Placement, YAxis};
