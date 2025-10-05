use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::collections::{HashMap, HashSet};

use crate::database::entities::{data_sources, graph_edges, graph_nodes, graphs, plan_dag_nodes};
use crate::database::entities::datasources::ExecutionState;

/// Service for merging data from multiple upstream sources
pub struct MergeBuilder {
    db: DatabaseConnection,
}

impl MergeBuilder {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Merge data from upstream sources
    /// Returns the created/updated graph entity
    /// Note: Merge nodes reuse the graphs table structure for intermediate results
    pub async fn merge_sources(
        &self,
        project_id: i32,
        plan_id: i32,
        node_id: String,
        name: String,
        upstream_node_ids: Vec<String>,
    ) -> Result<graphs::Model> {
        // Get or create graph entity (Merge uses same tables as Graph)
        let graph = self.get_or_create_graph(project_id, &node_id, name).await?;

        // Set to processing state
        let mut active: graphs::ActiveModel = graph.into();
        active = active.set_state(ExecutionState::Processing);
        let graph = active.update(&self.db).await?;

        // Fetch upstream data sources by reading plan_dag_nodes configs
        let mut data_sources_list = Vec::new();
        for upstream_id in &upstream_node_ids {
            // Check if upstream is a DataSource node or a Graph/Merge node
            let upstream_node = plan_dag_nodes::Entity::find_by_id(upstream_id)
                .filter(plan_dag_nodes::Column::PlanId.eq(plan_id))
                .one(&self.db)
                .await?
                .ok_or_else(|| anyhow!("Upstream node not found: {}", upstream_id))?;

            match upstream_node.node_type.as_str() {
                "DataSourceNode" => {
                    // Read from data_sources table
                    let data_source = self.get_upstream_data_source(plan_id, upstream_id).await?;
                    data_sources_list.push(DataSourceOrGraph::DataSource(data_source));
                }
                "GraphNode" | "MergeNode" => {
                    // Read from graphs table
                    let graph = self.get_upstream_graph(project_id, upstream_id).await?;
                    data_sources_list.push(DataSourceOrGraph::Graph(graph));
                }
                _ => {
                    return Err(anyhow!("Unsupported upstream node type for Merge: {}", upstream_node.node_type));
                }
            }
        }

        // Compute source hash for change detection
        let source_hash = self.compute_source_hash(&data_sources_list)?;

        // Check if recomputation is needed
        if let Some(existing_hash) = &graph.source_hash {
            if existing_hash == &source_hash {
                // No changes, return existing graph
                return Ok(graph);
            }
        }

        // Merge data from all sources
        let result = self
            .merge_data_from_sources(&graph, &data_sources_list)
            .await;

        match result {
            Ok((node_count, edge_count)) => {
                // Update to completed state
                let mut active: graphs::ActiveModel = graph.into();
                active = active.set_completed(source_hash, node_count as i32, edge_count as i32);
                let updated = active.update(&self.db).await?;
                Ok(updated)
            }
            Err(e) => {
                // Update to error state
                let mut active: graphs::ActiveModel = graph.into();
                active = active.set_error(e.to_string());
                let _updated = active.update(&self.db).await?;
                Err(e)
            }
        }
    }

    /// Get or create graph entity
    async fn get_or_create_graph(
        &self,
        project_id: i32,
        node_id: &str,
        name: String,
    ) -> Result<graphs::Model> {
        use crate::database::entities::graphs::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        // Try to find existing
        if let Some(graph) = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::NodeId.eq(node_id))
            .one(&self.db)
            .await?
        {
            return Ok(graph);
        }

        // Create new (let database auto-generate ID)
        let graph = graphs::ActiveModel {
            project_id: Set(project_id),
            node_id: Set(node_id.to_string()),
            name: Set(name),
            ..graphs::ActiveModel::new()
        };

