//! Graph data management tools for MCP

use axum_mcp::prelude::*;
use crate::mcp_new::tools::{get_required_param, get_optional_param};
use crate::services::{ImportService, ExportService, GraphService};
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get graph data management tools
pub fn get_graph_data_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "import_csv".to_string(),
            description: "Import graph data from CSV content".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to import data into"
                    },
                    "nodes_csv": {
                        "type": "string",
                        "description": "CSV content for nodes (optional)"
                    },
                    "edges_csv": {
                        "type": "string",
                        "description": "CSV content for edges (optional)"
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
        },
        Tool {
            name: "export_graph".to_string(),
            description: "Export graph data in various formats".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to export"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "csv", "dot", "gml", "plantuml", "mermaid"],
                        "description": "Export format"
                    }
                },
                "required": ["project_id", "format"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_graph_data".to_string(),
            description: "Retrieve graph structure (nodes, edges, layers)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project"
                    },
                    "include_nodes": {
                        "type": "boolean",
                        "description": "Include nodes in response (default: true)"
                    },
                    "include_edges": {
                        "type": "boolean",
                        "description": "Include edges in response (default: true)"
                    },
                    "include_layers": {
                        "type": "boolean",
                        "description": "Include layers in response (default: true)"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

/// Import CSV data
pub async fn import_csv(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let nodes_csv = get_optional_param(&arguments, "nodes_csv")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let edges_csv = get_optional_param(&arguments, "edges_csv")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let layers_csv = get_optional_param(&arguments, "layers_csv")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    if nodes_csv.is_none() && edges_csv.is_none() && layers_csv.is_none() {
        return Err(McpError::Validation {
            message: "At least one CSV type (nodes, edges, or layers) must be provided".to_string(),
        });
    }

    let import_service = ImportService::new(db.clone());
    let mut results = json!({
        "project_id": project_id,
        "imported": {}
    });

    // Import nodes if provided
    if let Some(csv_content) = nodes_csv {
        match import_service.import_nodes_from_csv(project_id, &csv_content).await {
            Ok(count) => {
                results["imported"]["nodes"] = json!({
                    "count": count,
                    "status": "success"
                });
            }
            Err(e) => {
                results["imported"]["nodes"] = json!({
                    "status": "error",
                    "error": e.to_string()
                });
            }
        }
    }

    // Import edges if provided
    if let Some(csv_content) = edges_csv {
        match import_service.import_edges_from_csv(project_id, &csv_content).await {
            Ok(count) => {
                results["imported"]["edges"] = json!({
                    "count": count,
                    "status": "success"
                });
            }
            Err(e) => {
                results["imported"]["edges"] = json!({
                    "status": "error",
                    "error": e.to_string()
                });
            }
        }
    }

    // Import layers if provided
    if let Some(csv_content) = layers_csv {
        match import_service.import_layers_from_csv(project_id, &csv_content).await {
            Ok(count) => {
                results["imported"]["layers"] = json!({
                    "count": count,
                    "status": "success"
                });
            }
            Err(e) => {
                results["imported"]["layers"] = json!({
                    "status": "error",
                    "error": e.to_string()
                });
            }
        }
    }

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&results).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Export graph data
pub async fn export_graph(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let format = get_required_param(&arguments, "format")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Format must be a string".to_string(),
        })?
        .to_string();

    let export_service = ExportService::new(db.clone());
    let graph_service = GraphService::new(db.clone());

    // Build graph from database
    let _graph = graph_service.build_graph_from_project(project_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to build graph: {}", e),
        })?;

    // Export in requested format
    let exported_content = export_service.export_graph(project_id, &format)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Export failed: {}", e),
        })?;

    let result = json!({
        "project_id": project_id,
        "format": format,
        "content": exported_content,
        "message": format!("Graph exported successfully as {}", format)
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Get graph data structure
pub async fn get_graph_data(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let include_nodes = get_optional_param(&arguments, "include_nodes")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_edges = get_optional_param(&arguments, "include_edges")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_layers = get_optional_param(&arguments, "include_layers")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let graph_service = GraphService::new(db.clone());
    let mut result = json!({
        "project_id": project_id
    });

    if include_nodes {
        let nodes = graph_service.get_nodes_for_project(project_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to get nodes: {}", e),
            })?;
        
        result["nodes"] = json!({
            "count": nodes.len(),
            "data": nodes
        });
    }

    if include_edges {
        let edges = graph_service.get_edges_for_project(project_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to get edges: {}", e),
            })?;
        
        result["edges"] = json!({
            "count": edges.len(),
            "data": edges
        });
    }

    if include_layers {
        let layers = graph_service.get_layers_for_project(project_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to get layers: {}", e),
            })?;
        
        result["layers"] = json!({
            "count": layers.len(),
            "data": layers
        });
    }

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}