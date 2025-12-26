//! Plan DAG manipulation tools for MCP clients.

use layercake_core::app_context::AppContext;
use layercake_core::app_context::{
    PlanDagEdgeRequest, PlanDagEdgeUpdateRequest, PlanDagNodePositionRequest, PlanDagNodeRequest,
    PlanDagNodeUpdateRequest,
};
use layercake_core::plan_dag::{PlanDagNodeType, Position};
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use axum_mcp::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn get_plan_dag_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "add_plan_dag_node".to_string(),
            description: "Create a new Plan DAG node".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "node_type": {"type": "string", "description": "Plan DAG node type (DataSetNode, GraphNode, TransformNode, FilterNode, MergeNode, GraphArtefactNode, TreeArtefactNode)"},
                    "position": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        },
                        "required": ["x", "y"],
                        "additionalProperties": false
                    },
                    "metadata": {"type": "object"},
                    "config": {"type": "object"}
                },
                "required": ["project_id", "node_type", "position"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "update_plan_dag_node".to_string(),
            description: "Update position, metadata, or config for an existing Plan DAG node"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "node_id": {"type": "string"},
                    "position": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        }
                    },
                    "metadata": {"type": "object"},
                    "config": {"type": "object"}
                },
                "required": ["project_id", "node_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "delete_plan_dag_node".to_string(),
            description: "Remove a Plan DAG node and its connected edges".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "node_id": {"type": "string"}
                },
                "required": ["project_id", "node_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "move_plan_dag_node".to_string(),
            description: "Move a Plan DAG node to a new position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "node_id": {"type": "string"},
                    "position": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        },
                        "required": ["x", "y"],
                        "additionalProperties": false
                    }
                },
                "required": ["project_id", "node_id", "position"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "batch_move_plan_dag_nodes".to_string(),
            description: "Move multiple Plan DAG nodes in a single operation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "nodes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "node_id": {"type": "string"},
                                "position": {
                                    "type": "object",
                                    "properties": {
                                        "x": {"type": "number"},
                                        "y": {"type": "number"}
                                    },
                                    "required": ["x", "y"],
                                    "additionalProperties": false
                                },
                                "source_position": {"type": "string"},
                                "target_position": {"type": "string"}
                            },
                            "required": ["node_id", "position"],
                            "additionalProperties": false
                        }
                    }
                },
                "required": ["project_id", "nodes"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "add_plan_dag_edge".to_string(),
            description: "Create a Plan DAG edge between two nodes".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "source": {"type": "string"},
                    "target": {"type": "string"},
                    "metadata": {"type": "object"}
                },
                "required": ["project_id", "source", "target"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "update_plan_dag_edge".to_string(),
            description: "Update metadata for a Plan DAG edge".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "edge_id": {"type": "string"},
                    "metadata": {"type": "object"}
                },
                "required": ["project_id", "edge_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "delete_plan_dag_edge".to_string(),
            description: "Delete a Plan DAG edge".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "edge_id": {"type": "string"}
                },
                "required": ["project_id", "edge_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

pub async fn add_plan_dag_node(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let node_type_str = get_required_param(&arguments, "node_type")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "node_type must be a string".to_string(),
        })?;
    let node_type = parse_node_type(node_type_str)?;

    let position_value = get_required_param(&arguments, "position")?;
    let position = parse_position(position_value)?;

    let metadata = get_optional_param(&arguments, "metadata")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let config = get_optional_param(&arguments, "config")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let request = PlanDagNodeRequest {
        node_type,
        position,
        metadata,
        config,
    };

    let node = app
        .create_plan_dag_node(project_id, None, request)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create Plan DAG node: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagNode": node
    }))
}

pub async fn update_plan_dag_node(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let node_id = get_required_param(&arguments, "node_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "node_id must be a string".to_string(),
        })?
        .to_string();

    let position = get_optional_param(&arguments, "position")
        .map(parse_position)
        .transpose()?;

    let metadata = get_optional_param(&arguments, "metadata").cloned();
    let config = get_optional_param(&arguments, "config").cloned();

    let request = PlanDagNodeUpdateRequest {
        position,
        metadata,
        config,
    };

    let node = app
        .update_plan_dag_node(project_id, None, node_id, request)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update Plan DAG node: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagNode": node
    }))
}

pub async fn delete_plan_dag_node(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let node_id = get_required_param(&arguments, "node_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "node_id must be a string".to_string(),
        })?
        .to_string();

    let node = app
        .delete_plan_dag_node(project_id, None, node_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to delete Plan DAG node: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagNode": node
    }))
}