        let graph = graph.insert(&self.db).await?;
        Ok(graph)
    }

    /// Get upstream data_source by reading plan_dag_node config
    async fn get_upstream_data_source(
        &self,
        plan_id: i32,
        node_id: &str,
    ) -> Result<data_sources::Model> {
        use sea_orm::{ColumnTrait, QueryFilter};

        // Find the plan_dag_node to get the config
        let dag_node = plan_dag_nodes::Entity::find_by_id(node_id)
            .filter(plan_dag_nodes::Column::PlanId.eq(plan_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Plan DAG node not found: {}", node_id))?;

        // Parse config to get dataSourceId
        let config: serde_json::Value = serde_json::from_str(&dag_node.config_json)
            .map_err(|e| anyhow!("Failed to parse node config: {}", e))?;

        let data_source_id = config.get("dataSourceId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| anyhow!("Node config does not have dataSourceId: {}", node_id))?;

        // Query the data_sources table
        data_sources::Entity::find_by_id(data_source_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("DataSource not found with id {}", data_source_id))
    }

    /// Get upstream graph (from Graph or Merge node)
    async fn get_upstream_graph(
        &self,
        project_id: i32,
        node_id: &str,
    ) -> Result<graphs::Model> {
        use crate::database::entities::graphs::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::NodeId.eq(node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Graph not found for node {}", node_id))
    }

    /// Compute hash of upstream sources for change detection
    fn compute_source_hash(&self, sources: &[DataSourceOrGraph]) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        for source in sources {
            match source {
                DataSourceOrGraph::DataSource(ds) => {
                    hasher.update(ds.id.to_string().as_bytes());
                    hasher.update(ds.filename.as_bytes());
                    if let Some(processed_at) = &ds.processed_at {
                        hasher.update(processed_at.to_rfc3339().as_bytes());
                    }
                }
                DataSourceOrGraph::Graph(graph) => {
                    hasher.update(graph.id.to_string().as_bytes());
                    hasher.update(graph.node_id.as_bytes());
                    if let Some(computed_date) = &graph.computed_date {
                        hasher.update(computed_date.to_rfc3339().as_bytes());
                    }
                }
            }
        }

        let hash = format!("{:x}", hasher.finalize());
        Ok(hash)
    }

    /// Merge data from all sources (data_sources + graphs)
    async fn merge_data_from_sources(
        &self,
        graph: &graphs::Model,
        sources: &[DataSourceOrGraph],
    ) -> Result<(usize, usize)> {
        // Clear existing graph data
        self.clear_graph_data(graph.id).await?;

        let mut all_nodes = HashMap::new();
        let mut all_edges = Vec::new();

        // Process each source
        for source in sources {
            match source {
                DataSourceOrGraph::DataSource(ds) => {
                    // Parse graph_json from data_sources
                    let graph_data: serde_json::Value = serde_json::from_str(&ds.graph_json)
                        .map_err(|e| anyhow!("Failed to parse graph JSON for {}: {}", ds.name, e))?;

                    self.extract_from_json(&mut all_nodes, &mut all_edges, &graph_data, &ds.data_type)?;
                }
                DataSourceOrGraph::Graph(upstream_graph) => {
                    // Read nodes and edges from graph_nodes and graph_edges tables
                    self.extract_from_graph_tables(&mut all_nodes, &mut all_edges, upstream_graph.id).await?;
                }
            }
        }

        // Validate edges reference existing nodes
        let node_ids: HashSet<_> = all_nodes.keys().cloned().collect();
        for edge in &all_edges {
            if !node_ids.contains(&edge.source) {
                return Err(anyhow!(
                    "Edge {} references non-existent source node: {}",
                    edge.id,
                    edge.source
                ));
            }
            if !node_ids.contains(&edge.target) {
                return Err(anyhow!(
                    "Edge {} references non-existent target node: {}",
                    edge.id,
                    edge.target
                ));
            }
        }

        // Insert nodes
        for (id, node_data) in all_nodes {
            let node = graph_nodes::ActiveModel {
                id: Set(id),
                graph_id: Set(graph.id),
                label: Set(node_data.label),
                layer: Set(node_data.layer),
                weight: Set(node_data.weight),
                is_partition: Set(node_data.is_partition),
                attrs: Set(node_data.attrs),
                created_at: Set(chrono::Utc::now()),
            };

            node.insert(&self.db).await?;
        }

        // Insert edges
        let edge_count = all_edges.len();
        for edge_data in all_edges {
            let edge = graph_edges::ActiveModel {
                id: Set(edge_data.id),
                graph_id: Set(graph.id),
                source: Set(edge_data.source),
                target: Set(edge_data.target),
                label: Set(edge_data.label),
                layer: Set(edge_data.layer),
                weight: Set(edge_data.weight),
                attrs: Set(edge_data.attrs),
                created_at: Set(chrono::Utc::now()),
            };

            edge.insert(&self.db).await?;
        }

        let node_count = node_ids.len();
        Ok((node_count, edge_count))
    }

    /// Extract nodes and edges from graph_json
    fn extract_from_json(
        &self,
        all_nodes: &mut HashMap<String, NodeData>,
        all_edges: &mut Vec<EdgeData>,
        graph_data: &serde_json::Value,
        data_type: &str,
    ) -> Result<()> {
        match data_type {
            "nodes" => {
                if let Some(nodes_array) = graph_data.get("nodes").and_then(|v| v.as_array()) {
                    for node_val in nodes_array {
                        let id = node_val["id"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Node missing 'id' field"))?
                            .to_string();

                        let node = NodeData {
                            label: node_val["label"].as_str().map(|s| s.to_string()),
                            layer: node_val["layer"].as_str().map(|s| s.to_string()),
                            weight: node_val["weight"].as_f64(),
                            is_partition: node_val["is_partition"].as_bool().unwrap_or(false),
                            attrs: Some(node_val.clone()),
                        };

                        all_nodes.insert(id, node);
                    }
                }
            }
            "edges" => {
                if let Some(edges_array) = graph_data.get("edges").and_then(|v| v.as_array()) {
                    for edge_val in edges_array {
                        let id = edge_val["id"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Edge missing 'id' field"))?
                            .to_string();
                        let source = edge_val["source"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Edge missing 'source' field"))?
                            .to_string();
                        let target = edge_val["target"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Edge missing 'target' field"))?
                            .to_string();

                        let edge = EdgeData {
                            id,
                            source,
                            target,
                            label: edge_val["label"].as_str().map(|s| s.to_string()),
                            layer: edge_val["layer"].as_str().map(|s| s.to_string()),
                            weight: edge_val["weight"].as_f64(),
                            attrs: Some(edge_val.clone()),
                        };

                        all_edges.push(edge);
                    }
                }
            }
            "graph" => {
                // Extract both nodes and edges
                if let Some(nodes_array) = graph_data.get("nodes").and_then(|v| v.as_array()) {
                    for node_val in nodes_array {
                        let id = node_val["id"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Node missing 'id' field"))?
                            .to_string();

                        let node = NodeData {
                            label: node_val["label"].as_str().map(|s| s.to_string()),
                            layer: node_val["layer"].as_str().map(|s| s.to_string()),
                            weight: node_val["weight"].as_f64(),
                            is_partition: node_val["is_partition"].as_bool().unwrap_or(false),
                            attrs: Some(node_val.clone()),
                        };

                        all_nodes.insert(id, node);
                    }
                }

                if let Some(edges_array) = graph_data.get("edges").and_then(|v| v.as_array()) {
                    for edge_val in edges_array {
                        let id = edge_val["id"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Edge missing 'id' field"))?
                            .to_string();
                        let source = edge_val["source"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Edge missing 'source' field"))?
                            .to_string();
                        let target = edge_val["target"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Edge missing 'target' field"))?
                            .to_string();

                        let edge = EdgeData {
                            id,
                            source,
                            target,
                            label: edge_val["label"].as_str().map(|s| s.to_string()),
                            layer: edge_val["layer"].as_str().map(|s| s.to_string()),
                            weight: edge_val["weight"].as_f64(),
                            attrs: Some(edge_val.clone()),
                        };

                        all_edges.push(edge);
                    }
                }
            }
            _ => {
                return Err(anyhow!("Unknown data type: {}", data_type));
            }
        }

        Ok(())
    }

    /// Extract nodes and edges from graph_nodes and graph_edges tables
    async fn extract_from_graph_tables(
        &self,
        all_nodes: &mut HashMap<String, NodeData>,
        all_edges: &mut Vec<EdgeData>,
        graph_id: i32,
    ) -> Result<()> {
        use sea_orm::{ColumnTrait, QueryFilter};

        // Read nodes
        let nodes = graph_nodes::Entity::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        for node in nodes {
            all_nodes.insert(
                node.id.clone(),
                NodeData {
                    label: node.label,
                    layer: node.layer,
                    weight: node.weight,
                    is_partition: node.is_partition,
                    attrs: node.attrs,
                },
            );
        }

        // Read edges
        let edges = graph_edges::Entity::find()
            .filter(graph_edges::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        for edge in edges {
            all_edges.push(EdgeData {
                id: edge.id,
                source: edge.source,
                target: edge.target,
                label: edge.label,
                layer: edge.layer,
                weight: edge.weight,
                attrs: edge.attrs,
            });
        }

        Ok(())
    }

    /// Clear existing graph data
    async fn clear_graph_data(&self, graph_id: i32) -> Result<()> {
        use crate::database::entities::graph_edges::{Column as EdgeColumn, Entity as EdgeEntity};
        use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as NodeEntity};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        // Delete edges
        EdgeEntity::delete_many()
            .filter(EdgeColumn::GraphId.eq(graph_id))
            .exec(&self.db)
            .await?;

        // Delete nodes
        NodeEntity::delete_many()
            .filter(NodeColumn::GraphId.eq(graph_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}

/// Enum to represent either a DataSource or a Graph (for upstream sources)
enum DataSourceOrGraph {
    DataSource(data_sources::Model),
    Graph(graphs::Model),
}

/// Internal node data structure
struct NodeData {
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    is_partition: bool,
    attrs: Option<serde_json::Value>,
}

/// Internal edge data structure
struct EdgeData {
    id: String,
    source: String,
    target: String,
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    attrs: Option<serde_json::Value>,
}
