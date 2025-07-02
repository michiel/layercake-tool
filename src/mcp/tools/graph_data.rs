//! Graph data management tools for MCP

use crate::mcp::protocol::Tool;
use crate::mcp::tools::{get_required_param, get_optional_param};
use crate::services::{ImportService, ExportService, GraphService};
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use std::sync::Arc;

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
        },
    ]
}

/// Import CSV data
pub async fn import_csv(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or("Project ID must be a number")? as i32;

    let nodes_csv = get_optional_param(&arguments, "nodes_csv")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let edges_csv = get_optional_param(&arguments, "edges_csv")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let layers_csv = get_optional_param(&arguments, "layers_csv")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    if nodes_csv.is_none() && edges_csv.is_none() && layers_csv.is_none() {
        return Err("At least one CSV type (nodes, edges, or layers) must be provided".to_string());
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

    Ok(results)
}

/// Export graph data
pub async fn export_graph(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or("Project ID must be a number")? as i32;

    let format = get_required_param(&arguments, "format")?
        .as_str()
        .ok_or("Format must be a string")?
        .to_string();

    let export_service = ExportService::new(db.clone());
    let graph_service = GraphService::new(db.clone());

    // Build graph from database
    let graph = graph_service.build_graph_from_project(project_id)
        .await
        .map_err(|e| format!("Failed to build graph: {}", e))?;

    // Export in requested format
    let exported_content = export_service.export_graph(project_id, &format)
        .await
        .map_err(|e| format!("Export failed: {}", e))?;

    Ok(json!({
        "project_id": project_id,
        "format": format,
        "content": exported_content,
        "message": format!("Graph exported successfully as {}", format)
    }))
}

/// Get graph data structure
pub async fn get_graph_data(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or("Project ID must be a number")? as i32;

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
            .map_err(|e| format!("Failed to get nodes: {}", e))?;
        
        result["nodes"] = json!({
            "count": nodes.len(),
            "data": nodes
        });
    }

    if include_edges {
        let edges = graph_service.get_edges_for_project(project_id)
            .await
            .map_err(|e| format!("Failed to get edges: {}", e))?;
        
        result["edges"] = json!({
            "count": edges.len(),
            "data": edges
        });
    }

    if include_layers {
        let layers = graph_service.get_layers_for_project(project_id)
            .await
            .map_err(|e| format!("Failed to get layers: {}", e))?;
        
        result["layers"] = json!({
            "count": layers.len(),
            "data": layers
        });
    }

    Ok(result)
}