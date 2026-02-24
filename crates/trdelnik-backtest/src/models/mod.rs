//! Trading models for backtesting

mod position;
mod trade;

pub use position::{Position, PositionSide};
pub use trade::{ExitReason, Trade};
