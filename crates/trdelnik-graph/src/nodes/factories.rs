//! Factory functions for creating indicator nodes.
//!
//! These functions provide a convenient API for creating indicator nodes.
//! Since indicators implement `Hash` and `Eq` on their parameters, CSE
//! (Common Subexpression Elimination) is automatic.
//!
//! # Example
//! ```ignore
//! use trdelnik_graph::{Graph, nodes::*};
//!
//! let mut graph = Graph::new();
//! let close = graph.add_node(Box::new(CloseNode::new()));
//!
//! // Simple single-output indicators
//! let sma20 = graph.add_node(sma(close, 20));
//! let ema12 = graph.add_node(ema(close, 12));
//! let rsi14 = graph.add_node(rsi(close, 14));
//!
//! // Multi-output indicator with field extraction
//! let (boll, middle, upper, lower) = bollinger_with_fields(&mut graph, close, 20, 2.0);
//!
//! // Or manually extract fields
//! let macd_node = graph.add_node(macd(close, 12, 26, 9));
//! let macd_line = graph.add_node(field(macd_node, "macd"));
//! let signal_line = graph.add_node(field(macd_node, "signal"));
//! ```

use crate::graph::Graph;
use crate::node::{BoxedNode, NodeId};
use crate::nodes::field::FieldNode;
use crate::nodes::generic::IndicatorNode;
use crate::nodes::pattern::{BarsSinceNode, CountWhenNode};
use trdelnik_indicators::{
    Atr, Bollinger, Cci, Chandelier, EfficiencyRatio, Ema, Keltner, Macd, Mfi, Obv, Ppo, Roc, Rsi,
    Sma, StdDev, Stochastic, Wma,
};

// ============================================================================
// f64-Input Indicator Factories (single period parameter)
// ============================================================================

/// Create a Simple Moving Average node
pub fn sma(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(input, Sma::new(period)))
}

/// Create an Exponential Moving Average node
pub fn ema(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(input, Ema::new(period)))
}

/// Create a Weighted Moving Average node
pub fn wma(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(input, Wma::new(period)))
}

/// Create a Relative Strength Index node
pub fn rsi(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(input, Rsi::new(period)))
}

/// Create a Standard Deviation node
pub fn std_dev(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(input, StdDev::new(period)))
}

/// Create a Rate of Change node
pub fn roc(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(input, Roc::new(period)))
}

/// Create an Efficiency Ratio node (Kaufman's Adaptive Moving Average component)
pub fn efficiency_ratio(input: NodeId, period: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(
        input,
        EfficiencyRatio::new(period),
    ))
}

// ============================================================================
// f64-Input Indicator Factories (multiple parameters)
// ============================================================================

/// Create a Bollinger Bands node (outputs struct with upper, middle, lower)
pub fn bollinger(input: NodeId, period: usize, std_dev_mult: f64) -> BoxedNode {
    Box::new(IndicatorNode::with_input(
        input,
        Bollinger::new(period, std_dev_mult),
    ))
}

/// Create a MACD node (outputs struct with macd, signal, histogram)
pub fn macd(input: NodeId, fast: usize, slow: usize, signal: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(
        input,
        Macd::new(fast, slow, signal),
    ))
}

/// Create a PPO node (outputs struct with ppo, signal, histogram)
pub fn ppo(input: NodeId, fast: usize, slow: usize, signal: usize) -> BoxedNode {
    Box::new(IndicatorNode::with_input(
        input,
        Ppo::new(fast, slow, signal),
    ))
}

// ============================================================================
// OHLC-Input Indicator Factories (uses context for H/L/C)
// ============================================================================

/// Create an Average True Range node
pub fn atr(period: usize) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Atr::new(period)))
}

/// Create a Stochastic Oscillator node (outputs struct with k, d)
pub fn stochastic(k_period: usize, d_period: usize) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Stochastic::new(
        k_period, d_period,
    )))
}

/// Create a Keltner Channel node (outputs struct with upper, middle, lower)
pub fn keltner(ema_period: usize, atr_period: usize, atr_mult: f64) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Keltner::new(
        ema_period, atr_period, atr_mult,
    )))
}

/// Create a Commodity Channel Index node with default constant (0.015)
pub fn cci(period: usize) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Cci::new(period, 0.015)))
}

