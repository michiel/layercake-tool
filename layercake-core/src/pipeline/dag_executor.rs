use anyhow::{anyhow, Result};
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::database::entities::plan_dag_nodes;
use crate::pipeline::{DatasourceImporter, GraphBuilder, MergeBuilder};

/// DAG executor that processes nodes in topological order
pub struct DagExecutor {
    db: DatabaseConnection,
    datasource_importer: DatasourceImporter,
    graph_builder: GraphBuilder,
    merge_builder: MergeBuilder,
}

impl DagExecutor {
    pub fn new(db: DatabaseConnection) -> Self {
        let datasource_importer = DatasourceImporter::new(db.clone());
        let graph_builder = GraphBuilder::new(db.clone());
        let merge_builder = MergeBuilder::new(db.clone());

        Self {
            db,
            datasource_importer,
            graph_builder,
            merge_builder,
        }
    }

    /// Execute a single node in the DAG
    /// This is called when a node is created, updated, or dependencies change
    pub async fn execute_node(
        &self,
        project_id: i32,
        plan_id: i32,
        node_id: &str,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)], // (source, target) pairs
    ) -> Result<()> {
        // Find the node
        let node = nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| anyhow!("Node not found: {}", node_id))?;

        // Parse node config to get details
        let config: serde_json::Value = serde_json::from_str(&node.config_json)?;
        let metadata: serde_json::Value = serde_json::from_str(&node.metadata_json)?;

        let node_name = metadata["label"].as_str().unwrap_or("Unnamed").to_string();

        match node.node_type.as_str() {
            "DataSourceNode" => {
                // Check if this is a reference to an existing data_source (has dataSourceId)
                // or a file import (has filePath)
                if config["dataSourceId"].is_number() {
                    // DataSource references existing data_sources entry - no execution needed
                    // Data is already in data_sources.graph_json from upload
                    return Ok(());
                } else if let Some(file_path) = config["filePath"].as_str() {
                    // Legacy path: import from file
                    self.datasource_importer
                        .import_datasource(
                            project_id,
                            node_id.to_string(),
                            node_name,
                            file_path.to_string(),
                        )
                        .await?;
                } else {
                    return Err(anyhow!(
                        "DataSource node must have either dataSourceId or filePath in config"
                    ));
                }
            }
            "MergeNode" => {
                // Get upstream node IDs (can be DataSource, Graph, or Merge nodes)
                let upstream_ids = self.get_upstream_nodes(node_id, edges);

                // Merge data from upstream sources
                self.merge_builder
                    .merge_sources(
                        project_id,
                        plan_id,
                        node_id.to_string(),
                        node_name,
                        upstream_ids,
                    )
                    .await?;
            }
            "GraphNode" => {
                // Get upstream node IDs
                let upstream_ids = self.get_upstream_nodes(node_id, edges);

                // Build graph from upstream datasources (reads from data_sources table)
                self.graph_builder
                    .build_graph(
                        project_id,
                        plan_id,
                        node_id.to_string(),
                        node_name,
                        upstream_ids,
                    )
                    .await?;
            }
            "OutputNode" => {
                // Output nodes deliver exports on demand; no proactive execution required
                return Ok(());
            }
            "TransformNode" | "CopyNode" => {
                // TODO: Implement these node types in future phases
                return Err(anyhow!("Node type {} not yet implemented", node.node_type));
            }
            _ => {
                return Err(anyhow!("Unknown node type: {}", node.node_type));
            }
        }

        Ok(())
    }

    /// Execute nodes in topological order
    /// This processes the entire DAG or a subgraph
    pub async fn execute_dag(
        &self,
        project_id: i32,
        plan_id: i32,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<()> {
        // Perform topological sort
        let sorted_nodes = self.topological_sort(nodes, edges)?;

        // Execute nodes in order
        for node_id in sorted_nodes {
            self.execute_node(project_id, plan_id, &node_id, nodes, edges)
                .await?;
        }

        Ok(())
    }

    /// Execute nodes affected by a change
    /// This identifies downstream nodes and executes them in order
    pub async fn execute_affected_nodes(
        &self,
        project_id: i32,
        plan_id: i32,
        changed_node_id: &str,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<()> {
        // Find all downstream nodes
        let affected = self.find_downstream_nodes(changed_node_id, nodes, edges);

        // Include the changed node itself
        let mut all_affected = vec![changed_node_id.to_string()];
        all_affected.extend(affected);

        // Filter nodes to only affected ones that require automatic execution.
        // Skip downstream OutputNodes since they are executed on-demand for previews/exports.
        let affected_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| {
                all_affected.contains(&n.id)
                    && (n.id == changed_node_id || n.node_type != "OutputNode")
            })
            .cloned()
            .collect();

        // Execute in topological order
        let sorted = self.topological_sort(&affected_nodes, edges)?;

        for node_id in sorted {
            self.execute_node(project_id, plan_id, &node_id, nodes, edges)
                .await?;
        }

        Ok(())
    }

    /// Perform topological sort on DAG nodes
    /// Returns node IDs in execution order
    fn topological_sort(
        &self,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node in nodes {
            in_degree.insert(node.id.clone(), 0);
            adj_list.insert(node.id.clone(), Vec::new());
        }

        // Build adjacency list and in-degree count
        for (source, target) in edges {
            // Only consider edges between nodes in our set
            if in_degree.contains_key(source) && in_degree.contains_key(target) {
                adj_list.get_mut(source).unwrap().push(target.clone());
                *in_degree.get_mut(target).unwrap() += 1;
            }
        }

        // Find nodes with no incoming edges
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            sorted.push(node_id.clone());

            // Reduce in-degree for neighbors
            if let Some(neighbors) = adj_list.get(&node_id) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        // Check for cycles
        if sorted.len() != nodes.len() {
            return Err(anyhow!("DAG contains a cycle"));
        }

        Ok(sorted)
    }

    /// Find all downstream nodes from a given node
    fn find_downstream_nodes(
        &self,
        start_node: &str,
        _nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node.to_string());

        while let Some(node_id) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            // Find outgoing edges
            for (source, target) in edges {
                if source == &node_id && !visited.contains(target) {
                    queue.push_back(target.clone());
                }
            }
        }

        // Remove start node from results
        visited.remove(start_node);
        visited.into_iter().collect()
    }

    /// Execute a node and all its upstream dependencies
    /// This ensures upstream nodes (like Merge) are executed before the target node
    pub async fn execute_with_dependencies(
        &self,
        project_id: i32,
        plan_id: i32,
        target_node_id: &str,
        nodes: &[plan_dag_nodes::Model],
        edges: &[(String, String)],
    ) -> Result<()> {
        // Find all upstream nodes (ancestors)
        let upstream = self.find_upstream_nodes(target_node_id, edges);

        // Include the target node itself
        let mut all_nodes_to_execute = upstream;
        all_nodes_to_execute.push(target_node_id.to_string());

        // Filter nodes to only those we need to execute
        let nodes_to_execute: Vec<_> = nodes
            .iter()
            .filter(|n| all_nodes_to_execute.contains(&n.id))
            .cloned()
            .collect();

        // Execute in topological order
        let sorted = self.topological_sort(&nodes_to_execute, edges)?;

        for node_id in sorted {
            self.execute_node(project_id, plan_id, &node_id, nodes, edges)
                .await?;
        }

        Ok(())
    }

    /// Find all upstream nodes (ancestors) from a given node
    fn find_upstream_nodes(&self, start_node: &str, edges: &[(String, String)]) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node.to_string());

        while let Some(node_id) = queue.pop_front() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            // Find incoming edges (upstream nodes)
            for (source, target) in edges {
                if target == &node_id && !visited.contains(source) {
                    queue.push_back(source.clone());
                }
            }
        }

        // Remove start node from results
        visited.remove(start_node);
        visited.into_iter().collect()
    }

    /// Get upstream node IDs for a given node
    fn get_upstream_nodes(&self, node_id: &str, edges: &[(String, String)]) -> Vec<String> {
        edges
            .iter()
            .filter(|(_, target)| target == node_id)
            .map(|(source, _)| source.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort() {
        let executor = DagExecutor::new(DatabaseConnection::default());

        // Create test nodes
        let nodes = vec![
            plan_dag_nodes::Model {
                id: "A".to_string(),
                plan_id: 1,
                node_type: "DataSource".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "B".to_string(),
                plan_id: 1,
                node_type: "DataSource".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "C".to_string(),
                plan_id: 1,
                node_type: "Graph".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        // Edges: A -> C, B -> C
        let edges = vec![
            ("A".to_string(), "C".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let sorted = executor.topological_sort(&nodes, &edges).unwrap();

        // C should come after both A and B
        let c_pos = sorted.iter().position(|id| id == "C").unwrap();
        let a_pos = sorted.iter().position(|id| id == "A").unwrap();
        let b_pos = sorted.iter().position(|id| id == "B").unwrap();

        assert!(c_pos > a_pos);
        assert!(c_pos > b_pos);
    }

    #[test]
    fn test_find_downstream_nodes() {
        let executor = DagExecutor::new(DatabaseConnection::default());

        let nodes = vec![
            plan_dag_nodes::Model {
                id: "A".to_string(),
                plan_id: 1,
                node_type: "DataSource".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "B".to_string(),
                plan_id: 1,
                node_type: "Graph".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            plan_dag_nodes::Model {
                id: "C".to_string(),
                plan_id: 1,
                node_type: "Graph".to_string(),
                position_x: 0.0,
                position_y: 0.0,
                source_position: None,
                target_position: None,
                metadata_json: "{}".to_string(),
                config_json: "{}".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        // Edges: A -> B -> C
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let downstream = executor.find_downstream_nodes("A", &nodes, &edges);

        assert!(downstream.contains(&"B".to_string()));
        assert!(downstream.contains(&"C".to_string()));
        assert_eq!(downstream.len(), 2);
    }
}
