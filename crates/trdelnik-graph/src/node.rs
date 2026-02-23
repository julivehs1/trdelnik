//! Node trait and NodeId for the computation graph

use crate::context::ExecutionContext;
use crate::value::Value;

/// A unique identifier for a node in the computation graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    /// Create a new NodeId
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Get the underlying index
    pub fn index(&self) -> usize {
        self.0
    }
}

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        NodeId(id)
    }
}

/// Trait for computation graph nodes.
///
/// Nodes are stateful computation units that receive input values
/// and produce output values. They can maintain internal state
/// (like running sums for SMA) across bar computations.
pub trait Node: Send + Sync {
    /// Return a signature for Common Subexpression Elimination (CSE).
    ///
    /// The signature format is: `{node_type}:{input_ids}:{parameters}`
    ///
    /// Examples:
    /// - `sma:0:20` (SMA of node 0 with period 20)
    /// - `ema:0:12` (EMA of node 0 with period 12)
    /// - `close` (Close node - no inputs or params)
    ///
    /// Return `None` if this node cannot participate in CSE
    /// (e.g., nodes with non-deterministic behavior).
    fn signature(&self) -> Option<String>;

    /// Get the input node IDs that this node depends on.
    fn inputs(&self) -> &[NodeId];

    /// Reset the node's internal state.
    ///
    /// This is called before processing a new series of bars.
    fn reset(&mut self);

    /// Compute the output value for the current bar.
    ///
    /// # Arguments
    /// * `ctx` - The execution context containing the current bar data
    /// * `inputs` - The input values from dependent nodes (in the same order as `inputs()`)
    ///
    /// # Returns
    /// The computed value for this bar
    fn compute(&mut self, ctx: &ExecutionContext, inputs: &[Value]) -> Value;

    /// Get the warmup period for this node.
    ///
    /// The warmup period is the number of bars required before
    /// this node can produce a valid output. During warmup,
    /// `compute` should return `Value::none_number()` or `Value::none_bool()`.
    fn warmup_period(&self) -> usize;

    /// Get the display name of this node.
    fn name(&self) -> &str;

    /// Clone this node into a new Box.
    ///
    /// This is used to implement Clone for Graph.
    fn clone_box(&self) -> Box<dyn Node>;
}

/// A boxed node for use in the graph
pub type BoxedNode = Box<dyn Node>;

/// Implement Clone for BoxedNode via clone_box
impl Clone for Box<dyn Node> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Helper macro to implement clone_box for node types that derive Clone.
///
/// Usage: `impl_clone_box!(MyNodeType);`
#[macro_export]
macro_rules! impl_clone_box {
    ($node_type:ty) => {
        fn clone_box(&self) -> Box<dyn $crate::node::Node> {
            Box::new(self.clone())
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.index(), 42);
        assert_eq!(id.0, 42);

        let id2: NodeId = 42.into();
        assert_eq!(id, id2);
    }
}
