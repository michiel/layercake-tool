use anyhow::{anyhow, Result};
use async_graphql::SimpleObject;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use std::collections::{HashMap, HashSet};

use crate::app_context::DataSetValidationSummary;
use crate::database::entities::common_types::{DataType, FileFormat};
use crate::database::entities::data_sets::{self};
use crate::database::entities::{
    dataset_graph_edges, dataset_graph_layers, dataset_graph_nodes, graph_data, graph_data_edges,
    graph_data_nodes,
};
use crate::database::entities::{plan_dag_edges, plan_dag_nodes, projects};
use crate::graph::{Edge, Graph, Layer, Node};
use crate::services::{file_type_detection, source_processing};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct DataSetAnnotation {
    pub title: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub body: String,
}

/// Service for managing DataSets with file processing capabilities
#[derive(Clone)]
pub struct DataSetService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct GraphSummaryData {
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub layers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GraphPageData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
    pub has_more: bool,
}

impl DataSetService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// DEPRECATED: Create a new DataSet from uploaded file data (old signature for compatibility)
    #[allow(dead_code)]
    pub async fn create_from_file_legacy(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sets::Model> {
        // Auto-detect format from filename
        let file_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;

        // Auto-detect type from filename (old behavior)
        let data_type = if filename.to_lowercase().contains("node") {
            DataType::Nodes
        } else if filename.to_lowercase().contains("edge") {
            DataType::Edges
        } else if filename.to_lowercase().contains("layer") {
            DataType::Layers
        } else if filename.to_lowercase().ends_with(".json") {
            DataType::Graph
        } else {
            return Err(anyhow!(
                "Cannot determine data type from filename: {}",
                filename
            ));
        };

        self.create_from_file(
            project_id,
            name,
            description,
            filename,
            file_format,
            file_data,
            Some(data_type),
        )
        .await
    }

    /// Create a new empty DataSet without file data
    pub async fn create_empty(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
    ) -> Result<data_sets::Model> {
        // Validate project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        let empty_graph_json = r#"{"nodes":[],"edges":[],"layers":[]}"#;

        // Create DataSet record without file data
        let data_set = data_sets::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            description: Set(description),
            file_format: Set("json".to_string()), // Use JSON as default format for empty datasets
            data_type: Set(DataType::Graph.as_ref().to_string()),
            origin: Set("manual_edit".to_string()),
            filename: Set(format!("{}.json", chrono::Utc::now().timestamp())),
            blob: Set(Vec::new()),
            file_size: Set(0),
            status: Set("active".to_string()),
            graph_json: Set(empty_graph_json.to_string()),
            processed_at: Set(Some(chrono::Utc::now())),
            ..data_sets::ActiveModel::new()
        };

        let data_set = data_set.insert(&self.db).await?;
        Ok(data_set)
    }

    /// Create a new DataSet from uploaded file data
    pub async fn create_from_file(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_format: FileFormat,
        file_data: Vec<u8>,
        tabular_data_type: Option<DataType>,
    ) -> Result<data_sets::Model> {
        // Validate project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        // Validate file extension matches declared format
        let detected_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;
        if detected_format != file_format {
            return Err(anyhow!(
                "File extension doesn't match declared format. Expected {}, got {}",
                file_format.as_ref(),
                detected_format.as_ref()
            ));
        }

        let resolved_data_type =
            Self::determine_data_type(&file_format, &file_data, tabular_data_type, None)?;

        // Create initial DataSet record
        let data_set = data_sets::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            description: Set(description),

            file_format: Set(file_format.as_ref().to_string()),
            data_type: Set(resolved_data_type.as_ref().to_string()),
            origin: Set("file_upload".to_string()),
            filename: Set(filename),
            blob: Set(file_data.clone()),
            file_size: Set(file_data.len() as i64),
            status: Set("processing".to_string()),
            graph_json: Set("{}".to_string()),
            annotations: Set(Some("[]".to_string())),
            ..data_sets::ActiveModel::new()
        };

        let data_set = data_set.insert(&self.db).await?;

        // Process the file
        let updated_data_set =
            match source_processing::process_file(&file_format, &resolved_data_type, &file_data)
                .await
            {
                Ok(graph_json) => {
                    // Update with successful processing
                    let mut active_model: data_sets::ActiveModel = data_set.into();
                    active_model.graph_json = Set(graph_json);
                    active_model.status = Set("active".to_string());
                    active_model.processed_at = Set(Some(chrono::Utc::now()));
                    active_model.updated_at = Set(chrono::Utc::now());

                    active_model.update(&self.db).await?
                }
                Err(e) => {
                    // Update with error
                    let mut active_model: data_sets::ActiveModel = data_set.into();
                    active_model.status = Set("error".to_string());
                    active_model.error_message = Set(Some(e.to_string()));
                    active_model.updated_at = Set(chrono::Utc::now());

                    let _updated = active_model.update(&self.db).await?;
                    return Err(e);
                }
            };

        Ok(updated_data_set)
    }

    /// Get DataSet by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<data_sets::Model>> {
        let data_set = data_sets::Entity::find_by_id(id).one(&self.db).await?;
        Ok(data_set)
    }

    /// Get all DataSets for a project
    #[allow(dead_code)]
    pub async fn get_by_project(&self, project_id: i32) -> Result<Vec<data_sets::Model>> {
        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(data_sets)
    }

    /// Create a new DataSet with auto-detected data type from file content
    pub async fn create_with_auto_detect(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sets::Model> {
        // Auto-detect format from filename
        let file_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;

        self.create_from_file(
            project_id,
            name,
            description,
            filename,
            file_format,
            file_data,
            None,
        )
        .await
    }

    /// Update DataSet metadata
    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<data_sets::Model> {
        let data_set = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        let mut active_model: data_sets::ActiveModel = data_set.into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }
        if let Some(description) = description {
            active_model.description = Set(Some(description));
        }
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(&self.db).await?;
        Ok(updated)
    }

    /// Update DataSet graph data (graph_json) directly
    /// Updates processed_at timestamp to trigger downstream re-execution
    pub async fn update_graph_data(&self, id: i32, graph_json: String) -> Result<data_sets::Model> {
        let data_set = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        let mut parsed: Graph = serde_json::from_str(&graph_json)?;
        parsed.sanitize_labels();
        let sanitized_graph_json = serde_json::to_string(&parsed)?;
        let txn = self.db.begin().await?;
        // Legacy tables removed; keep dataset_graph_* cleanup for back-compat but ignore errors if tables are gone
        let _ = dataset_graph_nodes::Entity::delete_many()
            .filter(dataset_graph_nodes::Column::DatasetId.eq(id))
            .exec(&txn)
            .await;
        let _ = dataset_graph_edges::Entity::delete_many()
            .filter(dataset_graph_edges::Column::DatasetId.eq(id))
            .exec(&txn)
            .await;
        let _ = dataset_graph_layers::Entity::delete_many()
            .filter(dataset_graph_layers::Column::DatasetId.eq(id))
            .exec(&txn)
            .await;

        // Ensure a graph_data row exists for this dataset (id is reused)
        let gd_existing = graph_data::Entity::find_by_id(id).one(&txn).await?;
        if gd_existing.is_none() {
            graph_data::ActiveModel {
                id: Set(id),
                project_id: Set(data_set.project_id),
                name: Set(data_set.name.clone()),
                source_type: Set("dataset".to_string()),
                dag_node_id: Set(None),
                file_format: Set(Some(data_set.file_format.clone())),
                origin: Set(Some(data_set.origin.clone())),
                filename: Set(Some(data_set.filename.clone())),
                blob: Set(None),
                file_size: Set(Some(data_set.file_size)),
                processed_at: Set(Some(chrono::Utc::now())),
                computed_date: Set(None),
                status: Set("active".to_string()),
                source_hash: Set(None),
                node_count: Set(0),
                edge_count: Set(0),
                last_edit_sequence: Set(0),
                has_pending_edits: Set(false),
                last_replay_at: Set(None),
                metadata: Set(None),
                annotations: Set(None),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
                error_message: Set(None),
            }
            .insert(&txn)
            .await?;
        }

        // Replace graph_data nodes/edges for this dataset (delete edges first for FK safety)
        graph_data_edges::Entity::delete_many()
            .filter(graph_data_edges::Column::GraphDataId.eq(id))
            .exec(&txn)
            .await?;
        graph_data_nodes::Entity::delete_many()
            .filter(graph_data_nodes::Column::GraphDataId.eq(id))
            .exec(&txn)
            .await?;

        let now = chrono::Utc::now();
        for node in &parsed.nodes {
            let model = graph_data_nodes::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                graph_data_id: Set(id),
                external_id: Set(node.id.clone()),
                label: Set(Some(node.label.clone())),
                layer: Set(Some(node.layer.clone())),
                weight: Set(Some(node.weight as f64)),
                is_partition: Set(node.is_partition),
                belongs_to: Set(node.belongs_to.clone()),
                comment: Set(node.comment.clone()),
                source_dataset_id: Set(node.dataset),
                attributes: Set(node.attributes.clone()),
                created_at: Set(now),
            };
            model.insert(&txn).await?;
        }

        for edge in &parsed.edges {
            let model = graph_data_edges::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                graph_data_id: Set(id),
                external_id: Set(edge.id.clone()),
                source: Set(edge.source.clone()),
                target: Set(edge.target.clone()),
                label: Set(Some(edge.label.clone())),
                layer: Set(Some(edge.layer.clone())),
                weight: Set(Some(edge.weight as f64)),
                comment: Set(edge.comment.clone()),
                source_dataset_id: Set(edge.dataset),
                attributes: Set(edge.attributes.clone()),
                created_at: Set(now),
            };
            model.insert(&txn).await?;
        }

        let mut active_model: data_sets::ActiveModel = data_set.into();
        active_model.graph_json = Set(sanitized_graph_json);
        active_model.processed_at = Set(Some(chrono::Utc::now()));
        active_model.updated_at = Set(chrono::Utc::now());

        // Update graph_data metadata counts/status
        let mut gd_active: graph_data::ActiveModel = graph_data::Entity::find_by_id(id)
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("graph_data missing for dataset {}", id))?
            .into();
        gd_active.node_count = Set(parsed.nodes.len() as i32);
        gd_active.edge_count = Set(parsed.edges.len() as i32);
        gd_active.status = Set("active".to_string());
        gd_active.updated_at = Set(now);
        gd_active.processed_at = Set(Some(now));
        gd_active.computed_date = Set(Some(now));
        gd_active.update(&txn).await?;

        let updated = active_model.update(&txn).await?;
        txn.commit().await?;
        Ok(updated)
    }

    /// Append an annotation (graph_json untouched)
    pub async fn update_annotation(
        &self,
        id: i32,
        title: String,
        body: String,
    ) -> Result<data_sets::Model> {
        let data_set = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        let mut active_model: data_sets::ActiveModel = data_set.into();
        let mut annotations: Vec<DataSetAnnotation> = active_model
            .annotations
            .clone()
            .into_value()
            .and_then(|v| match v {
                sea_orm::Value::String(Some(s)) => serde_json::from_str(&s).ok(),
                _ => None,
            })
            .unwrap_or_default();
        annotations.push(DataSetAnnotation {
            title,
            date: chrono::Utc::now(),
            body,
        });
        active_model.annotations = Set(Some(serde_json::to_string(&annotations)?));
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(&self.db).await?;
        Ok(updated)
    }

    pub async fn get_graph_summary(&self, dataset_id: i32) -> Result<GraphSummaryData> {
        let layer_rows = dataset_graph_layers::Entity::find()
            .filter(dataset_graph_layers::Column::DatasetId.eq(dataset_id))
            .all(&self.db)
            .await?;
        let node_count = dataset_graph_nodes::Entity::find()
            .filter(dataset_graph_nodes::Column::DatasetId.eq(dataset_id))
            .count(&self.db)
            .await?;
        let edge_count = dataset_graph_edges::Entity::find()
            .filter(dataset_graph_edges::Column::DatasetId.eq(dataset_id))
            .count(&self.db)
            .await?;
        if node_count > 0 || edge_count > 0 {
            let mut layers: Vec<String> = layer_rows.into_iter().map(|l| l.id).collect();
            layers.sort();
            return Ok(GraphSummaryData {
                node_count: node_count as usize,
                edge_count: edge_count as usize,
                layer_count: layers.len(),
                layers,
            });
        }

        let model = data_sets::Entity::find_by_id(dataset_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;
        let graph: Graph = serde_json::from_str(&model.graph_json)?;
        let mut layers: Vec<String> = graph.layers.iter().map(|l| l.id.clone()).collect();
        layers.sort();
        Ok(GraphSummaryData {
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            layer_count: graph.layers.len(),
            layers,
        })
    }

    pub async fn get_graph_page(
        &self,
        dataset_id: i32,
        limit: usize,
        offset: usize,
        filter_layers: Option<Vec<String>>,
    ) -> Result<GraphPageData> {
        let filter_set: Option<HashSet<String>> =
            filter_layers.clone().map(|v| v.into_iter().collect());
        let mut node_query = dataset_graph_nodes::Entity::find()
            .filter(dataset_graph_nodes::Column::DatasetId.eq(dataset_id))
            .order_by_asc(dataset_graph_nodes::Column::Id)
            .limit(limit as u64)
            .offset(offset as u64);
        if let Some(filter) = &filter_set {
            node_query =
                node_query.filter(dataset_graph_nodes::Column::Layer.is_in(filter.clone()));
        }
        let nodes_rows = node_query.all(&self.db).await?;
        let total_rows = dataset_graph_nodes::Entity::find()
            .filter(dataset_graph_nodes::Column::DatasetId.eq(dataset_id))
            .count(&self.db)
            .await? as usize;

        if !nodes_rows.is_empty() {
            let node_ids: HashSet<String> = nodes_rows.iter().map(|n| n.id.clone()).collect();
            let mut edges_rows = dataset_graph_edges::Entity::find()
                .filter(dataset_graph_edges::Column::DatasetId.eq(dataset_id))
                .all(&self.db)
                .await?;
            if let Some(filter) = &filter_set {
                edges_rows.retain(|e| filter.contains(&e.layer));
            }
            edges_rows.retain(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target));

            let mut layers_rows = dataset_graph_layers::Entity::find()
                .filter(dataset_graph_layers::Column::DatasetId.eq(dataset_id))
                .all(&self.db)
                .await?;
            if let Some(filter) = &filter_set {
                layers_rows.retain(|l| filter.contains(&l.id));
            }

            let nodes = nodes_rows
                .into_iter()
                .map(|n| Node {
                    id: n.id,
                    label: n.label,
                    layer: n.layer,
                    belongs_to: n.belongs_to,
                    weight: n.weight,
                    is_partition: n.is_partition,
                    comment: n.comment,
                    dataset: n.dataset,
                    attributes: n.attributes,
                })
                .collect();
            let edges = edges_rows
                .into_iter()
                .map(|e| Edge {
                    id: e.id,
                    source: e.source,
                    target: e.target,
                    label: e.label,
                    layer: e.layer,
                    weight: e.weight,
                    comment: e.comment,
                    dataset: e.dataset,
                    attributes: e.attributes,
                })
                .collect();
            let layers = layers_rows
                .into_iter()
                .map(|l| Layer {
                    id: l.id,
                    label: l.label,
                    background_color: l.background_color,
                    text_color: l.text_color,
                    border_color: l.border_color,
                    alias: None,
                    dataset: None,
                    attributes: None,
                })
                .collect();

            return Ok(GraphPageData {
                nodes,
                edges,
                layers,
                has_more: offset + limit < total_rows,
            });
        }

        let model = data_sets::Entity::find_by_id(dataset_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;
        let mut graph: Graph = serde_json::from_str(&model.graph_json)?;

        if let Some(layers) = filter_layers.clone() {
            let set: HashSet<String> = layers.into_iter().collect();
            graph.nodes.retain(|n| set.contains(&n.layer));
            graph.edges.retain(|e| set.contains(&e.layer));
            graph.layers.retain(|l| set.contains(&l.id));
        }

        graph.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        graph.edges.sort_by(|a, b| a.id.cmp(&b.id));
        let total = graph.nodes.len();
        let slice = graph.nodes.iter().skip(offset).take(limit);
        let node_ids: HashSet<String> = slice.clone().map(|n| n.id.clone()).collect();

        let nodes: Vec<Node> = slice.cloned().collect();
        let edges: Vec<Edge> = graph
            .edges
            .into_iter()
            .filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target))
            .collect();
        let has_more = offset + nodes.len() < total;

        Ok(GraphPageData {
            nodes,
            edges,
            layers: graph.layers,
            has_more,
        })
    }

    /// Update DataSet file and reprocess
    pub async fn update_file(
        &self,
        id: i32,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sets::Model> {
        let data_set = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        // Detect format from filename extension
        let file_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;

        // Determine appropriate data type (respect prior type if detection fails)
        let existing_data_type = data_set
            .get_data_type()
            .ok_or_else(|| anyhow!("Invalid data type in existing data source"))?;
        let resolved_data_type =
            Self::determine_data_type(&file_format, &file_data, None, Some(existing_data_type))?;

        // Update with new file data
        let mut active_model: data_sets::ActiveModel = data_set.into();
        active_model.filename = Set(filename);
        active_model.blob = Set(file_data.clone());
        active_model.file_size = Set(file_data.len() as i64);
        active_model.file_format = Set(file_format.as_ref().to_string());
        active_model.data_type = Set(resolved_data_type.as_ref().to_string());
        active_model.status = Set("processing".to_string());
        active_model.error_message = Set(None);
        active_model.updated_at = Set(chrono::Utc::now());

        let data_set = active_model.update(&self.db).await?;

        // Process the new file
        let updated_data_set =
            match source_processing::process_file(&file_format, &resolved_data_type, &file_data)
                .await
            {
                Ok(graph_json) => {
                    let mut active_model: data_sets::ActiveModel = data_set.into();
                    active_model.graph_json = Set(graph_json);
                    active_model.status = Set("active".to_string());
                    active_model.processed_at = Set(Some(chrono::Utc::now()));
                    active_model.updated_at = Set(chrono::Utc::now());

                    active_model.update(&self.db).await?
                }
                Err(e) => {
                    let mut active_model: data_sets::ActiveModel = data_set.into();
                    active_model.status = Set("error".to_string());
                    active_model.error_message = Set(Some(e.to_string()));
                    active_model.updated_at = Set(chrono::Utc::now());

                    let _updated = active_model.update(&self.db).await?;
                    return Err(e);
                }
            };

        Ok(updated_data_set)
    }

    /// Delete DataSet and clean up related plan DAG nodes
    pub async fn delete(&self, id: i32) -> Result<()> {
        let data_set = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        // Find and delete all plan_dag_nodes that reference this dataset
        let all_dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::NodeType.eq("DataSetNode"))
            .all(&self.db)
            .await?;

        for dag_node in all_dag_nodes {
            // Parse config to check if it references this dataset
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&dag_node.config_json) {
                if let Some(ds_id) = config.get("dataSetId").and_then(|v| v.as_i64()) {
                    if ds_id as i32 == data_set.id {
                        // Delete connected edges first
                        plan_dag_edges::Entity::delete_many()
                            .filter(plan_dag_edges::Column::SourceNodeId.eq(&dag_node.id))
                            .exec(&self.db)
                            .await?;

                        plan_dag_edges::Entity::delete_many()
                            .filter(plan_dag_edges::Column::TargetNodeId.eq(&dag_node.id))
                            .exec(&self.db)
                            .await?;

                        // Delete the node
                        plan_dag_nodes::Entity::delete_by_id(&dag_node.id)
                            .exec(&self.db)
                            .await?;
                    }
                }
            }
        }

        // Delete the dataset itself
        data_sets::Entity::delete_by_id(data_set.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Reprocess existing DataSet file
    pub async fn reprocess(&self, id: i32) -> Result<data_sets::Model> {
        let data_set = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        let file_format = data_set
            .get_file_format()
            .ok_or_else(|| anyhow!("Invalid file format"))?;
        let existing_data_type = data_set
            .get_data_type()
            .ok_or_else(|| anyhow!("Invalid data type"))?;
        let resolved_data_type = Self::determine_data_type(
            &file_format,
            &data_set.blob,
            None,
            Some(existing_data_type),
        )?;

        // Set to processing status
        let mut active_model: data_sets::ActiveModel = data_set.into();
        active_model.status = Set("processing".to_string());
        active_model.error_message = Set(None);
        active_model.data_type = Set(resolved_data_type.as_ref().to_string());
        active_model.updated_at = Set(chrono::Utc::now());

        let data_set = active_model.update(&self.db).await?;

        // Process the file
        let updated_data_set = match source_processing::process_file(
            &file_format,
            &resolved_data_type,
            &data_set.blob,
        )
        .await
        {
            Ok(graph_json) => {
                let mut active_model: data_sets::ActiveModel = data_set.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                active_model.update(&self.db).await?
            }
            Err(e) => {
                let mut active_model: data_sets::ActiveModel = data_set.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                let _updated = active_model.update(&self.db).await?;
                return Err(e);
            }
        };

        Ok(updated_data_set)
    }

    /// Validate the stored graph JSON for a dataset
    pub async fn validate(&self, id: i32) -> Result<DataSetValidationSummary> {
        let model = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSet not found"))?;

        let mut graph: Graph = if model.graph_json.trim().is_empty() {
            Graph::default()
        } else {
            serde_json::from_str(&model.graph_json)
                .map_err(|e| anyhow!("Failed to parse graph JSON: {}", e))?
        };

        if graph.name.is_empty() {
            graph.name = model.name.clone();
        }

        if graph.layers.is_empty() {
            let mut seen = HashSet::new();
            for node in &graph.nodes {
                if node.layer.is_empty() {
                    continue;
                }
                if seen.insert(node.layer.clone()) {
                    graph.layers.push(Layer::new(
                        &node.layer,
                        &node.layer,
                        "f2f4f7",
                        "0f172a",
                        "d0d5dd",
                    ));
                }
            }
        }

        let mut errors = Vec::new();
        let warnings = Vec::new();

        if let Err(mut validation_errors) = graph.verify_graph_integrity() {
            errors.append(&mut validation_errors);
        }

        let partition_lookup: HashMap<_, _> = graph
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.is_partition))
            .collect();

        for node in &graph.nodes {
            if let Some(parent_id) = &node.belongs_to {
                if let Some(is_partition) = partition_lookup.get(parent_id) {
                    if !is_partition {
                        errors.push(format!(
                            "Node id:[{}] belongs_to {} but parent is not marked as a partition node",
                            node.id, parent_id
                        ));
                    }
                }
            }
        }

        Ok(DataSetValidationSummary {
            data_set_id: model.id,
            project_id: model.project_id,
            is_valid: errors.is_empty(),
            errors,
            warnings,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            layer_count: graph.layers.len(),
            checked_at: chrono::Utc::now(),
        })
    }

    fn determine_data_type(
        file_format: &FileFormat,
        file_data: &[u8],
        manual_hint: Option<DataType>,
        fallback: Option<DataType>,
    ) -> Result<DataType> {
        match file_format {
            FileFormat::Csv | FileFormat::Tsv => {
                if let Some(hint) = manual_hint {
                    if matches!(hint, DataType::Nodes | DataType::Edges | DataType::Layers) {
                        return Ok(hint);
                    } else {
                        return Err(anyhow!(
                            "Tabular uploads only support Nodes, Edges, or Layers data types"
                        ));
                    }
                }

                match file_type_detection::detect_data_type(file_format, file_data) {
                    Ok(detected) => Ok(detected),
                    Err(err) => {
                        if let Some(fallback_type) = fallback {
                            if fallback_type.is_compatible_with_format(file_format) {
                                Ok(fallback_type)
                            } else {
                                Err(anyhow!(
                                    "Existing data type {} is incompatible with {} format",
                                    fallback_type.as_ref(),
                                    file_format.as_ref()
                                ))
                            }
                        } else {
                            Err(anyhow!(
                                "Unable to detect data type for {} upload: {}",
                                file_format.as_ref(),
                                err
                            ))
                        }
                    }
                }
            }
            FileFormat::Json => Ok(DataType::Graph),
            other => Err(anyhow!(
                "{} uploads are not supported for this operation",
                other.as_ref()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_detection() {
        assert_eq!(
            FileFormat::from_extension("test.csv"),
            Some(FileFormat::Csv)
        );
        assert_eq!(
            FileFormat::from_extension("test.tsv"),
            Some(FileFormat::Tsv)
        );
        assert_eq!(
            FileFormat::from_extension("test.json"),
            Some(FileFormat::Json)
        );
        assert_eq!(FileFormat::from_extension("unknown.txt"), None);
    }

    #[test]
    fn test_data_type_compatibility() {
        assert!(DataType::Nodes.is_compatible_with_format(&FileFormat::Csv));
        assert!(DataType::Edges.is_compatible_with_format(&FileFormat::Tsv));
        assert!(DataType::Graph.is_compatible_with_format(&FileFormat::Json));
        assert!(!DataType::Graph.is_compatible_with_format(&FileFormat::Csv));
    }
}
