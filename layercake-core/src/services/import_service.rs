use anyhow::Result;
use csv::ReaderBuilder;
use sea_orm::{DatabaseConnection, ActiveModelTrait, Set};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::database::entities::{
    nodes, edges, layers,
};

#[allow(dead_code)] // Reserved for future CSV import features
#[derive(Debug)]
pub struct CsvImportData {
    pub nodes_csv: Option<String>,
    pub edges_csv: Option<String>, 
    pub layers_csv: Option<String>,
}

pub struct ImportService {
    db: DatabaseConnection,
}

impl ImportService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Import nodes from CSV content (convenience method for MCP)
    pub async fn import_nodes_from_csv(&self, project_id: i32, csv_content: &str) -> Result<usize> {
        self.import_nodes(project_id, csv_content).await
    }

    /// Import edges from CSV content (convenience method for MCP)
    pub async fn import_edges_from_csv(&self, project_id: i32, csv_content: &str) -> Result<usize> {
        self.import_edges(project_id, csv_content).await
    }

    /// Import layers from CSV content (convenience method for MCP)
    pub async fn import_layers_from_csv(&self, project_id: i32, csv_content: &str) -> Result<usize> {
        self.import_layers(project_id, csv_content).await
    }

    #[allow(dead_code)] // Reserved for future CSV import features
    pub async fn import_csv_data(&self, project_id: i32, data: CsvImportData) -> Result<ImportResult> {
        let mut result = ImportResult {
            nodes_imported: 0,
            edges_imported: 0,
            layers_imported: 0,
            errors: Vec::new(),
        };

        // Import layers first (nodes may reference them)
        if let Some(layers_csv) = data.layers_csv {
            match self.import_layers(project_id, &layers_csv).await {
                Ok(count) => result.layers_imported = count,
                Err(e) => result.errors.push(format!("Layers import failed: {}", e)),
            }
        }

        // Import nodes
        if let Some(nodes_csv) = data.nodes_csv {
            match self.import_nodes(project_id, &nodes_csv).await {
                Ok(count) => result.nodes_imported = count,
                Err(e) => result.errors.push(format!("Nodes import failed: {}", e)),
            }
        }

        // Import edges last (they reference nodes)
        if let Some(edges_csv) = data.edges_csv {
            match self.import_edges(project_id, &edges_csv).await {
                Ok(count) => result.edges_imported = count,
                Err(e) => result.errors.push(format!("Edges import failed: {}", e)),
            }
        }

        info!("CSV import completed: {} nodes, {} edges, {} layers", 
              result.nodes_imported, result.edges_imported, result.layers_imported);

        Ok(result)
    }

    async fn import_nodes(&self, project_id: i32, csv_data: &str) -> Result<usize> {
        let mut reader = ReaderBuilder::new().from_reader(csv_data.as_bytes());
        let headers = reader.headers()?.clone();
        
        let mut count = 0;
        for record in reader.records() {
            let record = record?;
            
            // Extract standard fields
            let node_id = record.get(0).unwrap_or("").to_string();
            let label = record.get(1).unwrap_or(&node_id).to_string();
            let layer_id = record.get(2).and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });
            
            if node_id.is_empty() {
                warn!("Skipping node with empty ID");
                continue;
            }

            // Collect additional properties
            let mut properties = HashMap::new();
            for (i, value) in record.iter().enumerate() {
                if i >= 3 && i < headers.len() {
                    if let Some(header) = headers.get(i) {
                        if !value.is_empty() {
                            properties.insert(header.to_string(), Value::String(value.to_string()));
                        }
                    }
                }
            }

            let properties_json = if properties.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&properties)?)
            };

            let node = nodes::ActiveModel {
                project_id: Set(project_id),
                node_id: Set(node_id),
                label: Set(label),
                layer_id: Set(layer_id),
                properties: Set(properties_json),
                ..Default::default()
            };

            node.insert(&self.db).await?;
            count += 1;
        }

        Ok(count)
    }

    async fn import_edges(&self, project_id: i32, csv_data: &str) -> Result<usize> {
        let mut reader = ReaderBuilder::new().from_reader(csv_data.as_bytes());
        let headers = reader.headers()?.clone();
        
        let mut count = 0;
        for record in reader.records() {
            let record = record?;
            
            let source_node_id = record.get(0).unwrap_or("").to_string();
            let target_node_id = record.get(1).unwrap_or("").to_string();
            
            if source_node_id.is_empty() || target_node_id.is_empty() {
                warn!("Skipping edge with empty source or target");
                continue;
            }

            // Collect additional properties
            let mut properties = HashMap::new();
            for (i, value) in record.iter().enumerate() {
                if i >= 2 && i < headers.len() {
                    if let Some(header) = headers.get(i) {
                        if !value.is_empty() {
                            properties.insert(header.to_string(), Value::String(value.to_string()));
                        }
                    }
                }
            }

            let properties_json = if properties.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&properties)?)
            };

            let edge = edges::ActiveModel {
                project_id: Set(project_id),
                source_node_id: Set(source_node_id),
                target_node_id: Set(target_node_id),
                properties: Set(properties_json),
                ..Default::default()
            };

            edge.insert(&self.db).await?;
            count += 1;
        }

        Ok(count)
    }

    async fn import_layers(&self, project_id: i32, csv_data: &str) -> Result<usize> {
        let mut reader = ReaderBuilder::new().from_reader(csv_data.as_bytes());
        let headers = reader.headers()?.clone();
        
        let mut count = 0;
        for record in reader.records() {
            let record = record?;
            
            let layer_id = record.get(0).unwrap_or("").to_string();
            let name = record.get(1).unwrap_or(&layer_id).to_string();
            let color = record.get(2).and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });
            
            if layer_id.is_empty() {
                warn!("Skipping layer with empty ID");
                continue;
            }

            // Collect additional properties
            let mut properties = HashMap::new();
            for (i, value) in record.iter().enumerate() {
                if i >= 3 && i < headers.len() {
                    if let Some(header) = headers.get(i) {
                        if !value.is_empty() {
                            properties.insert(header.to_string(), Value::String(value.to_string()));
                        }
                    }
                }
            }

            let properties_json = if properties.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&properties)?)
            };

            let layer = layers::ActiveModel {
                project_id: Set(project_id),
                layer_id: Set(layer_id),
                name: Set(name),
                color: Set(color),
                properties: Set(properties_json),
                ..Default::default()
            };

            layer.insert(&self.db).await?;
            count += 1;
        }

        Ok(count)
    }
}

#[allow(dead_code)] // Reserved for future import result tracking
#[derive(Debug)]
pub struct ImportResult {
    pub nodes_imported: usize,
    pub edges_imported: usize,
    pub layers_imported: usize,
    pub errors: Vec<String>,
}