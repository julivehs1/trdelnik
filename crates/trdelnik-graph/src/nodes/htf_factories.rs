//! Factory functions for higher-timeframe indicator nodes.
//!
//! Wrappers around [`HtfIndicatorNode`] that build HTF candles by
//! aggregating every `every_n` primary bars and run the inner indicator
//! only on closed HTF bars (forward-filling between closes).
//!
//! Unlike the primary-stream factories in [`super::factories`], these
//! do not take a `NodeId` input — the inner indicator's input is built
//! from the per-bar `ExecutionContext` via [`HtfInput`].
//!
//! ```ignore
//! // 5m primary candles, run RSI(14) on 1h aggregates → every_n = 12.
//! let mut graph = Graph::new();
//! let htf_rsi = graph.add_node(htf_rsi(12, 14));
//! ```

use crate::node::BoxedNode;
use crate::nodes::htf::HtfIndicatorNode;
use trdelnik_indicators::{Atr, Ema, Rsi, Sma};

/// HTF Simple Moving Average over closed HTF bars.
pub fn htf_sma(every_n: usize, period: usize) -> BoxedNode {
    Box::new(HtfIndicatorNode::new(every_n, Sma::new(period)))
}

/// HTF Exponential Moving Average over closed HTF bars.
pub fn htf_ema(every_n: usize, period: usize) -> BoxedNode {
    Box::new(HtfIndicatorNode::new(every_n, Ema::new(period)))
}

/// HTF Relative Strength Index over closed HTF bars.
pub fn htf_rsi(every_n: usize, period: usize) -> BoxedNode {
    Box::new(HtfIndicatorNode::new(every_n, Rsi::new(period)))
}

/// HTF Average True Range over closed HTF bars (uses high/low/close from
/// the aggregated candle).
pub fn htf_atr(every_n: usize, period: usize) -> BoxedNode {
    Box::new(HtfIndicatorNode::new(every_n, Atr::new(period)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ExecutionContext;
    use crate::executor::Executor;
    use crate::graph::Graph;
    use trdelnik_core::{Candle, CandleSeries, Timestamp};

    fn make_series(closes: &[f64]) -> CandleSeries<Timestamp> {
        let mut s = CandleSeries::new();
        for (i, &c) in closes.iter().enumerate() {
            s.push(Candle::new(
                Timestamp((i as i64 + 1) * 1000),
                c,
                c + 0.5,
                c - 0.5,
                c,
                100.0,
            ));
        }
        s
    }

    #[test]
    fn htf_sma_runs_through_executor() {
        // 12 primary bars, every_n=4 → 3 HTF bars; SMA(2) is live from HTF bar 2.
        let mut graph = Graph::new();
        let _node = graph.add_node(htf_sma(4, 2));

        let mut exec = Executor::new(graph);
        let series = make_series(&[
            10.0, 11.0, 12.0, 13.0, // HTF1 closes at 13
            14.0, 15.0, 16.0, 17.0, // HTF2 closes at 17 → SMA(2) = (13+17)/2 = 15
            18.0, 19.0, 20.0, 21.0, // HTF3 closes at 21 → SMA(2) = (17+21)/2 = 19
        ]);
        let result = exec.process_series(&series);

        let outputs = result.get_output_f64(crate::node::NodeId(0));
        // Last 4 primary bars sit inside HTF3 → SMA(2) over [17, 21] = 19.0.
        assert_eq!(outputs[11], Some(19.0));
        // Bar at idx 7 closes HTF2 → SMA(2) over [13, 17] = 15.0.
        assert_eq!(outputs[7], Some(15.0));
        // Bars before any closed HTF or before SMA warmup are None.
        assert!(outputs[3].is_none());
    }

    #[test]
    fn htf_factories_distinct_signatures() {
        let _ = ExecutionContext::new(0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let a = htf_sma(4, 20);
        let b = htf_ema(4, 20);
        let c = htf_rsi(4, 20);
        let d = htf_atr(4, 20);
        assert_ne!(a.signature(), b.signature());
        assert_ne!(a.signature(), c.signature());
        assert_ne!(a.signature(), d.signature());
    }
}
