//! # trdelnik-graph
//!
//! Stateful, O(1)-per-bar indicator computation using a DAG (Directed Acyclic Graph)
//! execution model.
//!
//! ## Overview
//!
//! This crate provides a graph-based execution engine for computing technical indicators
//! efficiently. Each indicator is represented as a node in a computation graph, and the
//! engine processes bars one at a time, maintaining internal state for O(1) per-bar
//! computation.
//!
//! ## Key Features
//!
//! - **O(1) Per-Bar Computation**: Indicators like SMA, EMA, RSI maintain running state
//!   instead of recalculating from scratch each bar.
//! - **Common Subexpression Elimination (CSE)**: Duplicate nodes are automatically detected
//!   and eliminated, so `sma(close, 20)` is computed only once even if referenced multiple times.
//! - **Topological Ordering**: The graph automatically determines the correct execution order
//!   based on dependencies.
//! - **Type-Safe Values**: The `Value` enum supports both numeric and boolean values with
//!   proper handling of warmup periods.
//!
//! ## Example
//!
//! ```rust
//! use trdelnik_graph::{Graph, Executor, nodes::*};
//! use trdelnik_core::{CandleSeries, Candle, Timestamp, Timeframe, series::generate_sample_data};
//!
//! // Create a graph with SMA crossover
//! let mut graph = Graph::new();
//!
//! // Data source nodes
//! let close = graph.add_node(Box::new(CloseNode::new()));
//!
//! // Indicator nodes (using factory functions)
//! let sma20 = graph.add_node(sma(close, 20));
//! let sma50 = graph.add_node(sma(close, 50));
//!
//! // Signal node
//! let crossover = graph.add_node(Box::new(CrossOverNode::new(sma20, sma50)));
//!
//! // Create executor and process data
//! let mut executor = Executor::new(graph);
//! let series = generate_sample_data(200, Timeframe::H1);
//! let result = executor.process_series(&series);
//!
//! // Access results
//! let sma20_values = result.get_output_f64(sma20);
//! let signals = result.get_output(crossover);
//! ```
//!
//! ## Indicator Nodes
//!
//! ### Data Sources
//! - `CloseNode`, `OpenNode`, `HighNode`, `LowNode`, `VolumeNode` - OHLCV data
//! - `ConstNode` - Constant numeric value
//!
//! ### Factory Functions (Recommended)
//! - `sma(input, period)` - Simple Moving Average
//! - `ema(input, period)` - Exponential Moving Average
//! - `wma(input, period)` - Weighted Moving Average
//! - `rsi(input, period)` - Relative Strength Index
//! - `atr(period)` - Average True Range
//! - `bollinger(input, period, std_dev)` - Bollinger Bands
//! - `macd(input, fast, slow, signal)` - MACD
//! - `stochastic(k_period, d_period)` - Stochastic Oscillator
//! - `obv()` - On-Balance Volume
//! - `mfi(period)` - Money Flow Index
//!
//! ### Arithmetic
//! - `AddNode`, `SubNode`, `MulNode`, `DivNode` - Basic operations
//! - `NegNode`, `AbsNode` - Unary operations
//! - `MaxNode`, `MinNode` - Min/max of two values
//!
//! ### Comparison & Logic
//! - `GtNode`, `LtNode`, `GteNode`, `LteNode`, `EqNode` - Comparisons
//! - `CrossNode`, `CrossOverNode`, `CrossUnderNode` - Cross signals
//! - `AndNode`, `OrNode`, `NotNode` - Boolean logic

pub mod context;
pub mod executor;
pub mod graph;
pub mod node;
pub mod nodes;
pub mod output;
pub mod registry;
pub mod value;

// Re-export main types for convenience
pub use context::{ExecutionContext, ExecutionResult, OutputStore};
pub use executor::Executor;
pub use graph::Graph;
pub use node::{BoxedNode, Node, NodeId};
pub use output::{ExecutionResultExt, OutputBuilder};
pub use registry::IndicatorRegistry;
pub use value::Value;

// Re-export ring buffer from trdelnik-indicators for backwards compatibility
pub use trdelnik_indicators::{MinMaxRingBuffer, RingBuffer};

#[cfg(test)]
mod tests {
    use super::*;
    use nodes::*;
    use trdelnik_core::{Candle, CandleSeries, Timestamp};

    fn create_test_series() -> CandleSeries<Timestamp> {
        let mut series = CandleSeries::new();
        let prices = [
            100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5,
            107.0, 108.0, 107.5, 109.0, 110.0, 109.5, 111.0, 112.0, 111.5, 113.0,
        ];

        for (i, &price) in prices.iter().enumerate() {
            series.push(Candle::new(
                Timestamp(i as i64 * 3600000), // 1 hour intervals
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
    fn test_simple_graph() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let sma5 = graph.add_node(sma(close, 5));

        let mut executor = Executor::new(graph);
        let series = create_test_series();
        let result = executor.process_series(&series);

        // Check that we have values after warmup
        let sma_values = result.get_output_f64(sma5);
        assert_eq!(sma_values.len(), 20);

        // First 4 values should be None (warmup)
        assert!(sma_values[0].is_none());
        assert!(sma_values[3].is_none());

        // 5th value should be Some
        assert!(sma_values[4].is_some());
    }

    #[test]
    fn test_cse() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));

        // Add the same SMA twice
        let sma1 = graph.add_node(sma(close, 5));
        let sma2 = graph.add_node(sma(close, 5));

        // CSE should detect duplicate
        assert_eq!(sma1, sma2);
        assert_eq!(graph.len(), 2); // Only close and one SMA
    }

    #[test]
    fn test_complex_graph() {
        let mut graph = Graph::new();

        // Build a complex indicator graph
        let close = graph.add_node(Box::new(CloseNode::new()));
        let sma_fast = graph.add_node(sma(close, 5));
        let sma_slow = graph.add_node(sma(close, 10));
        let crossover = graph.add_node(Box::new(CrossOverNode::new(sma_fast, sma_slow)));

        let mut executor = Executor::new(graph);
        let series = create_test_series();
        let result = executor.process_series(&series);

        // Check crossover signals
        let signals = result.get_output(crossover);
        assert_eq!(signals.len(), 20);

        // First bars should be None (warmup)
        assert!(signals[0].is_none());
    }

    #[test]
    fn test_rsi() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let rsi14 = graph.add_node(rsi(close, 14));

        let mut executor = Executor::new(graph);
        let series = create_test_series();
        let result = executor.process_series(&series);

        let rsi_values = result.get_output_f64(rsi14);

        // RSI values should be between 0 and 100
        for val in rsi_values.iter().filter_map(|v| *v) {
            assert!(val >= 0.0 && val <= 100.0);
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let sma5 = graph.add_node(sma(close, 5));
        let diff = graph.add_node(Box::new(SubNode::new(close, sma5)));

        let mut executor = Executor::new(graph);
        let series = create_test_series();
        let result = executor.process_series(&series);

        let close_values = result.get_output_f64(close);
        let sma_values = result.get_output_f64(sma5);
        let diff_values = result.get_output_f64(diff);

        // After warmup, diff should equal close - sma
        for i in 4..20 {
            if let (Some(c), Some(s), Some(d)) = (close_values[i], sma_values[i], diff_values[i]) {
                assert!((d - (c - s)).abs() < 1e-10);
            }
        }
    }
}
