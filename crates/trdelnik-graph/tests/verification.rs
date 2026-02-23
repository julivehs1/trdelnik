//! Integration tests for graph-based indicators.

use trdelnik_core::{series::generate_sample_data, Timeframe};
use trdelnik_graph::{nodes::*, Executor, Graph};

#[test]
fn test_sma_produces_values() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));
    let sma_node = graph.add_node(sma(close, 20));

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);
    let values = result.get_output_f64(sma_node);

    // Should have None for first 19 values (warmup), then Some values
    assert!(values[18].is_none());
    assert!(values[19].is_some());
    assert!(values[99].is_some());
}

#[test]
fn test_ema_produces_values() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));
    let ema_node = graph.add_node(ema(close, 12));

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);
    let values = result.get_output_f64(ema_node);

    assert!(values[10].is_none());
    assert!(values[11].is_some());
    assert!(values[99].is_some());
}

#[test]
fn test_rsi_in_valid_range() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));
    let rsi_node = graph.add_node(rsi(close, 14));

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);
    let values = result.get_output_f64(rsi_node);

    for val in values.iter().flatten() {
        assert!(*val >= 0.0 && *val <= 100.0, "RSI should be between 0 and 100");
    }
}

#[test]
fn test_atr_positive() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let atr_node = graph.add_node(atr(14));

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);
    let values = result.get_output_f64(atr_node);

    for val in values.iter().flatten() {
        assert!(*val >= 0.0, "ATR should be non-negative");
    }
}

#[test]
fn test_bollinger_bands_relationship() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));
    let (_, middle, upper, lower) = bollinger_with_fields(&mut graph, close, 20, 2.0);

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);

    let middle_vals = result.get_output_f64(middle);
    let upper_vals = result.get_output_f64(upper);
    let lower_vals = result.get_output_f64(lower);

    // Upper >= Middle >= Lower
    for i in 19..100 {
        if let (Some(m), Some(u), Some(l)) = (middle_vals[i], upper_vals[i], lower_vals[i]) {
            assert!(u >= m, "Upper band should be >= middle");
            assert!(m >= l, "Middle should be >= lower band");
        }
    }
}

#[test]
fn test_stochastic_in_valid_range() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let (_, k, d) = stochastic_with_fields(&mut graph, 14, 3);

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);
    let k_vals = result.get_output_f64(k);
    let d_vals = result.get_output_f64(d);

    for val in k_vals.iter().flatten() {
        assert!(*val >= 0.0 && *val <= 100.0, "%K should be between 0 and 100");
    }
    for val in d_vals.iter().flatten() {
        assert!(*val >= 0.0 && *val <= 100.0, "%D should be between 0 and 100");
    }
}

#[test]
fn test_macd_produces_values() {
    let series = generate_sample_data(100, Timeframe::H1);

    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));
    let (_, macd_line, signal, histogram) = macd_with_fields(&mut graph, close, 12, 26, 9);

    let mut executor = Executor::new(graph);
    let result = executor.process_series(&series);

    let macd_vals = result.get_output_f64(macd_line);
    let signal_vals = result.get_output_f64(signal);
    let hist_vals = result.get_output_f64(histogram);

    // After warmup (26 + 9 - 1 = 34), should have values
    assert!(macd_vals[50].is_some());
    assert!(signal_vals[50].is_some());
    assert!(hist_vals[50].is_some());
}

#[test]
fn test_multiple_periods() {
    let series = generate_sample_data(200, Timeframe::H1);

    for period in [5, 10, 20, 50] {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let sma_node = graph.add_node(sma(close, period));

        let mut executor = Executor::new(graph);
        let result = executor.process_series(&series);
        let values = result.get_output_f64(sma_node);

        // First period-1 values should be None
        assert!(values[period - 2].is_none(), "SMA({}) warmup", period);
        assert!(values[period - 1].is_some(), "SMA({}) first value", period);
    }
}

#[test]
fn test_cse_same_indicator() {
    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));

    let sma1 = graph.add_node(sma(close, 20));
    let sma2 = graph.add_node(sma(close, 20));

    // CSE should return same NodeId
    assert_eq!(sma1, sma2);
}

#[test]
fn test_cse_different_params() {
    let mut graph = Graph::new();
    let close = graph.add_node(Box::new(CloseNode::new()));

    let sma1 = graph.add_node(sma(close, 20));
    let sma2 = graph.add_node(sma(close, 50));

    // Different params should return different NodeIds
    assert_ne!(sma1, sma2);
}
