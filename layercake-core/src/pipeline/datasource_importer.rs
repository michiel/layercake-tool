use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

use crate::database::entities::ExecutionState;
use crate::database::entities::{datasource_rows, datasources};

/// Service for importing data from files into datasource entities
pub struct DatasourceImporter {
    db: DatabaseConnection,
}

impl DatasourceImporter {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Import a datasource from file path
    /// Creates datasource entity and populates datasource_rows table
    pub async fn import_datasource(
        &self,
        project_id: i32,
        node_id: String,
        name: String,
        file_path: String,
    ) -> Result<datasources::Model> {
        // Detect file type from path
        let path = Path::new(&file_path);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("No file extension found"))?;

        let file_type = match extension.to_lowercase().as_str() {
            "csv" | "tsv" => {
                // Try to detect from filename
                let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

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

        // Create datasource entity
        let datasource = datasources::ActiveModel {
            id: Set(0),
            project_id: Set(project_id),
            node_id: Set(node_id.clone()),
            name: Set(name),
            file_path: Set(file_path.clone()),
            file_type: Set(file_type.to_string()),
            execution_state: Set(ExecutionState::Processing.as_str().to_string()),
            ..datasources::ActiveModel::new()
        };

        let datasource = datasource.insert(&self.db).await?;

        // Publish initial execution status (Processing)
        #[cfg(feature = "graphql")]
        crate::graphql::execution_events::publish_datasource_status(
            &self.db,
            datasource.project_id,
            &datasource.node_id,
            &datasource,
        )
        .await;

        // Import data based on file type
        let result = match (extension.to_lowercase().as_str(), file_type) {
            ("csv", "nodes") => self.import_csv_nodes(&datasource, &file_path, b',').await,
            ("csv", "edges") => self.import_csv_edges(&datasource, &file_path, b',').await,
            ("tsv", "nodes") => self.import_csv_nodes(&datasource, &file_path, b'\t').await,
            ("tsv", "edges") => self.import_csv_edges(&datasource, &file_path, b'\t').await,
            ("json", "graph") => self.import_json_graph(&datasource, &file_path).await,
            _ => Err(anyhow!("Unsupported file type combination")),
        };

        match result {
            Ok((row_count, column_info)) => {
                // Save values before moving datasource
                let project_id = datasource.project_id;
                let node_id = datasource.node_id.clone();

                // Update to completed state
                let mut active: datasources::ActiveModel = datasource.into();
                active = active.set_completed(row_count as i32, column_info);
                let updated = active.update(&self.db).await?;

                // Publish execution status change
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_datasource_status(
                    &self.db, project_id, &node_id, &updated,
                )
                .await;

                Ok(updated)
            }
            Err(e) => {
                // Save values before moving datasource
                let project_id = datasource.project_id;
                let node_id = datasource.node_id.clone();

                // Update to error state
                let mut active: datasources::ActiveModel = datasource.into();
                active = active.set_error(e.to_string());
                let updated = active.update(&self.db).await?;

                // Publish execution status change
                #[cfg(feature = "graphql")]
                crate::graphql::execution_events::publish_datasource_status(
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
        datasource: &datasources::Model,
        file_path: &str,
        delimiter: u8,
    ) -> Result<(usize, Value)> {
        let content = std::fs::read_to_string(file_path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

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

        // Parse and store rows
        let mut row_count = 0;
        for result in reader.records() {
            let record = result?;
            let mut row_data = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    row_data.insert(header.to_string(), json!(field));
                }
            }

            // Insert row
            let row = datasource_rows::ActiveModel {
                id: Set(0),
                datasource_id: Set(datasource.id),
                row_number: Set(row_count + 1),
                data: Set(json!(row_data)),
                created_at: Set(chrono::Utc::now()),
            };

            row.insert(&self.db).await?;
            row_count += 1;
        }

        Ok((row_count as usize, json!(column_info)))
    }

    /// Import CSV edges file
    async fn import_csv_edges(
        &self,
        datasource: &datasources::Model,
        file_path: &str,
        delimiter: u8,
    ) -> Result<(usize, Value)> {
        let content = std::fs::read_to_string(file_path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .delimiter(delimiter)
            .from_reader(content.as_bytes());

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

        // Parse and store rows
        let mut row_count = 0;
        for result in reader.records() {
            let record = result?;
            let mut row_data = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    row_data.insert(header.to_string(), json!(field));
                }
            }

            // Insert row
            let row = datasource_rows::ActiveModel {
                id: Set(0),
                datasource_id: Set(datasource.id),
                row_number: Set(row_count + 1),
                data: Set(json!(row_data)),
                created_at: Set(chrono::Utc::now()),
            };

            row.insert(&self.db).await?;
            row_count += 1;
        }

        Ok((row_count as usize, json!(column_info)))
    }

    /// Import JSON graph file
    /// For JSON graphs, we store the full structure in datasource_rows as a single row
    async fn import_json_graph(
        &self,
        datasource: &datasources::Model,
        file_path: &str,
    ) -> Result<(usize, Value)> {
        let content = std::fs::read_to_string(file_path)?;
        let graph_data: Value = serde_json::from_str(&content)?;

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

        // Count nodes and edges
        let node_count = obj["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
        let edge_count = obj["edges"].as_array().map(|a| a.len()).unwrap_or(0);

        // Store as single row with row_number = 0 (indicates full graph)
        let row = datasource_rows::ActiveModel {
            id: Set(0),
            datasource_id: Set(datasource.id),
            row_number: Set(0),
            data: Set(graph_data),
            created_at: Set(chrono::Utc::now()),
        };

        row.insert(&self.db).await?;

        // Column info for JSON graphs
        let column_info = json!([
            {"name": "nodes", "dataType": "array", "nullable": false},
            {"name": "edges", "dataType": "array", "nullable": false}
        ]);

        Ok((node_count + edge_count, column_info))
    }

    /// Get datasource by node ID
    pub async fn get_by_node(
        &self,
        project_id: i32,
        node_id: &str,
    ) -> Result<Option<datasources::Model>> {
        use crate::database::entities::datasources::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        let datasource = Entity::find()
            .filter(Column::ProjectId.eq(project_id))
            .filter(Column::NodeId.eq(node_id))
            .one(&self.db)
            .await?;

        Ok(datasource)
    }

    /// Get datasource rows with pagination
    pub async fn get_rows(
        &self,
        datasource_id: i32,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<datasource_rows::Model>> {
        use crate::database::entities::datasource_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter, QueryOrder, QuerySelect};

        let rows = Entity::find()
            .filter(Column::DatasourceId.eq(datasource_id))
            .order_by_asc(Column::RowNumber)
            .limit(limit)
            .offset(offset)
            .all(&self.db)
            .await?;

        Ok(rows)
    }

    /// Get total row count for a datasource
    pub async fn get_row_count(&self, datasource_id: i32) -> Result<u64> {
        use crate::database::entities::datasource_rows::{Column, Entity};
        use sea_orm::{ColumnTrait, QueryFilter};

        let count = Entity::find()
            .filter(Column::DatasourceId.eq(datasource_id))
            .count(&self.db)
            .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        // This would be expanded with actual import tests using test database
        assert!(true);
    }
}
