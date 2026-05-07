//! Computation graph with CSE (Common Subexpression Elimination) and topological sorting

use crate::node::{BoxedNode, NodeId};
use std::collections::HashMap;

/// A computation graph for indicator calculations.
///
/// The graph manages nodes and their dependencies, performs CSE
/// (Common Subexpression Elimination) to avoid redundant calculations,
/// and determines the execution order via topological sorting.
#[derive(Default, Clone)]
pub struct Graph {
    /// All nodes in the graph
    nodes: Vec<BoxedNode>,
    /// Execution order (topologically sorted node indices)
    execution_order: Vec<NodeId>,
    /// Signature map for CSE: signature -> node_id
    signature_map: HashMap<String, NodeId>,
    /// Whether the graph has been finalized
    finalized: bool,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            execution_order: Vec::new(),
            signature_map: HashMap::new(),
            finalized: false,
        }
    }

    /// Add a node to the graph.
    ///
    /// If a node with the same signature already exists (CSE),
    /// returns the existing node's ID instead of adding a duplicate.
    ///
    /// # Panics
    /// Panics if the graph has already been finalized.
    pub fn add_node(&mut self, node: BoxedNode) -> NodeId {
        if self.finalized {
            panic!("Cannot add nodes to a finalized graph");
        }

        // Check for CSE opportunity
        if let Some(signature) = node.signature() {
            if let Some(&existing_id) = self.signature_map.get(&signature) {
                return existing_id;
            }
            // New unique node
            let id = NodeId(self.nodes.len());
            self.signature_map.insert(signature, id);
            self.nodes.push(node);
            id
        } else {
            // Node doesn't participate in CSE
            let id = NodeId(self.nodes.len());
            self.nodes.push(node);
            id
        }
    }

    /// Finalize the graph by computing the topological execution order.
    ///
    /// After finalization, no more nodes can be added.
    ///
    /// # Panics
    /// Panics if the graph contains a cycle.
    pub fn finalize(&mut self) {
        if self.finalized {
            return;
        }

        self.execution_order = self.topological_sort();
        self.finalized = true;
    }

    /// Perform topological sort using Kahn's algorithm.
    fn topological_sort(&self) -> Vec<NodeId> {
        let n = self.nodes.len();
        if n == 0 {
            return Vec::new();
        }

        // Calculate in-degree for each node
        let mut in_degree = vec![0usize; n];
        for node in &self.nodes {
            for &input_id in node.inputs() {
                in_degree[input_id.0] += 0; // Inputs add to their targets
            }
        }

        // Actually, we need to track who depends on whom
        // in_degree[i] = number of nodes that node i depends on (not the other way)
        // For execution order, we need: a node can execute when all its inputs are ready

        // Rebuild: in_degree[i] = how many unprocessed inputs does node i have?
        let mut in_degree = vec![0usize; n];
        let mut dependents: Vec<Vec<NodeId>> = vec![Vec::new(); n];

        for (i, node) in self.nodes.iter().enumerate() {
            in_degree[i] = node.inputs().len();
            for &input_id in node.inputs() {
                dependents[input_id.0].push(NodeId(i));
            }
        }

        // Start with nodes that have no inputs
        let mut queue: Vec<NodeId> = (0..n)
            .filter(|&i| in_degree[i] == 0)
            .map(NodeId)
            .collect();

        let mut result = Vec::with_capacity(n);

        while let Some(node_id) = queue.pop() {
            result.push(node_id);

            // Reduce in-degree for all dependents
            for &dependent_id in &dependents[node_id.0] {
                in_degree[dependent_id.0] -= 1;
                if in_degree[dependent_id.0] == 0 {
                    queue.push(dependent_id);
                }
            }
        }

        if result.len() != n {
            panic!(
                "Graph contains a cycle! Processed {} of {} nodes",
                result.len(),
                n
            );
        }

        result
    }

    /// Get the execution order (topologically sorted).
    ///
    /// # Panics
    /// Panics if the graph has not been finalized.
    pub fn execution_order(&self) -> &[NodeId] {
        if !self.finalized {
            panic!("Graph must be finalized before getting execution order");
        }
        &self.execution_order
    }

    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<&BoxedNode> {
        self.nodes.get(id.0)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut BoxedNode> {
        self.nodes.get_mut(id.0)
    }

    /// Get the number of nodes in the graph
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Check if the graph has been finalized
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Reset all nodes in the graph
    pub fn reset_all(&mut self) {
        for node in &mut self.nodes {
            node.reset();
        }
    }

    /// Calculate the maximum warmup period across all nodes
    pub fn max_warmup_period(&self) -> usize {
        self.nodes.iter().map(|n| n.warmup_period()).max().unwrap_or(0)
    }

    /// Iterate over nodes in execution order
    pub fn iter_execution_order(&self) -> impl Iterator<Item = (NodeId, &BoxedNode)> {
        if !self.finalized {
            panic!("Graph must be finalized before iterating in execution order");
        }
        self.execution_order
            .iter()
            .map(move |&id| (id, &self.nodes[id.0]))
    }

    /// Get mutable access to nodes for execution
    pub fn nodes_mut(&mut self) -> &mut [BoxedNode] {
        &mut self.nodes
    }

    /// Export the graph as a Graphviz DOT string.
    pub fn to_dot(&self) -> String {
        use std::fmt::Write;
        let mut dot = String::new();
        writeln!(dot, "digraph G {{").unwrap();
        writeln!(dot, "    rankdir=LR;").unwrap();
        writeln!(
            dot,
            "    node [shape=box, style=filled, fontname=\"monospace\"];"
        )
        .unwrap();

        // Nodes
        for (i, node) in self.nodes.iter().enumerate() {
            let name = node.name();
            let color = node_color(name);
            let label = format!("{}\\n(#{})", name, i);
            writeln!(dot, "    n{} [label=\"{}\", fillcolor=\"{}\"];", i, label, color).unwrap();
        }

        // Edges
        for (i, node) in self.nodes.iter().enumerate() {
            let inputs = node.inputs();
            for (port, input_id) in inputs.iter().enumerate() {
                if inputs.len() > 1 {
                    writeln!(dot, "    n{} -> n{} [label=\"{}\"];", input_id.0, i, port).unwrap();
                } else {
                    writeln!(dot, "    n{} -> n{};", input_id.0, i).unwrap();
                }
            }
        }

        writeln!(dot, "}}").unwrap();
        dot
    }
}

