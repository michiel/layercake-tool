use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::database::entities::data_sources::{self, DataType, FileFormat};
use crate::database::entities::{plan_dag_edges, plan_dag_nodes, projects};
use crate::services::file_type_detection;

/// Service for managing DataSources with file processing capabilities
#[derive(Clone)]
pub struct DataSourceService {
    db: DatabaseConnection,
}

impl DataSourceService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// DEPRECATED: Create a new DataSource from uploaded file data (old signature for compatibility)
    pub async fn create_from_file_legacy(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sources::Model> {
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
            data_type,
            file_data,
        )
        .await
    }

    /// Create a new DataSource from uploaded file data
    pub async fn create_from_file(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_format: FileFormat,
        data_type: DataType,
        file_data: Vec<u8>,
    ) -> Result<data_sources::Model> {
        // Validate project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        // Validate format and type combination
        if !data_type.is_compatible_with_format(&file_format) {
            return Err(anyhow!(
                "Invalid combination: {} format cannot contain {} data",
                file_format.as_str(),
                data_type.as_str()
            ));
        }

        // Validate file extension matches declared format
        let detected_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;
        if detected_format != file_format {
            return Err(anyhow!(
                "File extension doesn't match declared format. Expected {}, got {}",
                file_format.as_str(),
                detected_format.as_str()
            ));
        }

        // Create initial DataSource record
        let data_source = data_sources::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            description: Set(description),

            file_format: Set(file_format.as_str().to_string()),
            data_type: Set(data_type.as_str().to_string()),
            filename: Set(filename),
            blob: Set(file_data.clone()),
            file_size: Set(file_data.len() as i64),
            status: Set("processing".to_string()),
            graph_json: Set("{}".to_string()),
            ..data_sources::ActiveModel::new()
        };

        let data_source = data_source.insert(&self.db).await?;

        // Process the file
        let updated_data_source = match self
            .process_file(&file_format, &data_type, &file_data)
            .await
        {
            Ok(graph_json) => {
                // Update with successful processing
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                active_model.update(&self.db).await?
            }
            Err(e) => {
                // Update with error
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                let _updated = active_model.update(&self.db).await?;
                return Err(e);
            }
        };

        Ok(updated_data_source)
    }

    /// Get DataSource by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<data_sources::Model>> {
        let data_source = data_sources::Entity::find_by_id(id).one(&self.db).await?;
        Ok(data_source)
    }

    /// Get all DataSources for a project
    #[allow(dead_code)]
    pub async fn get_by_project(&self, project_id: i32) -> Result<Vec<data_sources::Model>> {
        let data_sources = data_sources::Entity::find()
            .filter(data_sources::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(data_sources)
    }

    /// Create a new DataSource with auto-detected data type from file content
    pub async fn create_with_auto_detect(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<data_sources::Model> {
        // Auto-detect format from filename
        let file_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;

        // Auto-detect data type from file content using heuristics
        let data_type = file_type_detection::detect_data_type(&file_format, &file_data)
            .unwrap_or_else(|_| {
                // Fallback to filename-based detection if content detection fails
                let lowercase = filename.to_lowercase();
                if lowercase.contains("layer") {
                    DataType::Layers
                } else if lowercase.contains("edge") {
                    DataType::Edges
                } else if lowercase.contains("node") {
                    DataType::Nodes
                } else if file_format == FileFormat::Json {
                    DataType::Graph
                } else {
                    DataType::Nodes // Default fallback
                }
            });

        self.create_from_file(
            project_id,
            name,
            description,
            filename,
            file_format,
            data_type,
            file_data,
        )
        .await
    }

    /// Update DataSource metadata
    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<data_sources::Model> {
        let data_source = self
            .get_by_id(id)
            .await?
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
        let data_source = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        // Detect format from filename extension
        let file_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;

        // Get existing data type from the data source
        let data_type = data_source
            .get_data_type()
            .ok_or_else(|| anyhow!("Invalid data type in existing data source"))?;

        // Validate format/type combination
        if !data_type.is_compatible_with_format(&file_format) {
            return Err(anyhow!(
                "Invalid combination: {} format cannot contain {} data",
                file_format.as_str(),
                data_type.as_str()
            ));
        }

        // Update with new file data
        let mut active_model: data_sources::ActiveModel = data_source.into();
        active_model.filename = Set(filename);
        active_model.blob = Set(file_data.clone());
        active_model.file_size = Set(file_data.len() as i64);
        active_model.file_format = Set(file_format.as_str().to_string());
        active_model.status = Set("processing".to_string());
        active_model.error_message = Set(None);
        active_model.updated_at = Set(chrono::Utc::now());

        let data_source = active_model.update(&self.db).await?;

        // Process the new file
        let updated_data_source = match self
            .process_file(&file_format, &data_type, &file_data)
            .await
        {
            Ok(graph_json) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                active_model.update(&self.db).await?
            }
            Err(e) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                let _updated = active_model.update(&self.db).await?;
                return Err(e);
            }
        };

        Ok(updated_data_source)
    }

    /// Delete DataSource and clean up related plan DAG nodes
    pub async fn delete(&self, id: i32) -> Result<()> {
        let data_source = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        // Find and delete all plan_dag_nodes that reference this datasource
        let all_dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::NodeType.eq("DataSourceNode"))
            .all(&self.db)
            .await?;

        for dag_node in all_dag_nodes {
            // Parse config to check if it references this datasource
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&dag_node.config_json) {
                if let Some(ds_id) = config.get("dataSourceId").and_then(|v| v.as_i64()) {
                    if ds_id as i32 == data_source.id {
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

        // Delete the datasource itself
        data_sources::Entity::delete_by_id(data_source.id)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Reprocess existing DataSource file
    pub async fn reprocess(&self, id: i32) -> Result<data_sources::Model> {
        let data_source = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("DataSource not found"))?;

        let file_format = data_source
            .get_file_format()
            .ok_or_else(|| anyhow!("Invalid file format"))?;
        let data_type = data_source
            .get_data_type()
            .ok_or_else(|| anyhow!("Invalid data type"))?;

        // Set to processing status
        let mut active_model: data_sources::ActiveModel = data_source.into();
        active_model.status = Set("processing".to_string());
        active_model.error_message = Set(None);
        active_model.updated_at = Set(chrono::Utc::now());

        let data_source = active_model.update(&self.db).await?;

        // Process the file
        let updated_data_source = match self
            .process_file(&file_format, &data_type, &data_source.blob)
            .await
        {
            Ok(graph_json) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.graph_json = Set(graph_json);
                active_model.status = Set("active".to_string());
                active_model.processed_at = Set(Some(chrono::Utc::now()));
                active_model.updated_at = Set(chrono::Utc::now());

                active_model.update(&self.db).await?
            }
            Err(e) => {
                let mut active_model: data_sources::ActiveModel = data_source.into();
                active_model.status = Set("error".to_string());
                active_model.error_message = Set(Some(e.to_string()));
                active_model.updated_at = Set(chrono::Utc::now());

                let _updated = active_model.update(&self.db).await?;
                return Err(e);
            }
        };

        Ok(updated_data_source)
    }

    /// Process uploaded file data into graph JSON
    async fn process_file(
        &self,
        file_format: &FileFormat,
        data_type: &DataType,
        file_data: &[u8],
    ) -> Result<String> {
        match (file_format, data_type) {
            (FileFormat::Csv, DataType::Nodes) => {
                self.process_delimited_nodes(file_data, b',').await
            }
            (FileFormat::Csv, DataType::Edges) => {
                self.process_delimited_edges(file_data, b',').await
            }
            (FileFormat::Csv, DataType::Layers) => {
                self.process_delimited_layers(file_data, b',').await
            }
            (FileFormat::Tsv, DataType::Nodes) => {
                self.process_delimited_nodes(file_data, b'\t').await
            }
            (FileFormat::Tsv, DataType::Edges) => {
                self.process_delimited_edges(file_data, b'\t').await
            }
            (FileFormat::Tsv, DataType::Layers) => {
                self.process_delimited_layers(file_data, b'\t').await
            }
            (FileFormat::Json, DataType::Graph) => self.process_json_graph(file_data).await,
            _ => Err(anyhow!("Invalid format/type combination")),
        }
    }

    /// Process delimited nodes file (CSV or TSV)
    async fn process_delimited_nodes(&self, file_data: &[u8], delimiter: u8) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
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
                        "id" => {
                            node.insert("id".to_string(), json!(field));
                        }
                        "label" => {
                            node.insert("label".to_string(), json!(field));
                        }
                        "layer" => {
                            if !field.is_empty() {
                                node.insert("layer".to_string(), json!(field));
                            }
                        }
                        "x" => {
                            if let Ok(x) = field.parse::<f64>() {
                                node.insert("x".to_string(), json!(x));
                            }
                        }
                        "y" => {
                            if let Ok(y) = field.parse::<f64>() {
                                node.insert("y".to_string(), json!(y));
                            }
                        }
                        _ => {
                            // Store as metadata, skip empty strings
                            if !field.is_empty() {
                                node.insert(header.to_string(), json!(field));
                            }
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

    /// Process delimited edges file (CSV or TSV)
    async fn process_delimited_edges(&self, file_data: &[u8], delimiter: u8) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
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
                        "id" | "source" | "target" => {
                            edge.insert(header.to_string(), json!(field));
                        }
                        "label" => {
                            if !field.is_empty() {
                                edge.insert(header.to_string(), json!(field));
                            }
                        }
                        "weight" => {
                            if let Ok(weight) = field.parse::<f64>() {
                                edge.insert("weight".to_string(), json!(weight));
                            }
                        }
                        _ => {
                            // Store as metadata, skip empty strings
                            if !field.is_empty() {
                                edge.insert(header.to_string(), json!(field));
                            }
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

    /// Process delimited layers file (CSV or TSV)
    async fn process_delimited_layers(&self, file_data: &[u8], delimiter: u8) -> Result<String> {
        let content = String::from_utf8(file_data.to_vec())?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

        let headers = reader.headers()?.clone();
        println!("CSV Headers: {:?}", headers);
        let mut layers = Vec::new();

        // Validate required headers: must have ('id' or 'layer') and 'label'
        let has_id_col = headers.iter().any(|h| h == "id" || h == "layer");
        let has_label_col = headers.iter().any(|h| h == "label");

        if !has_id_col || !has_label_col {
            return Err(anyhow!(
                "CSV must contain 'label' and either 'id' or 'layer' columns"
            ));
        }

        for result in reader.records() {
            let record = result?;
            println!("CSV Record: {:?}", record);
            let mut layer = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    let key = if header == "layer" { "id" } else { header };
                    match key {
                        "id" | "label" => {
                            layer.insert(key.to_string(), json!(field));
                        }
                        "description" => {
                            if !field.is_empty() {
                                layer.insert(key.to_string(), json!(field));
                            }
                        }
                        "color" | "background" => {
                            if !field.is_empty() {
                                layer.insert("background_color".to_string(), json!(field));
                            }
                        }
                        "border" => {
                            if !field.is_empty() {
                                layer.insert("border_color".to_string(), json!(field));
                            }
                        }
                        "text" => {
                            if !field.is_empty() {
                                layer.insert("text_color".to_string(), json!(field));
                            }
                        }
                        "z_index" => {
                            if let Ok(z) = field.parse::<i32>() {
                                layer.insert("z_index".to_string(), json!(z));
                            }
                        }
                        _ => {
                            // Store as metadata, skip empty strings
                            if !field.is_empty() {
                                layer.insert(key.to_string(), json!(field));
                            }
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

        let obj = graph_data
            .as_object()
            .ok_or_else(|| anyhow!("JSON data is not a valid object"))?;

        // Ensure required fields exist
        if !obj.contains_key("nodes") || !obj.contains_key("edges") || !obj.contains_key("layers") {
            return Err(anyhow!(
                "JSON must contain 'nodes', 'edges', and 'layers' arrays"
            ));
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
            DatabaseConnection::default(),
        );

        let csv_data =
            b"id,label,layer,x,y\nnode1,Node 1,layer1,10.5,20.5\nnode2,Node 2,layer1,30.0,40.0";

        // This test would work with proper database setup
        // let result = service.process_csv_nodes(csv_data).await;
        // assert!(result.is_ok());
    }

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
