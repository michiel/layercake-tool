use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set};
use serde_json::{json, Map as JsonMap, Value};
use std::{collections::HashMap, mem, path::Path};

use crate::database::entities::graph_data;
use crate::database::entities::ExecutionState;
use crate::database::entities::{dataset_rows, datasets};
use crate::services::graph_data_service::{
    GraphDataCreate, GraphDataEdgeInput, GraphDataNodeInput, GraphDataService,
};
use sha2::{Digest, Sha256};

const DATASET_ROW_BATCH_SIZE: usize = 500;

/// Service for importing data from files into dataset entities
pub struct DatasourceImporter {
    db: DatabaseConnection,
    graph_data_service: std::sync::Arc<GraphDataService>,
    persist_dataset_rows: bool,
}

impl DatasourceImporter {
    pub fn new(
        db: DatabaseConnection,
        graph_data_service: std::sync::Arc<GraphDataService>,
    ) -> Self {
        let persist_dataset_rows = std::env::var("PIPELINE_PERSIST_DATASET_ROWS")
            .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Self {
            db,
            graph_data_service,
            persist_dataset_rows,
        }
    }

    /// Import a dataset from file path
    /// Creates dataset entity and populates dataset_rows table
    pub async fn import_dataset(
        &self,
        project_id: i32,
        node_id: String,
        name: String,
        file_path: String,
    ) -> Result<datasets::Model> {
        // Detect file type from path
        let path = Path::new(&file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("No file extension found"))?;
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        let file_type = match extension.to_lowercase().as_str() {
            "csv" | "tsv" => {
                // Try to detect from filename
                if filename.contains("node") {
                    "nodes"
                } else if filename.contains("edge") {
                    "edges"
                } else {
                    return Err(anyhow!("Cannot determine file type from filename: {}. Expected 'nodes' or 'edges' in filename", filename));
                }
            }
            "json" => "graph",
            _ => return Err(anyhow!("Unsupported file extension: {}", extension)),
        };

        // Read file bytes once for hashing and blob storage
        let file_bytes = std::fs::read(&file_path)?;
        let file_size = file_bytes.len() as i64;

        // Create dataset entity
        let dataset = datasets::ActiveModel {
            id: Set(0),
            project_id: Set(project_id),
            node_id: Set(node_id.clone()),
            name: Set(name.clone()),
            file_path: Set(file_path.clone()),
            file_type: Set(file_type.to_string()),
            execution_state: Set(ExecutionState::Processing.as_str().to_string()),
            ..datasets::ActiveModel::new()
        };

        let dataset = dataset.insert(&self.db).await?;

        // Create graph_data entry for this dataset import (processing state)
        let graph_data = self
            .graph_data_service
            .create(GraphDataCreate {
                project_id,
                name: name.clone(),
                source_type: "dataset".to_string(),
                dag_node_id: Some(node_id.clone()),
                file_format: Some(extension.to_lowercase()),
                origin: Some("file_path".to_string()),
                filename: Some(filename.to_string()),
                blob: Some(file_bytes.clone()),
                file_size: Some(file_size),
                processed_at: None,
                source_hash: None,
                computed_date: None,
                last_edit_sequence: Some(0),
                has_pending_edits: Some(false),
                last_replay_at: None,
                metadata: None,
                annotations: None,
                status: Some(graph_data::GraphDataStatus::Processing),
            })
            .await?;

        // Publish initial execution status (Processing)
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_dataset_status(
            &self.db,
            dataset.project_id,
            &dataset.node_id,
            &dataset,
        )
        .await;

        // Use shared content for parsing
        let file_content = String::from_utf8(file_bytes.clone())?;

        // Import data based on file type
        let result = match (extension.to_lowercase().as_str(), file_type) {
            ("csv", "nodes") => {
                self.import_csv_nodes(&dataset, &file_content, b',', &node_id)
                    .await
            }
            ("csv", "edges") => {
                self.import_csv_edges(&dataset, &file_content, b',', &node_id)
                    .await
            }
            ("tsv", "nodes") => {
                self.import_csv_nodes(&dataset, &file_content, b'\t', &node_id)
                    .await
            }
            ("tsv", "edges") => {
                self.import_csv_edges(&dataset, &file_content, b'\t', &node_id)
                    .await
            }
            ("json", "graph") => {
                self.import_json_graph(&dataset, &file_content, &node_id)
                    .await
            }
            _ => Err(anyhow!("Unsupported file type combination")),
        };

        match result {
            Ok((row_count, column_info, graph_nodes, graph_edges)) => {
                // Save values before moving dataset
                let project_id = dataset.project_id;
                let node_id = dataset.node_id.clone();

                // Persist graph_data contents and mark complete
                let source_hash = format!("{:x}", Sha256::digest(&file_bytes));
                if !graph_nodes.is_empty() {
                    self.graph_data_service
                        .replace_nodes(graph_data.id, graph_nodes)
                        .await?;
                }
                if !graph_edges.is_empty() {
                    self.graph_data_service
                        .replace_edges(graph_data.id, graph_edges)
                        .await?;
                }

                self.graph_data_service
                    .mark_complete(graph_data.id, source_hash)
                    .await?;

                // Update metadata and processed_at
                let mut graph_active: graph_data::ActiveModel = graph_data.into();
                graph_active.metadata = Set(Some(json!({
                    "columns": column_info,
                    "fileType": file_type,
                })));
                graph_active.processed_at = Set(Some(chrono::Utc::now()));
                graph_active.updated_at = Set(chrono::Utc::now());
                graph_active.update(&self.db).await?;

                // Update to completed state
                let mut active: datasets::ActiveModel = dataset.into();
                active = active.set_completed(row_count as i32, column_info);
                let updated = active.update(&self.db).await?;

                // Publish execution status change
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_dataset_status(
                    &self.db, project_id, &node_id, &updated,
                )
                .await;

                Ok(updated)
            }
            Err(e) => {
                // Save values before moving dataset
                let project_id = dataset.project_id;
                let node_id = dataset.node_id.clone();

                // Mark graph_data as error
                let mut graph_active: graph_data::ActiveModel = graph_data.into();
                graph_active.status = Set(graph_data::GraphDataStatus::Error.into());
                graph_active.error_message = Set(Some(e.to_string()));
                graph_active.updated_at = Set(chrono::Utc::now());
                let _ = graph_active.update(&self.db).await;

                // Update to error state
                let mut active: datasets::ActiveModel = dataset.into();
                active = active.set_error(e.to_string());
                let updated = active.update(&self.db).await?;

                // Publish execution status change
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_dataset_status(
                    &self.db, project_id, &node_id, &updated,
                )
                .await;

                Err(e)
            }
        }
    }