fn node_color(name: &str) -> &'static str {
    match name {
        "close" | "open" | "high" | "low" | "volume" | "const" => "#a8d5a2",
        "add" | "sub" | "mul" | "div" | "neg" | "abs" | "max" | "min" => "#a2c4d5",
        "gt" | "lt" | "gte" | "lte" | "eq" | "and" | "or" | "not" | "crossover"
        | "crossunder" | "cross" => "#d5c4a2",
        _ => "#c4a2d5",
    }
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Graph")
            .field("node_count", &self.nodes.len())
            .field("finalized", &self.finalized)
            .field("execution_order", &self.execution_order)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ExecutionContext;
    use crate::value::Value;

    // Simple test node for testing
    #[derive(Clone)]
    struct TestNode {
        name: String,
        inputs: Vec<NodeId>,
        value: f64,
    }

    impl TestNode {
        fn new(name: &str, inputs: Vec<NodeId>, value: f64) -> Self {
            Self {
                name: name.to_string(),
                inputs,
                value,
            }
        }
    }

    impl crate::node::Node for TestNode {
        fn signature(&self) -> Option<String> {
            let input_str: String = self.inputs.iter().map(|i| i.0.to_string()).collect::<Vec<_>>().join(",");
            Some(format!("test:{}:{}", input_str, self.value))
        }

        fn inputs(&self) -> &[NodeId] {
            &self.inputs
        }

        fn reset(&mut self) {}

        fn compute(&mut self, _ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
            Value::number(self.value)
        }

        fn warmup_period(&self) -> usize {
            0
        }

        fn name(&self) -> &str {
            &self.name
        }

        crate::impl_clone_box!(TestNode);
    }

    #[test]
    fn test_graph_add_node() {
        let mut graph = Graph::new();
        let id1 = graph.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let id2 = graph.add_node(Box::new(TestNode::new("b", vec![], 2.0)));

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn test_graph_cse() {
        let mut graph = Graph::new();

        // Add the same node twice - should return the same ID
        let id1 = graph.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let id2 = graph.add_node(Box::new(TestNode::new("a_duplicate", vec![], 1.0)));

        assert_eq!(id1, id2);
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn test_graph_topological_sort() {
        let mut graph = Graph::new();

        // Create a simple DAG: a -> b -> c
        let a = graph.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let b = graph.add_node(Box::new(TestNode::new("b", vec![a], 2.0)));
        let c = graph.add_node(Box::new(TestNode::new("c", vec![b], 3.0)));

        graph.finalize();

        let order = graph.execution_order();
        assert_eq!(order.len(), 3);

        // a must come before b, b must come before c
        let pos_a = order.iter().position(|&id| id == a).unwrap();
        let pos_b = order.iter().position(|&id| id == b).unwrap();
        let pos_c = order.iter().position(|&id| id == c).unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_graph_diamond() {
        let mut graph = Graph::new();

        // Diamond pattern: a -> b -> d
        //                  a -> c -> d
        let a = graph.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let b = graph.add_node(Box::new(TestNode::new("b", vec![a], 2.0)));
        let c = graph.add_node(Box::new(TestNode::new("c", vec![a], 3.0)));
        let d = graph.add_node(Box::new(TestNode::new("d", vec![b, c], 4.0)));

        graph.finalize();

        let order = graph.execution_order();
        assert_eq!(order.len(), 4);

        // a must come before b and c
        // b and c must come before d
        let pos_a = order.iter().position(|&id| id == a).unwrap();
        let pos_b = order.iter().position(|&id| id == b).unwrap();
        let pos_c = order.iter().position(|&id| id == c).unwrap();
        let pos_d = order.iter().position(|&id| id == d).unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    /// Node with a configurable warmup period for max_warmup_period tests.
    #[derive(Clone)]
    struct WarmupNode {
        warmup: usize,
    }

    impl crate::node::Node for WarmupNode {
        fn signature(&self) -> Option<String> {
            // Use a unique signature so CSE doesn't collapse them.
            Some(format!("warmup:{}", self.warmup))
        }
        fn inputs(&self) -> &[NodeId] {
            &[]
        }
        fn reset(&mut self) {}
        fn compute(&mut self, _ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
            Value::number(self.warmup as f64)
        }
        fn warmup_period(&self) -> usize {
            self.warmup
        }
        fn name(&self) -> &str {
            "warmup"
        }
        crate::impl_clone_box!(WarmupNode);
    }

    /// Node that records reset() calls in a shared counter (via a static atomic).
    #[derive(Clone)]
    struct ResetCountingNode {
        marker: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl crate::node::Node for ResetCountingNode {
        fn signature(&self) -> Option<String> {
            None
        }
        fn inputs(&self) -> &[NodeId] {
            &[]
        }
        fn reset(&mut self) {
            self.marker
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        fn compute(&mut self, _ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
            Value::number(0.0)
        }
        fn warmup_period(&self) -> usize {
            0
        }
        fn name(&self) -> &str {
            "reset_counter"
        }
        crate::impl_clone_box!(ResetCountingNode);
    }

    // ---------- Empty / basic state ----------

    #[test]
    fn test_default_graph_is_empty() {
        let g = Graph::default();
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
        assert!(!g.is_finalized());
    }

    #[test]
    fn test_finalize_empty_graph_succeeds() {
        let mut g = Graph::new();
        g.finalize();
        assert!(g.is_finalized());
        assert!(g.execution_order().is_empty());
    }

    #[test]
    fn test_finalize_idempotent() {
        let mut g = Graph::new();
        g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        g.finalize();
        let order_first = g.execution_order().to_vec();
        g.finalize(); // Second call must be a no-op
        assert_eq!(g.execution_order(), order_first.as_slice());
    }

    #[test]
    fn test_get_returns_some_for_valid_id() {
        let mut g = Graph::new();
        let id = g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        assert!(g.get(id).is_some());
        assert_eq!(g.get(id).unwrap().name(), "a");
    }

    #[test]
    fn test_get_returns_none_for_out_of_range() {
        let g = Graph::new();
        assert!(g.get(NodeId(99)).is_none());
    }

    #[test]
    fn test_get_mut_returns_some_for_valid_id() {
        let mut g = Graph::new();
        let id = g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        assert!(g.get_mut(id).is_some());
    }

    #[test]
    fn test_max_warmup_period_finds_largest() {
        let mut g = Graph::new();
        g.add_node(Box::new(WarmupNode { warmup: 5 }));
        g.add_node(Box::new(WarmupNode { warmup: 14 }));
        g.add_node(Box::new(WarmupNode { warmup: 3 }));
        assert_eq!(g.max_warmup_period(), 14);
    }

    #[test]
    fn test_max_warmup_period_empty_is_zero() {
        let g = Graph::new();
        assert_eq!(g.max_warmup_period(), 0);
    }

    #[test]
    fn test_reset_all_resets_every_node() {
        use std::sync::atomic::Ordering;
        use std::sync::Arc;

        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut g = Graph::new();
        // Two nodes that share the same Arc so we can confirm both got reset.
        g.add_node(Box::new(ResetCountingNode {
            marker: Arc::clone(&counter),
        }));
        g.add_node(Box::new(ResetCountingNode {
            marker: Arc::clone(&counter),
        }));
        g.reset_all();
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_nodes_mut_exposes_mutable_slice() {
        let mut g = Graph::new();
        g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let slice = g.nodes_mut();
        assert_eq!(slice.len(), 1);
    }

    // ---------- Iteration ----------

    #[test]
    fn test_iter_execution_order_yields_each_node_once() {
        let mut g = Graph::new();
        let a = g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let b = g.add_node(Box::new(TestNode::new("b", vec![a], 2.0)));
        let c = g.add_node(Box::new(TestNode::new("c", vec![b], 3.0)));
        g.finalize();
        let collected: Vec<NodeId> = g.iter_execution_order().map(|(id, _)| id).collect();
        assert_eq!(collected.len(), 3);
        assert!(collected.contains(&a));
        assert!(collected.contains(&b));
        assert!(collected.contains(&c));
    }

    // ---------- Panic paths ----------

    #[test]
    #[should_panic(expected = "Cannot add nodes to a finalized graph")]
    fn test_add_node_after_finalize_panics() {
        let mut g = Graph::new();
        g.finalize();
        g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
    }

    #[test]
    #[should_panic(expected = "Graph must be finalized")]
    fn test_execution_order_before_finalize_panics() {
        let g = Graph::new();
        let _ = g.execution_order();
    }

    #[test]
    #[should_panic(expected = "Graph must be finalized")]
    fn test_iter_execution_order_before_finalize_panics() {
        let g = Graph::new();
        let _ = g.iter_execution_order();
    }

    #[test]
    #[should_panic(expected = "cycle")]
    fn test_topological_sort_detects_cycle() {
        // Create a cycle by manipulating a node's inputs after it was added.
        let mut g = Graph::new();
        let a = g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let _b = g.add_node(Box::new(TestNode::new("b", vec![a], 2.0)));
        // Now mutate `a` so it depends on `b` — that closes the loop.
        let any_node = g.get_mut(a).unwrap();
        // Replace the node's inputs by replacing the node itself.
        *any_node = Box::new(TestNode::new("a", vec![NodeId(1)], 1.0));
        g.finalize();
    }

    // ---------- Debug + DOT ----------

    #[test]
    fn test_debug_impl_lists_node_count() {
        let mut g = Graph::new();
        g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let s = format!("{:?}", g);
        assert!(s.contains("Graph"));
        assert!(s.contains("node_count"));
    }

    #[test]
    fn test_to_dot_includes_header_and_nodes() {
        let mut g = Graph::new();
        let a = g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let _b = g.add_node(Box::new(TestNode::new("b", vec![a], 2.0)));
        let dot = g.to_dot();
        assert!(dot.starts_with("digraph G {"));
        assert!(dot.contains("rankdir=LR"));
        assert!(dot.contains("n0"));
        assert!(dot.contains("n1"));
        // Single-input edge format: no port label
        assert!(dot.contains("n0 -> n1;") || dot.contains("n0 -> n1 ["));
        assert!(dot.trim_end().ends_with('}'));
    }

    #[test]
    fn test_to_dot_multi_input_edge_uses_port_label() {
        let mut g = Graph::new();
        let a = g.add_node(Box::new(TestNode::new("a", vec![], 1.0)));
        let b = g.add_node(Box::new(TestNode::new("b", vec![], 2.0)));
        let _c = g.add_node(Box::new(TestNode::new("c", vec![a, b], 3.0)));
        let dot = g.to_dot();
        // Multi-input nodes get port labels
        assert!(dot.contains(r#"label="0""#));
        assert!(dot.contains(r#"label="1""#));
    }

    // ---------- node_color via to_dot ----------

    #[test]
    fn test_node_color_categories_via_dot() {
        // Use TestNode names that match each color category. Different `value`
        // per node so CSE doesn't collapse them (TestNode::signature uses it).
        let mut g = Graph::new();
        g.add_node(Box::new(TestNode::new("close", vec![], 1.0)));
        g.add_node(Box::new(TestNode::new("add", vec![], 2.0)));
        g.add_node(Box::new(TestNode::new("gt", vec![], 3.0)));
        g.add_node(Box::new(TestNode::new("custom_indicator", vec![], 4.0)));
        let dot = g.to_dot();
        // Category 1 (data nodes) → green
        assert!(dot.contains("#a8d5a2"));
        // Category 2 (arithmetic) → blue
        assert!(dot.contains("#a2c4d5"));
        // Category 3 (logical/comparison) → tan
        assert!(dot.contains("#d5c4a2"));
        // Default → purple
        assert!(dot.contains("#c4a2d5"));
    }

    // ---------- Non-CSE node retains its own ID ----------

    /// Node whose `signature()` returns None — does not participate in CSE.
    #[derive(Clone)]
    struct NoSigNode {
        i: u32,
    }
    impl crate::node::Node for NoSigNode {
        fn signature(&self) -> Option<String> {
            None
        }
        fn inputs(&self) -> &[NodeId] {
            &[]
        }
        fn reset(&mut self) {}
        fn compute(&mut self, _ctx: &ExecutionContext, _inputs: &[Value]) -> Value {
            Value::number(self.i as f64)
        }
        fn warmup_period(&self) -> usize {
            0
        }
        fn name(&self) -> &str {
            "nosig"
        }
        crate::impl_clone_box!(NoSigNode);
    }

    #[test]
    fn test_no_signature_node_skips_cse_path() {
        let mut g = Graph::new();
        let id1 = g.add_node(Box::new(NoSigNode { i: 1 }));
        let id2 = g.add_node(Box::new(NoSigNode { i: 2 }));
        // Without a signature, CSE never collapses them.
        assert_ne!(id1, id2);
        assert_eq!(g.len(), 2);
    }
}
