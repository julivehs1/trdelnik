//! Per-bar execution engine for the computation graph

use crate::context::{ExecutionContext, ExecutionResult, OutputStore};
use crate::graph::Graph;
use crate::node::NodeId;
use trdelnik_core::{AxisCoordinate, Candle, CandleSeries};

/// The executor processes bars through the computation graph.
///
/// Two usage modes are supported, both backed by the same execution path:
///
/// 1. **Streaming** — bars arrive one at a time (live data, paper trading,
///    incremental backtests). Call [`Executor::on_candle`] per bar; the
///    current outputs are available via [`Executor::output_store`]. The
///    executor tracks its own bar index. Call [`Executor::reset`] to
///    start a fresh stream.
/// 2. **Batch** — process a frozen `CandleSeries` in one shot via
///    [`Executor::process_series`]. Internally implemented as a
///    streaming loop with recording into an `ExecutionResult`.
pub struct Executor {
    graph: Graph,
    output_store: OutputStore,
    bar_count: usize,
}

impl Executor {
    /// Create a new executor with the given graph.
    ///
    /// The graph will be finalized if it hasn't been already.
    pub fn new(mut graph: Graph) -> Self {
        graph.finalize();
        let output_store = OutputStore::with_capacity(graph.len());
        Self {
            graph,
            output_store,
            bar_count: 0,
        }
    }

    /// Process a single bar from a low-level `ExecutionContext`.
    ///
    /// Use this when you need to drive the executor with a custom-built
    /// context (synthetic data, replays with custom bar indices, etc.).
    /// Most callers should prefer [`Executor::on_candle`] which builds
    /// the context from a `Candle` and tracks the bar index automatically.
    pub fn process_bar(&mut self, ctx: &ExecutionContext) {
        self.output_store.clear();

        // Process nodes in topological order
        let execution_order: Vec<NodeId> = self.graph.execution_order().to_vec();

        for node_id in execution_order {
            // Get inputs for this node
            let node = self.graph.get(node_id).unwrap();
            let input_ids = node.inputs().to_vec();
            let inputs = self.output_store.get_inputs(&input_ids);

            // Compute the node's output
            let node = self.graph.get_mut(node_id).unwrap();
            let output = node.compute(ctx, &inputs);

            // Store the output
            self.output_store.set(node_id, output);
        }
    }

    /// Stream a single candle through the graph.
    ///
    /// The executor builds an `ExecutionContext` from `candle` using its
    /// internal bar counter, runs every node once, and returns the
    /// resulting `OutputStore` snapshot. The bar counter advances by one.
    ///
    /// This is the primary streaming API. Use it for live feeds, paper
    /// trading, or any pipeline where bars arrive one by one.
    pub fn on_candle<X: AxisCoordinate>(&mut self, candle: &Candle<X>) -> &OutputStore {
        let ctx = ExecutionContext::from_candle(self.bar_count, candle);
        self.process_bar(&ctx);
        self.bar_count += 1;
        &self.output_store
    }

    /// Stream a candle and append its outputs to an `ExecutionResult`.
    ///
    /// Same as [`Executor::on_candle`], but also pushes the current
    /// outputs into `result`. Use when you want streaming semantics but
    /// still accumulate history (e.g. live backtests that need to plot a
    /// growing equity curve).
    pub fn on_candle_recording<X: AxisCoordinate>(
        &mut self,
        candle: &Candle<X>,
        result: &mut ExecutionResult,
    ) {
        let ctx = ExecutionContext::from_candle(self.bar_count, candle);
        self.process_bar(&ctx);
        self.bar_count += 1;
        result.push_bar(ctx.x, &self.output_store);
    }

    /// Get the current output store (after processing a bar)
    pub fn output_store(&self) -> &OutputStore {
        &self.output_store
    }

    /// Number of bars streamed since the last `reset` (or since construction).
    pub fn bar_count(&self) -> usize {
        self.bar_count
    }

    /// Process a complete candle series and return the accumulated results.
    ///
    /// Convenience over [`Executor::on_candle_recording`]: resets state,
    /// streams every candle, and returns the `ExecutionResult`.
    pub fn process_series<X: AxisCoordinate>(&mut self, series: &CandleSeries<X>) -> ExecutionResult {
        self.reset();
        let mut result = ExecutionResult::with_capacity(self.graph.len(), series.len());
        for candle in series.candles() {
            self.on_candle_recording(candle, &mut result);
        }
        result
    }

    /// Reset all node states, clear the output store, and rewind the bar
    /// counter — ready for a fresh stream.
    pub fn reset(&mut self) {
        self.graph.reset_all();
        self.output_store.clear();
        self.bar_count = 0;
    }

