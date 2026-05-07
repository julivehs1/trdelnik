//! Pattern-counting nodes — O(1) per bar.
//!
//! These cover the loop-style needs that the DSL deliberately doesn't support:
//!
//! - [`BarsSinceNode`] — bars elapsed since `cond` was last true. Backs
//!   `bars_since(cond)` in the DSL.
//! - [`CountWhenNode`] — how often `cond` was true in the last `period`
//!   bars. Backs `count_when(cond, period)` in the DSL.
//!
//! Both maintain incremental state and run in constant time per bar,
//! so they fit the streaming-graph execution model.

use std::collections::VecDeque;

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::Value;

// ============================================================================
// BarsSinceNode
// ============================================================================

/// Bars elapsed since `cond` was last true.
///
/// - On the bar where `cond` is `true`, the output is `0`.
/// - On the next bar, `1`. Then `2`, `3`, ...
/// - Before `cond` has ever been true, the output is `None`.
/// - When `cond` is `None` (undefined), state is unchanged and the
///   output is `None` for that bar.
#[derive(Debug, Clone)]
pub struct BarsSinceNode {
    input: NodeId,
    inputs_arr: [NodeId; 1],
    counter: Option<usize>,
}

impl BarsSinceNode {
    /// Build a bars_since node tracking `input` (must be a bool stream).
    pub fn new(input: NodeId) -> Self {
        Self {
            input,
            inputs_arr: [input],
            counter: None,
        }
    }
}

impl Node for BarsSinceNode {
    fn signature(&self) -> Option<String> {
        Some(format!("bars_since:{}", self.input.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs_arr
    }

    fn reset(&mut self) {
        self.counter = None;
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match inputs[0].as_bool() {
            Some(true) => {
                self.counter = Some(0);
                Value::number(0.0)
            }
            Some(false) => {
                self.counter = self.counter.map(|c| c + 1);
                match self.counter {
                    Some(c) => Value::number(c as f64),
                    None => Value::none_number(),
                }
            }
            None => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "bars_since"
    }

    crate::impl_clone_box!(BarsSinceNode);
}

// ============================================================================
// CountWhenNode
// ============================================================================

/// Rolling count of how often `cond` was true in the last `period` bars
/// (current bar inclusive).
///
/// During warmup (fewer than `period` bars seen) the output is `None`.
/// `None` inputs are counted as "not true" — they neither add nor reset
/// the counter, and they remain in the window so the output stays
/// well-defined once enough bars have passed.
#[derive(Debug, Clone)]
pub struct CountWhenNode {
    input: NodeId,
    inputs_arr: [NodeId; 1],
    period: usize,
    history: VecDeque<bool>,
    count: usize,
}

impl CountWhenNode {
    /// Build a count_when node tracking `input` over a window of `period`
    /// bars. `period` must be at least 1.
    pub fn new(input: NodeId, period: usize) -> Self {
        let period = period.max(1);
        Self {
            input,
            inputs_arr: [input],
            period,
            history: VecDeque::with_capacity(period),
            count: 0,
        }
    }
}

impl Node for CountWhenNode {
    fn signature(&self) -> Option<String> {
        Some(format!("count_when:{}:{}", self.input.0, self.period))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs_arr
    }

    fn reset(&mut self) {
        self.history.clear();
        self.count = 0;
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        let truthy = inputs[0].as_bool().unwrap_or(false);

        self.history.push_back(truthy);
        if truthy {
            self.count += 1;
        }
        if self.history.len() > self.period {
            if let Some(old) = self.history.pop_front() {
                if old {
                    self.count -= 1;
                }
            }
        }

        if self.history.len() == self.period {
            Value::number(self.count as f64)
        } else {
            Value::none_number()
        }
    }

    fn warmup_period(&self) -> usize {
        self.period.saturating_sub(1)
    }

    fn name(&self) -> &str {
        "count_when"
    }

    crate::impl_clone_box!(CountWhenNode);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::Executor;
    use crate::graph::Graph;
    use crate::nodes::comparison::GtNode;
    use crate::nodes::data::{CloseNode, ConstNode};
    use trdelnik_core::{Candle, CandleSeries, Timestamp};

    fn series(closes: &[f64]) -> CandleSeries<Timestamp> {
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

    /// Build a graph with `close > threshold` as the boolean condition,
    /// fed into a node produced by `make`. Returns the output node id.
    fn run_with_cond(
        closes: &[f64],
        threshold: f64,
        make: impl FnOnce(NodeId) -> Box<dyn Node>,
    ) -> Vec<Option<f64>> {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let thr = graph.add_node(Box::new(ConstNode::new(threshold)));
        let cond = graph.add_node(Box::new(GtNode::new(close, thr)));
        let out = graph.add_node(make(cond));

        let mut exec = Executor::new(graph);
        let r = exec.process_series(&series(closes));
        r.get_output_f64(out)
    }

    #[test]
    fn bars_since_counts_from_zero_on_event() {
        // close > 10? false, true, false, false, true, false
        let closes = [9.0, 11.0, 8.0, 5.0, 12.0, 7.0];
        let out = run_with_cond(&closes, 10.0, |c| Box::new(BarsSinceNode::new(c)));

        assert_eq!(
            out,
            vec![
                None,         // never true yet
                Some(0.0),    // event!
                Some(1.0),
                Some(2.0),
                Some(0.0),    // event again
                Some(1.0),
            ]
        );
    }

    #[test]
    fn bars_since_signature_distinguishes_input() {
        let a = BarsSinceNode::new(NodeId(0));
        let b = BarsSinceNode::new(NodeId(0));
        let c = BarsSinceNode::new(NodeId(1));
        assert_eq!(a.signature(), b.signature());
        assert_ne!(a.signature(), c.signature());
    }

    #[test]
    fn count_when_rolls_correctly() {
        // close > 10?  in 4-bar window, count of trues
        let closes = [9.0, 11.0, 12.0, 8.0, 13.0, 7.0];
        let out = run_with_cond(&closes, 10.0, |c| Box::new(CountWhenNode::new(c, 4)));

        // Bar 0..2: warmup (window not full yet).
        // Bar 3: window = [F, T, T, F] → 2
        // Bar 4: window = [T, T, F, T] → 3
        // Bar 5: window = [T, F, T, F] → 2
        assert_eq!(
            out,
            vec![None, None, None, Some(2.0), Some(3.0), Some(2.0)]
        );
    }

    #[test]
    fn count_when_period_one_is_indicator_of_current_bar() {
        let closes = [9.0, 11.0, 8.0];
        let out = run_with_cond(&closes, 10.0, |c| Box::new(CountWhenNode::new(c, 1)));
        assert_eq!(out, vec![Some(0.0), Some(1.0), Some(0.0)]);
    }

    #[test]
    fn count_when_signature_distinguishes_period() {
        let a = CountWhenNode::new(NodeId(0), 5);
        let b = CountWhenNode::new(NodeId(0), 5);
        let c = CountWhenNode::new(NodeId(0), 10);
        assert_eq!(a.signature(), b.signature());
        assert_ne!(a.signature(), c.signature());
    }
}
