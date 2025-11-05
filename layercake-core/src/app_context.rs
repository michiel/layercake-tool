use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::database::entities::{
    data_sources,
    data_sources::{DataType as DataSourceDataType, FileFormat as DataSourceFileFormat},
    graphs, plans, projects,
};
use crate::graphql::types::graph_node::GraphNode as GraphNodeDto;
use crate::graphql::types::layer::Layer as LayerDto;
use crate::graphql::types::plan_dag::{
    DataSourceExecutionMetadata, GraphExecutionMetadata, PlanDagEdge, PlanDagMetadata, PlanDagNode,
    PlanDagNodeType, Position,
};
use crate::plan::ExportFileType;
use crate::services::graph_analysis_service::{GraphAnalysisService, GraphConnectivityReport};
use crate::services::graph_edit_service::{
    GraphEditService, ReplaySummary as GraphEditReplaySummary,
};
use crate::services::plan_dag_service::PlanDagNodePositionUpdate;
use crate::services::{
    data_source_service::DataSourceService, datasource_bulk_service::DataSourceBulkService,
    ExportService, GraphService, ImportService, PlanDagService,
};

/// Shared application context exposing core services for GraphQL, MCP, and console layers.
#[derive(Clone)]
pub struct AppContext {
    db: DatabaseConnection,
    import_service: Arc<ImportService>,
    export_service: Arc<ExportService>,
    graph_service: Arc<GraphService>,
    data_source_service: Arc<DataSourceService>,
    data_source_bulk_service: Arc<DataSourceBulkService>,
    plan_dag_service: Arc<PlanDagService>,
    graph_edit_service: Arc<GraphEditService>,
    graph_analysis_service: Arc<GraphAnalysisService>,
}

