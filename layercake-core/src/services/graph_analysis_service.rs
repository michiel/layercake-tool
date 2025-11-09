use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Result;
use serde::Serialize;

use crate::graph::Graph;
use crate::services::GraphService;
use sea_orm::DatabaseConnection;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphConnectivityReport {
    pub graph_id: i32,
    pub node_count: usize,
    pub edge_count: usize,
    pub components: Vec<Vec<String>>,
}

pub struct GraphAnalysisService {
    db: DatabaseConnection,
}

impl GraphAnalysisService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn analyze_connectivity(&self, graph_id: i32) -> Result<GraphConnectivityReport> {
        let graph_service = GraphService::new(self.db.clone());
        let graph = graph_service.build_graph_from_dag_graph(graph_id).await?;
        let adjacency = build_adjacency(&graph);
        let components = find_connected_components(&adjacency);

        Ok(GraphConnectivityReport {
            graph_id,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            components,
        })
    }

    pub async fn find_paths(
        &self,
        graph_id: i32,
        source: &str,
        target: &str,
        max_paths: usize,
    ) -> Result<Vec<Vec<String>>> {
        let graph_service = GraphService::new(self.db.clone());
        let graph = graph_service.build_graph_from_dag_graph(graph_id).await?;
        let adjacency = build_adjacency(&graph);

        Ok(find_all_paths(&adjacency, source, target, max_paths))
    }
}

fn build_adjacency(graph: &Graph) -> HashMap<String, Vec<String>> {
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    for node in &graph.nodes {
        adjacency.entry(node.id.clone()).or_default();
    }

    for edge in &graph.edges {
        adjacency
            .entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        adjacency
            .entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
    }

    adjacency
}

fn find_connected_components(adjacency: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut components: Vec<Vec<String>> = Vec::new();

    for node in adjacency.keys() {
        if !visited.contains(node) {
            let mut component = Vec::new();
            dfs_component(node, adjacency, &mut visited, &mut component);
            if !component.is_empty() {
                components.push(component);
            }
        }
    }

    components
}

fn dfs_component(
    node: &str,
    adjacency: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    component: &mut Vec<String>,
) {
    visited.insert(node.to_string());
    component.push(node.to_string());

    if let Some(neighbors) = adjacency.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_component(neighbor, adjacency, visited, component);
            }
        }
    }
}

fn find_all_paths(
    adjacency: &HashMap<String, Vec<String>>,
    source: &str,
    target: &str,
    max_paths: usize,
) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back(vec![source.to_string()]);

    while let Some(current_path) = queue.pop_front() {
        if paths.len() >= max_paths {
            break;
        }

        // Safe: paths are initialized with at least one element and only extended
        let current_node = current_path
            .last()
            .expect("Current path should not be empty");

        if current_node == target {
            paths.push(current_path);
            continue;
        }

        if current_path.len() > 10 {
            continue;
        }

        if let Some(neighbors) = adjacency.get(current_node) {
            for neighbor in neighbors {
                if !current_path.contains(neighbor) {
                    let mut new_path = current_path.clone();
                    new_path.push(neighbor.clone());
                    queue.push_back(new_path);
                }
            }
        }
    }

    paths
}
