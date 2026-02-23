//! Per-bar execution engine for the computation graph

use crate::context::{ExecutionContext, ExecutionResult, OutputStore};
use crate::graph::Graph;
use crate::node::NodeId;
use trdelnik_core::{AxisCoordinate, CandleSeries};

/// The executor processes bars through the computation graph.
///
/// It maintains the graph's state and executes nodes in topological order
/// for each bar of data.
pub struct Executor {
    graph: Graph,
    output_store: OutputStore,
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
        }
    }

    /// Process a single bar and return the output store.
    ///
    /// This is useful for streaming/real-time processing where
    /// bars arrive one at a time.
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

    /// Get the current output store (after processing a bar)
    pub fn output_store(&self) -> &OutputStore {
        &self.output_store
    }

    /// Process a complete candle series and return the accumulated results.
    pub fn process_series<X: AxisCoordinate>(&mut self, series: &CandleSeries<X>) -> ExecutionResult {
        // Reset all node states
        self.graph.reset_all();

        let mut result = ExecutionResult::with_capacity(self.graph.len(), series.len());

        for (bar_index, candle) in series.candles().iter().enumerate() {
            let ctx = ExecutionContext::from_candle(bar_index, candle);
            self.process_bar(&ctx);
            result.push_bar(ctx.x, &self.output_store);
        }

        result
    }

    /// Reset all node states for a fresh run
    pub fn reset(&mut self) {
        self.graph.reset_all();
        self.output_store.clear();
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
}
