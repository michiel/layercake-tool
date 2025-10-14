use anyhow::Result;
use csv::ReaderBuilder;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

use crate::database::entities::layers;

pub struct ImportService {
    db: DatabaseConnection,
}

impl ImportService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Import layers from CSV content (convenience method for MCP)
    pub async fn import_layers_from_csv(&self, graph_id: i32, csv_content: &str) -> Result<usize> {
        self.import_layers(graph_id, csv_content).await
    }

    async fn import_layers(&self, graph_id: i32, csv_data: &str) -> Result<usize> {
        let mut reader = ReaderBuilder::new().from_reader(csv_data.as_bytes());
        let headers = reader.headers()?.clone();

        let mut count = 0;
        for record in reader.records() {
            let record = record?;

            let layer_id = record.get(0).unwrap_or("").to_string();
            let name = record.get(1).unwrap_or(&layer_id).to_string();
            let color = record.get(2).and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            });

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
                graph_id: Set(graph_id),
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
