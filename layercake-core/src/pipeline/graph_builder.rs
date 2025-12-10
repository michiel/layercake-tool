use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use super::layer_operations::insert_layers_to_db;
use super::persist_utils::{clear_graph_storage, insert_edge_batches, insert_node_batches};
use super::types::LayerData;
use crate::database::entities::ExecutionState;
use crate::database::entities::{
    data_sets, datasets, graph_edges, graph_nodes, graphs, plan_dag_nodes,
};
use crate::services::GraphEditService;
use tracing::{info, warn};

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

/// Service for building graphs from datasets
pub struct GraphBuilder {
    db: DatabaseConnection,
}

impl GraphBuilder {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Build a graph from upstream datasets
    /// Returns the created/updated graph entity
    /// Reads directly from data_sets table (no pipeline processing needed)
    pub async fn build_graph(
        &self,
        project_id: i32,
        _plan_id: i32,
        node_id: String,
        name: String,
        upstream_node_ids: Vec<String>,
    ) -> Result<graphs::Model> {
        // Get or create graph entity
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

        // Fetch upstream sources by reading plan_dag_nodes
        // Upstream can be DataSet nodes OR Graph/Merge nodes
        let mut data_sets_list = Vec::new();
        let mut data_set_cache = HashMap::new();
        for upstream_id in &upstream_node_ids {
            // Get the upstream node to check its type
            let upstream_node = plan_dag_nodes::Entity::find_by_id(upstream_id)
                .one(&self.db)
                .await?
                .ok_or_else(|| anyhow!("Upstream node not found: {}", upstream_id))?;

            match upstream_node.node_type.as_str() {
                "DataSetNode" => {
                    let data_set = self
                        .get_data_set_from_node(&upstream_node, &mut data_set_cache)
                        .await?;

                    // Check if data source is ready
                    if data_set.status != "active" {
                        return Err(anyhow!(
                            "Upstream data source {} is not ready (status: {})",
                            upstream_id,
                            data_set.status
                        ));
                    }
                    data_sets_list.push(data_set);
                }
                "GraphNode" | "MergeNode" | "TransformNode" | "FilterNode" => {
                    // Legacy path does not support chaining computed graphs
                    return Err(anyhow!(
                        "Cannot chain computed graphs in legacy GraphBuilder. \
                         Please migrate this node to use 'graphDataIds' in config instead of legacy upstream IDs. \
                         GraphDataBuilder natively supports chaining graph_data without conversion. \
                         See Phase 3 migration docs for details."
                    ));
                }
                _ => {
                    return Err(anyhow!(
                        "Unsupported upstream node type for Graph: {}",
                        upstream_node.node_type
                    ));
                }
            }
        }

        // Compute source hash for change detection
        let source_hash = self.compute_data_set_hash(&data_sets_list)?;

        // Check if recomputation is needed
        if let Some(existing_hash) = graph.source_hash.clone() {
            if existing_hash == source_hash {
                // Upstream content unchanged; restore completed state if needed
                let mut active: graphs::ActiveModel = graph.clone().into();
                active = active.set_completed(existing_hash, graph.node_count, graph.edge_count);
                let updated = active.update(&self.db).await?;

                // Publish execution status change so subscribers refresh state
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_graph_status(
                    &self.db, project_id, &node_id, &updated,
                )
                .await;

                return Ok(updated);
            }
        }

        // Build graph data from data_sets
        let result = self
            .build_graph_from_data_sets(&graph, &data_sets_list)
            .await;

        match result {
            Ok((node_count, edge_count)) => {
                // Update to completed state
                let mut active: graphs::ActiveModel = graph.clone().into();
                active = active.set_completed(source_hash, node_count as i32, edge_count as i32);
                let mut updated = active.update(&self.db).await?;

                // Check if there are pending edits to replay
                if updated.has_pending_edits {
                    info!("Graph {} has pending edits, starting replay", updated.id);

                    let edit_service = GraphEditService::new(self.db.clone());
                    match edit_service.replay_graph_edits(updated.id).await {
                        Ok(summary) => {
                            info!(
                                "Replay complete for graph {}: {} applied, {} skipped, {} failed",
                                updated.id, summary.applied, summary.skipped, summary.failed
                            );

                            // If any edits failed, log warning but don't fail the build
                            if summary.failed > 0 {
                                warn!(
                                    "Graph {} replay had {} failed edits",
                                    updated.id, summary.failed
                                );
                            }

                            // Refresh the graph entity to get updated metadata
                            updated = graphs::Entity::find_by_id(updated.id)
                                .one(&self.db)
                                .await?
                                .ok_or_else(|| anyhow!("Graph not found after replay"))?;
                        }
                        Err(e) => {
                            warn!("Failed to replay edits for graph {}: {}", updated.id, e);
                            // Don't fail the entire build, just log the error
                        }
                    }
                }

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
        use sea_orm::{ColumnTrait, QueryFilter};

        graphs::Entity::find()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .filter(graphs::Column::NodeId.eq(node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Graph not found for node: {}", node_id))
    }


    /// Compute hash of upstream data_sets for change detection
    fn compute_data_set_hash(&self, data_sets: &[data_sets::Model]) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        for ds in data_sets {
            hasher.update(ds.id.to_le_bytes());
            hasher.update(ds.graph_json.as_bytes());
            if let Some(processed_at) = &ds.processed_at {
                hasher.update(processed_at.to_rfc3339().as_bytes());
            }
        }

        let hash = format!("{:x}", hasher.finalize());
        Ok(hash)
    }

    /// Build graph from data_sets (reads graph_json directly)
    async fn build_graph_from_data_sets(
        &self,
        graph: &graphs::Model,
        data_sets: &[data_sets::Model],
    ) -> Result<(usize, usize)> {
        // Clear existing graph data within a transaction
        let txn = self.db.begin().await?;
        clear_graph_storage(&txn, graph.id).await?;

        let mut all_nodes = HashMap::new();
        let mut all_edges = Vec::new();
        let mut used_edge_ids = HashSet::new();
        let mut all_layers = HashMap::new(); // layer_id -> layer data

        // Process each data source
        for ds in data_sets {
            // Parse graph_json
            let graph_data: serde_json::Value = serde_json::from_str(&ds.graph_json)
                .map_err(|e| anyhow!("Failed to parse graph JSON for {}: {}", ds.name, e))?;

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
                        dataset_id: Some(ds.id),
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
                        id: allocate_edge_id(&id, Some(ds.id), &mut used_edge_ids),
                        source,
                        target,
                        label: edge_val["label"].as_str().map(|s| s.to_string()),
                        layer: edge_val["layer"].as_str().map(|s| s.to_string()),
                        weight: edge_val["weight"].as_f64(),
                        comment: edge_val["comment"].as_str().map(|s| s.to_string()),
                        attrs: Some(edge_val.clone()),
                        dataset_id: Some(ds.id),
                    };

                    all_edges.push(edge);
                }
            }

            let mut process_layer_array = |array: &Vec<serde_json::Value>| -> Result<()> {
                for layer_val in array {
                    let layer_id = layer_val["id"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Layer missing 'id' field"))?
                        .to_string();

                    if layer_id.is_empty() {
                        continue;
                    }

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

                    let alias = layer_val["alias"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let comment = layer_val["comment"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let mut properties = serde_json::Map::new();
                    if let Some(obj) = layer_val.as_object() {
                        for (key, value) in obj {
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
                        alias,
                        comment,
                        properties: properties_json,
                        dataset_id: Some(ds.id),
                    };

                    all_layers.insert(layer_id, layer);
                }
                Ok(())
            };

            if let Some(layers_array) = graph_data.get("graph_layers").and_then(|v| v.as_array()) {
                process_layer_array(layers_array)?;
            }

            if let Some(layers_array) = graph_data.get("layers").and_then(|v| v.as_array()) {
                process_layer_array(layers_array)?;
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

            // Validate edges don't reference partition nodes
            if let Some(source_node) = all_nodes.get(&edge.source) {
                if source_node.is_partition {
                    return Err(anyhow!(
                        "Edge {} has source node {} which is a partition (subflow). Edges cannot connect to partition nodes.",
                        edge.id,
                        edge.source
                    ));
                }
            }
            if let Some(target_node) = all_nodes.get(&edge.target) {
                if target_node.is_partition {
                    return Err(anyhow!(
                        "Edge {} has target node {} which is a partition (subflow). Edges cannot connect to partition nodes.",
                        edge.id,
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
            .map(|edge_data| graph_edges::ActiveModel {
                id: Set(edge_data.id),
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

        // Insert graph_layers using shared function
        insert_layers_to_db(&txn, graph.id, all_layers).await?;
        txn.commit().await?;

        let node_count = node_ids.len();
        Ok((node_count, edge_count))
    }

    /// Extract nodes from a nodes dataset
    #[allow(dead_code)]
    async fn extract_nodes_from_dataset(
        &self,
        dataset: &datasets::Model,
    ) -> Result<HashMap<String, NodeData>> {
        use crate::database::entities::dataset_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

        let rows = Entity::find()
            .filter(Column::DatasetNodeId.eq(dataset.id))
            .order_by_asc(Column::RowNumber)
            .all(&self.db)
            .await?;

        let mut nodes = HashMap::new();

        for row in rows {
            let data = row.data;
            let id = data["id"]
                .as_str()
                .ok_or_else(|| anyhow!("Node missing 'id' field"))?
                .to_string();

            let node = NodeData {
                label: data["label"].as_str().map(|s| s.to_string()),
                layer: data["layer"].as_str().map(|s| s.to_string()),
                weight: data["weight"].as_f64(),
                is_partition: parse_is_partition(&data["is_partition"]),
                belongs_to: data["belongs_to"].as_str().map(|s| s.to_string()),
                comment: data["comment"].as_str().map(|s| s.to_string()),
                attrs: Some(data.clone()),
                dataset_id: Some(dataset.id),
            };

            nodes.insert(id, node);
        }

        Ok(nodes)
    }

    /// Extract edges from an edges dataset
    #[allow(dead_code)]
    async fn extract_edges_from_dataset(&self, dataset: &datasets::Model) -> Result<Vec<EdgeData>> {
        use crate::database::entities::dataset_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

        let rows = Entity::find()
            .filter(Column::DatasetNodeId.eq(dataset.id))
            .order_by_asc(Column::RowNumber)
            .all(&self.db)
            .await?;

        let mut edges = Vec::new();

        for row in rows {
            let data = row.data;
            let id = data["id"]
                .as_str()
                .ok_or_else(|| anyhow!("Edge missing 'id' field"))?
                .to_string();
            let source = data["source"]
                .as_str()
                .ok_or_else(|| anyhow!("Edge missing 'source' field"))?
                .to_string();
            let target = data["target"]
                .as_str()
                .ok_or_else(|| anyhow!("Edge missing 'target' field"))?
                .to_string();

            let edge = EdgeData {
                id,
                source,
                target,
                label: data["label"].as_str().map(|s| s.to_string()),
                layer: data["layer"].as_str().map(|s| s.to_string()),
                weight: data["weight"].as_f64(),
                comment: data["comment"].as_str().map(|s| s.to_string()),
                attrs: Some(data.clone()),
                dataset_id: Some(dataset.id),
            };

            edges.push(edge);
        }

        Ok(edges)
    }

    /// Extract graph from JSON dataset
    #[allow(dead_code)]
    async fn extract_graph_from_json(
        &self,
        dataset: &datasets::Model,
    ) -> Result<(HashMap<String, NodeData>, Vec<EdgeData>)> {
        use crate::database::entities::dataset_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        // JSON graphs are stored as single row with row_number = 0
        let row = Entity::find()
            .filter(Column::DatasetNodeId.eq(dataset.id))
            .filter(Column::RowNumber.eq(0))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("JSON graph data not found"))?;

        let graph_data = row.data;

        // Extract nodes
        let mut nodes = HashMap::new();
        if let Some(nodes_array) = graph_data["nodes"].as_array() {
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
                    belongs_to: node_val["belongs_to"].as_str().map(|s| s.to_string()),
                    comment: node_val["comment"].as_str().map(|s| s.to_string()),
                    attrs: Some(node_val.clone()),
                    dataset_id: Some(dataset.id),
                };

                nodes.insert(id, node);
            }
        }

        // Extract edges
        let mut edges = Vec::new();
        if let Some(edges_array) = graph_data["edges"].as_array() {
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
                    comment: edge_val["comment"].as_str().map(|s| s.to_string()),
                    attrs: Some(edge_val.clone()),
                    dataset_id: Some(dataset.id),
                };

                edges.push(edge);
            }
        }

        Ok((nodes, edges))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn node_model_to_graph_json_preserves_comment_field() {
        let node = graph_nodes::Model {
            id: "node-1".to_string(),
            graph_id: 1,
            label: Some("Example".to_string()),
            layer: Some("app".to_string()),
            weight: Some(1.0),
            is_partition: false,
            belongs_to: None,
            comment: Some("keep me".to_string()),
            dataset_id: None,
            attrs: None,
            created_at: Utc::now(),
        };

        let json = node_model_to_graph_json(&node);
        assert_eq!(json["comment"].as_str(), Some("keep me"));
    }

    #[test]
    fn edge_model_to_graph_json_preserves_comment_field() {
        let edge = graph_edges::Model {
            id: "edge-1".to_string(),
            graph_id: 1,
            source: "node-1".to_string(),
            target: "node-2".to_string(),
            label: Some("Calls".to_string()),
            layer: Some("app".to_string()),
            weight: Some(1.0),
            comment: Some("retain me".to_string()),
            dataset_id: None,
            attrs: None,
            created_at: Utc::now(),
        };

        let json = edge_model_to_graph_json(&edge);
        assert_eq!(json["comment"].as_str(), Some("retain me"));
    }
}

/// Internal node data structure
struct NodeData {
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    is_partition: bool,
    belongs_to: Option<String>,
    comment: Option<String>,
    attrs: Option<Value>,
    dataset_id: Option<i32>,
}

/// Internal edge data structure
struct EdgeData {
    id: String,
    source: String,
    target: String,
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    comment: Option<String>,
    attrs: Option<Value>,
    dataset_id: Option<i32>,
}

fn allocate_edge_id(
    original_id: &str,
    dataset_id: Option<i32>,
    used_ids: &mut HashSet<String>,
) -> String {
    let mut candidate = original_id.to_string();
    if used_ids.insert(candidate.clone()) {
        return candidate;
    }

    let prefix = dataset_id
        .map(|id| format!("ds{}:", id))
        .unwrap_or_else(|| "edge:".to_string());
    let mut counter = 1;
    loop {
        candidate = format!("{}{}#{}", prefix, original_id, counter);
        if used_ids.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

// LayerData now imported from super::types (was previously defined here)
