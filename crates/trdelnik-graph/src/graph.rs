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
}
