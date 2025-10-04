use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::database::entities::{datasources, graph_edges, graph_nodes, graphs};
use crate::database::entities::datasources::ExecutionState;

/// Service for building graphs from datasources
pub struct GraphBuilder {
    db: DatabaseConnection,
}

impl GraphBuilder {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Build a graph from upstream datasources
    /// Returns the created/updated graph entity
    pub async fn build_graph(
        &self,
        project_id: i32,
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

        // Fetch upstream datasources
        let mut datasources_list = Vec::new();
        for upstream_id in &upstream_node_ids {
            let ds = self.get_datasource(project_id, upstream_id).await?;
            if !ds.is_ready() {
                return Err(anyhow!(
                    "Upstream datasource {} is not ready (state: {})",
                    upstream_id,
                    ds.execution_state
                ));
            }
            datasources_list.push(ds);
        }

        // Compute source hash for change detection
        let source_hash = self.compute_source_hash(&datasources_list)?;

        // Check if recomputation is needed
        if let Some(existing_hash) = &graph.source_hash {
            if existing_hash == &source_hash {
                // No changes, return existing graph
                return Ok(graph);
            }
        }

        // Build graph data from datasources
        let result = self
            .build_graph_from_datasources(&graph, &datasources_list)
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

        // Create new
        let graph = graphs::ActiveModel {
            id: Set(0),
            project_id: Set(project_id),
            node_id: Set(node_id.to_string()),
            name: Set(name),
            ..graphs::ActiveModel::new()
        };

        let graph = graph.insert(&self.db).await?;
        Ok(graph)
    }

    /// Get datasource by node ID
    async fn get_datasource(
        &self,
        project_id: i32,
        node_id: &str,
    ) -> Result<datasources::Model> {
        use crate::database::entities::datasources::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::NodeId.eq(node_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Datasource not found for node {}", node_id))
    }

    /// Compute hash of upstream datasources for change detection
    fn compute_source_hash(&self, datasources: &[datasources::Model]) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        for ds in datasources {
            // Hash datasource ID, file path, and import date
            hasher.update(ds.id.to_string().as_bytes());
            hasher.update(ds.file_path.as_bytes());
            if let Some(import_date) = &ds.import_date {
                hasher.update(import_date.to_rfc3339().as_bytes());
            }
        }

        let hash = format!("{:x}", hasher.finalize());
        Ok(hash)
    }

    /// Build graph from datasources
    async fn build_graph_from_datasources(
        &self,
        graph: &graphs::Model,
        datasources: &[datasources::Model],
    ) -> Result<(usize, usize)> {
        // Clear existing graph data
        self.clear_graph_data(graph.id).await?;

        // Separate datasources by type
        let mut nodes_sources = Vec::new();
        let mut edges_sources = Vec::new();
        let mut graph_sources = Vec::new();

        for ds in datasources {
            match ds.file_type.as_str() {
                "nodes" => nodes_sources.push(ds),
                "edges" => edges_sources.push(ds),
                "graph" => graph_sources.push(ds),
                _ => {
                    return Err(anyhow!("Unknown datasource file type: {}", ds.file_type))
                }
            }
        }

        let mut all_nodes = HashMap::new();
        let mut all_edges = Vec::new();

        // Process JSON graph sources first (they contain both nodes and edges)
        for ds in graph_sources {
            let (nodes, edges) = self.extract_graph_from_json(ds).await?;
            for (id, node) in nodes {
                all_nodes.insert(id, node);
            }
            all_edges.extend(edges);
        }

        // Process node datasources
        for ds in nodes_sources {
            let nodes = self.extract_nodes_from_datasource(ds).await?;
            for (id, node) in nodes {
                all_nodes.insert(id, node);
            }
        }

        // Process edge datasources
        for ds in edges_sources {
            let edges = self.extract_edges_from_datasource(ds).await?;
            all_edges.extend(edges);
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

    /// Extract nodes from a nodes datasource
    async fn extract_nodes_from_datasource(
        &self,
        datasource: &datasources::Model,
    ) -> Result<HashMap<String, NodeData>> {
        use crate::database::entities::datasource_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

        let rows = Entity::find()
            .filter(Column::DatasourceId.eq(datasource.id))
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
                is_partition: data["is_partition"].as_bool().unwrap_or(false),
                attrs: Some(data.clone()),
            };

            nodes.insert(id, node);
        }

        Ok(nodes)
    }

    /// Extract edges from an edges datasource
    async fn extract_edges_from_datasource(
        &self,
        datasource: &datasources::Model,
    ) -> Result<Vec<EdgeData>> {
        use crate::database::entities::datasource_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter, QueryOrder};

        let rows = Entity::find()
            .filter(Column::DatasourceId.eq(datasource.id))
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
                attrs: Some(data.clone()),
            };

            edges.push(edge);
        }

        Ok(edges)
    }

    /// Extract graph from JSON datasource
    async fn extract_graph_from_json(
        &self,
        datasource: &datasources::Model,
    ) -> Result<(HashMap<String, NodeData>, Vec<EdgeData>)> {
        use crate::database::entities::datasource_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        // JSON graphs are stored as single row with row_number = 0
        let row = Entity::find()
            .filter(Column::DatasourceId.eq(datasource.id))
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
                    attrs: Some(node_val.clone()),
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
                    attrs: Some(edge_val.clone()),
                };

                edges.push(edge);
            }
        }

        Ok((nodes, edges))
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

/// Internal node data structure
struct NodeData {
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    is_partition: bool,
    attrs: Option<Value>,
}

/// Internal edge data structure
struct EdgeData {
    id: String,
    source: String,
    target: String,
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    attrs: Option<Value>,
}
