//! Project management tools for MCP backed by shared AppContext helpers.

use layercake_core::app_context::{AppContext, ProjectUpdate};
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use axum_mcp::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn get_project_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_projects".to_string(),
            description: "List all available projects".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "create_project".to_string(),
            description: "Create a new project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "update_project".to_string(),
            description: "Update an existing project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"},
                    "name": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_project".to_string(),
            description: "Fetch details of a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"}
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "delete_project".to_string(),
            description: "Delete a project and its data".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {"type": "integer"}
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

pub async fn list_projects(app: &AppContext) -> McpResult<ToolsCallResult> {
    let projects = app.list_projects().await.map_err(|e| McpError::Internal {
        message: format!("Failed to list projects: {}", e),
    })?;

    create_success_response(&json!({
        "projects": projects,
        "count": projects.len()
    }))
}

pub async fn create_project(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let name = get_required_param(&arguments, "name")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Project name must be a string".to_string(),
        })?
        .to_string();
    let description = get_optional_param(&arguments, "description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let project = app
        .create_project(&layercake_core::auth::SystemActor::internal(), name, description, None)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create project: {}", e),
        })?;

    create_success_response(&json!({
        "project": project,
        "message": "Project created successfully"
    }))
}

pub async fn update_project(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let name = get_optional_param(&arguments, "name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let description_value = get_optional_param(&arguments, "description");
    let description = description_value
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let description_is_set = description_value.is_some();

    if name.is_none() && !description_is_set {
        return Err(McpError::Validation {
            message: "Provide at least one field to update".to_string(),
        });
    }

    let update = ProjectUpdate::new(name, description, description_is_set, None, None);
    let project = app
        .update_project(
            &layercake_core::auth::SystemActor::internal(),
            project_id,
            update,
        )
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update project: {}", e),
        })?;

    create_success_response(&json!({
        "project": project,
        "message": "Project updated successfully"
    }))
}

pub async fn get_project(arguments: Option<Value>, app: &AppContext) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let project = app
        .get_project(project_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to load project: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_project".to_string(),
            message: format!("Project with ID {} not found", project_id),
        })?;

    create_success_response(&json!({ "project": project }))
}

pub async fn delete_project(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    app.delete_project(&layercake_core::auth::SystemActor::internal(), project_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to delete project: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "message": "Project deleted successfully"
    }))
}
