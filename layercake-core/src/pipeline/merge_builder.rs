use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use super::layer_operations::{insert_layers_to_db, load_layers_from_db};
use super::persist_utils::{clear_graph_storage, insert_edge_batches, insert_node_batches};
use super::types::LayerData;
use crate::database::entities::ExecutionState;
use crate::database::entities::{data_sets, graph_edges, graph_nodes, graphs, plan_dag_nodes};

/// Helper function to parse is_partition from JSON Value (handles both boolean and string)
fn parse_is_partition(value: &Value) -> bool {
    // Try as boolean first
    if let Some(b) = value.as_bool() {
        return b;
    }

    // Try as string with truthy logic
    if let Some(s) = value.as_str() {
        let trimmed_lowercase = s.trim().to_lowercase();
        return matches!(trimmed_lowercase.as_str(), "true" | "y" | "yes" | "1");
    }

    // Default to false
    false
}

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
        _plan_id: i32,
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

        // Publish execution status change
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_graph_status(
            &self.db, project_id, &node_id, &graph,
        )
        .await;

        // Fetch upstream data sources by reading plan_dag_nodes configs
        let mut data_sets_list = Vec::new();
        let mut data_set_cache = HashMap::new();
        for upstream_id in &upstream_node_ids {
            // Check if upstream is a DataSet node or a Graph/Merge node
            // Note: Node IDs are globally unique, no need to filter by plan_id
            let upstream_node = plan_dag_nodes::Entity::find_by_id(upstream_id)
                .one(&self.db)
                .await?
                .ok_or_else(|| anyhow!("Upstream node not found: {}", upstream_id))?;

            match upstream_node.node_type.as_str() {
                "DataSetNode" => {
                    // Read from data_sets table using the node we just found
                    let data_set = self
                        .get_data_set_from_node(&upstream_node, &mut data_set_cache)
                        .await?;
                    data_sets_list.push(DataSetOrGraph::DataSet(data_set));
                }
                "GraphNode" | "MergeNode" => {
                    // Read from graphs table
                    let graph = self.get_upstream_graph(project_id, upstream_id).await?;
                    data_sets_list.push(DataSetOrGraph::Graph(graph));
                }
                _ => {
                    return Err(anyhow!(
                        "Unsupported upstream node type for Merge: {}",
                        upstream_node.node_type
                    ));
                }
            }
        }

        // Compute source hash for change detection
        let source_hash = self.compute_source_hash(&data_sets_list)?;

        // Check if recomputation is needed
        if let Some(existing_hash) = &graph.source_hash {
            if existing_hash == &source_hash {
                // No changes, return existing graph
                return Ok(graph);
            }
        }

        // Merge data from all sources
        let result = self.merge_data_from_sources(&graph, &data_sets_list).await;

        match result {
            Ok((node_count, edge_count)) => {
                // Update to completed state
                let mut active: graphs::ActiveModel = graph.into();
                active = active.set_completed(source_hash, node_count as i32, edge_count as i32);
                let updated = active.update(&self.db).await?;

                // Publish execution status change
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_graph_status(
                    &self.db, project_id, &node_id, &updated,
                )
                .await;

                Ok(updated)
            }
            Err(e) => {
                // Update to error state
                let mut active: graphs::ActiveModel = graph.into();
                active = active.set_error(e.to_string());
                let updated = active.update(&self.db).await?;

                // Publish execution status change
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_graph_status(
                    &self.db, project_id, &node_id, &updated,
                )
                .await;

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

    /// Get data_set from a plan_dag_node
    async fn get_data_set_from_node(
        &self,
        dag_node: &plan_dag_nodes::Model,
        cache: &mut HashMap<i32, data_sets::Model>,
    ) -> Result<data_sets::Model> {
        // Parse config to get dataSetId
        let config: serde_json::Value = serde_json::from_str(&dag_node.config_json)
            .map_err(|e| anyhow!("Failed to parse node config: {}", e))?;

        let data_set_id = config
            .get("dataSetId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| anyhow!("Node config does not have dataSetId: {}", dag_node.id))?;

        if let Some(cached) = cache.get(&data_set_id) {
            return Ok(cached.clone());
        }

        let data_set = data_sets::Entity::find_by_id(data_set_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found with id {}", data_set_id))?;

        cache.insert(data_set_id, data_set.clone());
        Ok(data_set)
    }

    /// Get upstream graph (from Graph or Merge node)
    async fn get_upstream_graph(&self, project_id: i32, node_id: &str) -> Result<graphs::Model> {
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
    fn compute_source_hash(&self, sources: &[DataSetOrGraph]) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        for source in sources {
            match source {
                DataSetOrGraph::DataSet(ds) => {
                    hasher.update(ds.id.to_le_bytes());
                    hasher.update(ds.data_type.as_bytes());
                    hasher.update(ds.graph_json.as_bytes());
                    if let Some(processed_at) = &ds.processed_at {
                        hasher.update(processed_at.to_rfc3339().as_bytes());
                    }
                }
                DataSetOrGraph::Graph(graph) => {
                    hasher.update(graph.id.to_le_bytes());
                    if let Some(hash) = &graph.source_hash {
                        hasher.update(hash.as_bytes());
                    }
                    hasher.update(graph.updated_at.timestamp_micros().to_le_bytes());
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

    /// Merge data from all sources (data_sets + graphs)
    async fn merge_data_from_sources(
        &self,
        graph: &graphs::Model,
        sources: &[DataSetOrGraph],
    ) -> Result<(usize, usize)> {
        let txn = self.db.begin().await?;
        clear_graph_storage(&txn, graph.id).await?;

        let mut all_nodes = HashMap::new();
        let mut all_edges: HashMap<String, EdgeData> = HashMap::new();
        let mut all_layers = HashMap::new(); // layer_id -> layer data

        // Process each source
        for source in sources {
            match source {
                DataSetOrGraph::DataSet(ds) => {
                    // Parse graph_json from data_sets
                    let graph_data: serde_json::Value = serde_json::from_str(&ds.graph_json)
                        .map_err(|e| {
                            anyhow!("Failed to parse graph JSON for {}: {}", ds.name, e)
                        })?;

                    self.extract_from_json(
                        &mut all_nodes,
                        &mut all_edges,
                        &mut all_layers,
                        &graph_data,
                        &ds.data_type,
                        Some(ds.id),
                    )?;
                }
                DataSetOrGraph::Graph(upstream_graph) => {
                    // Read nodes and edges from graph_nodes and graph_edges tables
                    self.extract_from_graph_tables(
                        &mut all_nodes,
                        &mut all_edges,
                        upstream_graph.id,
                    )
                    .await?;
                    // Also read layers from the upstream graph
                    self.extract_layers_from_graph(&mut all_layers, upstream_graph.id)
                        .await?;
                }
            }
        }

        // Validate edges reference existing nodes
        let node_ids: HashSet<_> = all_nodes.keys().cloned().collect();
        for (edge_id, edge) in &all_edges {
            if !node_ids.contains(&edge.source) {
                return Err(anyhow!(
                    "Edge {} references non-existent source node: {}",
                    edge_id,
                    edge.source
                ));
            }
            if !node_ids.contains(&edge.target) {
                return Err(anyhow!(
                    "Edge {} references non-existent target node: {}",
                    edge_id,
                    edge.target
                ));
            }

            // Validate edges don't reference partition nodes
            if let Some(source_node) = all_nodes.get(&edge.source) {
                if source_node.is_partition {
                    return Err(anyhow!(
                        "Edge {} has source node {} which is a partition (subflow). Edges cannot connect to partition nodes.",
                        edge_id,
                        edge.source
                    ));
                }
            }
            if let Some(target_node) = all_nodes.get(&edge.target) {
                if target_node.is_partition {
                    return Err(anyhow!(
                        "Edge {} has target node {} which is a partition (subflow). Edges cannot connect to partition nodes.",
                        edge_id,
                        edge.target
                    ));
                }
            }
        }

        // Insert nodes
        let node_models: Vec<_> = all_nodes
            .into_iter()
            .map(|(id, node_data)| graph_nodes::ActiveModel {
                id: Set(id),
                graph_id: Set(graph.id),
                label: Set(node_data.label),
                layer: Set(node_data.layer),
                weight: Set(node_data.weight),
                is_partition: Set(node_data.is_partition),
                belongs_to: Set(node_data.belongs_to),
                dataset_id: Set(node_data.dataset_id),
                attrs: Set(node_data.attrs),
                comment: Set(node_data.comment),
                created_at: Set(chrono::Utc::now()),
            })
            .collect();
        insert_node_batches(&txn, node_models).await?;

        // Insert edges
        let edge_count = all_edges.len();
        let edge_models: Vec<_> = all_edges
            .into_iter()
            .map(|(edge_id, edge_data)| graph_edges::ActiveModel {
                id: Set(edge_id),
                graph_id: Set(graph.id),
                source: Set(edge_data.source),
                target: Set(edge_data.target),
                label: Set(edge_data.label),
                layer: Set(edge_data.layer),
                weight: Set(edge_data.weight),
                dataset_id: Set(edge_data.dataset_id),
                attrs: Set(edge_data.attrs),
                comment: Set(edge_data.comment),
                created_at: Set(chrono::Utc::now()),
            })
            .collect();
        insert_edge_batches(&txn, edge_models).await?;

        // Insert layers using shared function
        insert_layers_to_db(&txn, graph.id, all_layers).await?;
        txn.commit().await?;

        let node_count = node_ids.len();
        Ok((node_count, edge_count))
    }

    /// Extract nodes, edges, and layers from graph_json
    fn extract_from_json(
        &self,
        all_nodes: &mut HashMap<String, NodeData>,
        all_edges: &mut HashMap<String, EdgeData>,
        all_layers: &mut HashMap<String, LayerData>,
        graph_data: &serde_json::Value,
        data_type: &str,
        dataset_id: Option<i32>,
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
                            is_partition: parse_is_partition(&node_val["is_partition"]),
                            belongs_to: node_val["belongs_to"]
                                .as_str()
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                            comment: node_val["comment"].as_str().map(|s| s.to_string()),
                            attrs: Some(node_val.clone()),
                            dataset_id,
                        };

                        merge_node(all_nodes, id, node);
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
                            source,
                            target,
                            label: edge_val["label"].as_str().map(|s| s.to_string()),
                            layer: edge_val["layer"].as_str().map(|s| s.to_string()),
                            weight: edge_val["weight"].as_f64(),
                            comment: edge_val["comment"].as_str().map(|s| s.to_string()),
                            attrs: Some(edge_val.clone()),
                            dataset_id,
                        };

                        merge_edge(all_edges, id, edge);
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
                            is_partition: parse_is_partition(&node_val["is_partition"]),
                            belongs_to: node_val["belongs_to"]
                                .as_str()
                                .filter(|s| !s.is_empty())
                                .map(|s| s.to_string()),
                            comment: node_val["comment"].as_str().map(|s| s.to_string()),
                            attrs: Some(node_val.clone()),
                            dataset_id,
                        };

                        merge_node(all_nodes, id, node);
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
                            source,
                            target,
                            label: edge_val["label"].as_str().map(|s| s.to_string()),
                            layer: edge_val["layer"].as_str().map(|s| s.to_string()),
                            weight: edge_val["weight"].as_f64(),
                            comment: edge_val["comment"].as_str().map(|s| s.to_string()),
                            attrs: Some(edge_val.clone()),
                            dataset_id,
                        };

                        merge_edge(all_edges, id, edge);
                    }
                }
            }
            "layers" => {
                // Extract layers from dataset
                if let Some(layers_array) = graph_data.get("layers").and_then(|v| v.as_array()) {
                    for layer_val in layers_array {
                        let layer_id = layer_val["id"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Layer missing 'id' field"))?
                            .to_string();

                        let name = layer_val["label"].as_str().unwrap_or(&layer_id).to_string();

                        let background_color = layer_val["background_color"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        let text_color = layer_val["text_color"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        let border_color = layer_val["border_color"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        let comment = layer_val["comment"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string());

                        // Extract other properties
                        let mut properties = serde_json::Map::new();
                        if let Some(obj) = layer_val.as_object() {
                            for (key, value) in obj {
                                // Skip fields that are now first-class fields
                                if !matches!(
                                    key.as_str(),
                                    "id" | "label"
                                        | "background_color"
                                        | "text_color"
                                        | "border_color"
                                        | "comment"
                                ) {
                                    properties.insert(key.clone(), value.clone());
                                }
                            }
                        }

                        let properties_json = if properties.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&properties)?)
                        };

                        let layer = LayerData {
                            name,
                            background_color,
                            text_color,
                            border_color,
                            comment,
                            properties: properties_json,
                            dataset_id,
                        };

                        all_layers.insert(layer_id, layer);
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
        all_edges: &mut HashMap<String, EdgeData>,
        graph_id: i32,
    ) -> Result<()> {
        use sea_orm::{ColumnTrait, QueryFilter};

        // Read nodes
        let nodes = graph_nodes::Entity::find()
            .filter(graph_nodes::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        for node in nodes {
            let id = node.id.clone();
            let node_data = NodeData {
                label: node.label,
                layer: node.layer,
                weight: node.weight,
                is_partition: node.is_partition,
                belongs_to: node.belongs_to,
                comment: node.comment,
                attrs: node.attrs,
                dataset_id: node.dataset_id,
            };
            merge_node(all_nodes, id, node_data);
        }

        // Read edges
        let edges = graph_edges::Entity::find()
            .filter(graph_edges::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        for edge in edges {
            let id = edge.id.clone();
            let edge_data = EdgeData {
                source: edge.source,
                target: edge.target,
                label: edge.label,
                layer: edge.layer,
                weight: edge.weight,
                comment: edge.comment,
                attrs: edge.attrs,
                dataset_id: edge.dataset_id,
            };
            merge_edge(all_edges, id, edge_data);
        }

        Ok(())
    }

    /// Extract layers from graph layers table using shared function
    async fn extract_layers_from_graph(
        &self,
        all_layers: &mut HashMap<String, LayerData>,
        graph_id: i32,
    ) -> Result<()> {
        let loaded_layers = load_layers_from_db(&self.db, graph_id).await?;
        all_layers.extend(loaded_layers);
        Ok(())
    }
}

/// Enum to represent either a DataSet or a Graph (for upstream sources)
enum DataSetOrGraph {
    DataSet(data_sets::Model),
    Graph(graphs::Model),
}

/// Internal node data structure
struct NodeData {
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    is_partition: bool,
    belongs_to: Option<String>,
    comment: Option<String>,
    attrs: Option<serde_json::Value>,
    dataset_id: Option<i32>,
}

/// Merge a node into the map, summing weights if the ID already exists
fn merge_node(all_nodes: &mut HashMap<String, NodeData>, id: String, node: NodeData) {
    use std::collections::hash_map::Entry;
    match all_nodes.entry(id) {
        Entry::Occupied(mut entry) => {
            // Node exists - sum weights, keep first node's other values
            let existing = entry.get_mut();
            let existing_weight = existing.weight.unwrap_or(1.0);
            let new_weight = node.weight.unwrap_or(1.0);
            existing.weight = Some(existing_weight + new_weight);
        }
        Entry::Vacant(entry) => {
            // New node - insert as-is
            entry.insert(node);
        }
    }
}

/// Merge an edge into the map, summing weights if the ID already exists
fn merge_edge(all_edges: &mut HashMap<String, EdgeData>, id: String, edge: EdgeData) {
    use std::collections::hash_map::Entry;
    match all_edges.entry(id) {
        Entry::Occupied(mut entry) => {
            // Edge exists - sum weights, keep first edge's other values
            let existing = entry.get_mut();
            let existing_weight = existing.weight.unwrap_or(1.0);
            let new_weight = edge.weight.unwrap_or(1.0);
            existing.weight = Some(existing_weight + new_weight);
        }
        Entry::Vacant(entry) => {
            // New edge - insert as-is
            entry.insert(edge);
        }
    }
}

/// Internal edge data structure
struct EdgeData {
    source: String,
    target: String,
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    comment: Option<String>,
    attrs: Option<serde_json::Value>,
    dataset_id: Option<i32>,
}

// LayerData now imported from super::types (was previously defined here)
