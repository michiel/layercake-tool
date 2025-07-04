//! Plan management tools for MCP

use axum_mcp::prelude::*;
use crate::database::entities::plans;
use crate::mcp::tools::{get_required_param, get_optional_param};
use sea_orm::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Get plan management tools
pub fn get_plan_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_plan".to_string(),
            description: "Create a new transformation plan".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project"
                    },
                    "name": {
                        "type": "string",
                        "description": "Name of the plan"
                    },
                    "plan_content": {
                        "type": "string",
                        "description": "JSON or YAML configuration for the plan"
                    },
                    "dependencies": {
                        "type": "array",
                        "items": {
                            "type": "integer"
                        },
                        "description": "List of plan IDs this plan depends on"
                    }
                },
                "required": ["project_id", "name", "plan_content"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "execute_plan".to_string(),
            description: "Execute a transformation plan".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan_id": {
                        "type": "integer",
                        "description": "ID of the plan to execute"
                    }
                },
                "required": ["plan_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_plan_status".to_string(),
            description: "Get the execution status of a plan".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan_id": {
                        "type": "integer",
                        "description": "ID of the plan to check"
                    }
                },
                "required": ["plan_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

/// Create a new plan
pub async fn create_plan(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let name = get_required_param(&arguments, "name")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Plan name must be a string".to_string(),
        })?
        .to_string();

    let plan_content = get_required_param(&arguments, "plan_content")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Plan content must be a string".to_string(),
        })?
        .to_string();

    let dependencies = get_optional_param(&arguments, "dependencies")
        .and_then(|v| v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect::<Vec<i32>>()
        }))
        .unwrap_or_default();

    // Validate content (try JSON first, then YAML for backward compatibility)
    let (plan_format, validated_content) = if let Ok(_) = serde_json::from_str::<serde_json::Value>(&plan_content) {
        ("json".to_string(), plan_content)
    } else if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&plan_content) {
        // Convert YAML to JSON
        let json_value = serde_json::to_value(yaml_value)
            .map_err(|e| McpError::Validation {
                message: format!("Failed to convert YAML to JSON: {}", e),
            })?;
        let json_content = serde_json::to_string_pretty(&json_value)
            .map_err(|e| McpError::Validation {
                message: format!("Failed to serialize JSON: {}", e),
            })?;
        ("json".to_string(), json_content)
    } else {
        return Err(McpError::Validation {
            message: "Invalid plan content: must be valid JSON or YAML".to_string(),
        });
    };

    let dependencies_json = serde_json::to_string(&dependencies)
        .map_err(|e| McpError::Internal {
            message: format!("Failed to serialize dependencies: {}", e),
        })?;

    let new_plan = plans::ActiveModel {
        project_id: Set(project_id),
        name: Set(name),
        plan_content: Set(validated_content),
        plan_format: Set(plan_format),
        plan_schema_version: Set("1.0.0".to_string()),
        dependencies: Set(Some(dependencies_json)),
        status: Set("pending".to_string()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let plan = plans::Entity::insert(new_plan)
        .exec_with_returning(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create plan: {}", e),
        })?;

    let result = json!({
        "id": plan.id,
        "project_id": plan.project_id,
        "name": plan.name,
        "status": plan.status,
        "dependencies": dependencies,
        "created_at": plan.created_at,
        "updated_at": plan.updated_at,
        "message": "Plan created successfully"
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Execute a plan (placeholder implementation)
pub async fn execute_plan(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    // Find the plan
    let plan = plans::Entity::find_by_id(plan_id)
        .one(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Database error: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "execute_plan".to_string(),
            message: format!("Plan with ID {} not found", plan_id),
        })?;

    // Check if plan is already running or completed
    if plan.status == "running" {
        let result = json!({
            "plan_id": plan_id,
            "status": "running",
            "message": "Plan is already running"
        });

        return Ok(ToolsCallResult {
            content: vec![ToolContent::Text {
                text: serde_json::to_string_pretty(&result).unwrap(),
            }],
            is_error: false,
            metadata: HashMap::new(),
        });
    }

    // Update status to running
    let mut plan_active: plans::ActiveModel = plan.clone().into();
    plan_active.status = Set("running".to_string());
    plan_active.updated_at = Set(chrono::Utc::now());
    
    plans::Entity::update(plan_active)
        .exec(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update plan status: {}", e),
        })?;

    // TODO: Implement actual plan execution using existing plan_execution module
    // For now, we'll simulate execution and mark as completed
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update status to completed
    let mut plan_active: plans::ActiveModel = plan.into();
    plan_active.status = Set("completed".to_string());
    plan_active.updated_at = Set(chrono::Utc::now());
    
    plans::Entity::update(plan_active)
        .exec(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update plan status: {}", e),
        })?;

    let result = json!({
        "plan_id": plan_id,
        "status": "completed",
        "message": "Plan executed successfully"
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}

/// Get plan status
pub async fn get_plan_status(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    let plan = plans::Entity::find_by_id(plan_id)
        .one(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Database error: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_plan_status".to_string(),
            message: format!("Plan with ID {} not found", plan_id),
        })?;

    let dependencies: Vec<i32> = plan.dependencies
        .as_ref()
        .and_then(|deps| serde_json::from_str(deps).ok())
        .unwrap_or_default();

    let result = json!({
        "id": plan.id,
        "project_id": plan.project_id,
        "name": plan.name,
        "status": plan.status,
        "dependencies": dependencies,
        "created_at": plan.created_at,
        "updated_at": plan.updated_at
    });

    Ok(ToolsCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(&result).unwrap(),
        }],
        is_error: false,
        metadata: HashMap::new(),
    })
}