/// Create a Commodity Channel Index node with custom constant
pub fn cci_with_constant(period: usize, constant: f64) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Cci::new(
        period, constant,
    )))
}

/// Create a Chandelier Exit node (outputs struct with long_exit, short_exit)
pub fn chandelier(period: usize, atr_mult: f64) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Chandelier::new(
        period, atr_mult,
    )))
}

// ============================================================================
// OHLCV-Input Indicator Factories (uses context for H/L/C/V)
// ============================================================================

/// Create an On-Balance Volume node
pub fn obv() -> BoxedNode {
    Box::new(IndicatorNode::from_context(Obv::new()))
}

/// Create a Money Flow Index node
pub fn mfi(period: usize) -> BoxedNode {
    Box::new(IndicatorNode::from_context(Mfi::new(period)))
}

// ============================================================================
// Pattern-Counting Factories
// ============================================================================

/// Create a `bars_since` node — bars elapsed since the bool input was last true.
pub fn bars_since(input: NodeId) -> BoxedNode {
    Box::new(BarsSinceNode::new(input))
}

/// Create a `count_when` node — rolling count of how often the bool input
/// was true over the last `period` bars.
pub fn count_when(input: NodeId, period: usize) -> BoxedNode {
    Box::new(CountWhenNode::new(input, period))
}

// ============================================================================
// Field Extraction
// ============================================================================

/// Create a field extraction node for multi-output indicators
pub fn field(input: NodeId, field_name: &str) -> BoxedNode {
    Box::new(FieldNode::new(input, field_name))
}

// ============================================================================
// Convenience Functions (add nodes to graph and return NodeIds)
// ============================================================================

/// Create Bollinger Bands with field extraction nodes.
///
/// Returns (boll_node, middle_node, upper_node, lower_node) NodeIds.
pub fn bollinger_with_fields(
    graph: &mut Graph,
    input: NodeId,
    period: usize,
    std_dev_mult: f64,
) -> (NodeId, NodeId, NodeId, NodeId) {
    let boll = graph.add_node(bollinger(input, period, std_dev_mult));
    let middle = graph.add_node(field(boll, "middle"));
    let upper = graph.add_node(field(boll, "upper"));
    let lower = graph.add_node(field(boll, "lower"));
    (boll, middle, upper, lower)
}

/// Create MACD with field extraction nodes.
///
/// Returns (macd_node, line_node, signal_node, histogram_node) NodeIds.
pub fn macd_with_fields(
    graph: &mut Graph,
    input: NodeId,
    fast: usize,
    slow: usize,
    signal: usize,
) -> (NodeId, NodeId, NodeId, NodeId) {
    let macd_node = graph.add_node(macd(input, fast, slow, signal));
    let line = graph.add_node(field(macd_node, "macd"));
    let signal_line = graph.add_node(field(macd_node, "signal"));
    let histogram = graph.add_node(field(macd_node, "histogram"));
    (macd_node, line, signal_line, histogram)
}

/// Create Stochastic with field extraction nodes.
///
/// Returns (stoch_node, k_node, d_node) NodeIds.
pub fn stochastic_with_fields(
    graph: &mut Graph,
    k_period: usize,
    d_period: usize,
) -> (NodeId, NodeId, NodeId) {
    let stoch = graph.add_node(stochastic(k_period, d_period));
    let k = graph.add_node(field(stoch, "k"));
    let d = graph.add_node(field(stoch, "d"));
    (stoch, k, d)
}

/// Create Keltner Channel with field extraction nodes.
///
/// Returns (keltner_node, middle_node, upper_node, lower_node) NodeIds.
pub fn keltner_with_fields(
    graph: &mut Graph,
    ema_period: usize,
    atr_period: usize,
    atr_mult: f64,
) -> (NodeId, NodeId, NodeId, NodeId) {
    let kelt = graph.add_node(keltner(ema_period, atr_period, atr_mult));
    let middle = graph.add_node(field(kelt, "middle"));
    let upper = graph.add_node(field(kelt, "upper"));
    let lower = graph.add_node(field(kelt, "lower"));
    (kelt, middle, upper, lower)
}

