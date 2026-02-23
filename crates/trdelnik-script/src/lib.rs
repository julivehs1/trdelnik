//! # TrdelScript
//!
//! A domain-specific language for developing trading strategies, inspired by Pine Script.
//!
//! TrdelScript provides a clean, expressive syntax for defining trading strategies that
//! compile to the high-performance `trdelnik-graph` computation engine.
//!
//! ## Example
//!
//! ```ignore
//! use trdelnik_script::compile;
//!
//! let source = r#"
//!     strategy "SMA Crossover"
//!     param fast_period: int = 12
//!     param slow_period: int = 26
//!
//!     let fast = sma(close, fast_period)
//!     let slow = sma(close, slow_period)
//!
//!     entry long when crossover(fast, slow)
//!     entry short when crossunder(fast, slow)
//!
//!     stop_loss 2%
//!     take_profit 6%
//!
//!     plot fast color=blue
//!     plot slow color=red
//! "#;
//!
//! let strategy = compile(source).expect("compilation failed");
//!
//! // Use the compiled strategy
//! println!("Strategy: {:?}", strategy.name);
//! println!("Has long entry: {}", strategy.entry_long.is_some());
//! ```
//!
//! ## Syntax Overview
//!
//! ### Strategy Declaration
//! ```text
//! strategy "Strategy Name" timeframe = H1
//! ```
//!
//! ### Parameters
//! ```text
//! param name: int = 20
//! param ratio: float = 2.0
//! param enabled: bool = true
//! ```
//!
//! ### Variable Bindings
//! ```text
//! // Simple binding
//! let fast = sma(close, 12)
//!
//! // Destructuring for multi-output indicators
//! let { upper, middle, lower } = bollinger(close, 20, 2.0)
//! let { macd, signal, histogram } = macd(close, 12, 26, 9)
//! ```
//!
//! ### Entry/Exit Signals
//! ```text
//! entry long when crossover(fast, slow)
//! entry short when crossunder(fast, slow)
//! exit all when rsi(close, 14) > 80
//! exit long when close < stop_price
//! ```
//!
//! ### Risk Management
//! ```text
//! stop_loss 2%
//! take_profit 6%
//! ```
//!
//! ### Plotting
//! ```text
//! plot sma(close, 20) color=blue
//! plot rsi(close, 14) panel="RSI"
//! ```
//!
//! ## Available Functions
//!
//! ### Moving Averages
//! - `sma(input, period)` - Simple Moving Average
//! - `ema(input, period)` - Exponential Moving Average
//! - `wma(input, period)` - Weighted Moving Average
//!
//! ### Oscillators
//! - `rsi(input, period)` - Relative Strength Index
//! - `stochastic(k_period, d_period)` - Stochastic Oscillator (returns {k, d})
//! - `cci(period)` - Commodity Channel Index
//!
//! ### Trend Indicators
//! - `macd(input, fast, slow, signal)` - MACD (returns {macd, signal, histogram})
//! - `ppo(input, fast, slow, signal)` - Percentage Price Oscillator
//!
//! ### Volatility
//! - `bollinger(input, period, std_dev)` - Bollinger Bands (returns {upper, middle, lower})
//! - `atr(period)` - Average True Range
//! - `keltner(ema_period, atr_period, atr_mult)` - Keltner Channel
//! - `std_dev(input, period)` - Standard Deviation
//!
//! ### Volume
//! - `obv()` - On-Balance Volume
//! - `mfi(period)` - Money Flow Index
//!
//! ### Signal Functions
//! - `crossover(a, b)` - True when a crosses above b
//! - `crossunder(a, b)` - True when a crosses below b
//! - `cross(a, b)` - True when a crosses b (either direction)
//!
//! ## Data Sources
//! - `open` - Opening price
//! - `high` - High price
//! - `low` - Low price
//! - `close` - Closing price
//! - `volume` - Volume

pub mod ast;
pub mod compiler;
pub mod error;
pub mod integration;
pub mod lexer;
pub mod parser;
pub mod semantic;

// Re-export main types
pub use ast::Script;
pub use compiler::{CompiledStrategy, Compiler, CompileError, PlotConfig};
pub use error::TrdelScriptError;
pub use lexer::{lex, LexError, Token};
pub use parser::parse;
pub use semantic::{SemanticAnalyzer, SemanticError};

// Re-export graph types for convenience
pub use trdelnik_graph::{Executor, Graph, NodeId};

use std::collections::HashMap;
use std::path::Path;

