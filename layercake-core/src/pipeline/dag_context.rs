use std::collections::HashMap;

use crate::graph::Graph;

/// Holds in-memory graph materializations during DAG execution to avoid
/// repeatedly hydrating the same nodes from the database.
#[derive(Default)]
pub struct DagExecutionContext {
    graph_cache: HashMap<String, Graph>,
    dataset_cache: HashMap<i32, Graph>,
}

impl DagExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve a cached graph for the given DAG node.
    pub fn graph(&self, node_id: &str) -> Option<Graph> {
        self.graph_cache.get(node_id).cloned()
    }

    /// Cache the latest graph for a DAG node.
    pub fn set_graph(&mut self, node_id: impl Into<String>, graph: Graph) {
        self.graph_cache.insert(node_id.into(), graph);
    }

    /// Retrieve a cached dataset graph representation.
    pub fn dataset_graph(&self, data_set_id: i32) -> Option<Graph> {
        self.dataset_cache.get(&data_set_id).cloned()
    }

    /// Cache a dataset graph representation for reuse.
    pub fn set_dataset_graph(&mut self, data_set_id: i32, graph: Graph) {
        self.dataset_cache.insert(data_set_id, graph);
    }
}
