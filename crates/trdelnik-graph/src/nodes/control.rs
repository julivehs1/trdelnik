//! Control-flow nodes used by the script DSL.
//!
//! - [`LagNode`] — value of an upstream node `n` bars ago. Backs the
//!   `expr[n]` history-access syntax.
//! - [`SelectNode`] — picks one of two values based on a boolean
//!   condition. Backs the `if cond then a else b` expression.
//!
//! Both nodes are O(1) per bar and participate in CSE via stable
//! signatures.

use std::collections::VecDeque;

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::Value;

// ============================================================================
// LagNode
// ============================================================================

/// Returns the value of `input` from `lag` bars ago.
///
/// During the first `lag` bars the output is `None` (the warmup period).
#[derive(Debug, Clone)]
pub struct LagNode {
    input: NodeId,
    inputs_arr: [NodeId; 1],
    lag: usize,
    history: VecDeque<Value>,
}

impl LagNode {
    /// Build a lag node with the given input and lag distance.
    /// `lag = 0` is allowed and is just an identity passthrough.
    pub fn new(input: NodeId, lag: usize) -> Self {
        Self {
            input,
            inputs_arr: [input],
            lag,
            history: VecDeque::with_capacity(lag + 1),
        }
    }
}

impl Node for LagNode {
    fn signature(&self) -> Option<String> {
        Some(format!("lag:{}:{}", self.input.0, self.lag))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs_arr
    }

    fn reset(&mut self) {
        self.history.clear();
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        let current = inputs.first().cloned().unwrap_or_else(Value::none_number);
        self.history.push_back(current);
        // Trim oldest until we keep at most `lag + 1` entries.
        while self.history.len() > self.lag + 1 {
            self.history.pop_front();
        }
        if self.history.len() == self.lag + 1 {
            self.history.front().cloned().unwrap_or_else(Value::none_number)
        } else {
            Value::none_number()
        }
    }

    fn warmup_period(&self) -> usize {
        self.lag
    }

