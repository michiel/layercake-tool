//! Project resources for MCP

use crate::mcp::protocol::Resource;
use crate::database::entities::projects;
use sea_orm::*;
use serde_json::{json, Value};

/// Get project resources
pub fn get_project_resources() -> Vec<Resource> {
    vec![
        Resource {
            uri: "layercake://project/list".to_string(),
            name: "Project List".to_string(),
            description: Some("List of all available projects".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

/// Get project resource content
pub async fn get_project_resource_content(
    uri: &str,
    db: &DatabaseConnection,
) -> Result<Value, String> {
    match uri {
        "layercake://project/list" => get_project_list(db).await,
        uri if uri.starts_with("layercake://project/") => {
            // Parse project ID from URI like "layercake://project/123"
            let parts: Vec<&str> = uri.split('/').collect();
            if parts.len() >= 3 {
                if let Ok(project_id) = parts[2].parse::<i32>() {
                    get_project_details(project_id, db).await
                } else {
                    Err("Invalid project ID in URI".to_string())
                }
            } else {
                Err("Invalid project URI format".to_string())
            }
        }
        _ => Err(format!("Unknown project resource: {}", uri)),
    }
}

/// Get list of all projects
async fn get_project_list(db: &DatabaseConnection) -> Result<Value, String> {
    let projects = projects::Entity::find()
        .all(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    let project_list: Vec<Value> = projects
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "description": p.description,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
                "resource_uri": format!("layercake://project/{}", p.id)
            })
        })
        .collect();

    Ok(json!({
        "projects": project_list,
        "count": project_list.len(),
        "generated_at": chrono::Utc::now()
    }))
}

/// Get detailed project information
async fn get_project_details(project_id: i32, db: &DatabaseConnection) -> Result<Value, String> {
    let project = projects::Entity::find_by_id(project_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Project not found")?;

    Ok(json!({
        "id": project.id,
        "name": project.name,
        "description": project.description,
        "created_at": project.created_at,
        "updated_at": project.updated_at,
        "related_resources": {
            "graph_data": format!("layercake://graph/{}/data", project.id),
            "nodes": format!("layercake://graph/{}/nodes", project.id),
            "edges": format!("layercake://graph/{}/edges", project.id),
            "layers": format!("layercake://graph/{}/layers", project.id)
        }
    }))
}