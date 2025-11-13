use async_graphql::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::database::entities::{project_collaborators, user_sessions, users};
use crate::graphql::errors::StructuredError;
use crate::plan::{ExportFileType, RenderConfig, RenderConfigOrientation, RenderConfigTheme};

/// Generate a unique node ID based on node type
pub fn generate_node_id_from_ids(
    node_type: &crate::graphql::types::PlanDagNodeType,
    _existing_node_ids: &[&str],
) -> String {
    use crate::graphql::types::PlanDagNodeType;

    let prefix = match node_type {
        PlanDagNodeType::DataSet => "dataset",
        PlanDagNodeType::Graph => "graph",
        PlanDagNodeType::Transform => "transform",
        PlanDagNodeType::Filter => "filter",
        PlanDagNodeType::Merge => "merge",
        PlanDagNodeType::GraphArtefact => "graphartefact",
        PlanDagNodeType::TreeArtefact => "treeartefact",
    };

    // Generate a globally unique ID using UUID to prevent collisions across projects/plans
    let uuid = Uuid::new_v4().simple().to_string();
    let short_uuid = uuid.chars().take(12).collect::<String>();

    format!("{}_{}", prefix, short_uuid)
}

/// Generate a unique edge ID
pub fn generate_edge_id(_source: &str, _target: &str) -> String {
    // Generate a globally unique ID using UUID to prevent collisions
    let uuid = Uuid::new_v4().simple().to_string();
    let short_uuid = uuid.chars().take(12).collect::<String>();

    format!("edge_{}", short_uuid)
}

/// Stored graph artefact node configuration
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredGraphArtefactNodeConfig {
    pub render_target: Option<String>,
    pub output_path: Option<String>,
    pub render_config: Option<StoredRenderConfig>,
}

/// Stored tree artefact node configuration (currently shares the same fields)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredTreeArtefactNodeConfig {
    pub render_target: Option<String>,
    pub output_path: Option<String>,
    pub render_config: Option<StoredRenderConfig>,
}

/// Stored render configuration
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<String>,
    pub use_default_styling: Option<bool>,
    pub theme: Option<String>,
}

impl StoredRenderConfig {
    pub fn into_render_config(self) -> RenderConfig {
        RenderConfig {
            contain_nodes: self.contain_nodes.unwrap_or(true),
            orientation: self
                .orientation
                .as_deref()
                .map(parse_orientation)
                .unwrap_or(RenderConfigOrientation::TB),
            use_default_styling: self.use_default_styling.unwrap_or(true),
            theme: self
                .theme
                .as_deref()
                .map(parse_theme)
                .unwrap_or(RenderConfigTheme::Light),
        }
    }
}

/// Parse orientation string
pub fn parse_orientation(value: &str) -> RenderConfigOrientation {
    match value {
        "LR" | "lr" | "Lr" | "lR" => RenderConfigOrientation::LR,
        _ => RenderConfigOrientation::TB,
    }
}

/// Parse theme string
pub fn parse_theme(value: &str) -> RenderConfigTheme {
    match value {
        "DARK" | "dark" | "Dark" => RenderConfigTheme::Dark,
        _ => RenderConfigTheme::Light,
    }
}

