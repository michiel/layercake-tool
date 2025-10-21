use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::database::entities::data_sources::{self, DataType, FileFormat};
use crate::database::entities::{plan_dag_edges, plan_dag_nodes, projects};
use crate::services::{file_type_detection, source_processing};

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
        let updated_data_source =
            match source_processing::process_file(&file_format, &data_type, &file_data).await {
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
        let updated_data_source =
            match source_processing::process_file(&file_format, &data_type, &file_data).await {
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
        let updated_data_source = match source_processing::process_file(
            &file_format,
            &data_type,
            &data_source.blob,
        )
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
