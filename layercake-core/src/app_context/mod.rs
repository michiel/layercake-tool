use std::sync::Arc;

use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::auth::{Actor, AllowAllAuthorizer, Authorizer};
use crate::database::entities::{data_sets, graph_data, plans, projects};
use crate::errors::{CoreError, CoreResult};
use crate::services::graph_analysis_service::GraphAnalysisService;
use crate::services::graph_edit_service::GraphEditService;
use crate::services::plan_service::PlanService;
use crate::services::{
    code_analysis_service::CodeAnalysisService, data_set_service::DataSetService,
    dataset_bulk_service::DataSetBulkService, AuthorizationService, ExportService, GraphService,
    ImportService, PlanDagService, ProjectRole,
};
use layercake_genai::{config::EmbeddingProviderConfig, services::DataAcquisitionService};

mod data_set_operations;
mod graph_operations;
mod library_operations;
mod plan_dag_operations;
mod plan_operations;
mod preview_operations;
mod project_operations;
mod story_operations;

/// Shared application context exposing core services for GraphQL, MCP, and console layers.
#[derive(Clone)]
pub struct AppContext {
    db: DatabaseConnection,
    authorizer: Arc<dyn Authorizer + Send + Sync>,
    import_service: Arc<ImportService>,
    export_service: Arc<ExportService>,
    graph_service: Arc<GraphService>,
    data_set_service: Arc<DataSetService>,
    data_set_bulk_service: Arc<DataSetBulkService>,
    plan_dag_service: Arc<PlanDagService>,
    plan_service: Arc<PlanService>,
    graph_edit_service: Arc<GraphEditService>,
    graph_analysis_service: Arc<GraphAnalysisService>,
    data_acquisition_service: Arc<DataAcquisitionService>,
    code_analysis_service: Arc<CodeAnalysisService>,
}

impl AppContext {
    pub fn new(db: DatabaseConnection) -> Self {
        let provider_hint = std::env::var("LAYERCAKE_EMBEDDING_PROVIDER")
            .ok()
            .or_else(|| std::env::var("LAYERCAKE_CHAT_PROVIDER").ok());
        let provider_config = EmbeddingProviderConfig::from_env();
        let data_acquisition_service = Arc::new(DataAcquisitionService::new(
            db.clone(),
            provider_hint,
            provider_config,
        ));
        Self::with_data_acquisition_and_authorizer(
            db,
            data_acquisition_service,
            Arc::new(AllowAllAuthorizer),
        )
    }

    pub fn with_data_acquisition(
        db: DatabaseConnection,
        data_acquisition_service: Arc<DataAcquisitionService>,
    ) -> Self {
        Self::with_data_acquisition_and_authorizer(
            db,
            data_acquisition_service,
            Arc::new(AllowAllAuthorizer),
        )
    }

