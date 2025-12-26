//! Plan management tools for MCP backed by shared AppContext helpers.

use layercake_core::app_context::AppContext;
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use layercake_core::services::plan_service::{PlanCreateRequest, PlanUpdateRequest};
use axum_mcp::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

fn parse_dependencies(value: Option<&Value>) -> McpResult<Option<Vec<i32>>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => {
            let mut deps = Vec::new();
            for item in items {
                let id = item.as_i64().ok_or_else(|| McpError::Validation {
                    message: "Dependencies must be an array of numbers".to_string(),
                })? as i32;
                deps.push(id);
            }
            Ok(Some(deps))
        }
        Some(_) => Err(McpError::Validation {
            message: "Dependencies must be an array of numbers".to_string(),
        }),
    }
}

fn parse_dependencies_update(value: Option<&Value>) -> McpResult<(Option<Vec<i32>>, bool)> {
    match value {
        None => Ok((None, false)),
        Some(Value::Null) => Ok((None, true)),
        Some(Value::Array(items)) => {
            let mut deps = Vec::new();
            for item in items {
                let id = item.as_i64().ok_or_else(|| McpError::Validation {
                    message: "Dependencies must be an array of numbers".to_string(),
                })? as i32;
                deps.push(id);
            }
            Ok((Some(deps), true))
        }
        Some(_) => Err(McpError::Validation {
            message: "Dependencies must be an array of numbers".to_string(),
        }),
    }
}

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
                    "yaml_content": {
                        "type": "string",
                        "description": "YAML configuration for the plan"
                    },
                    "dependencies": {
                        "type": "array",
                        "items": {
                            "type": "integer"
                        },
                        "description": "List of plan IDs this plan depends on"
                    }
                },
                "required": ["project_id", "name", "yaml_content"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "update_plan".to_string(),
            description: "Update an existing transformation plan".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan_id": {
                        "type": "integer",
                        "description": "ID of the plan to update"
                    },
                    "name": {
                        "type": "string",
                        "description": "Updated name of the plan"
                    },
                    "yaml_content": {
                        "type": "string",
                        "description": "Updated YAML content for the plan"
                    },
                    "dependencies": {
                        "type": ["array", "null"],
                        "items": { "type": "integer" },
                        "description": "Optional list of plan IDs this plan depends on"
                    }
                },
                "required": ["plan_id", "name", "yaml_content"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_plan".to_string(),
            description: "Fetch details of a plan by ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan_id": {
                        "type": "integer",
                        "description": "ID of the plan to retrieve"
                    }
                },
                "required": ["plan_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "delete_plan".to_string(),
            description: "Delete a plan and its associated DAG data".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "plan_id": {
                        "type": "integer",
                        "description": "ID of the plan to delete"
                    }
                },
                "required": ["plan_id"],
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
        Tool {
            name: "get_plan_dag".to_string(),
            description: "Load the plan DAG definition for a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project whose Plan DAG should be returned"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

/// Create a new plan
pub async fn create_plan(arguments: Option<Value>, app: &AppContext) -> McpResult<ToolsCallResult> {
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

    let yaml_content = get_required_param(&arguments, "yaml_content")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "YAML content must be a string".to_string(),
        })?
        .to_string();

    let dependencies = parse_dependencies(get_optional_param(&arguments, "dependencies"))?;

    // Validate YAML content
    serde_yaml::from_str::<serde_yaml::Value>(&yaml_content).map_err(|e| McpError::Validation {
        message: format!("Invalid YAML content: {}", e),
    })?;

    let request = PlanCreateRequest {
        project_id,
        name,
        description: None,
        tags: None,
        yaml_content,
        dependencies,
        status: None,
    };

    let plan = app
        .create_plan(&layercake_core::auth::SystemActor::internal(), request)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create plan: {}", e),
        })?;

    create_success_response(&json!({
        "plan": plan,
        "message": "Plan created successfully"
    }))
}

