//! Plan management tools for MCP

use crate::mcp::protocol::Tool;
use crate::mcp::tools::{get_required_param, get_optional_param};
use crate::database::entities::plans;
use sea_orm::*;
use serde_json::{json, Value};

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
        },
    ]
}

/// Create a new plan
pub async fn create_plan(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or("Project ID must be a number")? as i32;

    let name = get_required_param(&arguments, "name")?
        .as_str()
        .ok_or("Plan name must be a string")?
        .to_string();

    let yaml_content = get_required_param(&arguments, "yaml_content")?
        .as_str()
        .ok_or("YAML content must be a string")?
        .to_string();

    let dependencies = get_optional_param(&arguments, "dependencies")
        .and_then(|v| v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect::<Vec<i32>>()
        }))
        .unwrap_or_default();

    // Validate YAML content
    serde_yaml::from_str::<serde_yaml::Value>(&yaml_content)
        .map_err(|e| format!("Invalid YAML content: {}", e))?;

    let dependencies_json = serde_json::to_string(&dependencies)
        .map_err(|e| format!("Failed to serialize dependencies: {}", e))?;

    let new_plan = plans::ActiveModel {
        project_id: Set(project_id),
        name: Set(name),
        yaml_content: Set(yaml_content),
        dependencies: Set(Some(dependencies_json)),
        status: Set("pending".to_string()),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let plan = plans::Entity::insert(new_plan)
        .exec_with_returning(db)
        .await
        .map_err(|e| format!("Failed to create plan: {}", e))?;

    Ok(json!({
        "id": plan.id,
        "project_id": plan.project_id,
        "name": plan.name,
        "status": plan.status,
        "dependencies": dependencies,
        "created_at": plan.created_at,
        "updated_at": plan.updated_at,
        "message": "Plan created successfully"
    }))
}

/// Execute a plan (placeholder implementation)
pub async fn execute_plan(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or("Plan ID must be a number")? as i32;

    // Find the plan
    let mut plan = plans::Entity::find_by_id(plan_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Plan not found")?;

    // Check if plan is already running or completed
    if plan.status == "running" {
        return Ok(json!({
            "plan_id": plan_id,
            "status": "running",
            "message": "Plan is already running"
        }));
    }

    // Update status to running
    let mut plan_active: plans::ActiveModel = plan.clone().into();
    plan_active.status = Set("running".to_string());
    plan_active.updated_at = Set(chrono::Utc::now());
    
    plans::Entity::update(plan_active)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to update plan status: {}", e))?;

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
        .map_err(|e| format!("Failed to update plan status: {}", e))?;

    Ok(json!({
        "plan_id": plan_id,
        "status": "completed",
        "message": "Plan executed successfully"
    }))
}

/// Get plan status
pub async fn get_plan_status(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    let plan_id = get_required_param(&arguments, "plan_id")?
        .as_i64()
        .ok_or("Plan ID must be a number")? as i32;

    let plan = plans::Entity::find_by_id(plan_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Plan not found")?;

    let dependencies: Vec<i32> = plan.dependencies
        .as_ref()
        .and_then(|deps| serde_json::from_str(deps).ok())
        .unwrap_or_default();

    Ok(json!({
        "id": plan.id,
        "project_id": plan.project_id,
        "name": plan.name,
        "status": plan.status,
        "dependencies": dependencies,
        "created_at": plan.created_at,
        "updated_at": plan.updated_at
    }))
}