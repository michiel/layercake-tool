//! Graph data management tools for MCP

use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use crate::services::{ExportService, GraphService, ImportService};
use axum_mcp::prelude::*;
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get graph data management tools
pub fn get_graph_data_tools() -> Vec<Tool> {
    vec![Tool {
        name: "import_csv".to_string(),
        description: "Import graph data from CSV content".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {
                    "type": "integer",
                    "description": "ID of the project to import data into"
                },
                "layers_csv": {
                    "type": "string",
                    "description": "CSV content for layers (optional)"
                }
            },
            "required": ["project_id"],
            "additionalProperties": false
        }),
        metadata: HashMap::new(),
    }]
}

