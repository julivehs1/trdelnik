//! Field extraction node for multi-output indicators.
//!
//! FieldNode extracts a specific field from a struct Value, enabling
//! access to individual outputs of multi-output indicators like Bollinger Bands.

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::Value;

/// Node that extracts a field from a struct Value.
///
/// This enables efficient reuse of multi-output indicators by computing
/// them once and extracting individual fields as needed.
///
/// # Example
/// ```ignore
/// // First, create a Bollinger node that outputs a struct
/// let boll = graph.add_node(bollinger(close, 20, 2.0));
///
/// // Then extract individual fields
/// let middle = graph.add_node(field(boll, "middle"));
/// let upper = graph.add_node(field(boll, "upper"));
/// let lower = graph.add_node(field(boll, "lower"));
/// ```
#[derive(Debug, Clone)]
pub struct FieldNode {
    input: NodeId,
    field_name: String,
}

impl FieldNode {
    /// Create a new field extraction node
    pub fn new(input: NodeId, field_name: impl Into<String>) -> Self {
        Self {
            input,
            field_name: field_name.into(),
        }
    }

    /// Get the input node ID
    pub fn input(&self) -> NodeId {
        self.input
    }

    /// Get the field name being extracted
    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}

impl Node for FieldNode {
    fn signature(&self) -> Option<String> {
        Some(format!("field:{}:{}", self.input.0, self.field_name))
    }

    fn inputs(&self) -> &[NodeId] {
        std::slice::from_ref(&self.input)
    }

    fn reset(&mut self) {
        // FieldNode has no state to reset
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        inputs[0]
            .get_field(&self.field_name)
            .unwrap_or(Value::none_number())
    }

    fn warmup_period(&self) -> usize {
        // The warmup is determined by the parent node
        0
    }

    fn name(&self) -> &str {
        &self.field_name
    }

    crate::impl_clone_box!(FieldNode);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_ctx() -> ExecutionContext {
        ExecutionContext::new(0, 100.0, 101.0, 99.0, 100.0, 1000.0, 0.0)
    }

    #[test]
    fn test_field_node_extracts_field() {
        let mut node = FieldNode::new(NodeId(0), "middle");

        let mut fields = BTreeMap::new();
        fields.insert("upper".to_string(), Value::number(110.0));
        fields.insert("middle".to_string(), Value::number(100.0));
        fields.insert("lower".to_string(), Value::number(90.0));
        let struct_value = Value::structure(fields);

        let result = node.compute(&create_ctx(), &[struct_value]);

        assert_eq!(result.as_number(), Some(100.0));
    }

    #[test]
    fn test_field_node_missing_field() {
        let mut node = FieldNode::new(NodeId(0), "nonexistent");

        let mut fields = BTreeMap::new();
        fields.insert("upper".to_string(), Value::number(110.0));
        let struct_value = Value::structure(fields);

        let result = node.compute(&create_ctx(), &[struct_value]);

        assert!(result.is_none());
    }

    #[test]
    fn test_field_node_none_struct() {
        let mut node = FieldNode::new(NodeId(0), "middle");

        let result = node.compute(&create_ctx(), &[Value::none_struct()]);

        assert!(result.is_none());
    }

    #[test]
    fn test_field_node_signature() {
        let node = FieldNode::new(NodeId(5), "upper");
        assert_eq!(node.signature(), Some("field:5:upper".to_string()));
    }

    #[test]
    fn test_field_node_cse_same_field() {
        // Two field nodes with the same input and field should have the same signature
        let node1 = FieldNode::new(NodeId(3), "middle");
        let node2 = FieldNode::new(NodeId(3), "middle");

        assert_eq!(node1.signature(), node2.signature());
    }

    #[test]
    fn test_field_node_cse_different_field() {
        // Two field nodes with different fields should have different signatures
        let node1 = FieldNode::new(NodeId(3), "upper");
        let node2 = FieldNode::new(NodeId(3), "lower");

        assert_ne!(node1.signature(), node2.signature());
    }

    #[test]
    fn test_field_node_input_getter() {
        let n = FieldNode::new(NodeId(42), "x");
        assert_eq!(n.input(), NodeId(42));
    }

    #[test]
    fn test_field_node_field_name_getter() {
        let n = FieldNode::new(NodeId(0), "middle");
        assert_eq!(n.field_name(), "middle");
    }

    #[test]
    fn test_field_node_name_returns_field_name() {
        let n = FieldNode::new(NodeId(0), "histogram");
        assert_eq!(n.name(), "histogram");
    }

    #[test]
    fn test_field_node_inputs_slice() {
        let n = FieldNode::new(NodeId(7), "x");
        assert_eq!(n.inputs(), &[NodeId(7)][..]);
    }

    #[test]
    fn test_field_node_warmup_is_zero() {
        let n = FieldNode::new(NodeId(0), "x");
        assert_eq!(n.warmup_period(), 0);
    }

    #[test]
    fn test_field_node_reset_is_noop() {
        let mut n = FieldNode::new(NodeId(0), "x");
        n.reset();
        // After reset, compute behaves identically to before.
        let mut fields = BTreeMap::new();
        fields.insert("x".to_string(), Value::number(1.0));
        let r = n.compute(&create_ctx(), &[Value::structure(fields)]);
        assert_eq!(r.as_number(), Some(1.0));
    }

    #[test]
    fn test_field_node_clone_box_preserves_field_name() {
        let n: Box<dyn Node> = Box::new(FieldNode::new(NodeId(0), "k"));
        let cloned = n.clone_box();
        assert_eq!(cloned.signature(), n.signature());
        assert_eq!(cloned.name(), "k");
    }
}