/// Parse and compile a TrdelScript source string.
///
/// This is the main entry point for compiling TrdelScript code.
///
/// # Example
///
/// ```ignore
/// use trdelnik_script::compile;
///
/// let source = r#"
///     let fast = sma(close, 12)
///     let slow = sma(close, 26)
///     entry long when crossover(fast, slow)
/// "#;
///
/// let strategy = compile(source)?;
/// ```
pub fn compile(source: &str) -> Result<CompiledStrategy, TrdelScriptError> {
    compile_with_params(source, HashMap::new())
}

/// Parse and compile a TrdelScript source string with parameter overrides.
///
/// Allows specifying custom values for parameters defined in the script.
///
/// # Example
///
/// ```ignore
/// use trdelnik_script::compile_with_params;
/// use std::collections::HashMap;
///
/// let source = r#"
///     param period: int = 20
///     let ma = sma(close, period)
/// "#;
///
/// let mut params = HashMap::new();
/// params.insert("period".to_string(), 50.0);
///
/// let strategy = compile_with_params(source, params)?;
/// ```
pub fn compile_with_params(
    source: &str,
    params: HashMap<String, f64>,
) -> Result<CompiledStrategy, TrdelScriptError> {
    // 1. Lex
    let tokens = lex(source)?;

    // 2. Parse
    let script = parse(&tokens).map_err(TrdelScriptError::Parse)?;

    // 3. Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(&script)
        .map_err(TrdelScriptError::Semantic)?;

    // 4. Compile to graph
    let strategy = Compiler::compile_with_params(&script, params)?;

    Ok(strategy)
}

/// Load and compile a TrdelScript file.
///
/// # Example
///
/// ```ignore
/// use trdelnik_script::compile_file;
/// use std::path::Path;
///
/// let strategy = compile_file(Path::new("strategy.trdl"))?;
/// ```
pub fn compile_file(path: &Path) -> Result<CompiledStrategy, TrdelScriptError> {
    let source = std::fs::read_to_string(path)?;
    compile(&source)
}

/// Load and compile a TrdelScript file with parameter overrides.
pub fn compile_file_with_params(
    path: &Path,
    params: HashMap<String, f64>,
) -> Result<CompiledStrategy, TrdelScriptError> {
    let source = std::fs::read_to_string(path)?;
    compile_with_params(&source, params)
}

/// Parse a TrdelScript source string into an AST without compiling.
///
/// Useful for syntax validation or AST inspection.
pub fn parse_source(source: &str) -> Result<Script, TrdelScriptError> {
    let tokens = lex(source)?;
    parse(&tokens).map_err(TrdelScriptError::Parse)
}