    /// Get the underlying graph
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get mutable access to the graph
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Get the maximum warmup period across all nodes
    pub fn max_warmup_period(&self) -> usize {
        self.graph.max_warmup_period()
    }
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            .field("graph", &self.graph)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::value::Value;

    // Simple test node that outputs the close price
    #[derive(Clone)]
    struct CloseTestNode;

    impl Node for CloseTestNode {
        fn signature(&self) -> Option<String> {
            Some("close".to_string())
        }

        fn inputs(&self) -> &[NodeId] {
            &[]
        }

        fn reset(&mut self) {}

        fn compute(&mut self, ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
            Value::number(ctx.close)
        }

        fn warmup_period(&self) -> usize {
            0
        }

        fn name(&self) -> &str {
            "Close"
        }

        crate::impl_clone_box!(CloseTestNode);
    }

    // Simple test node that doubles its input
    #[derive(Clone)]
    struct DoubleNode {
        input: NodeId,
    }

    impl Node for DoubleNode {
        fn signature(&self) -> Option<String> {
            Some(format!("double:{}", self.input.0))
        }

        fn inputs(&self) -> &[NodeId] {
            std::slice::from_ref(&self.input)
        }

        fn reset(&mut self) {}

        fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
            inputs[0].as_number().map(|v| v * 2.0).into()
        }

        fn warmup_period(&self) -> usize {
            0
        }

        fn name(&self) -> &str {
            "Double"
        }

