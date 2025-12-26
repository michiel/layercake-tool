//! Graph editing tools for MCP clients backed by shared AppContext helpers.

use layercake_core::app_context::{AppContext, GraphLayerUpdateRequest, GraphNodeUpdateRequest};
use crate::graphql::types::graph_node::GraphNode as GraphNodeDto;
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use axum_mcp::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn get_graph_edit_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "update_graph_node".to_string(),
            description: "Update a graph node's metadata (label, layer, attributes, belongsTo)"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "graph_id": {"type": "integer"},
                    "node_id": {"type": "string"},
                    "label": {"type": "string"},
                    "layer": {"type": "string"},
                    "attrs": {"type": "object"},
                    "attributes": {"type": "object"},
                    "belongs_to": {"type": "string"}
                },
                "required": ["graph_id", "node_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "bulk_update_graph_data".to_string(),
            description: "Bulk update graph nodes and layers in a single request".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "graph_id": {"type": "integer"},
                    "nodes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "node_id": {"type": "string"},
                                "label": {"type": "string"},
                                "layer": {"type": "string"},
                                "attrs": {"type": "object"}
                            },
                            "required": ["node_id"],
                            "additionalProperties": false
                        }
                    },
                    "layers": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "integer"},
                                "name": {"type": "string"},
                                "properties": {"type": "object"}
                            },
                            "required": ["id"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["graph_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "replay_graph_edits".to_string(),
            description: "Replay all pending graph edits for a graph".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "graph_id": {"type": "integer"}
                },
                "required": ["graph_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

pub async fn update_graph_node(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let graph_id = parse_required_i32(&arguments, "graph_id")?;
    let node_id = parse_required_string(&arguments, "node_id")?;
    let label = parse_optional_string(&arguments, "label");
    let layer = parse_optional_string(&arguments, "layer");
    let attrs = normalize_attributes(
        get_optional_param(&arguments, "attrs").cloned(),
        get_optional_param(&arguments, "attributes").cloned(),
    )?;
    let belongs_to = parse_optional_string(&arguments, "belongs_to");

    let node = app
        .update_graph_node(graph_id, node_id, label, layer, attrs, belongs_to)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update graph node: {}", e),
        })?;

    let node = GraphNodeDto::from(node);
    create_success_response(&json_node(&node))
}

pub async fn bulk_update_graph_data(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let graph_id = parse_required_i32(&arguments, "graph_id")?;

    let node_requests = parse_node_updates(&arguments)?;
    let layer_requests = parse_layer_updates(&arguments)?;

    app.bulk_update_graph_data(graph_id, node_requests, layer_requests)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to bulk update graph data: {}", e),
        })?;

    create_success_response(&json!({
        "graphId": graph_id,
        "message": "Graph data updated successfully"
    }))
}

pub async fn replay_graph_edits(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let graph_id = parse_required_i32(&arguments, "graph_id")?;

    let summary = app
        .replay_graph_edits(graph_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to replay graph edits: {}", e),
        })?;

    let details: Vec<Value> = summary
        .details
        .into_iter()
        .map(|detail| {
            json!({
                "sequenceNumber": detail.sequence_number,
                "targetType": detail.target_type,
                "targetId": detail.target_id,
                "operation": detail.operation,
                "result": detail.result,
                "message": detail.message
            })
        })
        .collect();

    create_success_response(&json!({
        "graphId": graph_id,
        "summary": {
            "total": summary.total,
            "applied": summary.applied,
            "skipped": summary.skipped,
            "failed": summary.failed,
            "details": details
        }
    }))
}

fn parse_required_i32(arguments: &Option<Value>, key: &str) -> McpResult<i32> {
    get_required_param(arguments, key)?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: format!("{key} must be an integer"),
        })
        .map(|value| value as i32)
}

fn parse_required_string(arguments: &Option<Value>, key: &str) -> McpResult<String> {
    get_required_param(arguments, key)?
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| McpError::Validation {
            message: format!("{key} must be a string"),
        })
}

fn parse_optional_string(arguments: &Option<Value>, key: &str) -> Option<String> {
    get_optional_param(arguments, key)
        .and_then(|value| value.as_str())
        .map(|s| s.to_string())
}

fn normalize_attributes(
    attrs: Option<Value>,
    attributes: Option<Value>,
) -> McpResult<Option<Value>> {
    let candidate = attributes.or(attrs);
    if let Some(val) = candidate {
        let map = val.as_object().ok_or_else(|| McpError::Validation {
            message: "attributes must be an object with string keys and string/integer values"
                .to_string(),
        })?;
        for (key, value) in map {
            if key.trim().is_empty() {
                return Err(McpError::Validation {
                    message: "attribute keys must be non-empty strings".to_string(),
                });
            }
            if !(value.is_string() || value.as_i64().is_some()) {
                return Err(McpError::Validation {
                    message: format!("attribute '{}' must be a string or integer value", key),
                });
            }
        }
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

fn parse_node_updates(arguments: &Option<Value>) -> McpResult<Vec<GraphNodeUpdateRequest>> {
    let nodes = get_optional_param(arguments, "nodes")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut requests = Vec::with_capacity(nodes.len());
    for node in nodes {
        let node_id = node
            .get("node_id")
            .and_then(Value::as_str)
            .ok_or_else(|| McpError::Validation {
                message: "Each node entry must include node_id".to_string(),
            })?
            .to_string();

        let label = node
            .get("label")
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let layer = node
            .get("layer")
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let attrs =
            normalize_attributes(node.get("attrs").cloned(), node.get("attributes").cloned())?;

        requests.push(GraphNodeUpdateRequest {
            node_id,
            label,
            layer,
            attributes: attrs,
            belongs_to: None,
        });
    }

    Ok(requests)
}

fn parse_layer_updates(arguments: &Option<Value>) -> McpResult<Vec<GraphLayerUpdateRequest>> {
    let layers = get_optional_param(arguments, "layers")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();

    let mut requests = Vec::with_capacity(layers.len());
    for layer in layers {
        let id = layer
            .get("id")
            .and_then(Value::as_i64)
            .ok_or_else(|| McpError::Validation {
                message: "Each layer entry must include id".to_string(),
            })? as i32;

        let name = layer
            .get("name")
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let alias = layer
            .get("alias")
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let properties = layer.get("properties").cloned();

        requests.push(GraphLayerUpdateRequest {
            id,
            name,
            alias,
            properties,
        });
    }

    Ok(requests)
}

fn json_node(node: &GraphNodeDto) -> Value {
    json!({
        "id": node.id,
        "graphId": node.graph_id,
        "label": node.label,
        "layer": node.layer,
        "weight": node.weight,
        "isPartition": node.is_partition,
        "belongsTo": node.belongs_to,
        "attrs": node.attrs.clone(),
        "attributes": node.attrs,
        "datasetId": node.dataset_id,
        "createdAt": node.created_at
    })
}