pub async fn move_plan_dag_node(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let node_id = get_required_param(&arguments, "node_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "node_id must be a string".to_string(),
        })?
        .to_string();
    let position_value = get_required_param(&arguments, "position")?;
    let position = parse_position(position_value)?;

    let node = app
        .move_plan_dag_node(project_id, None, node_id, position)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to move Plan DAG node: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagNode": node
    }))
}

pub async fn batch_move_plan_dag_nodes(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let nodes_value = get_required_param(&arguments, "nodes")?
        .as_array()
        .ok_or_else(|| McpError::Validation {
            message: "nodes must be an array".to_string(),
        })?;

    let mut requests = Vec::with_capacity(nodes_value.len());
    for entry in nodes_value {
        let node_id = entry
            .get("node_id")
            .and_then(Value::as_str)
            .ok_or_else(|| McpError::Validation {
                message: "Each node entry must include node_id".to_string(),
            })?
            .to_string();
        let position_value = entry.get("position").ok_or_else(|| McpError::Validation {
            message: "Each node entry must include position".to_string(),
        })?;
        let position = parse_position(position_value)?;
        let source_position = entry
            .get("source_position")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let target_position = entry
            .get("target_position")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        requests.push(PlanDagNodePositionRequest {
            node_id,
            position,
            source_position,
            target_position,
        });
    }

    let nodes = app
        .batch_move_plan_dag_nodes(project_id, None, requests)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to batch move Plan DAG nodes: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagNodes": nodes
    }))
}

pub async fn add_plan_dag_edge(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let source = get_required_param(&arguments, "source")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "source must be a string".to_string(),
        })?
        .to_string();
    let target = get_required_param(&arguments, "target")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "target must be a string".to_string(),
        })?
        .to_string();
    let metadata = get_optional_param(&arguments, "metadata")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let request = PlanDagEdgeRequest {
        source,
        target,
        metadata,
    };

    let edge = app
        .create_plan_dag_edge(project_id, None, request)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create Plan DAG edge: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagEdge": edge
    }))
}

pub async fn update_plan_dag_edge(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let edge_id = get_required_param(&arguments, "edge_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "edge_id must be a string".to_string(),
        })?
        .to_string();

    let metadata = get_optional_param(&arguments, "metadata").cloned();
    let request = PlanDagEdgeUpdateRequest { metadata };

    let edge = app
        .update_plan_dag_edge(project_id, None, edge_id, request)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update Plan DAG edge: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagEdge": edge
    }))
}

pub async fn delete_plan_dag_edge(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = extract_project_id(&arguments)?;
    let edge_id = get_required_param(&arguments, "edge_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "edge_id must be a string".to_string(),
        })?
        .to_string();

    let edge = app
        .delete_plan_dag_edge(project_id, None, edge_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to delete Plan DAG edge: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDagEdge": edge
    }))
}

fn extract_project_id(arguments: &Option<Value>) -> McpResult<i32> {
    get_required_param(arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "project_id must be an integer".to_string(),
        })
        .map(|id| id as i32)
}

fn parse_node_type(value: &str) -> McpResult<PlanDagNodeType> {
    match value {
        "DataSetNode" | "DataSet" => Ok(PlanDagNodeType::DataSet),
        "GraphNode" | "Graph" => Ok(PlanDagNodeType::Graph),
        "TransformNode" | "Transform" => Ok(PlanDagNodeType::Transform),
        "FilterNode" | "Filter" => Ok(PlanDagNodeType::Filter),
        "MergeNode" | "Merge" => Ok(PlanDagNodeType::Merge),
        "GraphArtefactNode" | "GraphArtefact" | "OutputNode" | "Output" => {
            Ok(PlanDagNodeType::GraphArtefact)
        }
        "TreeArtefactNode" | "TreeArtefact" => Ok(PlanDagNodeType::TreeArtefact),
        "ProjectionNode" | "Projection" => Ok(PlanDagNodeType::Projection),
        "StoryNode" | "Story" => Ok(PlanDagNodeType::Story),
        "SequenceArtefactNode" | "SequenceArtefact" => Ok(PlanDagNodeType::SequenceArtefact),
        _ => Err(McpError::Validation {
            message: format!("Unsupported node_type '{}'", value),
        }),
    }
}

fn parse_position(value: &Value) -> McpResult<Position> {
    let x = value
        .get("x")
        .and_then(Value::as_f64)
        .ok_or_else(|| McpError::Validation {
            message: "position.x must be a number".to_string(),
        })?;
    let y = value
        .get("y")
        .and_then(Value::as_f64)
        .ok_or_else(|| McpError::Validation {
            message: "position.y must be a number".to_string(),
        })?;

    Ok(Position { x, y })
}