/// Validate a TrdelScript source string without compiling.
///
/// Performs lexing, parsing, and semantic analysis.
pub fn validate(source: &str) -> Result<(), TrdelScriptError> {
    let tokens = lex(source)?;
    let script = parse(&tokens).map_err(TrdelScriptError::Parse)?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(&script)
        .map_err(TrdelScriptError::Semantic)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use trdelnik_core::{Candle, CandleSeries, Timestamp};

    fn create_test_series() -> CandleSeries<Timestamp> {
        let mut series = CandleSeries::new();
        let prices = [
            100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5,
            107.0, 108.0, 107.5, 109.0, 110.0, 109.5, 111.0, 112.0, 111.5, 113.0,
            114.0, 113.5, 115.0, 116.0, 115.5, 117.0, 118.0, 117.5, 119.0, 120.0,
        ];

        for (i, &price) in prices.iter().enumerate() {
            series.push(Candle::new(
                Timestamp(i as i64 * 3600000),
                price - 0.5,
                price + 1.0,
                price - 1.0,
                price,
                1000.0 + i as f64 * 100.0,
            ));
        }
        series
    }

    #[test]
    fn test_full_compilation() {
        let source = r#"
            strategy "Test Strategy"
            let fast = sma(close, 5)
            let slow = sma(close, 10)
            entry long when crossover(fast, slow)
        "#;

        let strategy = compile(source).unwrap();
        assert!(strategy.entry_long.is_some());
        assert!(strategy.entry_short.is_none());
        assert_eq!(strategy.name, Some("Test Strategy".to_string()));
    }

    #[test]
    fn test_strategy_execution() {
        let source = r#"
            let fast = sma(close, 5)
            let slow = sma(close, 10)
            entry long when crossover(fast, slow)
        "#;

        let strategy = compile(source).unwrap();
        let mut executor = Executor::new(strategy.graph);

        let series = create_test_series();
        let result = executor.process_series(&series);

        // Verify signals were computed
        if let Some(entry_long) = strategy.entry_long {
            let signals = result.get_output(entry_long);
            assert_eq!(signals.len(), 30);
        }
    }

    #[test]
    fn test_bollinger_destructuring() {
        let source = r#"
            let { upper, middle, lower } = bollinger(close, 5, 2.0)
            entry long when close > upper
        "#;

        let strategy = compile(source).unwrap();
        let mut executor = Executor::new(strategy.graph);

        let series = create_test_series();
        let result = executor.process_series(&series);

        // Should have computed without error
        assert!(strategy.entry_long.is_some());
        let signals = result.get_output(strategy.entry_long.unwrap());
        assert!(!signals.is_empty());
    }

    #[test]
    fn test_macd_strategy() {
        let source = r#"
            let { macd, signal, histogram } = macd(close, 3, 5, 3)
            entry long when crossover(macd, signal)
            entry short when crossunder(macd, signal)
        "#;

        let strategy = compile(source).unwrap();
        assert!(strategy.entry_long.is_some());
        assert!(strategy.entry_short.is_some());

        let mut executor = Executor::new(strategy.graph);
        let series = create_test_series();
        let result = executor.process_series(&series);

        // Check we got results
        let long_signals = result.get_output(strategy.entry_long.unwrap());
        let short_signals = result.get_output(strategy.entry_short.unwrap());
        assert_eq!(long_signals.len(), 30);
        assert_eq!(short_signals.len(), 30);
    }

    #[test]
    fn test_complex_conditions() {
        let source = r#"
            let fast = sma(close, 5)
            let slow = sma(close, 10)
            entry long when crossover(fast, slow) and rsi(close, 5) < 70
        "#;

        let strategy = compile(source).unwrap();
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_stochastic_strategy() {
        let source = r#"
            let { k, d } = stochastic(5, 3)
            entry long when crossover(k, d) and k < 30
            entry short when crossunder(k, d) and k > 70
        "#;

        let strategy = compile(source).unwrap();
        assert!(strategy.entry_long.is_some());
        assert!(strategy.entry_short.is_some());
    }

    #[test]
    fn test_with_params() {
        let source = r#"
            param fast_period: int = 5
            param slow_period: int = 10
            let fast = sma(close, fast_period)
            let slow = sma(close, slow_period)
            entry long when crossover(fast, slow)
        "#;

        // Compile with default params
        let strategy1 = compile(source).unwrap();
        assert!(strategy1.entry_long.is_some());

        // Compile with custom params
        let mut params = HashMap::new();
        params.insert("fast_period".to_string(), 3.0);
        params.insert("slow_period".to_string(), 8.0);

        let strategy2 = compile_with_params(source, params).unwrap();
        assert!(strategy2.entry_long.is_some());
    }

    #[test]
    fn test_validation() {
        let valid_source = "let x = sma(close, 20)";
        assert!(validate(valid_source).is_ok());

        let invalid_source = "let x = unknown_function(close)";
        assert!(validate(invalid_source).is_err());
    }

    #[test]
    fn test_parse_only() {
        let source = r#"
            strategy "Test"
            let x = sma(close, 20)
        "#;

        let script = parse_source(source).unwrap();
        assert!(script.strategy.is_some());
        assert_eq!(script.statements.len(), 1);
    }

    #[test]
    fn test_error_reporting() {
        let source = "entry long when undefined_var";
        let result = compile(source);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let report = err.to_report_string(source);
        assert!(report.contains("Undefined variable"));
    }

    #[test]
    fn test_exit_signals() {
        let source = r#"
            exit all when rsi(close, 5) > 80
            exit long when close < sma(close, 5)
        "#;

        let strategy = compile(source).unwrap();
        assert_eq!(strategy.exit_signals.len(), 2);
    }

    #[test]
    fn test_plots() {
        use trdelnik_core::Color;

        let source = r#"
            let fast = sma(close, 5)
            let slow = sma(close, 10)
            plot fast color=blue
            plot slow color=red
        "#;

        let strategy = compile(source).unwrap();
        assert_eq!(strategy.plots.len(), 2);
        assert_eq!(strategy.plots[0].color, Color::from_name("blue"));
        assert_eq!(strategy.plots[1].color, Color::from_name("red"));
    }

    #[test]
    fn test_risk_management() {
        let source = r#"
            stop_loss 2%
            take_profit 6%
        "#;

        let strategy = compile(source).unwrap();
        assert_eq!(strategy.stop_loss, Some(2.0));
        assert_eq!(strategy.take_profit, Some(6.0));
    }

    #[test]
    fn test_arithmetic_expressions() {
        let source = r#"
            let mid = (high + low) / 2
            let range = high - low
            entry long when mid > sma(close, 5)
        "#;

        let strategy = compile(source).unwrap();
        assert!(strategy.entry_long.is_some());
    }

    #[test]
    fn test_atr_and_ohlc_indicators() {
        let source = r#"
            let atr_val = atr(5)
            entry long when close > sma(close, 5) + atr_val
        "#;

        let strategy = compile(source).unwrap();
        assert!(strategy.entry_long.is_some());
    }
}