    pub fn with_data_acquisition_and_authorizer(
        db: DatabaseConnection,
        data_acquisition_service: Arc<DataAcquisitionService>,
        authorizer: Arc<dyn Authorizer + Send + Sync>,
    ) -> Self {
        let import_service = Arc::new(ImportService::new(db.clone()));
        let export_service = Arc::new(ExportService::new(db.clone()));
        let graph_service = Arc::new(GraphService::new(db.clone()));
        let plan_dag_service = Arc::new(PlanDagService::new(db.clone()));
        let plan_service = Arc::new(PlanService::new(db.clone()));
        let graph_edit_service = Arc::new(GraphEditService::new(db.clone()));
        let graph_analysis_service = Arc::new(GraphAnalysisService::new(db.clone()));
        let data_set_service = Arc::new(DataSetService::new(db.clone()));
        let data_set_bulk_service = Arc::new(DataSetBulkService::new(db.clone()));
        let code_analysis_service = Arc::new(CodeAnalysisService::new(db.clone()));

        Self {
            db,
            authorizer,
            import_service,
            export_service,
            graph_service,
            data_set_service,
            data_set_bulk_service,
            plan_dag_service,
            plan_service,
            graph_edit_service,
            graph_analysis_service,
            data_acquisition_service,
            code_analysis_service,
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn authorizer(&self) -> &Arc<dyn Authorizer + Send + Sync> {
        &self.authorizer
    }

    pub fn authorize(
        &self,
        actor: &crate::auth::Actor,
        action: &str,
    ) -> crate::errors::CoreResult<()> {
        self.authorizer.authorize(actor, action)
    }

    async fn authorize_project_role(
        &self,
        actor: &Actor,
        project_id: i32,
        required_role: ProjectRole,
        action: &str,
    ) -> CoreResult<()> {
        self.authorize(actor, action)?;

        if actor.is_system() {
            return Ok(());
        }

        let user_id = actor
            .user_id
            .ok_or_else(|| CoreError::unauthorized("User is not authenticated"))?;
        let auth_service = AuthorizationService::new(self.db.clone());
        auth_service
            .check_project_access(user_id, project_id, Some(required_role))
            .await?;

        Ok(())
    }

    async fn authorize_project_read(&self, actor: &Actor, project_id: i32) -> CoreResult<()> {
        self.authorize_project_role(actor, project_id, ProjectRole::Viewer, "read:project")
            .await
    }

    async fn authorize_project_write(&self, actor: &Actor, project_id: i32) -> CoreResult<()> {
        self.authorize_project_role(actor, project_id, ProjectRole::Editor, "write:project")
            .await
    }

    async fn authorize_project_admin(&self, actor: &Actor, project_id: i32) -> CoreResult<()> {
        self.authorize_project_role(actor, project_id, ProjectRole::Owner, "admin:project")
            .await
    }

    async fn authorize_graph_write(&self, actor: &Actor, graph_id: i32) -> CoreResult<()> {
        let project_id = self.project_id_for_graph(graph_id).await?;
        self.authorize_project_write(actor, project_id).await
    }

    async fn authorize_data_set_write(&self, actor: &Actor, data_set_id: i32) -> CoreResult<()> {
        let project_id = self.project_id_for_data_set(data_set_id).await?;
        self.authorize_project_write(actor, project_id).await
    }

    async fn project_id_for_graph(&self, graph_id: i32) -> CoreResult<i32> {
        let graph = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph {}: {}", graph_id, e)))?
            .ok_or_else(|| CoreError::not_found("Graph", graph_id.to_string()))?;

        Ok(graph.project_id)
    }

    async fn project_id_for_data_set(&self, data_set_id: i32) -> CoreResult<i32> {
        let data_set = data_sets::Entity::find_by_id(data_set_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load data set {}: {}", data_set_id, e))
            })?
            .ok_or_else(|| CoreError::not_found("DataSet", data_set_id.to_string()))?;

        Ok(data_set.project_id)
    }

    async fn project_id_for_plan(&self, plan_id: i32) -> CoreResult<i32> {
        let plan = plans::Entity::find_by_id(plan_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load plan {}: {}", plan_id, e)))?
            .ok_or_else(|| CoreError::not_found("Plan", plan_id.to_string()))?;

        Ok(plan.project_id)
    }

    pub fn import_service(&self) -> &Arc<ImportService> {
        &self.import_service
    }

    pub fn export_service(&self) -> &Arc<ExportService> {
        &self.export_service
    }

    pub fn graph_service(&self) -> &Arc<GraphService> {
        &self.graph_service
    }

    #[allow(dead_code)]
    pub fn data_set_service(&self) -> &Arc<DataSetService> {
        &self.data_set_service
    }

    #[allow(dead_code)]
    pub fn data_set_bulk_service(&self) -> &Arc<DataSetBulkService> {
        &self.data_set_bulk_service
    }

    pub fn plan_dag_service(&self) -> &Arc<PlanDagService> {
        &self.plan_dag_service
    }

    pub fn plan_service(&self) -> &Arc<PlanService> {
        &self.plan_service
    }

    pub fn code_analysis_service(&self) -> &Arc<CodeAnalysisService> {
        &self.code_analysis_service
    }

    #[allow(dead_code)]
    pub fn graph_edit_service(&self) -> &Arc<GraphEditService> {
        &self.graph_edit_service
    }

    #[allow(dead_code)]
    pub fn graph_analysis_service(&self) -> &Arc<GraphAnalysisService> {
        &self.graph_analysis_service
    }

    pub fn data_acquisition_service(&self) -> &Arc<DataAcquisitionService> {
        &self.data_acquisition_service
    }
}

// ----- Public types -----

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub import_export_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<projects::Model> for ProjectSummary {
    fn from(model: projects::Model) -> Self {
        let tags = serde_json::from_str::<Vec<String>>(&model.tags).unwrap_or_default();
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            tags,
            import_export_path: model.import_export_path,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct ProjectArchiveFile {
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub description_is_set: bool,
    pub tags: Option<Vec<String>>,
    pub import_export_path: Option<Option<String>>,
}

impl ProjectUpdate {
    pub fn new(
        name: Option<String>,
        description: Option<String>,
        description_is_set: bool,
        tags: Option<Vec<String>>,
        import_export_path: Option<Option<String>>,
    ) -> Self {
        Self {
            name,
            description,
            description_is_set,
            tags,
            import_export_path,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<plans::Model> for PlanSummary {
    fn from(model: plans::Model) -> Self {
        let dependencies = model
            .dependencies
            .and_then(|value| serde_json::from_str::<Vec<i32>>(&value).ok());
        let tags = serde_json::from_str::<Vec<String>>(&model.tags).unwrap_or_default();

        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            description: model.description,
            tags,
            yaml_content: model.yaml_content,
            dependencies,
            status: model.status,
            version: model.version,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSetSummary {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub file_format: String,
    pub origin: String,
    pub filename: String,
    pub graph_json: String,
    pub status: String,
    pub error_message: Option<String>,
    pub file_size: i64,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub node_count: Option<usize>,
    pub edge_count: Option<usize>,
    pub layer_count: Option<usize>,
    pub has_layers: bool,
    pub annotations: Vec<crate::services::data_set_service::DataSetAnnotation>,
}

impl From<data_sets::Model> for DataSetSummary {
    fn from(model: data_sets::Model) -> Self {
        let (node_count, edge_count, layer_count) = summarize_graph_counts(&model.graph_json);
        let has_layers = layer_count.unwrap_or(0) > 0;
        let annotations = model
            .annotations
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_default();
        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            description: model.description,
            file_format: model.file_format,
            origin: model.origin,
            filename: model.filename,
            graph_json: model.graph_json,
            status: model.status,
            error_message: model.error_message,
            file_size: model.file_size,
            processed_at: model.processed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
            node_count,
            edge_count,
            layer_count,
            has_layers,
            annotations,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSetValidationSummary {
    pub data_set_id: i32,
    pub project_id: i32,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub checked_at: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphValidationSummary {
    pub graph_id: i32,
    pub project_id: i32,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub checked_at: DateTime<Utc>,
}

use crate::database::entities::common_types::{
    DataType as DataSetDataType, FileFormat as DataSetFileFormat,
};

#[derive(Clone)]
pub struct DataSetFileCreateRequest {
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_format: DataSetFileFormat,
    pub tabular_data_type: Option<DataSetDataType>,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct DataSetEmptyCreateRequest {
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone)]
pub struct BulkDataSetUpload {
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct DataSetUpdateRequest {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub new_file: Option<DataSetFileReplacement>,
}

#[derive(Clone)]
pub struct DataSetFileReplacement {
    pub filename: String,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum DataSetExportFormat {
    Xlsx,
    Ods,
}

impl DataSetExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            DataSetExportFormat::Xlsx => "xlsx",
            DataSetExportFormat::Ods => "ods",
        }
    }
}

#[derive(Clone)]
pub struct DataSetExportRequest {
    pub project_id: i32,
    pub data_set_ids: Vec<i32>,
    pub format: DataSetExportFormat,
}

#[derive(Clone)]
pub struct DataSetExportResult {
    pub data: Vec<u8>,
    pub filename: String,
    pub format: DataSetExportFormat,
}

#[derive(Clone, Copy)]
pub enum DataSetImportFormat {
    Xlsx,
    Ods,
}

impl DataSetImportFormat {
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        if lower.ends_with(".xlsx") {
            Some(DataSetImportFormat::Xlsx)
        } else if lower.ends_with(".ods") {
            Some(DataSetImportFormat::Ods)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct DataSetImportRequest {
    pub project_id: i32,
    pub format: DataSetImportFormat,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSetImportOutcome {
    pub data_sets: Vec<DataSetSummary>,
    pub created_count: i32,
    pub updated_count: i32,
}

use crate::plan_dag::{
    PlanDagEdge, PlanDagMetadata, PlanDagNode, PlanDagNodeType, Position,
};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDagSnapshot {
    pub version: String,
    pub nodes: Vec<PlanDagNode>,
    pub edges: Vec<PlanDagEdge>,
    pub metadata: PlanDagMetadata,
}

#[derive(Clone)]
pub struct PlanDagNodeRequest {
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub metadata: Value,
    pub config: Value,
}

#[derive(Clone)]
pub struct PlanDagNodeUpdateRequest {
    pub position: Option<Position>,
    pub metadata: Option<Value>,
    pub config: Option<Value>,
}

#[derive(Clone)]
pub struct PlanDagNodePositionRequest {
    pub node_id: String,
    pub position: Position,
    pub source_position: Option<String>,
    pub target_position: Option<String>,
}

#[derive(Clone)]
pub struct PlanDagEdgeRequest {
    pub source: String,
    pub target: String,
    pub metadata: Value,
}

#[derive(Clone)]
pub struct PlanDagEdgeUpdateRequest {
    pub metadata: Option<Value>,
}

#[derive(Clone)]
pub struct GraphNodeUpdateRequest {
    pub node_id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub attributes: Option<Value>,
    pub belongs_to: Option<String>,
}

#[derive(Clone)]
pub struct GraphLayerUpdateRequest {
    pub id: i32,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub properties: Option<Value>,
}

pub fn summarize_graph_counts(graph_json: &str) -> (Option<usize>, Option<usize>, Option<usize>) {
    serde_json::from_str::<Value>(graph_json)
        .ok()
        .map(|parsed| {
            let node_count = parsed
                .get("nodes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len());
            let edge_count = parsed
                .get("edges")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len());
            let layer_count = parsed
                .get("layers")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len());
            (node_count, edge_count, layer_count)
        })
        .unwrap_or((None, None, None))
}

#[derive(Clone)]
pub struct StoryExportResult {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryImportResult {
    pub imported_stories: Vec<StoryImportSummary>,
    pub created_count: i32,
    pub updated_count: i32,
    pub errors: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryImportSummary {
    pub id: i32,
    pub name: String,
    pub sequence_count: i32,
}
