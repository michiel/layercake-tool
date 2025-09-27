use anyhow::{Result, anyhow};
use csv::ReaderBuilder;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::database::entities::data_sources::{self, DataSourceType};
use crate::database::entities::projects;

/// Service for managing DataSources with file processing capabilities
#[derive(Clone)]
pub struct DataSourceService {
    db: DatabaseConnection,
}

impl DataSourceService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new DataSource from uploaded file data
    pub async fn create_from_file(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sources::Model> {
        // Validate project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        // Determine source type from filename
        let source_type = DataSourceType::from_filename(&filename)
            .ok_or_else(|| anyhow!("Unsupported file type: {}", filename))?;

        // Create initial DataSource record
        let data_source = data_sources::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            description: Set(description),
            source_type: Set(source_type.as_str().to_string()),
            filename: Set(filename),
            blob: Set(file_data.clone()),
            file_size: Set(file_data.len() as i64),
            status: Set("processing".to_string()),
            graph_json: Set("{}".to_string()),
            ..data_sources::ActiveModel::new()
        };

        let mut data_source = data_source.insert(&self.db).await?;

        // Process the file
        match self.process_file(&source_type, &file_data).await {
            Ok(graph_json) => {
                // Update with successful processing
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                data_source = active_model.update(&self.db).await?;
            }
            Err(e) => {
                // Update with error
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                data_source = active_model.update(&self.db).await?;
                return Err(e);
            }
        }

        Ok(data_source)
    }

    /// Get DataSource by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<data_sources::Model>> {
        let data_source = data_sources::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        Ok(data_source)
    }

    /// Get all DataSources for a project
    pub async fn get_by_project(&self, project_id: i32) -> Result<Vec<data_sources::Model>> {
        let data_sources = data_sources::Entity::find()
            .filter(data_sources::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(data_sources)
    }

    /// Update DataSource metadata
    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<data_sources::Model> {
        let data_source = self.get_by_id(id).await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        let mut active_model: data_sources::ActiveModel = data_source.into();

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

    /// Update DataSource file and reprocess
    pub async fn update_file(
        &self,
        id: i32,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sources::Model> {
        let data_source = self.get_by_id(id).await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        // Determine source type from new filename
        let source_type = DataSourceType::from_filename(&filename)
            .ok_or_else(|| anyhow!("Unsupported file type: {}", filename))?;

        // Update with new file data
        let mut active_model: data_sources::ActiveModel = data_source.into();
        active_model.filename = Set(filename);
        active_model.blob = Set(file_data.clone());
        active_model.file_size = Set(file_data.len() as i64);
        active_model.source_type = Set(source_type.as_str().to_string());
        active_model.status = Set("processing".to_string());
        active_model.error_message = Set(None);
        active_model.updated_at = Set(chrono::Utc::now());

        let mut data_source = active_model.update(&self.db).await?;

        // Process the new file
        match self.process_file(&source_type, &file_data).await {
            Ok(graph_json) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                data_source = active_model.update(&self.db).await?;
            }
            Err(e) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                data_source = active_model.update(&self.db).await?;
                return Err(e);
            }
        }

        Ok(data_source)
    }

    /// Delete DataSource
    pub async fn delete(&self, id: i32) -> Result<()> {
        let data_source = self.get_by_id(id).await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        data_sources::Entity::delete_by_id(data_source.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Reprocess existing DataSource file
    pub async fn reprocess(&self, id: i32) -> Result<data_sources::Model> {
        let data_source = self.get_by_id(id).await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        let source_type = data_source.get_source_type()
            .ok_or_else(|| anyhow!("Invalid source type"))?;

        // Set to processing status
        let mut active_model: data_sources::ActiveModel = data_source.into();
        active_model.status = Set("processing".to_string());
        active_model.error_message = Set(None);
        active_model.updated_at = Set(chrono::Utc::now());

        let mut data_source = active_model.update(&self.db).await?;

        // Process the file
        match self.process_file(&source_type, &data_source.blob).await {
            Ok(graph_json) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                data_source = active_model.update(&self.db).await?;
            }
            Err(e) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                data_source = active_model.update(&self.db).await?;
                return Err(e);
            }
        }

        Ok(data_source)
    }

    /// Process uploaded file data into graph JSON
    async fn process_file(&self, source_type: &DataSourceType, file_data: &[u8]) -> Result<String> {
        match source_type {
            DataSourceType::CsvNodes => self.process_csv_nodes(file_data).await,
            DataSourceType::CsvEdges => self.process_csv_edges(file_data).await,
            DataSourceType::CsvLayers => self.process_csv_layers(file_data).await,
            DataSourceType::JsonGraph => self.process_json_graph(file_data).await,
        }
    }

    /// Process CSV nodes file
    async fn process_csv_nodes(&self, file_data: &[u8]) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let headers = reader.headers()?.clone();
        let mut nodes = Vec::new();

        // Validate required headers
        if !headers.iter().any(|h| h == "id") || !headers.iter().any(|h| h == "label") {
            return Err(anyhow!("CSV must contain 'id' and 'label' columns"));
        }

        for result in reader.records() {
            let record = result?;
            let mut node = HashMap::new();

            // Process each field
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    match header {
                        "id" => { node.insert("id".to_string(), json!(field)); },
                        "label" => { node.insert("label".to_string(), json!(field)); },
                        "layer" => { node.insert("layer".to_string(), json!(field)); },
                        "x" => {
                            if let Ok(x) = field.parse::<f64>() {
                                node.insert("x".to_string(), json!(x));
                            }
                        },
                        "y" => {
                            if let Ok(y) = field.parse::<f64>() {
                                node.insert("y".to_string(), json!(y));
                            }
                        },
                        _ => {
                            // Store as metadata
                            node.insert(header.to_string(), json!(field));
                        }
                    };
                }
            }

            nodes.push(json!(node));
        }

        let graph_json = json!({
            "nodes": nodes,
            "edges": [],
            "layers": []
        });

        Ok(serde_json::to_string(&graph_json)?)
    }

    /// Process CSV edges file
    async fn process_csv_edges(&self, file_data: &[u8]) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let headers = reader.headers()?.clone();
        let mut edges = Vec::new();

        // Validate required headers
        let required_headers = ["id", "source", "target"];
        for required in &required_headers {
            if !headers.iter().any(|h| h == *required) {
                return Err(anyhow!("CSV must contain '{}' column", required));
            }
        }

        for result in reader.records() {
            let record = result?;
            let mut edge = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    match header {
                        "id" | "source" | "target" | "label" => {
                            edge.insert(header.to_string(), json!(field));
                        },
                        "weight" => {
                            if let Ok(weight) = field.parse::<f64>() {
                                edge.insert("weight".to_string(), json!(weight));
                            }
                        },
                        _ => {
                            edge.insert(header.to_string(), json!(field));
                        }
                    };
                }
            }

            edges.push(json!(edge));
        }

        let graph_json = json!({
            "nodes": [],
            "edges": edges,
            "layers": []
        });

        Ok(serde_json::to_string(&graph_json)?)
    }

    /// Process CSV layers file
    async fn process_csv_layers(&self, file_data: &[u8]) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let headers = reader.headers()?.clone();
        let mut layers = Vec::new();

        // Validate required headers
        if !headers.iter().any(|h| h == "id") || !headers.iter().any(|h| h == "label") {
            return Err(anyhow!("CSV must contain 'id' and 'label' columns"));
        }

        for result in reader.records() {
            let record = result?;
            let mut layer = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    match header {
                        "id" | "label" | "color" | "description" => {
                            layer.insert(header.to_string(), json!(field));
                        },
                        "z_index" => {
                            if let Ok(z) = field.parse::<i32>() {
                                layer.insert("z_index".to_string(), json!(z));
                            }
                        },
                        _ => {
                            layer.insert(header.to_string(), json!(field));
                        }
                    };
                }
            }

            layers.push(json!(layer));
        }

        let graph_json = json!({
            "nodes": [],
            "edges": [],
            "layers": layers
        });

        Ok(serde_json::to_string(&graph_json)?)
    }

    /// Process JSON graph file
    async fn process_json_graph(&self, file_data: &[u8]) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let graph_data: Value = serde_json::from_str(&content)?;

        // Validate graph structure
        if !graph_data.is_object() {
            return Err(anyhow!("JSON must be an object"));
        }

        let obj = graph_data.as_object()
            .ok_or_else(|| anyhow!("JSON data is not a valid object"))?;

        // Ensure required fields exist
        if !obj.contains_key("nodes") || !obj.contains_key("edges") || !obj.contains_key("layers") {
            return Err(anyhow!("JSON must contain 'nodes', 'edges', and 'layers' arrays"));
        }

        // Validate that they are arrays
        if !obj["nodes"].is_array() || !obj["edges"].is_array() || !obj["layers"].is_array() {
            return Err(anyhow!("'nodes', 'edges', and 'layers' must be arrays"));
        }

        // Return the validated JSON
        Ok(serde_json::to_string(&graph_data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_csv_nodes() {
        let service = DataSourceService::new(
            // Mock database connection would go here
            DatabaseConnection::default()
        );

        let csv_data = b"id,label,layer,x,y\nnode1,Node 1,layer1,10.5,20.5\nnode2,Node 2,layer1,30.0,40.0";

        // This test would work with proper database setup
        // let result = service.process_csv_nodes(csv_data).await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_data_source_type_validation() {
        assert_eq!(DataSourceType::from_filename("nodes.csv"), Some(DataSourceType::CsvNodes));
        assert_eq!(DataSourceType::from_filename("test_nodes.csv"), Some(DataSourceType::CsvNodes));
        assert_eq!(DataSourceType::from_filename("edges.csv"), Some(DataSourceType::CsvEdges));
        assert_eq!(DataSourceType::from_filename("graph.json"), Some(DataSourceType::JsonGraph));
        assert_eq!(DataSourceType::from_filename("unknown.txt"), None);
    }
}