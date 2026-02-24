//! Configuration types for backtesting

mod backtest_config;
mod risk_config;

pub use backtest_config::{BacktestConfig, BacktestConfigBuilder, BacktestConfigSnapshot};
pub use risk_config::{RiskConfig, TrailingStopConfig};