        crate::impl_clone_box!(DoubleNode);
    }

    #[test]
    fn test_executor_single_bar() {
        let mut graph = Graph::new();
        let close_id = graph.add_node(Box::new(CloseTestNode));
        let double_id = graph.add_node(Box::new(DoubleNode { input: close_id }));

        let mut executor = Executor::new(graph);
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        executor.process_bar(&ctx);

        let close_output = executor.output_store().get(close_id).unwrap();
        let double_output = executor.output_store().get(double_id).unwrap();

        assert_eq!(close_output.as_number(), Some(105.0));
        assert_eq!(double_output.as_number(), Some(210.0));
    }

    #[test]
    fn test_executor_series() {
        use trdelnik_core::{Candle, Timestamp};

        let mut graph = Graph::new();
        let close_id = graph.add_node(Box::new(CloseTestNode));

        let mut executor = Executor::new(graph);

        let mut series = CandleSeries::new();
        series.push(Candle::new(Timestamp(1000), 100.0, 110.0, 95.0, 105.0, 1000.0));
        series.push(Candle::new(Timestamp(2000), 105.0, 115.0, 100.0, 110.0, 1500.0));
        series.push(Candle::new(Timestamp(3000), 110.0, 120.0, 105.0, 115.0, 2000.0));

        let result = executor.process_series(&series);

        assert_eq!(result.len(), 3);
        let closes = result.get_output_f64(close_id);
        assert_eq!(closes, vec![Some(105.0), Some(110.0), Some(115.0)]);
    }

    fn streaming_test_series() -> (CandleSeries<trdelnik_core::Timestamp>, Vec<f64>) {
        use trdelnik_core::{Candle, Timestamp};
        let mut series = CandleSeries::new();
        let closes = [105.0, 110.0, 115.0, 120.0, 125.0];
        for (i, &c) in closes.iter().enumerate() {
            series.push(Candle::new(
                Timestamp((i as i64 + 1) * 1000),
                c - 5.0,
                c + 5.0,
                c - 10.0,
                c,
                1000.0,
            ));
        }
        (series, closes.to_vec())
    }

    #[test]
    fn test_streaming_on_candle() {
        let mut graph = Graph::new();
        let close_id = graph.add_node(Box::new(CloseTestNode));
        let mut executor = Executor::new(graph);

        let (series, closes) = streaming_test_series();

        for (i, candle) in series.candles().iter().enumerate() {
            let close_value = executor.on_candle(candle).get(close_id).unwrap().as_number();
            assert_eq!(executor.bar_count(), i + 1);
            assert_eq!(close_value, Some(closes[i]));
        }
    }

    #[test]
    fn test_streaming_matches_batch() {
        // Same graph, same data: streaming and batch must agree on every bar.
        let mut g1 = Graph::new();
        let close1 = g1.add_node(Box::new(CloseTestNode));
        let dbl1 = g1.add_node(Box::new(DoubleNode { input: close1 }));
        let mut e1 = Executor::new(g1);

        let mut g2 = Graph::new();
        let close2 = g2.add_node(Box::new(CloseTestNode));
        let dbl2 = g2.add_node(Box::new(DoubleNode { input: close2 }));
        let mut e2 = Executor::new(g2);

        let (series, _) = streaming_test_series();
        let batch = e1.process_series(&series);

        let mut streamed = ExecutionResult::with_capacity(2, series.len());
        for candle in series.candles() {
            e2.on_candle_recording(candle, &mut streamed);
        }

        assert_eq!(batch.get_output_f64(close1), streamed.get_output_f64(close2));
        assert_eq!(batch.get_output_f64(dbl1), streamed.get_output_f64(dbl2));
        assert_eq!(batch.x_values(), streamed.x_values());
    }

    #[test]
    fn test_reset_rewinds_bar_count() {
        let mut graph = Graph::new();
        graph.add_node(Box::new(CloseTestNode));
        let mut executor = Executor::new(graph);
        let (series, _) = streaming_test_series();

        for candle in series.candles() {
            executor.on_candle(candle);
        }
        assert_eq!(executor.bar_count(), series.len());

        executor.reset();
        assert_eq!(executor.bar_count(), 0);
    }

    #[test]
    fn test_executor_finalizes_unfinalized_graph() {
        // Graph::new() returns an unfinalized graph; Executor::new must
        // finalize it.
        let mut graph = Graph::new();
        let _ = graph.add_node(Box::new(CloseTestNode));
        assert!(!graph.is_finalized());
        let exec = Executor::new(graph);
        assert!(exec.graph().is_finalized());
    }

    #[test]
    fn test_graph_accessor_returns_finalized_graph() {
        let mut g = Graph::new();
        g.add_node(Box::new(CloseTestNode));
        let exec = Executor::new(g);
        assert!(exec.graph().is_finalized());
        assert_eq!(exec.graph().len(), 1);
    }

    #[test]
    fn test_graph_mut_can_reset_via_executor() {
        let mut g = Graph::new();
        g.add_node(Box::new(CloseTestNode));
        let mut exec = Executor::new(g);
        // Reset via the mutable accessor.
        exec.graph_mut().reset_all();
    }

    #[test]
    fn test_max_warmup_period_proxies_graph() {
        let mut g = Graph::new();
        g.add_node(Box::new(CloseTestNode));
        let exec = Executor::new(g);
        assert_eq!(exec.max_warmup_period(), exec.graph().max_warmup_period());
    }

    #[test]
    fn test_output_store_after_process_bar() {
        let mut g = Graph::new();
        let close = g.add_node(Box::new(CloseTestNode));
        let mut exec = Executor::new(g);
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);
        exec.process_bar(&ctx);
        let store = exec.output_store();
        assert_eq!(store.get(close).unwrap().as_number(), Some(105.0));
    }

    #[test]
    fn test_process_bar_clears_previous_bar_outputs() {
        // Successive process_bar calls overwrite — ensure the second bar's
        // output reflects the new context, not stale data.
        let mut g = Graph::new();
        let close = g.add_node(Box::new(CloseTestNode));
        let mut exec = Executor::new(g);

        let ctx_a = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);
        exec.process_bar(&ctx_a);

        let ctx_b = ExecutionContext::new(1, 200.0, 220.0, 195.0, 215.0, 2000.0, 0.0);
        exec.process_bar(&ctx_b);

        let store = exec.output_store();
        assert_eq!(store.get(close).unwrap().as_number(), Some(215.0));
    }

    #[test]
    fn test_reset_after_process_bar_clears_output_store() {
        let mut g = Graph::new();
        let close = g.add_node(Box::new(CloseTestNode));
        let mut exec = Executor::new(g);
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);
        exec.process_bar(&ctx);
        assert!(exec.output_store().get(close).is_some());

        exec.reset();
        assert!(exec.output_store().get(close).is_none());
    }

    #[test]
    fn test_debug_impl_includes_graph() {
        let mut g = Graph::new();
        g.add_node(Box::new(CloseTestNode));
        let exec = Executor::new(g);
        let s = format!("{:?}", exec);
        assert!(s.contains("Executor"));
        assert!(s.contains("graph"));
    }

    #[test]
    fn test_process_series_resets_bar_count_first() {
        use trdelnik_core::{Candle, Timestamp};

        let mut g = Graph::new();
        let _ = g.add_node(Box::new(CloseTestNode));
        let mut exec = Executor::new(g);

        // Stream a few bars first to advance the counter.
        let mut s = CandleSeries::new();
        s.push(Candle::new(Timestamp(0), 1.0, 2.0, 0.5, 1.5, 1.0));
        s.push(Candle::new(Timestamp(1000), 1.0, 2.0, 0.5, 1.5, 1.0));
        for c in s.candles() {
            exec.on_candle(c);
        }
        assert_eq!(exec.bar_count(), 2);

        // process_series should reset internally → counter ends at series.len().
        let _ = exec.process_series(&s);
        assert_eq!(exec.bar_count(), s.len());
    }
}
