//! Graph data resources for MCP

use crate::mcp::protocol::Resource;
use crate::services::GraphService;
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};

/// Get graph data resources
pub fn get_graph_data_resources() -> Vec<Resource> {
    vec![
        Resource {
            uri: "layercake://graph/schema".to_string(),
            name: "Graph Data Schema".to_string(),
            description: Some("Schema definition for graph data structures".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

/// Get graph data resource content
pub async fn get_graph_data_resource_content(
    uri: &str,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    match uri {
        "layercake://graph/schema" => get_graph_schema().await,
        uri if uri.starts_with("layercake://graph/") => {
            // Parse URIs like "layercake://graph/123/data", "layercake://graph/123/nodes", etc.
            let parts: Vec<&str> = uri.split('/').collect();
            if parts.len() >= 4 {
                if let Ok(project_id) = parts[2].parse::<i32>() {
                    match parts[3] {
                        "data" => get_complete_graph_data(project_id, db).await,
                        "nodes" => get_nodes_data(project_id, db).await,
                        "edges" => get_edges_data(project_id, db).await,
                        "layers" => get_layers_data(project_id, db).await,
                        resource_type => Err(format!("Unknown graph resource type: {}", resource_type)),
                    }
                } else {
                    Err("Invalid project ID in URI".to_string())
                }
            } else {
                Err("Invalid graph URI format".to_string())
            }
        }
        _ => Err(format!("Unknown graph resource: {}", uri)),
    }
}

/// Get graph data schema
async fn get_graph_schema() -> Result<Value, String> {
    Ok(json!({
        "schema_version": "1.0.0",
        "description": "Layercake graph data schema",
        "entities": {
            "node": {
                "properties": {
                    "node_id": {
                        "type": "string",
                        "description": "Unique identifier for the node",
                        "required": true
                    },
                    "label": {
                        "type": "string", 
                        "description": "Display label for the node",
                        "required": true
                    },
                    "layer_id": {
                        "type": "string",
                        "description": "ID of the layer this node belongs to",
                        "required": false
                    },
                    "properties": {
                        "type": "object",
                        "description": "Additional node properties as JSON",
                        "required": false
                    }
                }
            },
            "edge": {
                "properties": {
                    "source_node_id": {
                        "type": "string",
                        "description": "ID of the source node",
                        "required": true
                    },
                    "target_node_id": {
                        "type": "string",
                        "description": "ID of the target node", 
                        "required": true
                    },
                    "properties": {
                        "type": "object",
                        "description": "Additional edge properties as JSON",
                        "required": false
                    }
                }
            },
            "layer": {
                "properties": {
                    "layer_id": {
                        "type": "string",
                        "description": "Unique identifier for the layer",
                        "required": true
                    },
                    "name": {
                        "type": "string",
                        "description": "Display name for the layer",
                        "required": true
                    },
                    "color": {
                        "type": "string",
                        "description": "Color code for the layer (hex format)",
                        "required": false
                    },
                    "properties": {
                        "type": "object",
                        "description": "Additional layer properties as JSON",
                        "required": false
                    }
                }
            }
        }
    }))
}

/// Get complete graph data for a project
async fn get_complete_graph_data(project_id: i32, db: &DatabaseConnection) -> Result<Value, String> {
    let graph_service = GraphService::new(db.clone());
    
    let nodes = graph_service.get_nodes_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get nodes: {}", e))?;
        
    let edges = graph_service.get_edges_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get edges: {}", e))?;
        
    let layers = graph_service.get_layers_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get layers: {}", e))?;

    Ok(json!({
        "project_id": project_id,
        "graph_data": {
            "nodes": {
                "count": nodes.len(),
                "data": nodes
            },
            "edges": {
                "count": edges.len(),
                "data": edges
            },
            "layers": {
                "count": layers.len(),
                "data": layers
            }
        },
        "metadata": {
            "generated_at": chrono::Utc::now(),
            "total_entities": nodes.len() + edges.len() + layers.len()
        }
    }))
}

/// Get nodes data for a project
async fn get_nodes_data(project_id: i32, db: &DatabaseConnection) -> Result<Value, String> {
    let graph_service = GraphService::new(db.clone());
    
    let nodes = graph_service.get_nodes_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get nodes: {}", e))?;

    Ok(json!({
        "project_id": project_id,
        "nodes": {
            "count": nodes.len(),
            "data": nodes
        },
        "generated_at": chrono::Utc::now()
    }))
}

/// Get edges data for a project
async fn get_edges_data(project_id: i32, db: &DatabaseConnection) -> Result<Value, String> {
    let graph_service = GraphService::new(db.clone());
    
    let edges = graph_service.get_edges_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get edges: {}", e))?;

    Ok(json!({
        "project_id": project_id,
        "edges": {
            "count": edges.len(),
            "data": edges
        },
        "generated_at": chrono::Utc::now()
    }))
}

/// Get layers data for a project
async fn get_layers_data(project_id: i32, db: &DatabaseConnection) -> Result<Value, String> {
    let graph_service = GraphService::new(db.clone());
    
    let layers = graph_service.get_layers_for_project(project_id)
        .await
        .map_err(|e| format!("Failed to get layers: {}", e))?;

    Ok(json!({
        "project_id": project_id,
        "layers": {
            "count": layers.len(),
            "data": layers
        },
        "generated_at": chrono::Utc::now()
    }))
}