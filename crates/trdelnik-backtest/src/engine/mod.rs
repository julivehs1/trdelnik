//! Backtesting engine

mod backtester;
mod broker;
mod state;

pub use backtester::Backtester;
pub use broker::BacktestBroker;
pub use state::{BacktestState, EquityPoint};
