//! Error types for strategy optimization

use thiserror::Error;

/// Errors that can occur during strategy optimization
#[derive(Debug, Error)]
pub enum OptimizeError {
    /// Compilation error from trdelnik-script
    #[error("Compilation error: {0}")]
    Compile(#[from] trdelnik_script::TrdelScriptError),

    /// Backtest error
    #[error("Backtest error: {0}")]
    Backtest(#[from] trdelnik_backtest::BacktestError),

    /// Parse error (for script source)
    #[error("Parse error: {0}")]
    Parse(String),

    /// Unknown parameter name
    #[error("Unknown parameter: {0}")]
    UnknownParam(String),

    /// No valid results after optimization
    #[error("No valid results")]
    NoValidResults,

    /// Empty parameter space (no parameters defined)
    #[error("Empty parameter space")]
    EmptyParamSpace,
}

/// Result type alias for optimization operations
pub type OptimizeResult<T> = std::result::Result<T, OptimizeError>;
