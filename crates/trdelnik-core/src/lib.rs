//! # Trdelnik Core
//!
//! Core types for the trdelnik trading chart library.
//! This crate contains no GUI dependencies and can be used in any context.
//!
//! ## Main Types
//!
//! - [`AxisCoordinate`] - Trait for X-axis coordinate types
//! - [`Timestamp`], [`Slot`], [`BlockNumber`], [`Index`] - Built-in coordinate types
//! - [`Candle`] - Generic OHLCV candle with configurable X coordinate
//! - [`CandleSeries`] - Collection of candles
//! - [`Signal`], [`SignalSeries`] - Trading signals
//! - [`Timeframe`] - Candle timeframe definitions
//! - [`PlotData`] - Pure-data output of `Plottable` indicators

pub mod axis;
pub mod candle;
pub mod color;
pub mod indicator_output;
pub mod plot_data;
pub mod series;
pub mod signal;
pub mod timeframe;
pub mod value;

// Re-exports
pub use axis::{AxisCoordinate, BlockNumber, Index, Slot, Timestamp};
pub use candle::{Candle, TimestampCandle};
pub use color::Color;
pub use indicator_output::{
    HistogramBar, IndicatorLine, IndicatorMarker, MarkerShape, YAxis,
};
pub use plot_data::{HLine, HLineStyle, PlotData};
pub use series::{generate_sample_data, CandleSeries};
pub use signal::{Signal, SignalDirection, SignalSeries, SignalStrength};
pub use timeframe::Timeframe;
pub use value::{OutputToValue, Value};