/// Ensure a local user session exists, creating one if needed
pub async fn ensure_local_user_session(
    db: &sea_orm::DatabaseConnection,
    session_id: &str,
    project_id: i32,
) -> async_graphql::Result<users::Model> {
    if let Some(existing_session) = user_sessions::Entity::find()
        .filter(user_sessions::Column::SessionId.eq(session_id))
        .filter(user_sessions::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(|e| {
            StructuredError::database("user_sessions::Entity::find (ensure_local_user_session)", e)
        })?
    {
        if let Some(user) = users::Entity::find_by_id(existing_session.user_id)
            .one(db)
            .await
            .map_err(|e| {
                StructuredError::database(
                    "users::Entity::find_by_id (ensure_local_user_session)",
                    e,
                )
            })?
        {
            ensure_project_collaborator(db, project_id, user.id).await?;
            return Ok(user);
        }
    }

    let sanitized: String = session_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let identifier = if sanitized.is_empty() {
        format!("sess{}", chrono::Utc::now().timestamp_micros())
    } else {
        sanitized
    };

    let username = format!("local_{identifier}");
    let email = format!("{identifier}@local.layercake");

    let mut user_active = users::ActiveModel::new();
    user_active.email = Set(email);
    user_active.username = Set(username.clone());
    user_active.display_name = Set("Local User".to_string());
    user_active.password_hash = Set(String::new());
    user_active.avatar_color = Set("#3b82f6".to_string());
    user_active.is_active = Set(true);
    user_active.user_type = Set("local".to_string());
    user_active.scoped_project_id = Set(Some(project_id));
    user_active.api_key_hash = Set(None);
    user_active.organisation_id = Set(None);
    user_active.created_at = Set(chrono::Utc::now());
    user_active.updated_at = Set(chrono::Utc::now());
    user_active.last_login_at = Set(None);

    let user = user_active.insert(db).await.map_err(|e| {
        StructuredError::database("users::ActiveModel::insert (ensure_local_user_session)", e)
    })?;

    ensure_project_collaborator(db, project_id, user.id).await?;

    let mut session_active = user_sessions::ActiveModel::new(user.id, username, project_id);
    session_active.session_id = Set(session_id.to_string());
    session_active.auth_method = Set("local".to_string());
    session_active.auth_context = Set(Some(json!({ "source": "local" }).to_string()));

    session_active.insert(db).await.map_err(|e| {
        StructuredError::database(
            "user_sessions::ActiveModel::insert (ensure_local_user_session)",
            e,
        )
    })?;

    Ok(user)
}

/// Ensure a user is a collaborator on a project
pub async fn ensure_project_collaborator(
    db: &sea_orm::DatabaseConnection,
    project_id: i32,
    user_id: i32,
) -> async_graphql::Result<()> {
    if let Some(collaborator) = project_collaborators::Entity::find()
        .filter(project_collaborators::Column::ProjectId.eq(project_id))
        .filter(project_collaborators::Column::UserId.eq(user_id))
        .filter(project_collaborators::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(|e| {
            StructuredError::database(
                "project_collaborators::Entity::find (ensure_project_collaborator)",
                e,
            )
        })?
    {
        if collaborator.invitation_status != "accepted" {
            let mut active: project_collaborators::ActiveModel = collaborator.into();
            active.invitation_status = Set("accepted".to_string());
            active.joined_at = Set(Some(chrono::Utc::now()));
            active.updated_at = Set(chrono::Utc::now());
            active.update(db).await.map_err(|e| {
                StructuredError::database("project_collaborators::ActiveModel::update", e)
            })?;
        }
        return Ok(());
    }

    let collaborator = project_collaborators::ActiveModel::new(
        project_id,
        user_id,
        project_collaborators::ProjectRole::Viewer,
        None,
    )
    .accept_invitation();

    collaborator.insert(db).await.map_err(|e| {
        StructuredError::database(
            "project_collaborators::ActiveModel::insert (ensure_project_collaborator)",
            e,
        )
    })?;

    Ok(())
}

/// Get file extension for render target format
pub fn get_extension_for_format(format: &str) -> &str {
    match format {
        "DOT" => "dot",
        "GML" => "gml",
        "JSON" => "json",
        "CSV" => "csv",
        "CSVNodes" => "csv",
        "CSVEdges" => "csv",
        "PlantUML" => "puml",
        "PlantUmlMindmap" => "puml",
        "Mermaid" => "mermaid",
        "MermaidMindmap" => "mmd",
        "MermaidTreemap" => "mmd",
        _ => "txt",
    }
}

/// Get MIME type for render target format
pub fn get_mime_type_for_format(format: &str) -> String {
    match format {
        "DOT" => "text/vnd.graphviz",
        "GML" => "text/plain",
        "JSON" => "application/json",
        "CSV" | "CSVNodes" | "CSVEdges" => "text/csv",
        "PlantUML" | "PlantUmlMindmap" => "text/plain",
        "Mermaid" | "MermaidMindmap" | "MermaidTreemap" => "text/plain",
        _ => "text/plain",
    }
    .to_string()
}

/// Parse render target string to ExportFileType enum
pub fn parse_export_format(format: &str) -> Result<ExportFileType> {
    match format {
        "DOT" => Ok(ExportFileType::DOT),
        "GML" => Ok(ExportFileType::GML),
        "JSON" => Ok(ExportFileType::JSON),
        "PlantUML" => Ok(ExportFileType::PlantUML),
        "PlantUmlMindmap" => Ok(ExportFileType::PlantUmlMindmap),
        "Mermaid" => Ok(ExportFileType::Mermaid),
        "MermaidMindmap" => Ok(ExportFileType::MermaidMindmap),
        "MermaidTreemap" => Ok(ExportFileType::MermaidTreemap),
        "CSVNodes" => Ok(ExportFileType::CSVNodes),
        "CSVEdges" => Ok(ExportFileType::CSVEdges),
        "CSV" => Ok(ExportFileType::CSVNodes), // Default CSV to nodes
        _ => Err(StructuredError::bad_request(format!(
            "Unsupported export format: {}",
            format
        ))),
    }
}

/// Result types for mutations
#[derive(async_graphql::SimpleObject)]
pub struct PlanExecutionResult {
    pub success: bool,
    pub message: String,
    pub output_files: Vec<String>,
}

#[derive(async_graphql::SimpleObject)]
pub struct NodeExecutionResult {
    pub success: bool,
    pub message: String,
    pub node_id: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct ExecutionActionResult {
    pub success: bool,
    pub message: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct ExportNodeOutputResult {
    pub success: bool,
    pub message: String,
    pub content: String, // Base64 encoded
    pub filename: String,
    pub mime_type: String,
}
