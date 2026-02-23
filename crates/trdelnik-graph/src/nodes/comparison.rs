//! Comparison and signal nodes (Gt, Lt, Cross, CrossOver, CrossUnder)

use crate::context::ExecutionContext;
use crate::node::{Node, NodeId};
use crate::value::Value;

/// Greater than node: output = input_a > input_b
#[derive(Debug, Clone)]
pub struct GtNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl GtNode {
    /// Create a new greater than node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for GtNode {
    fn signature(&self) -> Option<String> {
        Some(format!("gt:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::bool(a > b),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Gt"
    }

    crate::impl_clone_box!(GtNode);
}

/// Less than node: output = input_a < input_b
#[derive(Debug, Clone)]
pub struct LtNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl LtNode {
    /// Create a new less than node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for LtNode {
    fn signature(&self) -> Option<String> {
        Some(format!("lt:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::bool(a < b),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Lt"
    }

    crate::impl_clone_box!(LtNode);
}

/// Greater than or equal node: output = input_a >= input_b
#[derive(Debug, Clone)]
pub struct GteNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl GteNode {
    /// Create a new greater than or equal node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for GteNode {
    fn signature(&self) -> Option<String> {
        Some(format!("gte:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::bool(a >= b),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Gte"
    }

    crate::impl_clone_box!(GteNode);
}

/// Less than or equal node: output = input_a <= input_b
#[derive(Debug, Clone)]
pub struct LteNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl LteNode {
    /// Create a new less than or equal node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for LteNode {
    fn signature(&self) -> Option<String> {
        Some(format!("lte:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::bool(a <= b),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Lte"
    }

    crate::impl_clone_box!(LteNode);
}

/// Equal node: output = input_a == input_b (with epsilon comparison for floats)
#[derive(Debug, Clone)]
pub struct EqNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
    epsilon: f64,
}

impl EqNode {
    /// Create a new equal node with default epsilon (1e-10)
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self::with_epsilon(input_a, input_b, 1e-10)
    }

    /// Create with custom epsilon for float comparison
    pub fn with_epsilon(input_a: NodeId, input_b: NodeId, epsilon: f64) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
            epsilon,
        }
    }
}

impl Node for EqNode {
    fn signature(&self) -> Option<String> {
        Some(format!("eq:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => Value::bool((a - b).abs() < self.epsilon),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Eq"
    }

    crate::impl_clone_box!(EqNode);
}

/// CrossOver node: output = true when input_a crosses above input_b
///
/// A crossover occurs when:
/// - Previous bar: input_a <= input_b
/// - Current bar: input_a > input_b
#[derive(Debug, Clone)]
pub struct CrossOverNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
    prev_a: Option<f64>,
    prev_b: Option<f64>,
}

impl CrossOverNode {
    /// Create a new crossover node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
            prev_a: None,
            prev_b: None,
        }
    }
}

impl Node for CrossOverNode {
    fn signature(&self) -> Option<String> {
        Some(format!("crossover:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {
        self.prev_a = None;
        self.prev_b = None;
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        let (curr_a, curr_b) = match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                self.prev_a = None;
                self.prev_b = None;
                return Value::none_bool();
            }
        };

        let crossed = match (self.prev_a, self.prev_b) {
            (Some(prev_a), Some(prev_b)) => {
                prev_a <= prev_b && curr_a > curr_b
            }
            _ => false,
        };

        self.prev_a = Some(curr_a);
        self.prev_b = Some(curr_b);

        Value::bool(crossed)
    }

    fn warmup_period(&self) -> usize {
        1 // Need at least one previous bar
    }

    fn name(&self) -> &str {
        "CrossOver"
    }

    crate::impl_clone_box!(CrossOverNode);
}

/// CrossUnder node: output = true when input_a crosses below input_b
///
/// A crossunder occurs when:
/// - Previous bar: input_a >= input_b
/// - Current bar: input_a < input_b
#[derive(Debug, Clone)]
pub struct CrossUnderNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
    prev_a: Option<f64>,
    prev_b: Option<f64>,
}

impl CrossUnderNode {
    /// Create a new crossunder node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
            prev_a: None,
            prev_b: None,
        }
    }
}

impl Node for CrossUnderNode {
    fn signature(&self) -> Option<String> {
        Some(format!("crossunder:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {
        self.prev_a = None;
        self.prev_b = None;
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        let (curr_a, curr_b) = match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                self.prev_a = None;
                self.prev_b = None;
                return Value::none_bool();
            }
        };

        let crossed = match (self.prev_a, self.prev_b) {
            (Some(prev_a), Some(prev_b)) => {
                prev_a >= prev_b && curr_a < curr_b
            }
            _ => false,
        };

        self.prev_a = Some(curr_a);
        self.prev_b = Some(curr_b);

        Value::bool(crossed)
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "CrossUnder"
    }

    crate::impl_clone_box!(CrossUnderNode);
}

/// Cross node: output = true when input_a crosses input_b (either direction)
#[derive(Debug, Clone)]
pub struct CrossNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
    prev_a: Option<f64>,
    prev_b: Option<f64>,
}

impl CrossNode {
    /// Create a new cross node
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
            prev_a: None,
            prev_b: None,
        }
    }
}