    /// Import CSV nodes file
    async fn import_csv_nodes(
        &self,
        dataset: &datasets::Model,
        file_content: &str,
        delimiter: u8,
        _dag_node_id: &str,
    ) -> Result<(
        usize,
        Value,
        Vec<GraphDataNodeInput>,
        Vec<GraphDataEdgeInput>,
    )> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
            .from_reader(file_content.as_bytes());

        let headers = reader.headers()?.clone();

        // Validate required columns
        if !headers.iter().any(|h| h == "id") {
            return Err(anyhow!("CSV must contain 'id' column"));
        }

        // Build column info
        let mut column_info = Vec::new();
        for header in headers.iter() {
            column_info.push(json!({
                "name": header,
                "dataType": "string", // We could infer types but string is safe
                "nullable": true
            }));
        }

        // Parse and optionally store rows
        let persist_rows = self.persist_dataset_rows;
        let mut graph_nodes = Vec::new();
        let mut row_count = 0usize;
        let mut batch = Vec::with_capacity(DATASET_ROW_BATCH_SIZE);

        for result in reader.records() {
            let record = result?;
            row_count += 1;

            // Build graph_data node input
            let id_value = record
                .get(headers.iter().position(|h| h == "id").unwrap())
                .unwrap_or("")
                .to_string();

            let label = headers
                .iter()
                .position(|h| h == "label")
                .and_then(|idx| record.get(idx).map(|s| s.to_string()))
                .filter(|s| !s.is_empty())
                .or_else(|| Some(id_value.clone()));

            let layer = headers
                .iter()
                .position(|h| h == "layer")
                .and_then(|idx| record.get(idx).map(|s| s.to_string()))
                .filter(|s| !s.is_empty());

            let mut attributes = JsonMap::new();
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    // Skip id/label/layer since they map to dedicated fields
                    if header == "id" || header == "label" || header == "layer" {
                        continue;
                    }
                    attributes.insert(header.to_string(), json!(field));
                }
            }

            graph_nodes.push(GraphDataNodeInput {
                external_id: id_value,
                label,
                layer,
                weight: None,
                is_partition: None,
                belongs_to: None,
                comment: None,
                source_dataset_id: Some(dataset.id),
                attributes: if attributes.is_empty() {
                    None
                } else {
                    Some(Value::Object(attributes))
                },
                created_at: Some(chrono::Utc::now()),
            });

            if persist_rows {
                let mut row_data = HashMap::new();
                for (i, field) in record.iter().enumerate() {
                    if let Some(header) = headers.get(i) {
                        row_data.insert(header.to_string(), json!(field));
                    }
                }

                let row = dataset_rows::ActiveModel {
                    id: Set(0),
                    dataset_node_id: Set(dataset.id),
                    row_number: Set(row_count as i32),
                    data: Set(json!(row_data)),
                    created_at: Set(chrono::Utc::now()),
                };

                batch.push(row);
                if batch.len() >= DATASET_ROW_BATCH_SIZE {
                    self.flush_dataset_rows(&mut batch).await?;
                }
            }
        }

        if persist_rows {
            self.flush_dataset_rows(&mut batch).await?;
        }

        Ok((row_count, json!(column_info), graph_nodes, Vec::new()))
    }

    /// Import CSV edges file
    async fn import_csv_edges(
        &self,
        dataset: &datasets::Model,
        file_content: &str,
        delimiter: u8,
        _dag_node_id: &str,
    ) -> Result<(
        usize,
        Value,
        Vec<GraphDataNodeInput>,
        Vec<GraphDataEdgeInput>,
    )> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
            .from_reader(file_content.as_bytes());

        let headers = reader.headers()?.clone();

        // Validate required columns
        let required = ["id", "source", "target"];
        for req in &required {
            if !headers.iter().any(|h| h == *req) {
                return Err(anyhow!("CSV must contain '{}' column", req));
            }
        }

        // Build column info
        let mut column_info = Vec::new();
        for header in headers.iter() {
            column_info.push(json!({
                "name": header,
                "dataType": "string",
                "nullable": true
            }));
        }

        let persist_rows = self.persist_dataset_rows;
        let mut graph_edges = Vec::new();
        let mut row_count = 0usize;
        let mut batch = Vec::with_capacity(DATASET_ROW_BATCH_SIZE);

        for result in reader.records() {
            let record = result?;
            row_count += 1;

            let id_value = record
                .get(headers.iter().position(|h| h == "id").unwrap())
                .unwrap_or("")
                .to_string();
            let source = record
                .get(headers.iter().position(|h| h == "source").unwrap())
                .unwrap_or("")
                .to_string();
            let target = record
                .get(headers.iter().position(|h| h == "target").unwrap())
                .unwrap_or("")
                .to_string();

            let label = headers
                .iter()
                .position(|h| h == "label")
                .and_then(|idx| record.get(idx).map(|s| s.to_string()))
                .filter(|s| !s.is_empty());

            let layer = headers
                .iter()
                .position(|h| h == "layer")
                .and_then(|idx| record.get(idx).map(|s| s.to_string()))
                .filter(|s| !s.is_empty());

            let weight = headers
                .iter()
                .position(|h| h == "weight")
                .and_then(|idx| record.get(idx))
                .and_then(|s| s.parse::<f64>().ok());

            let mut attributes = JsonMap::new();
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    if header == "id" || header == "source" || header == "target" {
                        continue;
                    }
                    attributes.insert(header.to_string(), json!(field));
                }
            }

            graph_edges.push(GraphDataEdgeInput {
                external_id: id_value,
                source,
                target,
                label,
                layer,
                weight,
                comment: None,
                source_dataset_id: Some(dataset.id),
                attributes: if attributes.is_empty() {
                    None
                } else {
                    Some(Value::Object(attributes))
                },
                created_at: Some(chrono::Utc::now()),
            });

            if persist_rows {
                let mut row_data = HashMap::new();
                for (i, field) in record.iter().enumerate() {
                    if let Some(header) = headers.get(i) {
                        row_data.insert(header.to_string(), json!(field));
                    }
                }

                let row = dataset_rows::ActiveModel {
                    id: Set(0),
                    dataset_node_id: Set(dataset.id),
                    row_number: Set(row_count as i32),
                    data: Set(json!(row_data)),
                    created_at: Set(chrono::Utc::now()),
                };

                batch.push(row);
                if batch.len() >= DATASET_ROW_BATCH_SIZE {
                    self.flush_dataset_rows(&mut batch).await?;
                }
            }
        }

        if persist_rows {
            self.flush_dataset_rows(&mut batch).await?;
        }

        Ok((row_count, json!(column_info), Vec::new(), graph_edges))
    }

    /// Import JSON graph file
    /// For JSON graphs, we store the full structure in dataset_rows as a single row
    async fn import_json_graph(
        &self,
        dataset: &datasets::Model,
        file_content: &str,
        _dag_node_id: &str,
    ) -> Result<(
        usize,
        Value,
        Vec<GraphDataNodeInput>,
        Vec<GraphDataEdgeInput>,
    )> {
        let graph_data: Value = serde_json::from_str(file_content)?;

        // Validate structure
        if !graph_data.is_object() {
            return Err(anyhow!("JSON must be an object"));
        }

        let obj = graph_data
            .as_object()
            .ok_or_else(|| anyhow!("JSON data is not a valid object"))?;

        // Ensure required fields
        if !obj.contains_key("nodes") || !obj.contains_key("edges") {
            return Err(anyhow!("JSON must contain 'nodes' and 'edges' arrays"));
        }

        // Validate arrays
        if !obj["nodes"].is_array() || !obj["edges"].is_array() {
            return Err(anyhow!("'nodes' and 'edges' must be arrays"));
        }

        // Convert nodes/edges to graph_data inputs
        let mut graph_nodes = Vec::new();
        let mut graph_edges = Vec::new();

        if let Some(nodes) = obj["nodes"].as_array() {
            for node in nodes {
                let id = node["id"].as_str().unwrap_or_default().to_string();
                let label = node["label"]
                    .as_str()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .or_else(|| Some(id.clone()));
                let layer = node["layer"]
                    .as_str()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());
                let weight = node["weight"].as_f64();
                let attributes = node["attributes"].as_object().cloned().map(Value::Object);

                graph_nodes.push(GraphDataNodeInput {
                    external_id: id,
                    label,
                    layer,
                    weight,
                    is_partition: node["is_partition"].as_bool(),
                    belongs_to: node["belongs_to"].as_str().map(|s| s.to_string()),
                    comment: node["comment"].as_str().map(|s| s.to_string()),
                    source_dataset_id: Some(dataset.id),
                    attributes,
                    created_at: Some(chrono::Utc::now()),
                });
            }
        }

        if let Some(edges) = obj["edges"].as_array() {
            for edge in edges {
                let id = edge["id"].as_str().unwrap_or_default().to_string();
                let source = edge["source"].as_str().unwrap_or_default().to_string();
                let target = edge["target"].as_str().unwrap_or_default().to_string();
                let label = edge["label"]
                    .as_str()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());
                let layer = edge["layer"]
                    .as_str()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());
                let weight = edge["weight"].as_f64();
                let attributes = edge["attributes"].as_object().cloned().map(Value::Object);

                graph_edges.push(GraphDataEdgeInput {
                    external_id: id,
                    source,
                    target,
                    label,
                    layer,
                    weight,
                    comment: edge["comment"].as_str().map(|s| s.to_string()),
                    source_dataset_id: Some(dataset.id),
                    attributes,
                    created_at: Some(chrono::Utc::now()),
                });
            }
        }

        if self.persist_dataset_rows {
            let row = dataset_rows::ActiveModel {
                id: Set(0),
                dataset_node_id: Set(dataset.id),
                row_number: Set(0),
                data: Set(graph_data.clone()),
                created_at: Set(chrono::Utc::now()),
            };

            row.insert(&self.db).await?;
        }

        // Column info for JSON graphs
        let column_info = json!([
            {"name": "nodes", "dataType": "array", "nullable": false},
            {"name": "edges", "dataType": "array", "nullable": false}
        ]);

        Ok((
            graph_nodes.len() + graph_edges.len(),
            column_info,
            graph_nodes,
            graph_edges,
        ))
    }

    /// Get dataset by node ID
    #[allow(dead_code)]
    pub async fn get_by_node(
        &self,
        project_id: i32,
        node_id: &str,
    ) -> Result<Option<datasets::Model>> {
        use crate::database::entities::datasets::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        let dataset = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::NodeId.eq(node_id))
            .one(&self.db)
            .await?;

        Ok(dataset)
    }

    /// Get dataset rows with pagination
    #[allow(dead_code)]
    pub async fn get_rows(
        &self,
        dataset_id: i32,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<dataset_rows::Model>> {
        use crate::database::entities::dataset_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter, QueryOrder, QuerySelect};

        let rows = Entity::find()
            .filter(Column::DatasetNodeId.eq(dataset_id))
            .order_by_asc(Column::RowNumber)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok(rows)
    }

    /// Get total row count for a dataset
    #[allow(dead_code)]
    pub async fn get_row_count(&self, dataset_id: i32) -> Result<u64> {
        use crate::database::entities::dataset_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        let count = Entity::find()
            .filter(Column::DatasetNodeId.eq(dataset_id))
            .count(&self.db)
            .await?;

        Ok(count)
    }

    async fn flush_dataset_rows(&self, batch: &mut Vec<dataset_rows::ActiveModel>) -> Result<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let rows = mem::take(batch);
        dataset_rows::Entity::insert_many(rows)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_file_type_detection() {
        // This would be expanded with actual import tests using test database
        assert!(true);
    }
}
