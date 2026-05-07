//! Data source nodes for accessing OHLCV bar data

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::Value;

/// Node that outputs the closing price of the current bar
#[derive(Debug, Clone, Default)]
pub struct CloseNode;

impl CloseNode {
    pub fn new() -> Self {
        Self
    }
}

impl Node for CloseNode {
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

    crate::impl_clone_box!(CloseNode);
}

/// Node that outputs the opening price of the current bar
#[derive(Debug, Clone, Default)]
pub struct OpenNode;

impl OpenNode {
    pub fn new() -> Self {
        Self
    }
}

impl Node for OpenNode {
    fn signature(&self) -> Option<String> {
        Some("open".to_string())
    }

    fn inputs(&self) -> &[NodeId] {
        &[]
    }

    fn reset(&mut self) {}

    fn compute(&mut self, ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
        Value::number(ctx.open)
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Open"
    }

    crate::impl_clone_box!(OpenNode);
}

/// Node that outputs the highest price of the current bar
#[derive(Debug, Clone, Default)]
pub struct HighNode;

impl HighNode {
    pub fn new() -> Self {
        Self
    }
}

impl Node for HighNode {
    fn signature(&self) -> Option<String> {
        Some("high".to_string())
    }

    fn inputs(&self) -> &[NodeId] {
        &[]
    }

    fn reset(&mut self) {}

    fn compute(&mut self, ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
        Value::number(ctx.high)
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "High"
    }

    crate::impl_clone_box!(HighNode);
}

/// Node that outputs the lowest price of the current bar
#[derive(Debug, Clone, Default)]
pub struct LowNode;

impl LowNode {
    pub fn new() -> Self {
        Self
    }
}

impl Node for LowNode {
    fn signature(&self) -> Option<String> {
        Some("low".to_string())
    }

    fn inputs(&self) -> &[NodeId] {
        &[]
    }

    fn reset(&mut self) {}

    fn compute(&mut self, ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
        Value::number(ctx.low)
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Low"
    }

    crate::impl_clone_box!(LowNode);
}

/// Node that outputs the volume of the current bar
#[derive(Debug, Clone, Default)]
pub struct VolumeNode;

impl VolumeNode {
    pub fn new() -> Self {
        Self
    }
}

impl Node for VolumeNode {
    fn signature(&self) -> Option<String> {
        Some("volume".to_string())
    }

    fn inputs(&self) -> &[NodeId] {
        &[]
    }

    fn reset(&mut self) {}

    fn compute(&mut self, ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
        Value::number(ctx.volume)
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Volume"
    }

    crate::impl_clone_box!(VolumeNode);
}

/// Node that outputs a constant numeric value
#[derive(Debug, Clone)]
pub struct ConstNode {
    value: f64,
}

impl ConstNode {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Node for ConstNode {
    fn signature(&self) -> Option<String> {
        Some(format!("const:{}", self.value))
    }

    fn inputs(&self) -> &[NodeId] {
        &[]
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
        Value::number(self.value)
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Const"
    }

    crate::impl_clone_box!(ConstNode);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_node() {
        let mut node = CloseNode::new();
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        let result = node.compute(&ctx, &[]);
        assert_eq!(result.as_number(), Some(105.0));
    }

    #[test]
    fn test_open_node() {
        let mut node = OpenNode::new();
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        let result = node.compute(&ctx, &[]);
        assert_eq!(result.as_number(), Some(100.0));
    }

    #[test]
    fn test_high_node() {
        let mut node = HighNode::new();
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        let result = node.compute(&ctx, &[]);
        assert_eq!(result.as_number(), Some(110.0));
    }

    #[test]
    fn test_low_node() {
        let mut node = LowNode::new();
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        let result = node.compute(&ctx, &[]);
        assert_eq!(result.as_number(), Some(95.0));
    }

    #[test]
    fn test_volume_node() {
        let mut node = VolumeNode::new();
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        let result = node.compute(&ctx, &[]);
        assert_eq!(result.as_number(), Some(1000.0));
    }

    #[test]
    fn test_const_node() {
        let mut node = ConstNode::new(42.5);
        let ctx = ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0);

        let result = node.compute(&ctx, &[]);
        assert_eq!(result.as_number(), Some(42.5));
    }

    fn ctx() -> ExecutionContext {
        ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0)
    }

    #[test]
    fn test_data_node_signatures() {
        assert_eq!(CloseNode::new().signature().as_deref(), Some("close"));
        assert_eq!(OpenNode::new().signature().as_deref(), Some("open"));
        assert_eq!(HighNode::new().signature().as_deref(), Some("high"));
        assert_eq!(LowNode::new().signature().as_deref(), Some("low"));
        assert_eq!(VolumeNode::new().signature().as_deref(), Some("volume"));
    }

    #[test]
    fn test_const_node_signature_includes_value() {
        let a = ConstNode::new(1.0);
        let b = ConstNode::new(2.0);
        assert_eq!(a.signature().as_deref(), Some("const:1"));
        assert_ne!(a.signature(), b.signature());
    }

    #[test]
    fn test_data_node_names() {
        assert_eq!(CloseNode::new().name(), "Close");
        assert_eq!(OpenNode::new().name(), "Open");
        assert_eq!(HighNode::new().name(), "High");
        assert_eq!(LowNode::new().name(), "Low");
        assert_eq!(VolumeNode::new().name(), "Volume");
        assert_eq!(ConstNode::new(0.0).name(), "Const");
    }

    #[test]
    fn test_data_nodes_have_no_inputs() {
        assert!(CloseNode::new().inputs().is_empty());
        assert!(OpenNode::new().inputs().is_empty());
        assert!(HighNode::new().inputs().is_empty());
        assert!(LowNode::new().inputs().is_empty());
        assert!(VolumeNode::new().inputs().is_empty());
        assert!(ConstNode::new(0.0).inputs().is_empty());
    }

    #[test]
    fn test_data_nodes_warmup_zero() {
        assert_eq!(CloseNode::new().warmup_period(), 0);
        assert_eq!(OpenNode::new().warmup_period(), 0);
        assert_eq!(HighNode::new().warmup_period(), 0);
        assert_eq!(LowNode::new().warmup_period(), 0);
        assert_eq!(VolumeNode::new().warmup_period(), 0);
        assert_eq!(ConstNode::new(0.0).warmup_period(), 0);
    }

    #[test]
    fn test_data_nodes_default_matches_new() {
        let a = CloseNode::default();
        let b = CloseNode::new();
        assert_eq!(a.signature(), b.signature());

        // Spot-check the others compile via Default
        let _ = OpenNode::default();
        let _ = HighNode::default();
        let _ = LowNode::default();
        let _ = VolumeNode::default();
    }

    #[test]
    fn test_data_nodes_reset_is_noop() {
        // None of the data nodes hold state — reset must not change compute output.
        let mut close = CloseNode::new();
        let r1 = close.compute(&ctx(), &[]);
        close.reset();
        let r2 = close.compute(&ctx(), &[]);
        assert_eq!(r1.as_number(), r2.as_number());
    }

    #[test]
    fn test_data_nodes_clone_box() {
        let nodes: Vec<Box<dyn Node>> = vec![
            Box::new(CloseNode::new()),
            Box::new(OpenNode::new()),
            Box::new(HighNode::new()),
            Box::new(LowNode::new()),
            Box::new(VolumeNode::new()),
            Box::new(ConstNode::new(7.0)),
        ];
        for n in nodes {
            let cloned = n.clone_box();
            assert_eq!(cloned.signature(), n.signature());
        }
    }
}