impl Node for CrossNode {
    fn signature(&self) -> Option<String> {
        Some(format!("cross:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {
        self.prev_a = None;
        self.prev_b = None;
    }

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        let (curr_a, curr_b) = match (inputs[0].as_number(), inputs[1].as_number()) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                self.prev_a = None;
                self.prev_b = None;
                return Value::none_bool();
            }
        };

        let crossed = match (self.prev_a, self.prev_b) {
            (Some(prev_a), Some(prev_b)) => {
                // Cross over or cross under
                (prev_a <= prev_b && curr_a > curr_b) ||
                (prev_a >= prev_b && curr_a < curr_b)
            }
            _ => false,
        };

        self.prev_a = Some(curr_a);
        self.prev_b = Some(curr_b);

        Value::bool(crossed)
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "Cross"
    }

    crate::impl_clone_box!(CrossNode);
}

/// Logical AND node: output = input_a && input_b
#[derive(Debug, Clone)]
pub struct AndNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl AndNode {
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for AndNode {
    fn signature(&self) -> Option<String> {
        Some(format!("and:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_bool(), inputs[1].as_bool()) {
            (Some(a), Some(b)) => Value::bool(a && b),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "And"
    }

    crate::impl_clone_box!(AndNode);
}

/// Logical OR node: output = input_a || input_b
#[derive(Debug, Clone)]
pub struct OrNode {
    input_a: NodeId,
    input_b: NodeId,
    inputs: [NodeId; 2],
}

impl OrNode {
    pub fn new(input_a: NodeId, input_b: NodeId) -> Self {
        Self {
            input_a,
            input_b,
            inputs: [input_a, input_b],
        }
    }
}

impl Node for OrNode {
    fn signature(&self) -> Option<String> {
        Some(format!("or:{}:{}", self.input_a.0, self.input_b.0))
    }

    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match (inputs[0].as_bool(), inputs[1].as_bool()) {
            (Some(a), Some(b)) => Value::bool(a || b),
            _ => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Or"
    }

    crate::impl_clone_box!(OrNode);
}

/// Logical NOT node: output = !input
#[derive(Debug, Clone)]
pub struct NotNode {
    input: NodeId,
}

impl NotNode {
    pub fn new(input: NodeId) -> Self {
        Self { input }
    }
}

impl Node for NotNode {
    fn signature(&self) -> Option<String> {
        Some(format!("not:{}", self.input.0))
    }

    fn inputs(&self) -> &[NodeId] {
        std::slice::from_ref(&self.input)
    }

    fn reset(&mut self) {}

    fn compute(&mut self, _ctx: &ExecutionContext, inputs: &[Value]) -> Value {
        match inputs[0].as_bool() {
            Some(a) => Value::bool(!a),
            None => Value::none_bool(),
        }
    }

    fn warmup_period(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "Not"
    }

    crate::impl_clone_box!(NotNode);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_ctx() -> ExecutionContext {
        ExecutionContext::new(0, 100.0, 110.0, 95.0, 105.0, 1000.0, 0.0)
    }

    #[test]
    fn test_gt_node() {
        let mut node = GtNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::number(5.0)]);
        assert_eq!(result.as_bool(), Some(true));

        let result = node.compute(&ctx, &[Value::number(5.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_lt_node() {
        let mut node = LtNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(5.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(true));

        let result = node.compute(&ctx, &[Value::number(10.0), Value::number(5.0)]);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_crossover_node() {
        let mut node = CrossOverNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        // First bar - no cross possible
        let result = node.compute(&ctx, &[Value::number(5.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(false));

        // Second bar - a crosses above b
        let result = node.compute(&ctx, &[Value::number(15.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(true));

        // Third bar - still above, no new cross
        let result = node.compute(&ctx, &[Value::number(20.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_crossunder_node() {
        let mut node = CrossUnderNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        // First bar - no cross possible
        let result = node.compute(&ctx, &[Value::number(15.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(false));

        // Second bar - a crosses below b
        let result = node.compute(&ctx, &[Value::number(5.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(true));

        // Third bar - still below, no new cross
        let result = node.compute(&ctx, &[Value::number(3.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_cross_node() {
        let mut node = CrossNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        // Setup: a below b
        node.compute(&ctx, &[Value::number(5.0), Value::number(10.0)]);

        // Cross over
        let result = node.compute(&ctx, &[Value::number(15.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(true));

        // No cross
        let result = node.compute(&ctx, &[Value::number(20.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(false));

        // Cross under
        let result = node.compute(&ctx, &[Value::number(5.0), Value::number(10.0)]);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_and_node() {
        let mut node = AndNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::bool(true), Value::bool(true)]);
        assert_eq!(result.as_bool(), Some(true));

        let result = node.compute(&ctx, &[Value::bool(true), Value::bool(false)]);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_or_node() {
        let mut node = OrNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::bool(false), Value::bool(false)]);
        assert_eq!(result.as_bool(), Some(false));

        let result = node.compute(&ctx, &[Value::bool(true), Value::bool(false)]);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_not_node() {
        let mut node = NotNode::new(NodeId(0));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::bool(true)]);
        assert_eq!(result.as_bool(), Some(false));

        let result = node.compute(&ctx, &[Value::bool(false)]);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_comparison_with_none() {
        let mut node = GtNode::new(NodeId(0), NodeId(1));
        let ctx = create_ctx();

        let result = node.compute(&ctx, &[Value::number(10.0), Value::none_number()]);
        assert!(result.is_none());
    }
}
