//! Project management tools for MCP

use axum_mcp::prelude::*;
use crate::database::entities::projects;
use crate::mcp::tools::{get_required_param, get_optional_param};
use sea_orm::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get project management tools
pub fn get_project_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_projects".to_string(),
            description: "List all available graph projects".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "create_project".to_string(),
            description: "Create a new graph project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the project"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional description of the project"
                    }
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_project".to_string(),
            description: "Get details of a specific project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to retrieve"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "delete_project".to_string(),
            description: "Delete a project and all its data".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to delete"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

/// List all projects
pub async fn list_projects(db: &DatabaseConnection) -> McpResult<ToolsCallResult> {
    let projects = projects::Entity::find()
        .all(db)
        .await
        .map_err(|e| McpError::Internal { message: format!("Database error: {}", e) })?;

    let project_list: Vec<Value> = projects
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "created_at": p.created_at,
                "updated_at": p.updated_at
            })
        })
        .collect();

    let result = json!({
        "projects": project_list,
        "count": project_list.len()
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Create a new project
pub async fn create_project(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let name = get_required_param(&arguments, "name")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Project name must be a string".to_string(),
        })?
        .to_string();

    let description = get_optional_param(&arguments, "description")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let new_project = projects::ActiveModel {
        name: Set(name),
        description: Set(description),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let project = projects::Entity::insert(new_project)
        .exec_with_returning(db)
        .await
        .map_err(|e| McpError::Internal { message: format!("Failed to create project: {}", e) })?;

    let result = json!({
        "id": project.id,
        "name": project.name,
        "description": project.description,
        "created_at": project.created_at,
        "updated_at": project.updated_at,
        "message": "Project created successfully"
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Get project details
pub async fn get_project(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let project = projects::Entity::find_by_id(project_id)
        .one(db)
        .await
        .map_err(|e| McpError::Internal { message: format!("Database error: {}", e) })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_project".to_string(),
            message: format!("Project with ID {} not found", project_id),
        })?;

    let result = json!({
        "id": project.id,
        "name": project.name,
        "description": project.description,
        "created_at": project.created_at,
        "updated_at": project.updated_at
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Delete a project
pub async fn delete_project(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    // Check if project exists
    let project = projects::Entity::find_by_id(project_id)
        .one(db)
        .await
        .map_err(|e| McpError::Internal { message: format!("Database error: {}", e) })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "delete_project".to_string(),
            message: format!("Project with ID {} not found", project_id),
        })?;

    // Delete the project (cascade will handle related data)
    projects::Entity::delete_by_id(project_id)
        .exec(db)
        .await
        .map_err(|e| McpError::Internal { message: format!("Failed to delete project: {}", e) })?;

    let result = json!({
        "project_id": project_id,
        "project_name": project.name,
        "message": "Project deleted successfully"
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}