/// Update an existing plan
pub async fn update_plan(arguments: Option<Value>, app: &AppContext) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    let name = get_required_param(&arguments, "name")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Plan name must be a string".to_string(),
        })?
        .to_string();

    let yaml_content = get_required_param(&arguments, "yaml_content")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "YAML content must be a string".to_string(),
        })?
        .to_string();

    let (dependencies, dependencies_is_set) =
        parse_dependencies_update(get_optional_param(&arguments, "dependencies"))?;

    // Validate YAML content
    serde_yaml::from_str::<serde_yaml::Value>(&yaml_content).map_err(|e| McpError::Validation {
        message: format!("Invalid YAML content: {}", e),
    })?;

    let update = PlanUpdateRequest {
        name: Some(name),
        description: None,
        tags: None,
        yaml_content: Some(yaml_content),
        dependencies,
        dependencies_is_set,
        status: None,
    };

    let plan = app
        .update_plan(&layercake_core::auth::SystemActor::internal(), plan_id, update)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update plan: {}", e),
        })?;

    create_success_response(&json!({
        "plan": plan,
        "message": "Plan updated successfully"
    }))
}

/// Retrieve a plan by ID
pub async fn get_plan(arguments: Option<Value>, app: &AppContext) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    let plan = app
        .get_plan(plan_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to load plan: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_plan".to_string(),
            message: format!("Plan with ID {} not found", plan_id),
        })?;

    create_success_response(&json!({ "plan": plan }))
}

/// Delete a plan by ID
pub async fn delete_plan(arguments: Option<Value>, app: &AppContext) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    app.delete_plan(&layercake_core::auth::SystemActor::internal(), plan_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to delete plan: {}", e),
        })?;

    create_success_response(&json!({
        "planId": plan_id,
        "message": "Plan deleted successfully"
    }))
}

/// Execute a plan (placeholder implementation)
pub async fn execute_plan(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    // Find the plan
    let plan = app
        .get_plan(plan_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to load plan: {}", e),
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
                text: serde_json::to_string_pretty(&result)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize result to JSON: {}", e))?,
            }],
            is_error: false,
            metadata: HashMap::new(),
        });
    }

    // Update status to running
    app.update_plan(
        &layercake_core::auth::SystemActor::internal(),
        plan_id,
        PlanUpdateRequest {
            name: None,
            description: None,
            tags: None,
            yaml_content: None,
            dependencies: None,
            dependencies_is_set: false,
            status: Some("running".to_string()),
        },
    )
    .await
    .map_err(|e| McpError::Internal {
        message: format!("Failed to update plan status: {}", e),
    })?;

    // Execute the plan using the existing plan_execution module
    let execution_result = layercake_core::plan_execution::execute_plan(plan.yaml_content.clone(), false);

    let (status, error_message) = match execution_result {
        Ok(_) => ("completed".to_string(), None),
        Err(e) => {
            tracing::error!("Plan execution failed: {}", e);
            (
                "failed".to_string(),
                Some(format!("Plan execution failed: {}", e)),
            )
        }
    };

    // Update status based on execution result
    app.update_plan(
        &layercake_core::auth::SystemActor::internal(),
        plan_id,
        PlanUpdateRequest {
            name: None,
            description: None,
            tags: None,
            yaml_content: None,
            dependencies: None,
            dependencies_is_set: false,
            status: Some(status.clone()),
        },
    )
    .await
    .map_err(|e| McpError::Internal {
        message: format!("Failed to update plan status: {}", e),
    })?;

    let result = json!({
        "plan_id": plan_id,
        "status": status,
        "message": error_message.unwrap_or_else(|| "Plan executed successfully".to_string())
    });

    create_success_response(&result)
}

/// Get plan status
pub async fn get_plan_status(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Plan ID must be a number".to_string(),
        })? as i32;

    let plan = app
        .get_plan(plan_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to load plan: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_plan_status".to_string(),
            message: format!("Plan with ID {} not found", plan_id),
        })?;

    let result = json!({
        "id": plan.id,
        "project_id": plan.project_id,
        "name": plan.name,
        "status": plan.status,
        "dependencies": plan.dependencies.unwrap_or_default(),
        "created_at": plan.created_at,
        "updated_at": plan.updated_at
    });

    create_success_response(&result)
}

/// Get the Plan DAG for a project
pub async fn get_plan_dag(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "Project ID must be a number".to_string(),
        })? as i32;

    let snapshot = app
        .load_plan_dag(project_id, None)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to load Plan DAG: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_plan_dag".to_string(),
            message: format!("Project with ID {} not found", project_id),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "planDag": snapshot
    }))
}
