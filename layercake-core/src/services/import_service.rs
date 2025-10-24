use anyhow::Result;
use csv::ReaderBuilder;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

use crate::database::entities::graph_layers;

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

            if layer_id.is_empty() {
                warn!("Skipping layer with empty ID");
                continue;
            }

            // Extract color fields and other properties from headers
            let mut background_color = None;
            let mut text_color = None;
            let mut border_color = None;
            let mut comment = None;
            let mut properties = HashMap::new();

            for (i, value) in record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    if !value.is_empty() {
                        match header {
                            "background_color" => background_color = Some(value.to_string()),
                            "text_color" => text_color = Some(value.to_string()),
                            "border_color" => border_color = Some(value.to_string()),
                            "comment" => comment = Some(value.to_string()),
                            // Skip layer_id and name columns (0 and 1)
                            _ if i > 1 => {
                                properties.insert(header.to_string(), Value::String(value.to_string()));
                            }
                            _ => {}
                        }
                    }
                }
            }

            let properties_json = if properties.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&properties)?)
            };

            let layer = graph_layers::ActiveModel {
                graph_id: Set(graph_id),
                layer_id: Set(layer_id),
                name: Set(name),
                background_color: Set(background_color),
                text_color: Set(text_color),
                border_color: Set(border_color),
                comment: Set(comment),
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
