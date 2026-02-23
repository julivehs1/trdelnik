//! Arithmetic operation nodes (Add, Sub, Mul, Div)

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::Value;

/// Addition node: output = input_a + input_b
#[derive(Debug, Clone)]
pub struct AddNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl AddNode {
    /// Create a new addition node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for AddNode {
    fn signature(&self) -> Option<String> {
        Some(format!("add:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::number(a + b),
            _ => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Add"
    }

    crate::impl_clone_box!(AddNode);
}

/// Subtraction node: output = input_a - input_b
#[derive(Debug, Clone)]
pub struct SubNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl SubNode {
    /// Create a new subtraction node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for SubNode {
    fn signature(&self) -> Option<String> {
        Some(format!("sub:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::number(a - b),
            _ => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Sub"
    }

    crate::impl_clone_box!(SubNode);
}

/// Multiplication node: output = input_a * input_b
#[derive(Debug, Clone)]
pub struct MulNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl MulNode {
    /// Create a new multiplication node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for MulNode {
    fn signature(&self) -> Option<String> {
        Some(format!("mul:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::number(a * b),
            _ => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Mul"
    }

    crate::impl_clone_box!(MulNode);
}

/// Division node: output = input_a / input_b
#[derive(Debug, Clone)]
pub struct DivNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl DivNode {
    /// Create a new division node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for DivNode {
    fn signature(&self) -> Option<String> {
        Some(format!("div:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => {
                if b == 0.0 {
                    Value::none_number() // Avoid division by zero
                } else {
                    Value::number(a / b)
                }
            }
            _ => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Div"
    }

    crate::impl_clone_box!(DivNode);
}

/// Negation node: output = -input
#[derive(Debug, Clone)]
pub struct NegNode {
    input: NodeId,
}

impl NegNode {
    /// Create a new negation node
    pub fn new(input: NodeId) -> Self {
        Self { input }
    }
}

impl Node for NegNode {
    fn signature(&self) -> Option<String> {
        Some(format!("neg:{}", self.input.0))
    }

    fn inputs(&self) -> &[NodeId] {
        std::slice::from_ref(&self.input)
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match inputs[0].as_number() {
            Some(a) => Value::number(-a),
            None => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Neg"
    }

    crate::impl_clone_box!(NegNode);
}

/// Absolute value node: output = |input|
#[derive(Debug, Clone)]
pub struct AbsNode {
    input: NodeId,
}

impl AbsNode {
    /// Create a new absolute value node
    pub fn new(input: NodeId) -> Self {
        Self { input }
    }
}

impl Node for AbsNode {
    fn signature(&self) -> Option<String> {
        Some(format!("abs:{}", self.input.0))
    }

    fn inputs(&self) -> &[NodeId] {
        std::slice::from_ref(&self.input)
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match inputs[0].as_number() {
            Some(a) => Value::number(a.abs()),
            None => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Abs"
    }

    crate::impl_clone_box!(AbsNode);
}

/// Maximum of two inputs: output = max(input_a, input_b)
#[derive(Debug, Clone)]
pub struct MaxNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl MaxNode {
    /// Create a new max node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for MaxNode {
    fn signature(&self) -> Option<String> {
        Some(format!("max:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::number(a.max(b)),
            _ => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Max"
    }

    crate::impl_clone_box!(MaxNode);
}

/// Minimum of two inputs: output = min(input_a, input_b)
#[derive(Debug, Clone)]
pub struct MinNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl MinNode {
    /// Create a new min node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for MinNode {
    fn signature(&self) -> Option<String> {
        Some(format!("min:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::number(a.min(b)),
            _ => Value::none_number(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Min"
    }

    crate::impl_clone_box!(MinNode);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_ctx() -> ExecutionContext {
        ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0)
    }

    #[test]
    fn test_add_node() {
        let mut node = AddNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::number(5.0)]);
        assert_eq!(result.as_number(), Some(15.0));
    }

    #[test]
    fn test_sub_node() {
        let mut node = SubNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::number(3.0)]);
        assert_eq!(result.as_number(), Some(7.0));
    }

    #[test]
    fn test_mul_node() {
        let mut node = MulNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(4.0), Value::number(5.0)]);
        assert_eq!(result.as_number(), Some(20.0));
    }

    #[test]
    fn test_div_node() {
        let mut node = DivNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(20.0), Value::number(4.0)]);
        assert_eq!(result.as_number(), Some(5.0));
    }

    #[test]
    fn test_div_by_zero() {
        let mut node = DivNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(20.0), Value::number(0.0)]);
        assert!(result.is_none());
    }

    #[test]
    fn test_neg_node() {
        let mut node = NegNode::new(NodeId(0));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(5.0)]);
        assert_eq!(result.as_number(), Some(-5.0));

        let result = node.compute(&ctx, &[Value::number(-3.0)]);
        assert_eq!(result.as_number(), Some(3.0));
    }

    #[test]
    fn test_abs_node() {
        let mut node = AbsNode::new(NodeId(0));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(-5.0)]);
        assert_eq!(result.as_number(), Some(5.0));

        let result = node.compute(&ctx, &[Value::number(3.0)]);
        assert_eq!(result.as_number(), Some(3.0));
    }

    #[test]
    fn test_max_node() {
        let mut node = MaxNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::number(5.0)]);
        assert_eq!(result.as_number(), Some(10.0));

        let result = node.compute(&ctx, &[Value::number(3.0), Value::number(7.0)]);
        assert_eq!(result.as_number(), Some(7.0));
    }

    #[test]
    fn test_min_node() {
        let mut node = MinNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::number(5.0)]);
        assert_eq!(result.as_number(), Some(5.0));

        let result = node.compute(&ctx, &[Value::number(3.0), Value::number(7.0)]);
        assert_eq!(result.as_number(), Some(3.0));
    }

    #[test]
    fn test_arithmetic_with_none() {
        let mut node = AddNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::none_number()]);
        assert!(result.is_none());
    }
}
