//! Error types for backtesting

use thiserror::Error;

/// Errors that can occur during backtesting
#[derive(Debug, Error)]
pub enum BacktestError {
    /// No entry signals defined in the strategy
    #[error("Strategy has no entry signals defined")]
    NoEntrySignals,

    /// Insufficient data to run the backtest
    #[error("Insufficient data: required {required} bars, but only {actual} available")]
    InsufficientData {
        /// Minimum number of bars required
        required: usize,
        /// Actual number of bars available
        actual: usize,
    },

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Position sizing error
    #[error("Position sizing error: {0}")]
    PositionSizing(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Risk limit exceeded
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),
}

/// Result type alias for backtest operations
pub type BacktestResult<T> = std::result::Result<T, BacktestError>;
