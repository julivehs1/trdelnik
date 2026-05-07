//! # Trdelnik Indicators
//!
//! Technical analysis indicators for the trdelnik trading chart library.
//! This crate provides pure mathematical calculations with no GUI dependencies.
//!
//! ## Usage
//!
//! All indicators implement the [`Indicator`] trait and process data incrementally:
//!
//! ```rust
//! use trdelnik_indicators::{Indicator, Sma};
//!
//! let mut sma = Sma::new(3);
//! assert_eq!(sma.next(1.0), None);  // Warmup
//! assert_eq!(sma.next(2.0), None);  // Warmup
//! assert_eq!(sma.next(3.0), Some(2.0));  // Ready!
//! ```
//!
//! ## Input Types
//!
//! Indicators work with different input types:
//! - `f64` - Single value (close price)
//! - [`Ohlc`] - High, Low, Close
//! - [`Ohlcv`] - High, Low, Close, Volume
//!
//! ## Available Indicators
//!
//! ### Moving Averages
//! - [`Sma`] - Simple Moving Average
//! - [`Ema`] - Exponential Moving Average
//! - [`Wma`] - Weighted Moving Average
//!
//! ### Momentum
//! - [`Rsi`] - Relative Strength Index
//! - [`Macd`] - Moving Average Convergence Divergence
//! - [`Ppo`] - Percentage Price Oscillator
//! - [`Roc`] - Rate of Change
//! - [`Stochastic`] - Stochastic Oscillator
//! - [`Cci`] - Commodity Channel Index
//!
//! ### Volatility
//! - [`Atr`] - Average True Range
//! - [`Bollinger`] - Bollinger Bands
//! - [`Keltner`] - Keltner Channel
//! - [`StdDev`] - Standard Deviation
//! - [`Chandelier`] - Chandelier Exit
//!
//! ### Volume
//! - [`Obv`] - On-Balance Volume
//! - [`Mfi`] - Money Flow Index
//!
//! ### Trend / Stops
//! - [`FisherTransform`] - Fisher Transform
//! - [`ChandeKrollStop`] - Chande Kroll Stop
//!
//! ### Other
//! - [`EfficiencyRatio`] - Kaufman's Efficiency Ratio

pub mod params;
pub mod plottable;
pub mod ring_buffer;
pub mod indicator;

mod sma;
mod ema;
mod wma;
mod rsi;
mod std_dev;
mod atr;
mod obv;
mod roc;
mod bollinger;
mod stochastic;
mod macd;
mod ppo;
mod cci;
mod keltner;
mod chandelier;
mod mfi;
mod efficiency_ratio;
mod fisher_transform;
mod chande_kroll_stop;

// Re-export ring buffer utilities
pub use ring_buffer::{MinMaxRingBuffer, RingBuffer};

// Re-export parameter types
pub use params::{
    all_indicators, IndicatorMeta, IndicatorParams, ParamDef, ParamType, ParamValue,
};

// Re-export the derive macros
pub use trdelnik_indicator_derive::{indicator, IndicatorValue};

// Re-export core types
pub use indicator::{Indicator, Ohlc, Ohlcv};

// Re-export plottable trait and helpers
pub use plottable::Plottable;

// Re-export all indicators
pub use sma::Sma;
pub use ema::Ema;
pub use wma::Wma;
pub use rsi::Rsi;
pub use std_dev::StdDev;
pub use atr::Atr;
pub use obv::Obv;
pub use roc::Roc;
pub use bollinger::{Bollinger, BollingerValue};
pub use stochastic::{Stochastic, StochasticValue};
pub use macd::{Macd, MacdValue};
pub use ppo::{Ppo, PpoValue};
pub use cci::Cci;
pub use keltner::{Keltner, KeltnerValue};
pub use chandelier::{Chandelier, ChandelierValue};
pub use mfi::Mfi;
pub use efficiency_ratio::EfficiencyRatio;
pub use fisher_transform::{FisherTransform, FisherTransformValue};
pub use chande_kroll_stop::{ChandeKrollStop, ChandeKrollStopValue};
