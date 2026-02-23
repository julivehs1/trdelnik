//! Execution context and output storage for the graph executor

use crate::node::NodeId;
use crate::value::Value;
use std::collections::HashMap;

/// The execution context provides access to the current bar's OHLCV data
/// and any additional context needed during computation.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Current bar index (0-based)
    pub bar_index: usize,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
    /// X-axis value (timestamp or other coordinate)
    pub x: f64,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(bar_index: usize, open: f64, high: f64, low: f64, close: f64, volume: f64, x: f64) -> Self {
        Self {
            bar_index,
            open,
            high,
            low,
            close,
            volume,
            x,
        }
    }

    /// Create from a candle
    pub fn from_candle<X: trdelnik_core::AxisCoordinate>(
        bar_index: usize,
        candle: &trdelnik_core::Candle<X>,
    ) -> Self {
        Self {
            bar_index,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
            x: candle.x.to_plot_value(),
        }
    }
}

/// Storage for node outputs during execution.
///
/// This stores the most recent output of each node so that
/// downstream nodes can access their inputs.
#[derive(Debug, Clone, Default)]
pub struct OutputStore {
    outputs: HashMap<NodeId, Value>,
}

impl OutputStore {
    /// Create a new output store
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            outputs: HashMap::with_capacity(capacity),
        }
    }

    /// Store a node's output
    pub fn set(&mut self, node_id: NodeId, value: Value) {
        self.outputs.insert(node_id, value);
    }

    /// Get a node's output
    pub fn get(&self, node_id: NodeId) -> Option<Value> {
        self.outputs.get(&node_id).cloned()
    }

    /// Get multiple outputs for a list of node IDs
    pub fn get_inputs(&self, node_ids: &[NodeId]) -> Vec<Value> {
        node_ids
            .iter()
            .map(|id| self.get(*id).unwrap_or_default())
            .collect()
    }

    /// Clear all outputs
    pub fn clear(&mut self) {
        self.outputs.clear();
    }
}

/// Accumulated results from processing a series of bars.
///
/// This stores the history of outputs for each node across all bars.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output history for each node: node_id -> Vec<Value>
    outputs: HashMap<NodeId, Vec<Value>>,
    /// X-axis values for each bar
    x_values: Vec<f64>,
}

impl ExecutionResult {
    /// Create a new execution result
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
            x_values: Vec::new(),
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(node_count: usize, bar_count: usize) -> Self {
        let mut outputs = HashMap::with_capacity(node_count);
        for i in 0..node_count {
            outputs.insert(NodeId(i), Vec::with_capacity(bar_count));
        }
        Self {
            outputs,
            x_values: Vec::with_capacity(bar_count),
        }
    }

    /// Record outputs for a bar
    pub fn push_bar(&mut self, x: f64, store: &OutputStore) {
        self.x_values.push(x);
        for (&node_id, value) in store.outputs.iter() {
            self.outputs
                .entry(node_id)
                .or_insert_with(Vec::new)
                .push(value.clone());
        }
    }

    /// Get the output history for a node
    pub fn get_output(&self, node_id: NodeId) -> &[Value] {
        self.outputs.get(&node_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the output history as Option<f64> values
    pub fn get_output_f64(&self, node_id: NodeId) -> Vec<Option<f64>> {
        self.get_output(node_id)
            .iter()
            .map(|v| v.to_option_f64())
            .collect()
    }

    /// Get the X-axis values
    pub fn x_values(&self) -> &[f64] {
        &self.x_values
    }

    /// Get the number of bars processed
    pub fn len(&self) -> usize {
        self.x_values.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.x_values.is_empty()
    }
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);
        assert_eq!(ctx.bar_index, 0);
        assert_eq!(ctx.open, 100.0);
        assert_eq!(ctx.close, 105.0);
    }

    #[test]
    fn test_output_store() {
        let mut store = OutputStore::new();
        let id = NodeId(0);

        store.set(id, Value::number(42.0));
        assert_eq!(store.get(id), Some(Value::number(42.0)));

        let inputs = store.get_inputs(&[id, NodeId(1)]);
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].as_number(), Some(42.0));
        assert_eq!(inputs[1].as_number(), None); // NodeId(1) not set
    }

    #[test]
    fn test_execution_result() {
        let mut result = ExecutionResult::new();
        let mut store = OutputStore::new();

        store.set(NodeId(0), Value::number(1.0));
        store.set(NodeId(1), Value::number(2.0));
        result.push_bar(1000.0, &store);

        store.set(NodeId(0), Value::number(3.0));
        store.set(NodeId(1), Value::number(4.0));
        result.push_bar(2000.0, &store);

        assert_eq!(result.len(), 2);
        assert_eq!(result.x_values(), &[1000.0, 2000.0]);
        assert_eq!(result.get_output_f64(NodeId(0)), vec![Some(1.0), Some(3.0)]);
        assert_eq!(result.get_output_f64(NodeId(1)), vec![Some(2.0), Some(4.0)]);
    }
}
