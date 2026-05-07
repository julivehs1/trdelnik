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

    // ---------- Meta methods (signature, inputs, name, warmup, reset) ----------

    #[test]
    fn test_node_metadata_for_each_binary() {
        let cases: Vec<(&str, &str)> = vec![
            ("Add", "add:0:1"),
            ("Sub", "sub:0:1"),
            ("Mul", "mul:0:1"),
            ("Div", "div:0:1"),
            ("Max", "max:0:1"),
            ("Min", "min:0:1"),
        ];

        let nodes: Vec<Box<dyn Node>> = vec![
            Box::new(AddNode::new(NodeId(0), NodeId(1))),
            Box::new(SubNode::new(NodeId(0), NodeId(1))),
            Box::new(MulNode::new(NodeId(0), NodeId(1))),
            Box::new(DivNode::new(NodeId(0), NodeId(1))),
            Box::new(MaxNode::new(NodeId(0), NodeId(1))),
            Box::new(MinNode::new(NodeId(0), NodeId(1))),
        ];

        for (n, (expected_name, expected_sig)) in nodes.iter().zip(cases.iter()) {
            assert_eq!(n.name(), *expected_name);
            assert_eq!(n.signature().as_deref(), Some(*expected_sig));
            assert_eq!(n.inputs(), &[NodeId(0), NodeId(1)][..]);
            assert_eq!(n.warmup_period(), 0);
        }
    }

    #[test]
    fn test_unary_node_metadata() {
        let neg = NegNode::new(NodeId(7));
        assert_eq!(neg.name(), "Neg");
        assert_eq!(neg.signature().as_deref(), Some("neg:7"));
        assert_eq!(neg.inputs(), &[NodeId(7)][..]);
        assert_eq!(neg.warmup_period(), 0);

        let abs = AbsNode::new(NodeId(3));
        assert_eq!(abs.name(), "Abs");
        assert_eq!(abs.signature().as_deref(), Some("abs:3"));
        assert_eq!(abs.inputs(), &[NodeId(3)][..]);
        assert_eq!(abs.warmup_period(), 0);
    }

    #[test]
    fn test_reset_is_noop_for_pure_arithmetic() {
        // Arithmetic nodes hold no state; reset must not affect compute output.
        let ctx = create_ctx();
        let mut a = AddNode::new(NodeId(0), NodeId(1));
        let r1 = a.compute(&ctx, &[Value::number(1.0), Value::number(2.0)]);
        a.reset();
        let r2 = a.compute(&ctx, &[Value::number(1.0), Value::number(2.0)]);
        assert_eq!(r1.as_number(), r2.as_number());
    }

    #[test]
    fn test_clone_box_returns_independent_boxed_node() {
        let original: Box<dyn Node> = Box::new(AddNode::new(NodeId(2), NodeId(3)));
        let cloned: Box<dyn Node> = original.clone_box();
        assert_eq!(cloned.signature(), original.signature());
        assert_eq!(cloned.name(), "Add");
    }

    // ---------- Missing edge cases ----------

    #[test]
    fn test_neg_with_none_returns_none() {
        let mut n = NegNode::new(NodeId(0));
        let r = n.compute(&create_ctx(), &[Value::none_number()]);
        assert!(r.is_none());
    }

    #[test]
    fn test_abs_with_none_returns_none() {
        let mut n = AbsNode::new(NodeId(0));
        let r = n.compute(&create_ctx(), &[Value::none_number()]);
        assert!(r.is_none());
    }

    #[test]
    fn test_max_min_with_none_returns_none() {
        let ctx = create_ctx();
        let mut mx = MaxNode::new(NodeId(0), NodeId(1));
        assert!(mx.compute(&ctx, &[Value::none_number(), Value::number(1.0)]).is_none());
        let mut mn = MinNode::new(NodeId(0), NodeId(1));
        assert!(mn.compute(&ctx, &[Value::number(1.0), Value::none_number()]).is_none());
    }

    #[test]
    fn test_sub_mul_with_none_returns_none() {
        let ctx = create_ctx();
        let mut s = SubNode::new(NodeId(0), NodeId(1));
        assert!(s.compute(&ctx, &[Value::none_number(), Value::number(1.0)]).is_none());
        let mut m = MulNode::new(NodeId(0), NodeId(1));
        assert!(m.compute(&ctx, &[Value::number(1.0), Value::none_number()]).is_none());
    }

    #[test]
    fn test_div_with_none_returns_none() {
        let mut d = DivNode::new(NodeId(0), NodeId(1));
        assert!(d
            .compute(&create_ctx(), &[Value::none_number(), Value::number(1.0)])
            .is_none());
    }
}
