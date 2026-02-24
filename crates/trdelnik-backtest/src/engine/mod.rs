//! Backtesting engine

mod backtester;
mod state;

pub use backtester::Backtester;
pub use state::{BacktestState, EquityPoint};
