use async_graphql::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::database::entities::{project_collaborators, user_sessions, users};
use crate::graphql::errors::StructuredError;
use crate::plan::{
    ExportFileType, GraphvizLayout, GraphvizRenderOptions, MermaidDisplay, MermaidLook,
    MermaidRenderOptions, RenderConfig, RenderConfigBuiltInStyle, RenderConfigOrientation,
    RenderConfigTheme, RenderTargetOptions,
};

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
    pub apply_layers: Option<bool>,
    pub built_in_styles: Option<String>,
    pub target_options: Option<StoredRenderTargetOptions>,
    pub use_default_styling: Option<bool>,
    pub theme: Option<String>,
    pub add_node_comments_as_notes: Option<bool>,
    pub note_position: Option<String>,
    pub use_node_weight: Option<bool>,
    pub use_edge_weight: Option<bool>,
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
            apply_layers: self
                .apply_layers
                .or_else(|| self.use_default_styling.map(|_| true))
                .unwrap_or(true),
            built_in_styles: self
                .built_in_styles
                .as_deref()
                .map(parse_builtin_style)
                .or_else(|| match (self.use_default_styling, self.theme.as_deref()) {
                    (Some(false), _) => Some(RenderConfigBuiltInStyle::None),
                    (Some(true), Some(theme)) => {
                        if matches!(parse_theme(theme), RenderConfigTheme::Dark) {
                            Some(RenderConfigBuiltInStyle::Dark)
                        } else {
                            Some(RenderConfigBuiltInStyle::Light)
                        }
                    }
                    (Some(true), None) => Some(RenderConfigBuiltInStyle::Light),
                    (None, Some(theme)) => {
                        if matches!(parse_theme(theme), RenderConfigTheme::Dark) {
                            Some(RenderConfigBuiltInStyle::Dark)
                        } else {
                            Some(RenderConfigBuiltInStyle::Light)
                        }
                    }
                    (None, _) => None,
                })
                .unwrap_or(RenderConfigBuiltInStyle::Light),
            target_options: self
                .target_options
                .map(|options| options.into_render_target_options())
                .unwrap_or_default(),
            add_node_comments_as_notes: self.add_node_comments_as_notes.unwrap_or(false),
            note_position: self
                .note_position
                .as_deref()
                .map(parse_note_position)
                .unwrap_or(crate::plan::NotePosition::Left),
            use_node_weight: self.use_node_weight.unwrap_or(true),
            use_edge_weight: self.use_edge_weight.unwrap_or(true),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRenderTargetOptions {
    pub graphviz: Option<StoredGraphvizRenderOptions>,
    pub mermaid: Option<StoredMermaidRenderOptions>,
}

impl StoredRenderTargetOptions {
    pub fn into_render_target_options(self) -> RenderTargetOptions {
        RenderTargetOptions {
            graphviz: self.graphviz.map(|opts| opts.into_graphviz_options()),
            mermaid: self.mermaid.map(|opts| opts.into_mermaid_options()),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredGraphvizRenderOptions {
    pub layout: Option<String>,
    pub overlap: Option<bool>,
    pub splines: Option<bool>,
    pub nodesep: Option<f32>,
    pub ranksep: Option<f32>,
    pub comment_style: Option<String>,
}

impl StoredGraphvizRenderOptions {
    pub fn into_graphviz_options(self) -> GraphvizRenderOptions {
        let mut options = GraphvizRenderOptions::default();
        if let Some(layout) = self.layout.as_deref() {
            options.layout = parse_graphviz_layout(layout);
        }
        if let Some(overlap) = self.overlap {
            options.overlap = overlap;
        }
        if let Some(splines) = self.splines {
            options.splines = splines;
        }
        if let Some(nodesep) = self.nodesep {
            options.nodesep = nodesep;
        }
        if let Some(ranksep) = self.ranksep {
            options.ranksep = ranksep;
        }
        if let Some(style) = self.comment_style.as_deref() {
            options.comment_style = parse_graphviz_comment_style(style);
        }
        options
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredMermaidRenderOptions {
    pub look: Option<String>,
    pub display: Option<String>,
}

impl StoredMermaidRenderOptions {
    pub fn into_mermaid_options(self) -> MermaidRenderOptions {
        let mut options = MermaidRenderOptions::default();
        if let Some(look) = self.look.as_deref() {
            options.look = parse_mermaid_look(look);
        }
        if let Some(display) = self.display.as_deref() {
            options.display = parse_mermaid_display(display);
        }
        options
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

pub fn parse_builtin_style(value: &str) -> RenderConfigBuiltInStyle {
    match value {
        "none" | "NONE" | "None" => RenderConfigBuiltInStyle::None,
        "dark" | "DARK" | "Dark" => RenderConfigBuiltInStyle::Dark,
        _ => RenderConfigBuiltInStyle::Light,
    }
}

pub fn parse_note_position(value: &str) -> crate::plan::NotePosition {
    match value {
        "right" | "RIGHT" | "Right" => crate::plan::NotePosition::Right,
        "top" | "TOP" | "Top" => crate::plan::NotePosition::Top,
        "bottom" | "BOTTOM" | "Bottom" => crate::plan::NotePosition::Bottom,
        _ => crate::plan::NotePosition::Left,
    }
}

pub fn parse_graphviz_layout(value: &str) -> GraphvizLayout {
    match value {
        "neato" | "NEATO" => GraphvizLayout::Neato,
        "fdp" | "FDP" => GraphvizLayout::Fdp,
        "circo" | "CIRCO" => GraphvizLayout::Circo,
        _ => GraphvizLayout::Dot,
    }
}

pub fn parse_graphviz_comment_style(value: &str) -> crate::plan::GraphvizCommentStyle {
    match value {
        "tooltip" | "TOOLTIP" | "Tooltip" => crate::plan::GraphvizCommentStyle::Tooltip,
        _ => crate::plan::GraphvizCommentStyle::Label,
    }
}

pub fn parse_mermaid_look(value: &str) -> MermaidLook {
    match value {
        "handDrawn" | "HAND_DRAWN" | "HANDDRAWN" => MermaidLook::HandDrawn,
        _ => MermaidLook::Default,
    }
}

pub fn parse_mermaid_display(value: &str) -> MermaidDisplay {
    match value {
        "compact" | "COMPACT" => MermaidDisplay::Compact,
        _ => MermaidDisplay::Full,
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
        "PlantUmlWbs" => "puml",
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
        "PlantUML" | "PlantUmlMindmap" | "PlantUmlWbs" => "text/plain",
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
        "PlantUmlWbs" => Ok(ExportFileType::PlantUmlWbs),
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