    fn name(&self) -> &str {
        "lag"
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

// ============================================================================
// SelectNode
// ============================================================================

/// Picks `then_input` when `cond` is `true`, `else_input` when `false`.
/// When the condition is `None` (warmup or undecidable), the output is
/// `None` — it does not silently fall through to the else branch.
#[derive(Debug, Clone)]
pub struct SelectNode {
    inputs_arr: [NodeId; 3],
}

impl SelectNode {
    /// Build a select with condition, then-branch, else-branch.
    pub fn new(cond: NodeId, then_input: NodeId, else_input: NodeId) -> Self {
        Self {
            inputs_arr: [cond, then_input, else_input],
        }
    }
}

impl Node for SelectNode {
    fn signature(&self) -> Option<String> {
        Some(format!(
            "select:{}:{}:{}",
            self.inputs_arr[0].0, self.inputs_arr[1].0, self.inputs_arr[2].0
        ))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs_arr
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match inputs[0].as_bool() {
            Some(true) => inputs[1].clone(),
            Some(false) => inputs[2].clone(),
            None => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "select"
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::Executor;
    use crate::graph::Graph;
    use crate::nodes::data::CloseNode;
    use crate::nodes::factories::sma;
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

    #[test]
    fn lag_returns_previous_bar() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let lag1 = graph.add_node(Box::new(LagNode::new(close, 1)));

        let mut exec = Executor::new(graph);
        let s = series(&[10.0, 11.0, 12.0, 13.0]);
        let r = exec.process_series(&s);

        let lag_values = r.get_output_f64(lag1);
        assert_eq!(lag_values, vec![None, Some(10.0), Some(11.0), Some(12.0)]);
    }

    #[test]
    fn lag_zero_is_identity() {
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let lag0 = graph.add_node(Box::new(LagNode::new(close, 0)));

        let mut exec = Executor::new(graph);
        let s = series(&[10.0, 11.0, 12.0]);
        let r = exec.process_series(&s);

        assert_eq!(
            r.get_output_f64(lag0),
            vec![Some(10.0), Some(11.0), Some(12.0)]
        );
    }

    #[test]
    fn select_picks_branch_by_condition() {
        // Hand-build a graph where cond toggles between then and else.
        // We can't easily fabricate a bool node here, so use a comparison.
        use crate::nodes::comparison::GtNode;
        use crate::nodes::data::ConstNode;

        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let threshold = graph.add_node(Box::new(ConstNode::new(11.0)));
        let cond = graph.add_node(Box::new(GtNode::new(close, threshold)));
        let then_v = graph.add_node(Box::new(ConstNode::new(1.0)));
        let else_v = graph.add_node(Box::new(ConstNode::new(0.0)));
        let sel = graph.add_node(Box::new(SelectNode::new(cond, then_v, else_v)));

        let mut exec = Executor::new(graph);
        let s = series(&[10.0, 11.0, 12.0, 9.0]);
        let r = exec.process_series(&s);

        // Bar 0: 10 > 11 → false → 0
        // Bar 1: 11 > 11 → false → 0
        // Bar 2: 12 > 11 → true  → 1
        // Bar 3: 9 > 11  → false → 0
        assert_eq!(
            r.get_output_f64(sel),
            vec![Some(0.0), Some(0.0), Some(1.0), Some(0.0)]
        );
    }

    #[test]
    fn lag_signature_distinguishes_distance() {
        let close = NodeId(0);
        let a = LagNode::new(close, 1);
        let b = LagNode::new(close, 1);
        let c = LagNode::new(close, 2);
        assert_eq!(a.signature(), b.signature());
        assert_ne!(a.signature(), c.signature());
    }

    #[test]
    fn lag_works_through_an_indicator() {
        // sma(5)[1] — yesterday's SMA.
        let mut graph = Graph::new();
        let close = graph.add_node(Box::new(CloseNode::new()));
        let s5 = graph.add_node(sma(close, 3));
        let lag = graph.add_node(Box::new(LagNode::new(s5, 1)));

        let mut exec = Executor::new(graph);
        let s = series(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let r = exec.process_series(&s);

        let sma_vals = r.get_output_f64(s5);
        let lag_vals = r.get_output_f64(lag);
        // Lag(1) is None until SMA emits, then trails by one bar.
        assert_eq!(lag_vals[0], None);
        assert_eq!(lag_vals[1], None);
        assert_eq!(lag_vals[2], None); // SMA(3) emits at bar 2; lag holds prev which is None
        assert_eq!(lag_vals[3], sma_vals[2]);
        assert_eq!(lag_vals[4], sma_vals[3]);
    }

    fn ctx() -> ExecutionContext {
        ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0)
    }

    // ---------- LagNode metadata ----------

    #[test]
    fn lag_metadata() {
        let n = LagNode::new(NodeId(3), 5);
        assert_eq!(n.name(), "lag");
        assert_eq!(n.inputs(), &[NodeId(3)][..]);
        assert_eq!(n.warmup_period(), 5);
    }

    #[test]
    fn lag_reset_clears_history() {
        let mut n = LagNode::new(NodeId(0), 1);
        n.compute(&ctx(), &[Value::number(1.0)]);
        n.compute(&ctx(), &[Value::number(2.0)]);
        n.reset();
        // After reset, the next call has no history again → returns None.
        let r = n.compute(&ctx(), &[Value::number(3.0)]);
        assert!(r.is_none());
    }

    #[test]
    fn lag_clone_box_preserves_signature() {
        let n: Box<dyn Node> = Box::new(LagNode::new(NodeId(7), 2));
        let cloned = n.clone_box();
        assert_eq!(cloned.signature(), n.signature());
    }

    #[test]
    fn lag_with_empty_inputs_treats_as_none() {
        // The compute path that defends against missing inputs.
        let mut n = LagNode::new(NodeId(0), 0);
        let r = n.compute(&ctx(), &[]);
        // lag=0 acts as identity, so the result mirrors the (None) input.
        assert!(r.is_none());
    }

    // ---------- SelectNode metadata + paths ----------

    #[test]
    fn select_metadata() {
        let n = SelectNode::new(NodeId(1), NodeId(2), NodeId(3));
        assert_eq!(n.name(), "select");
        assert_eq!(n.inputs(), &[NodeId(1), NodeId(2), NodeId(3)][..]);
        assert_eq!(n.warmup_period(), 0);
        assert_eq!(n.signature().as_deref(), Some("select:1:2:3"));
    }

    #[test]
    fn select_returns_none_when_condition_is_none() {
        let mut n = SelectNode::new(NodeId(0), NodeId(1), NodeId(2));
        let r = n.compute(
            &ctx(),
            &[
                Value::none_bool(),
                Value::number(1.0),
                Value::number(0.0),
            ],
        );
        assert!(r.is_none());
    }

    #[test]
    fn select_reset_is_noop() {
        let mut n = SelectNode::new(NodeId(0), NodeId(1), NodeId(2));
        n.reset();
        let r = n.compute(
            &ctx(),
            &[Value::bool(true), Value::number(7.0), Value::number(0.0)],
        );
        assert_eq!(r.as_number(), Some(7.0));
    }

    #[test]
    fn select_clone_box() {
        let n: Box<dyn Node> = Box::new(SelectNode::new(NodeId(0), NodeId(1), NodeId(2)));
        let cloned = n.clone_box();
        assert_eq!(cloned.signature(), n.signature());
    }
}