/// Create PPO with field extraction nodes.
///
/// Returns (ppo_node, line_node, signal_node, histogram_node) NodeIds.
pub fn ppo_with_fields(
    graph: &mut Graph,
    input: NodeId,
    fast: usize,
    slow: usize,
    signal: usize,
) -> (NodeId, NodeId, NodeId, NodeId) {
    let ppo_node = graph.add_node(ppo(input, fast, slow, signal));
    let line = graph.add_node(field(ppo_node, "ppo"));
    let signal_line = graph.add_node(field(ppo_node, "signal"));
    let histogram = graph.add_node(field(ppo_node, "histogram"));
    (ppo_node, line, signal_line, histogram)
}

/// Create Chandelier Exit with field extraction nodes.
///
/// Returns (chandelier_node, long_exit_node, short_exit_node) NodeIds.
pub fn chandelier_with_fields(
    graph: &mut Graph,
    period: usize,
    atr_mult: f64,
) -> (NodeId, NodeId, NodeId) {
    let ch = graph.add_node(chandelier(period, atr_mult));
    let long_exit = graph.add_node(field(ch, "long_exit"));
    let short_exit = graph.add_node(field(ch, "short_exit"));
    (ch, long_exit, short_exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::data::CloseNode;
    use crate::Executor;
    use trdelnik_core::{Candle, CandleSeries, Timestamp};

    fn create_test_series() -> CandleSeries<Timestamp> {
        let mut series = CandleSeries::new();
        let prices = [
            100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5, 107.0, 108.0,
            107.5, 109.0, 110.0, 109.5, 111.0, 112.0, 111.5, 113.0,
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
    fn test_sma_factory() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let sma5 = graph.add_node(sma(close, 5));

        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());

        let values = result.get_output_f64(sma5);
        assert!(values[4].is_some());
    }

    #[test]
    fn test_bollinger_with_fields_factory() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let (_, middle, upper, lower) = bollinger_with_fields(&mut graph, close, 5, 2.0);

        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());

        let middle_values = result.get_output_f64(middle);
        let upper_values = result.get_output_f64(upper);
        let lower_values = result.get_output_f64(lower);

        // Check that upper > middle > lower after warmup
        for i in 4..middle_values.len() {
            if let (Some(m), Some(u), Some(l)) =
                (middle_values[i], upper_values[i], lower_values[i])
            {
                assert!(u >= m, "upper should be >= middle");
                assert!(m >= l, "middle should be >= lower");
            }
        }
    }

    #[test]
    fn test_cse_with_factories() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));

        // Adding the same indicator twice should return the same NodeId (CSE)
        let sma1 = graph.add_node(sma(close, 20));
        let sma2 = graph.add_node(sma(close, 20));

        assert_eq!(sma1, sma2);
    }

    fn run_with_close<F>(f: F) -> Vec<Option<f64>>
    where
        F: FnOnce(&mut Graph, NodeId) -> NodeId,
    {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let target = f(&mut graph, close);
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        result.get_output_f64(target)
    }

    fn run_no_input<F>(f: F) -> Vec<Option<f64>>
    where
        F: FnOnce(&mut Graph) -> NodeId,
    {
        let mut graph = Graph::new();
        let target = f(&mut graph);
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        result.get_output_f64(target)
    }

    // ---------- f64-input single-period factories ----------

    #[test]
    fn test_ema_factory() {
        let v = run_with_close(|g, c| g.add_node(ema(c, 5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_wma_factory() {
        let v = run_with_close(|g, c| g.add_node(wma(c, 5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_rsi_factory() {
        let v = run_with_close(|g, c| g.add_node(rsi(c, 5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_std_dev_factory() {
        let v = run_with_close(|g, c| g.add_node(std_dev(c, 5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_roc_factory() {
        let v = run_with_close(|g, c| g.add_node(roc(c, 3)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_efficiency_ratio_factory() {
        let v = run_with_close(|g, c| g.add_node(efficiency_ratio(c, 5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    // ---------- f64-input multi-param factories ----------

    #[test]
    fn test_macd_factory_via_field() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let macd_node = graph.add_node(macd(close, 3, 5, 2));
        let line = graph.add_node(field(macd_node, "macd"));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(line);
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_ppo_factory_via_field() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let ppo_node = graph.add_node(ppo(close, 3, 5, 2));
        let line = graph.add_node(field(ppo_node, "ppo"));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(line);
        assert!(v.iter().any(|x| x.is_some()));
    }

    // ---------- OHLC-input factories ----------

    #[test]
    fn test_atr_factory() {
        let v = run_no_input(|g| g.add_node(atr(5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_stochastic_factory_via_field() {
        let mut graph = Graph::new();
        let stoch = graph.add_node(stochastic(3, 2));
        let k = graph.add_node(field(stoch, "k"));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(k);
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_keltner_factory_via_field() {
        let mut graph = Graph::new();
        let kelt = graph.add_node(keltner(5, 5, 2.0));
        let middle = graph.add_node(field(kelt, "middle"));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(middle);
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_cci_factory() {
        let v = run_no_input(|g| g.add_node(cci(5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_cci_with_constant_factory_uses_custom_constant() {
        let mut g1 = Graph::new();
        let n1 = g1.add_node(cci(5));
        let mut g2 = Graph::new();
        let n2 = g2.add_node(cci_with_constant(5, 0.03));
        // Different constants produce different signatures (no CSE collision):
        let s1 = g1.get(n1).unwrap().signature().unwrap();
        let s2 = g2.get(n2).unwrap().signature().unwrap();
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_chandelier_factory_via_field() {
        let mut graph = Graph::new();
        let ch = graph.add_node(chandelier(5, 3.0));
        let long_exit = graph.add_node(field(ch, "long_exit"));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(long_exit);
        assert!(v.iter().any(|x| x.is_some()));
    }

    // ---------- OHLCV-input factories ----------

    #[test]
    fn test_obv_factory() {
        let v = run_no_input(|g| g.add_node(obv()));
        // OBV starts at first bar
        assert!(v[0].is_some());
    }

    #[test]
    fn test_mfi_factory() {
        let v = run_no_input(|g| g.add_node(mfi(5)));
        assert!(v.iter().any(|x| x.is_some()));
    }

    // ---------- Pattern-counting factories ----------

    #[test]
    fn test_bars_since_factory() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let const_node = graph.add_node(Box::new(crate::nodes::data::ConstNode::new(105.0)));
        let gt = graph.add_node(Box::new(crate::nodes::comparison::GtNode::new(close, const_node)));
        let bs = graph.add_node(bars_since(gt));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(bs);
        // Should produce some non-None values once a "true" event occurs
        assert!(v.iter().any(|x| x.is_some()));
    }

    #[test]
    fn test_count_when_factory() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let const_node = graph.add_node(Box::new(crate::nodes::data::ConstNode::new(105.0)));
        let gt = graph.add_node(Box::new(crate::nodes::comparison::GtNode::new(close, const_node)));
        let cw = graph.add_node(count_when(gt, 5));
        let mut executor = Executor::new(graph);
        let result = executor.process_series(&create_test_series());
        let v = result.get_output_f64(cw);
        assert!(v.iter().any(|x| x.is_some()));
    }

    // ---------- Multi-output convenience factories ----------

    #[test]
    fn test_macd_with_fields_returns_4_node_ids() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let (n, line, sig, hist) = macd_with_fields(&mut graph, close, 3, 5, 2);
        assert_ne!(n, line);
        assert_ne!(line, sig);
        assert_ne!(sig, hist);
    }

    #[test]
    fn test_stochastic_with_fields_returns_3_node_ids() {
        let mut graph = Graph::new();
        let (n, k, d) = stochastic_with_fields(&mut graph, 3, 2);
        assert_ne!(n, k);
        assert_ne!(k, d);
    }

    #[test]
    fn test_keltner_with_fields_returns_4_node_ids() {
        let mut graph = Graph::new();
        let (n, mid, up, low) = keltner_with_fields(&mut graph, 5, 5, 2.0);
        assert_ne!(n, mid);
        assert_ne!(up, low);
    }

    #[test]
    fn test_ppo_with_fields_returns_4_node_ids() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let (n, line, sig, hist) = ppo_with_fields(&mut graph, close, 3, 5, 2);
        assert_ne!(n, line);
        assert_ne!(sig, hist);
    }

    #[test]
    fn test_chandelier_with_fields_returns_3_node_ids() {
        let mut graph = Graph::new();
        let (n, le, se) = chandelier_with_fields(&mut graph, 5, 3.0);
        assert_ne!(n, le);
        assert_ne!(le, se);
    }
}
