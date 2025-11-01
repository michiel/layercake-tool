//! Graph analysis tools for MCP clients.

use crate::app_context::AppContext;
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use axum_mcp::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn get_analysis_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_connectivity".to_string(),
            description: "Analyze connectivity for a specific graph".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "graph_id": {
                        "type": "integer",
                        "description": "Graph identifier to analyze"
                    }
                },
                "required": ["graph_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "find_paths".to_string(),
            description: "Find simple paths between two nodes in a graph".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "graph_id": {
                        "type": "integer",
                        "description": "Graph identifier to analyze"
                    },
                    "source_node": {
                        "type": "string",
                        "description": "Source node identifier"
                    },
                    "target_node": {
                        "type": "string",
                        "description": "Target node identifier"
                    },
                    "max_paths": {
                        "type": "integer",
                        "description": "Maximum number of paths to return (default: 10)"
                    }
                },
                "required": ["graph_id", "source_node", "target_node"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

pub async fn analyze_connectivity(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let graph_id = get_required_param(&arguments, "graph_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "graph_id must be an integer".to_string(),
        })? as i32;

    let report = app
        .analyze_graph_connectivity(graph_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to analyze graph connectivity: {}", e),
        })?;

    create_success_response(&json!({
        "graphId": report.graph_id,
        "nodeCount": report.node_count,
        "edgeCount": report.edge_count,
        "componentCount": report.components.len(),
        "components": report.components
    }))
}

pub async fn find_paths(arguments: Option<Value>, app: &AppContext) -> McpResult<ToolsCallResult> {
    let graph_id = get_required_param(&arguments, "graph_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "graph_id must be an integer".to_string(),
        })? as i32;

    let source_node = get_required_param(&arguments, "source_node")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "source_node must be a string".to_string(),
        })?
        .to_string();

    let target_node = get_required_param(&arguments, "target_node")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "target_node must be a string".to_string(),
        })?
        .to_string();

    let max_paths = get_optional_param(&arguments, "max_paths")
        .and_then(|value| value.as_i64())
        .unwrap_or(10)
        .max(1) as usize;

    let paths = app
        .find_graph_paths(
            graph_id,
            source_node.clone(),
            target_node.clone(),
            max_paths,
        )
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to find graph paths: {}", e),
        })?;

    create_success_response(&json!({
        "graphId": graph_id,
        "sourceNode": source_node,
        "targetNode": target_node,
        "maxPaths": max_paths,
        "paths": paths
    }))
}