impl AppContext {
    pub fn new(db: DatabaseConnection) -> Self {
        let import_service = Arc::new(ImportService::new(db.clone()));
        let export_service = Arc::new(ExportService::new(db.clone()));
        let graph_service = Arc::new(GraphService::new(db.clone()));
        let plan_dag_service = Arc::new(PlanDagService::new(db.clone()));
        let graph_edit_service = Arc::new(GraphEditService::new(db.clone()));
        let graph_analysis_service = Arc::new(GraphAnalysisService::new(db.clone()));
        let data_source_service = Arc::new(DataSourceService::new(db.clone()));
        let data_source_bulk_service = Arc::new(DataSourceBulkService::new(db.clone()));

        Self {
            db,
            import_service,
            export_service,
            graph_service,
            data_source_service,
            data_source_bulk_service,
            plan_dag_service,
            graph_edit_service,
            graph_analysis_service,
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn import_service(&self) -> Arc<ImportService> {
        self.import_service.clone()
    }

    pub fn export_service(&self) -> Arc<ExportService> {
        self.export_service.clone()
    }

    pub fn graph_service(&self) -> Arc<GraphService> {
        self.graph_service.clone()
    }

    pub fn data_source_service(&self) -> Arc<DataSourceService> {
        self.data_source_service.clone()
    }

    pub fn data_source_bulk_service(&self) -> Arc<DataSourceBulkService> {
        self.data_source_bulk_service.clone()
    }

    pub fn plan_dag_service(&self) -> Arc<PlanDagService> {
        self.plan_dag_service.clone()
    }

    pub fn graph_edit_service(&self) -> Arc<GraphEditService> {
        self.graph_edit_service.clone()
    }

    pub fn graph_analysis_service(&self) -> Arc<GraphAnalysisService> {
        self.graph_analysis_service.clone()
    }

    // ----- Project helpers -------------------------------------------------
    pub async fn list_projects(&self) -> Result<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        Ok(projects.into_iter().map(ProjectSummary::from).collect())
    }

    pub async fn get_project(&self, id: i32) -> Result<Option<ProjectSummary>> {
        let project = projects::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", id, e))?;

        Ok(project.map(ProjectSummary::from))
    }

    pub async fn create_project(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<ProjectSummary> {
        let now = Utc::now();
        let project = projects::ActiveModel {
            name: Set(name),
            description: Set(description),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let project = project
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to create project: {}", e))?;

        Ok(ProjectSummary::from(project))
    }

    pub async fn update_project(&self, id: i32, update: ProjectUpdate) -> Result<ProjectSummary> {
        let project = projects::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Project {} not found", id))?;

        let mut active: projects::ActiveModel = project.into();
        if let Some(name) = update.name {
            active.name = Set(name);
        }
        if update.description_is_set {
            active.description = Set(update.description);
        }
        active.updated_at = Set(Utc::now());

        let project = active
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to update project {}: {}", id, e))?;

        Ok(ProjectSummary::from(project))
    }

    pub async fn delete_project(&self, id: i32) -> Result<()> {
        let result = projects::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to delete project {}: {}", id, e))?;

        if result.rows_affected == 0 {
            return Err(anyhow!("Project {} not found", id));
        }

        Ok(())
    }

    // ----- Plan summary helpers -------------------------------------------

    pub async fn list_plans(&self, project_id: Option<i32>) -> Result<Vec<PlanSummary>> {
        let mut query = plans::Entity::find().order_by_desc(plans::Column::UpdatedAt);

        if let Some(project_id) = project_id {
            query = query.filter(plans::Column::ProjectId.eq(project_id));
        }

        let plans = query
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list plans: {}", e))?;

        Ok(plans.into_iter().map(PlanSummary::from).collect())
    }

    pub async fn get_plan(&self, id: i32) -> Result<Option<PlanSummary>> {
        let plan = plans::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn get_plan_for_project(&self, project_id: i32) -> Result<Option<PlanSummary>> {
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn create_plan(&self, request: PlanCreateRequest) -> Result<PlanSummary> {
        let PlanCreateRequest {
            project_id,
            name,
            yaml_content,
            dependencies,
            status,
        } = request;

        let dependencies_json = match dependencies {
            Some(values) => Some(
                serde_json::to_string(&values)
                    .map_err(|e| anyhow!("Invalid plan dependencies: {}", e))?,
            ),
            None => None,
        };

        let now = Utc::now();
        let plan = plans::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            yaml_content: Set(yaml_content),
            dependencies: Set(dependencies_json),
            status: Set(status.unwrap_or_else(|| "pending".to_string())),
            version: Set(1),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let plan = plan
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to create plan: {}", e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn update_plan(&self, id: i32, update: PlanUpdateRequest) -> Result<PlanSummary> {
        let plan = plans::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Plan {} not found", id))?;

        let PlanUpdateRequest {
            name,
            yaml_content,
            dependencies,
            dependencies_is_set,
            status,
        } = update;

        let mut active: plans::ActiveModel = plan.into();

        if let Some(name) = name {
            active.name = Set(name);
        }

        if let Some(content) = yaml_content {
            active.yaml_content = Set(content);
        }

        if dependencies_is_set {
            let dependencies_json = match dependencies {
                Some(values) => Some(
                    serde_json::to_string(&values)
                        .map_err(|e| anyhow!("Invalid plan dependencies: {}", e))?,
                ),
                None => None,
            };
            active.dependencies = Set(dependencies_json);
        }

        if let Some(status) = status {
            active.status = Set(status);
        }

        active.updated_at = Set(Utc::now());

        let plan = active
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to update plan {}: {}", id, e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn delete_plan(&self, id: i32) -> Result<()> {
        let result = plans::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to delete plan {}: {}", id, e))?;

        if result.rows_affected == 0 {
            return Err(anyhow!("Plan {} not found", id));
        }

        Ok(())
    }

    // ----- Data source helpers ---------------------------------------------

    pub async fn list_data_sources(&self, project_id: i32) -> Result<Vec<DataSourceSummary>> {
        let data_sources = data_sources::Entity::find()
            .filter(data_sources::Column::ProjectId.eq(project_id))
            .order_by_asc(data_sources::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to list data sources for project {}: {}",
                    project_id,
                    e
                )
            })?;

        Ok(data_sources
            .into_iter()
            .map(DataSourceSummary::from)
            .collect())
    }

    pub async fn available_data_sources(&self, project_id: i32) -> Result<Vec<DataSourceSummary>> {
        self.list_data_sources(project_id).await
    }

    pub async fn get_data_source(&self, id: i32) -> Result<Option<DataSourceSummary>> {
        let data_source = data_sources::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load data source {}: {}", id, e))?;

        Ok(data_source.map(DataSourceSummary::from))
    }

    pub async fn create_data_source_from_file(
        &self,
        request: DataSourceFileCreateRequest,
    ) -> Result<DataSourceSummary> {
        let DataSourceFileCreateRequest {
            project_id,
            name,
            description,
            filename,
            file_format,
            data_type,
            file_bytes,
        } = request;

        let created = self
            .data_source_service
            .create_from_file(
                project_id,
                name,
                description,
                filename,
                file_format,
                data_type,
                file_bytes,
            )
            .await
            .map_err(|e| anyhow!("Failed to create data source from file: {}", e))?;

        self.attach_data_source_to_plan(project_id, &created)
            .await?;

        Ok(DataSourceSummary::from(created))
    }

    pub async fn create_empty_data_source(
        &self,
        request: DataSourceEmptyCreateRequest,
    ) -> Result<DataSourceSummary> {
        let DataSourceEmptyCreateRequest {
            project_id,
            name,
            description,
            data_type,
        } = request;

        let created = self
            .data_source_service
            .create_empty(project_id, name, description, data_type)
            .await
            .map_err(|e| anyhow!("Failed to create empty data source: {}", e))?;

        self.attach_data_source_to_plan(project_id, &created)
            .await?;

        Ok(DataSourceSummary::from(created))
    }

    pub async fn bulk_upload_data_sources(
        &self,
        project_id: i32,
        uploads: Vec<BulkDataSourceUpload>,
    ) -> Result<Vec<DataSourceSummary>> {
        let mut results = Vec::new();

        for upload in uploads {
            let created = self
                .data_source_service
                .create_with_auto_detect(
                    project_id,
                    upload.name.clone(),
                    upload.description.clone(),
                    upload.filename.clone(),
                    upload.file_bytes.clone(),
                )
                .await
                .map_err(|e| anyhow!("Failed to import data source {}: {}", upload.filename, e))?;

            self.attach_data_source_to_plan(project_id, &created)
                .await?;
            results.push(DataSourceSummary::from(created));
        }

        Ok(results)
    }

    pub async fn update_data_source(
        &self,
        request: DataSourceUpdateRequest,
    ) -> Result<DataSourceSummary> {
        let DataSourceUpdateRequest {
            id,
            name,
            description,
            new_file,
        } = request;

        let (mut model, had_new_file) = if let Some(file) = new_file {
            let updated = self
                .data_source_service
                .update_file(id, file.filename, file.file_bytes)
                .await
                .map_err(|e| anyhow!("Failed to update data source file {}: {}", id, e))?;
            (updated, true)
        } else {
            let updated = self
                .data_source_service
                .update(id, name.clone(), description.clone())
                .await
                .map_err(|e| anyhow!("Failed to update data source {}: {}", id, e))?;
            (updated, false)
        };

        if had_new_file && (name.is_some() || description.is_some()) {
            model = self
                .data_source_service
                .update(id, name, description)
                .await
                .map_err(|e| anyhow!("Failed to update metadata for data source {}: {}", id, e))?;
        }

        Ok(DataSourceSummary::from(model))
    }

    pub async fn update_data_source_graph_json(
        &self,
        id: i32,
        graph_json: String,
    ) -> Result<DataSourceSummary> {
        let model = self
            .data_source_service
            .update_graph_data(id, graph_json)
            .await
            .map_err(|e| anyhow!("Failed to update graph data for data source {}: {}", id, e))?;

        Ok(DataSourceSummary::from(model))
    }

    pub async fn reprocess_data_source(&self, id: i32) -> Result<DataSourceSummary> {
        let model = self
            .data_source_service
            .reprocess(id)
            .await
            .map_err(|e| anyhow!("Failed to reprocess data source {}: {}", id, e))?;

        Ok(DataSourceSummary::from(model))
    }

    pub async fn delete_data_source(&self, id: i32) -> Result<()> {
        self.data_source_service
            .delete(id)
            .await
            .map_err(|e| anyhow!("Failed to delete data source {}: {}", id, e))
    }

    pub async fn export_data_sources(
        &self,
        request: DataSourceExportRequest,
    ) -> Result<DataSourceExportResult> {
        let bytes = match request.format {
            DataSourceExportFormat::Xlsx => self
                .data_source_bulk_service
                .export_to_xlsx(&request.data_source_ids)
                .await
                .map_err(|e| anyhow!("Failed to export datasources to XLSX: {}", e))?,
            DataSourceExportFormat::Ods => self
                .data_source_bulk_service
                .export_to_ods(&request.data_source_ids)
                .await
                .map_err(|e| anyhow!("Failed to export datasources to ODS: {}", e))?,
        };

        let filename = format!(
            "datasources_export_{}.{}",
            chrono::Utc::now().timestamp(),
            request.format.extension()
        );

        Ok(DataSourceExportResult {
            data: bytes,
            filename,
            format: request.format,
        })
    }

    pub async fn import_data_sources(
        &self,
        request: DataSourceImportRequest,
    ) -> Result<DataSourceImportOutcome> {
        let result = match request.format {
            DataSourceImportFormat::Xlsx => self
                .data_source_bulk_service
                .import_from_xlsx(request.project_id, &request.file_bytes)
                .await
                .map_err(|e| anyhow!("Failed to import datasources from XLSX: {}", e))?,
            DataSourceImportFormat::Ods => self
                .data_source_bulk_service
                .import_from_ods(request.project_id, &request.file_bytes)
                .await
                .map_err(|e| anyhow!("Failed to import datasources from ODS: {}", e))?,
        };

        if result.imported_ids.is_empty() {
            return Ok(DataSourceImportOutcome {
                data_sources: Vec::new(),
                created_count: result.created_count,
                updated_count: result.updated_count,
            });
        }

        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let models = data_sources::Entity::find()
            .filter(data_sources::Column::Id.is_in(result.imported_ids.clone()))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load imported datasources: {}", e))?;

        for model in &models {
            self.attach_data_source_to_plan(model.project_id, model)
                .await
                .ok();
        }

        Ok(DataSourceImportOutcome {
            data_sources: models.into_iter().map(DataSourceSummary::from).collect(),
            created_count: result.created_count,
            updated_count: result.updated_count,
        })
    }

    async fn attach_data_source_to_plan(
        &self,
        project_id: i32,
        data_source: &data_sources::Model,
    ) -> Result<()> {
        let nodes = self
            .plan_dag_service
            .get_nodes(project_id)
            .await
            .unwrap_or_default();

        let already_attached = nodes.iter().any(|node| {
            serde_json::from_str::<Value>(&node.config)
                .ok()
                .and_then(|config| config.get("dataSourceId").and_then(|id| id.as_i64()))
                .map(|id| id as i32 == data_source.id)
                .unwrap_or(false)
        });

        if already_attached {
            return Ok(());
        }

        let position = Position {
            x: 100.0,
            y: 100.0 + (nodes.len() as f64 * 120.0),
        };

        let metadata = json!({ "label": data_source.name });
        let config = json!({
            "dataSourceId": data_source.id,
            "filename": data_source.filename,
            "dataType": data_source.data_type.to_lowercase(),
        });

        let _ = self
            .create_plan_dag_node(
                project_id,
                PlanDagNodeRequest {
                    node_type: PlanDagNodeType::DataSource,
                    position,
                    metadata,
                    config,
                },
            )
            .await?;

        Ok(())
    }

    // ----- Plan DAG helpers -------------------------------------------------
    pub async fn load_plan_dag(&self, project_id: i32) -> Result<Option<PlanDagSnapshot>> {
        let project = match projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", project_id, e))?
        {
            Some(project) => project,
            None => return Ok(None),
        };

        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?;

        if let Some(plan) = plan {
            let mut nodes = self
                .plan_dag_service
                .get_nodes(project_id)
                .await
                .map_err(|e| anyhow!("Failed to load Plan DAG nodes: {}", e))?;
            let edges = self
                .plan_dag_service
                .get_edges(project_id)
                .await
                .map_err(|e| anyhow!("Failed to load Plan DAG edges: {}", e))?;

            for idx in 0..nodes.len() {
                let node_type = nodes[idx].node_type;
                let node_id = nodes[idx].id.clone();

                match node_type {
                    PlanDagNodeType::DataSource => {
                        if let Ok(config) =
                            serde_json::from_str::<serde_json::Value>(&nodes[idx].config)
                        {
                            if let Some(data_source_id) = config
                                .get("dataSourceId")
                                .and_then(|v| v.as_i64())
                                .map(|v| v as i32)
                            {
                                if let Some(data_source) =
                                    data_sources::Entity::find_by_id(data_source_id)
                                        .one(&self.db)
                                        .await
                                        .map_err(|e| {
                                            anyhow!(
                                                "Failed to load data source {}: {}",
                                                data_source_id,
                                                e
                                            )
                                        })?
                                {
                                    let execution_state = match data_source.status.as_str() {
                                        "active" => "completed",
                                        "processing" => "processing",
                                        "error" => "error",
                                        _ => "not_started",
                                    }
                                    .to_string();

                                    nodes[idx].datasource_execution =
                                        Some(DataSourceExecutionMetadata {
                                            data_source_id: data_source.id,
                                            filename: data_source.filename.clone(),
                                            status: data_source.status.clone(),
                                            processed_at: data_source
                                                .processed_at
                                                .map(|d| d.to_rfc3339()),
                                            execution_state,
                                            error_message: data_source.error_message.clone(),
                                        });
                                }
                            }
                        }
                    }
                    PlanDagNodeType::Graph => {
                        if let Some(graph) = graphs::Entity::find()
                            .filter(graphs::Column::ProjectId.eq(project_id))
                            .filter(graphs::Column::NodeId.eq(node_id.clone()))
                            .one(&self.db)
                            .await
                            .map_err(|e| {
                                anyhow!(
                                    "Failed to load graph execution for node {}: {}",
                                    node_id,
                                    e
                                )
                            })?
                        {
                            nodes[idx].graph_execution = Some(GraphExecutionMetadata {
                                graph_id: graph.id,
                                node_count: graph.node_count,
                                edge_count: graph.edge_count,
                                execution_state: graph.execution_state.clone(),
                                computed_date: graph.computed_date.map(|d| d.to_rfc3339()),
                                error_message: graph.error_message.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            let metadata = PlanDagMetadata {
                version: plan.version.to_string(),
                name: Some(plan.name.clone()),
                description: None,
                created: Some(plan.created_at.to_rfc3339()),
                last_modified: Some(plan.updated_at.to_rfc3339()),
                author: None,
            };

            Ok(Some(PlanDagSnapshot {
                version: metadata.version.clone(),
                nodes,
                edges,
                metadata,
            }))
        } else {
            let metadata = PlanDagMetadata {
                version: "1.0".to_string(),
                name: Some(format!("{} Plan DAG", project.name)),
                description: project.description.clone(),
                created: Some(project.created_at.to_rfc3339()),
                last_modified: Some(project.updated_at.to_rfc3339()),
                author: None,
            };

            Ok(Some(PlanDagSnapshot {
                version: metadata.version.clone(),
                nodes: Vec::new(),
                edges: Vec::new(),
                metadata,
            }))
        }
    }

    // ----- Plan DAG mutations ----------------------------------------------

    pub async fn create_plan_dag_node(
        &self,
        project_id: i32,
        request: PlanDagNodeRequest,
    ) -> Result<PlanDagNode> {
        // Ensure plan exists before inspecting existing nodes
        self.plan_dag_service
            .get_or_create_plan(project_id)
            .await
            .map_err(|e| anyhow!("Failed to prepare plan for project {}: {}", project_id, e))?;

        let existing_nodes = self
            .plan_dag_service
            .get_nodes(project_id)
            .await
            .unwrap_or_default();

        let node_id = generate_node_id(&request.node_type, &existing_nodes)?;
        let node_type = node_type_storage_name(&request.node_type).to_string();
        let metadata_json = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Invalid node metadata: {}", e))?;
        let config_json = serde_json::to_string(&request.config)
            .map_err(|e| anyhow!("Invalid node config: {}", e))?;

        self.plan_dag_service
            .create_node(
                project_id,
                node_id,
                node_type,
                request.position,
                metadata_json,
                config_json,
            )
            .await
    }

    pub async fn update_plan_dag_node(
        &self,
        project_id: i32,
        node_id: String,
        updates: PlanDagNodeUpdateRequest,
    ) -> Result<PlanDagNode> {
        let metadata_json = if let Some(metadata) = updates.metadata {
            Some(
                serde_json::to_string(&metadata)
                    .map_err(|e| anyhow!("Invalid node metadata: {}", e))?,
            )
        } else {
            None
        };

        let config_json = if let Some(config) = updates.config {
            Some(
                serde_json::to_string(&config)
                    .map_err(|e| anyhow!("Invalid node config: {}", e))?,
            )
        } else {
            None
        };

        self.plan_dag_service
            .update_node(
                project_id,
                node_id,
                updates.position,
                metadata_json,
                config_json,
            )
            .await
    }

    pub async fn delete_plan_dag_node(
        &self,
        project_id: i32,
        node_id: String,
    ) -> Result<PlanDagNode> {
        self.plan_dag_service.delete_node(project_id, node_id).await
    }

    pub async fn move_plan_dag_node(
        &self,
        project_id: i32,
        node_id: String,
        position: Position,
    ) -> Result<PlanDagNode> {
        self.plan_dag_service
            .move_node(project_id, node_id, position)
            .await
    }

    pub async fn batch_move_plan_dag_nodes(
        &self,
        project_id: i32,
        positions: Vec<PlanDagNodePositionRequest>,
    ) -> Result<Vec<PlanDagNode>> {
        let updates = positions
            .into_iter()
            .map(|p| PlanDagNodePositionUpdate {
                node_id: p.node_id,
                position: p.position,
                source_position: p.source_position,
                target_position: p.target_position,
            })
            .collect();

        self.plan_dag_service
            .batch_move_nodes(project_id, updates)
            .await
    }

    pub async fn create_plan_dag_edge(
        &self,
        project_id: i32,
        request: PlanDagEdgeRequest,
    ) -> Result<PlanDagEdge> {
        // Ensure plan exists before creating edge
        self.plan_dag_service
            .get_or_create_plan(project_id)
            .await
            .map_err(|e| anyhow!("Failed to prepare plan for project {}: {}", project_id, e))?;

        let edge_id = generate_edge_id(&request.source, &request.target);
        let metadata_json = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Invalid edge metadata: {}", e))?;

        self.plan_dag_service
            .create_edge(
                project_id,
                edge_id,
                request.source,
                request.target,
                metadata_json,
            )
            .await
    }

    pub async fn update_plan_dag_edge(
        &self,
        project_id: i32,
        edge_id: String,
        updates: PlanDagEdgeUpdateRequest,
    ) -> Result<PlanDagEdge> {
        let metadata_json = if let Some(metadata) = updates.metadata {
            Some(
                serde_json::to_string(&metadata)
                    .map_err(|e| anyhow!("Invalid edge metadata: {}", e))?,
            )
        } else {
            None
        };

        self.plan_dag_service
            .update_edge(project_id, edge_id, metadata_json)
            .await
    }

    pub async fn delete_plan_dag_edge(
        &self,
        project_id: i32,
        edge_id: String,
    ) -> Result<PlanDagEdge> {
        self.plan_dag_service.delete_edge(project_id, edge_id).await
    }

    // ----- Graph editing helpers ------------------------------------------

    pub async fn update_graph_node(
        &self,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attrs: Option<Value>,
        belongs_to: Option<String>,
    ) -> Result<GraphNodeDto> {
        use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as GraphNodes};
        use sea_orm::{ColumnTrait, QueryFilter};

        let old_node = GraphNodes::find()
            .filter(NodeColumn::GraphId.eq(graph_id))
            .filter(NodeColumn::Id.eq(&node_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load graph node {}: {}", node_id, e))?;

        let belongs_to_param = belongs_to.as_ref().map(|value| {
            if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        });

        let updated_node = self
            .graph_service
            .update_graph_node(
                graph_id,
                node_id.clone(),
                label.clone(),
                layer.clone(),
                attrs.clone(),
                belongs_to_param.clone(),
            )
            .await
            .map_err(|e| anyhow!("Failed to update graph node {}: {}", node_id, e))?;

        if let Some(old_node) = old_node {
            if let Some(new_label) = &label {
                if old_node.label.as_ref() != Some(new_label) {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("label".to_string()),
                            old_node.label.as_ref().map(|l| json!(l)),
                            Some(json!(new_label)),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_layer) = &layer {
                let old_layer_value = old_node.layer.clone().unwrap_or_default();
                if &old_layer_value != new_layer {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("layer".to_string()),
                            if old_layer_value.is_empty() {
                                None
                            } else {
                                Some(json!(old_layer_value))
                            },
                            Some(json!(new_layer)),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_attrs) = &attrs {
                if old_node.attrs.as_ref() != Some(new_attrs) {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("attrs".to_string()),
                            old_node.attrs.clone(),
                            Some(new_attrs.clone()),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_belongs_to) = belongs_to_param.clone() {
                if old_node.belongs_to != new_belongs_to {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("belongsTo".to_string()),
                            old_node.belongs_to.as_ref().map(|b| json!(b)),
                            new_belongs_to.as_ref().map(|b| json!(b)),
                            None,
                            true,
                        )
                        .await;
                }
            }
        }

        Ok(GraphNodeDto::from(updated_node))
    }

    pub async fn update_layer_properties(
        &self,
        layer_id: i32,
        name: Option<String>,
        properties: Option<Value>,
    ) -> Result<LayerDto> {
        use crate::database::entities::graph_layers::Entity as Layers;

        let old_layer = Layers::find_by_id(layer_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load layer {}: {}", layer_id, e))?;

        let updated_layer = self
            .graph_service
            .update_layer_properties(layer_id, name.clone(), properties.clone())
            .await
            .map_err(|e| anyhow!("Failed to update layer {}: {}", layer_id, e))?;

        if let Some(old_layer) = old_layer {
            if let Some(new_name) = &name {
                if &old_layer.name != new_name {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            old_layer.graph_id,
                            "layer".to_string(),
                            old_layer.layer_id.clone(),
                            "update".to_string(),
                            Some("name".to_string()),
                            Some(json!(old_layer.name)),
                            Some(json!(new_name)),
                            None,
                            true,
                        )
                        .await;
                }
            }

            if let Some(new_properties) = &properties {
                let old_props = old_layer
                    .properties
                    .and_then(|p| serde_json::from_str::<Value>(&p).ok());

                if old_props.as_ref() != Some(new_properties) {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            old_layer.graph_id,
                            "layer".to_string(),
                            old_layer.layer_id.clone(),
                            "update".to_string(),
                            Some("properties".to_string()),
                            old_props,
                            Some(new_properties.clone()),
                            None,
                            true,
                        )
                        .await;
                }
            }
        }

        Ok(LayerDto::from(updated_layer))
    }

    pub async fn bulk_update_graph_data(
        &self,
        graph_id: i32,
        node_updates: Vec<GraphNodeUpdateRequest>,
        layer_updates: Vec<GraphLayerUpdateRequest>,
    ) -> Result<()> {
        for node_update in node_updates {
            self.update_graph_node(
                graph_id,
                node_update.node_id,
                node_update.label,
                node_update.layer,
                node_update.attrs,
                node_update.belongs_to,
            )
            .await?;
        }

        for layer_update in layer_updates {
            self.update_layer_properties(
                layer_update.id,
                layer_update.name,
                layer_update.properties,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn replay_graph_edits(&self, graph_id: i32) -> Result<GraphEditReplaySummary> {
        self.graph_edit_service
            .replay_graph_edits(graph_id)
            .await
            .map_err(|e| anyhow!("Failed to replay graph edits: {}", e))
    }

    pub async fn analyze_graph_connectivity(
        &self,
        graph_id: i32,
    ) -> Result<GraphConnectivityReport> {
        self.graph_analysis_service
            .analyze_connectivity(graph_id)
            .await
            .map_err(|e| anyhow!("Failed to analyze graph connectivity: {}", e))
    }

    pub async fn find_graph_paths(
        &self,
        graph_id: i32,
        source_node: String,
        target_node: String,
        max_paths: usize,
    ) -> Result<Vec<Vec<String>>> {
        self.graph_analysis_service
            .find_paths(graph_id, &source_node, &target_node, max_paths)
            .await
            .map_err(|e| anyhow!("Failed to find graph paths: {}", e))
    }
    pub async fn preview_graph_export(
        &self,
        graph_id: i32,
        format: ExportFileType,
        max_rows: Option<usize>,
    ) -> Result<String> {
        let graph = self
            .graph_service
            .build_graph_from_dag_graph(graph_id)
            .await
            .map_err(|e| anyhow!("Failed to load graph {}: {}", graph_id, e))?;

        let content = self
            .export_service
            .export_to_string(&graph, &format)
            .map_err(|e| anyhow!("Failed to render graph export: {}", e))?;

        Ok(apply_preview_limit(content, format, max_rows))
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<projects::Model> for ProjectSummary {
    fn from(model: projects::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub description_is_set: bool,
}

impl ProjectUpdate {
    pub fn new(
        name: Option<String>,
        description: Option<String>,
        description_is_set: bool,
    ) -> Self {
        Self {
            name,
            description,
            description_is_set,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
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

        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            yaml_content: model.yaml_content,
            dependencies,
            status: model.status,
            version: model.version,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct PlanCreateRequest {
    pub project_id: i32,
    pub name: String,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
    pub status: Option<String>,
}

#[derive(Clone)]
pub struct PlanUpdateRequest {
    pub name: Option<String>,
    pub yaml_content: Option<String>,
    pub dependencies: Option<Vec<i32>>,
    pub dependencies_is_set: bool,
    pub status: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceSummary {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub file_format: String,
    pub data_type: String,
    pub filename: String,
    pub graph_json: String,
    pub status: String,
    pub error_message: Option<String>,
    pub file_size: i64,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<data_sources::Model> for DataSourceSummary {
    fn from(model: data_sources::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            description: model.description,
            file_format: model.file_format,
            data_type: model.data_type,
            filename: model.filename,
            graph_json: model.graph_json,
            status: model.status,
            error_message: model.error_message,
            file_size: model.file_size,
            processed_at: model.processed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct DataSourceFileCreateRequest {
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_format: DataSourceFileFormat,
    pub data_type: DataSourceDataType,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct DataSourceEmptyCreateRequest {
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub data_type: DataSourceDataType,
}

#[derive(Clone)]
pub struct BulkDataSourceUpload {
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct DataSourceUpdateRequest {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub new_file: Option<DataSourceFileReplacement>,
}

#[derive(Clone)]
pub struct DataSourceFileReplacement {
    pub filename: String,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum DataSourceExportFormat {
    Xlsx,
    Ods,
}

impl DataSourceExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            DataSourceExportFormat::Xlsx => "xlsx",
            DataSourceExportFormat::Ods => "ods",
        }
    }
}

#[derive(Clone)]
pub struct DataSourceExportRequest {
    pub project_id: i32,
    pub data_source_ids: Vec<i32>,
    pub format: DataSourceExportFormat,
}

#[derive(Clone)]
pub struct DataSourceExportResult {
    pub data: Vec<u8>,
    pub filename: String,
    pub format: DataSourceExportFormat,
}

#[derive(Clone, Copy)]
pub enum DataSourceImportFormat {
    Xlsx,
    Ods,
}

impl DataSourceImportFormat {
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        if lower.ends_with(".xlsx") {
            Some(DataSourceImportFormat::Xlsx)
        } else if lower.ends_with(".ods") {
            Some(DataSourceImportFormat::Ods)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct DataSourceImportRequest {
    pub project_id: i32,
    pub format: DataSourceImportFormat,
    pub file_bytes: Vec<u8>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceImportOutcome {
    pub data_sources: Vec<DataSourceSummary>,
    pub created_count: i32,
    pub updated_count: i32,
}

#[derive(Clone, Serialize)]
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
    pub attrs: Option<Value>,
    pub belongs_to: Option<String>,
}

#[derive(Clone)]
pub struct GraphLayerUpdateRequest {
    pub id: i32,
    pub name: Option<String>,
    pub properties: Option<Value>,
}

fn node_type_prefix(node_type: &PlanDagNodeType) -> &'static str {
    match node_type {
        PlanDagNodeType::DataSource => "datasource",
        PlanDagNodeType::Graph => "graph",
        PlanDagNodeType::Transform => "transform",
        PlanDagNodeType::Filter => "filter",
        PlanDagNodeType::Merge => "merge",
        PlanDagNodeType::Copy => "copy",
        PlanDagNodeType::Output => "output",
    }
}

fn node_type_storage_name(node_type: &PlanDagNodeType) -> &'static str {
    match node_type {
        PlanDagNodeType::DataSource => "DataSourceNode",
        PlanDagNodeType::Graph => "GraphNode",
        PlanDagNodeType::Transform => "TransformNode",
        PlanDagNodeType::Filter => "FilterNode",
        PlanDagNodeType::Merge => "MergeNode",
        PlanDagNodeType::Copy => "CopyNode",
        PlanDagNodeType::Output => "OutputNode",
    }
}

fn generate_node_id(node_type: &PlanDagNodeType, existing_nodes: &[PlanDagNode]) -> Result<String> {
    let prefix = node_type_prefix(node_type);
    let regex = Regex::new(r"_(\d+)$").map_err(|e| anyhow!("Invalid regex: {}", e))?;

    let max_number = existing_nodes
        .iter()
        .filter(|node| node.id.starts_with(prefix))
        .filter_map(|node| {
            regex
                .captures(&node.id)
                .and_then(|caps| caps.get(1))
                .and_then(|m| m.as_str().parse::<i32>().ok())
        })
        .max()
        .unwrap_or(0);

    Ok(format!("{}_{}", prefix, format!("{:03}", max_number + 1)))
}

fn generate_edge_id(source: &str, target: &str) -> String {
    const MAX_ID_LEN: usize = 50;
    const RANDOM_LEN: usize = 12;
    // Keep generated IDs readable while respecting validation rules.
    let random_segment = {
        let uuid = Uuid::new_v4().simple().to_string();
        uuid.chars().take(RANDOM_LEN).collect::<String>()
    };

    let mut source_segment = source.chars().take(16).collect::<String>();
    let mut target_segment = target.chars().take(16).collect::<String>();

    let mut candidate = format!(
        "edge_{}_{}_{}",
        source_segment, target_segment, random_segment
    );

    if candidate.len() > MAX_ID_LEN {
        source_segment = source.chars().take(10).collect();
        target_segment = target.chars().take(10).collect();
        candidate = format!(
            "edge_{}_{}_{}",
            source_segment, target_segment, random_segment
        );
    }

    if candidate.len() > MAX_ID_LEN {
        format!("edge_{}", random_segment)
    } else {
        candidate
    }
}

fn apply_preview_limit(content: String, format: ExportFileType, max_rows: Option<usize>) -> String {
    match (format, max_rows) {
        (
            ExportFileType::CSVNodes | ExportFileType::CSVEdges | ExportFileType::CSVMatrix,
            Some(limit),
        ) => {
            let mut limited_lines = Vec::new();

            for (index, line) in content.lines().enumerate() {
                if index == 0 || index <= limit {
                    limited_lines.push(line.to_string());
                } else {
                    break;
                }
            }

            limited_lines.join("\n")
        }
        _ => content,
    }
}
