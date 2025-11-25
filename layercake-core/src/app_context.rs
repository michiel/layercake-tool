use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, Write};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use zip::{result::ZipError, write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::database::entities::common_types::{
    DataType as DataSetDataType, FileFormat as DataSetFileFormat,
};
use crate::database::entities::{
    data_sets, graphs, layer_aliases, library_items, plan_dag_edges, plan_dag_nodes, plans,
    project_layers, projects, sequences, stories,
};
use crate::export::{
    sequence_renderer::SequenceRenderConfigResolved, to_mermaid_sequence, to_plantuml_sequence,
};
use crate::graphql::types::graph_node::GraphNode as GraphNodeDto;
use crate::graphql::types::layer::Layer as LayerDto;
use crate::graphql::types::plan_dag::config::SequenceArtefactRenderTarget;
use crate::graphql::types::plan_dag::{
    DataSetExecutionMetadata, GraphExecutionMetadata, PlanDagEdge, PlanDagMetadata, PlanDagNode,
    PlanDagNodeType, Position,
};
use crate::plan::{ExportFileType, RenderConfig};
use crate::sequence_context::{apply_render_config, build_story_context};
use crate::services::graph_analysis_service::{GraphAnalysisService, GraphConnectivityReport};
use crate::services::graph_edit_service::{
    GraphEditService, ReplaySummary as GraphEditReplaySummary,
};
use crate::services::library_item_service::{
    LibraryItemService, ITEM_TYPE_PROJECT, ITEM_TYPE_PROJECT_TEMPLATE,
};
use crate::services::plan_dag_service::PlanDagNodePositionUpdate;
use crate::services::plan_service::{PlanCreateRequest, PlanService, PlanUpdateRequest};
use crate::services::{
    data_set_service::DataSetService, dataset_bulk_service::DataSetBulkService, ExportService,
    GraphService, ImportService, PlanDagService,
};
use layercake_data_acquisition::{
    config::EmbeddingProviderConfig,
    entities::{
        file_tags as kb_file_tags, files as kb_files, kb_documents as kb_docs, tags as kb_tags,
        vector_index_state as kb_vector_state,
    },
    services::DataAcquisitionService,
};

/// Shared application context exposing core services for GraphQL, MCP, and console layers.
#[derive(Clone)]
pub struct AppContext {
    db: DatabaseConnection,
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
        Self::with_data_acquisition(db, data_acquisition_service)
    }

    pub fn with_data_acquisition(
        db: DatabaseConnection,
        data_acquisition_service: Arc<DataAcquisitionService>,
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

        Self {
            db,
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
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
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

    // ----- Project helpers -------------------------------------------------
    pub async fn list_projects(&self) -> Result<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        Ok(projects.into_iter().map(ProjectSummary::from).collect())
    }

    pub async fn list_projects_filtered(
        &self,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        // If tags filter is provided, filter projects by tags
        let filtered_projects = if let Some(filter_tags) = tags {
            if filter_tags.is_empty() {
                projects
            } else {
                projects
                    .into_iter()
                    .filter(|project| {
                        let project_tags: Vec<String> =
                            serde_json::from_str(&project.tags).unwrap_or_default();
                        // Check if any filter tag matches any project tag
                        filter_tags
                            .iter()
                            .any(|filter_tag| project_tags.contains(filter_tag))
                    })
                    .collect()
            }
        } else {
            projects
        };

        Ok(filtered_projects
            .into_iter()
            .map(ProjectSummary::from)
            .collect())
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
        tags: Option<Vec<String>>,
    ) -> Result<ProjectSummary> {
        let now = Utc::now();
        let tags_json =
            serde_json::to_string(&tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
        let project = projects::ActiveModel {
            name: Set(name),
            description: Set(description),
            tags: Set(tags_json),
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
        if let Some(tags) = update.tags {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            active.tags = Set(tags_json);
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

    #[allow(dead_code)]
    pub async fn list_plans(&self, project_id: Option<i32>) -> Result<Vec<PlanSummary>> {
        let plans = if let Some(project_id) = project_id {
            self.plan_service
                .list_plans(project_id)
                .await
                .map_err(|e| anyhow!("Failed to list plans: {}", e))?
        } else {
            plans::Entity::find()
                .order_by_desc(plans::Column::UpdatedAt)
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to list plans: {}", e))?
        };

        Ok(plans.into_iter().map(PlanSummary::from).collect())
    }

    pub async fn get_plan(&self, id: i32) -> Result<Option<PlanSummary>> {
        let plan = self
            .plan_service
            .get_plan(id)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn get_plan_for_project(&self, project_id: i32) -> Result<Option<PlanSummary>> {
        let plan = self
            .plan_service
            .get_default_plan(project_id)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn create_plan(&self, request: PlanCreateRequest) -> Result<PlanSummary> {
        let plan = self
            .plan_service
            .create_plan(request)
            .await
            .map_err(|e| anyhow!("Failed to create plan: {}", e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn update_plan(&self, id: i32, update: PlanUpdateRequest) -> Result<PlanSummary> {
        let plan = self
            .plan_service
            .update_plan(id, update)
            .await
            .map_err(|e| anyhow!("Failed to update plan {}: {}", id, e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn delete_plan(&self, id: i32) -> Result<()> {
        self.plan_service
            .delete_plan(id)
            .await
            .map_err(|e| anyhow!("Failed to delete plan {}: {}", id, e))
    }

    pub async fn duplicate_plan(&self, id: i32, name: String) -> Result<PlanSummary> {
        let plan = self
            .plan_service
            .duplicate_plan(id, name)
            .await
            .map_err(|e| anyhow!("Failed to duplicate plan {}: {}", id, e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn resolve_plan_model(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> Result<plans::Model> {
        if let Some(plan_id) = plan_id {
            let plan = self
                .plan_service
                .get_plan(plan_id)
                .await
                .map_err(|e| anyhow!("Failed to load plan {}: {}", plan_id, e))?
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

            if plan.project_id != project_id {
                return Err(anyhow!(
                    "Plan {} does not belong to project {}",
                    plan_id,
                    project_id
                ));
            }

            Ok(plan)
        } else {
            self.plan_service
                .get_default_plan(project_id)
                .await
                .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?
                .ok_or_else(|| anyhow!("Project {} has no plan", project_id))
        }
    }

    // ----- Data set helpers ---------------------------------------------

    pub async fn list_data_sets(&self, project_id: i32) -> Result<Vec<DataSetSummary>> {
        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .order_by_asc(data_sets::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list data sets for project {}: {}", project_id, e))?;

        Ok(data_sets.into_iter().map(DataSetSummary::from).collect())
    }

    pub async fn available_data_sets(&self, project_id: i32) -> Result<Vec<DataSetSummary>> {
        self.list_data_sets(project_id).await
    }

    pub async fn get_data_set(&self, id: i32) -> Result<Option<DataSetSummary>> {
        let data_set = data_sets::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load data set {}: {}", id, e))?;

        Ok(data_set.map(DataSetSummary::from))
    }

    pub async fn create_data_set_from_file(
        &self,
        request: DataSetFileCreateRequest,
    ) -> Result<DataSetSummary> {
        let DataSetFileCreateRequest {
            project_id,
            name,
            description,
            filename,
            file_format,
            tabular_data_type,
            file_bytes,
        } = request;

        let created = self
            .data_set_service
            .create_from_file(
                project_id,
                name,
                description,
                filename,
                file_format,
                file_bytes,
                tabular_data_type,
            )
            .await
            .map_err(|e| anyhow!("Failed to create data set from file: {}", e))?;

        Ok(DataSetSummary::from(created))
    }

    pub async fn create_empty_data_set(
        &self,
        request: DataSetEmptyCreateRequest,
    ) -> Result<DataSetSummary> {
        let DataSetEmptyCreateRequest {
            project_id,
            name,
            description,
        } = request;

        let created = self
            .data_set_service
            .create_empty(project_id, name, description)
            .await
            .map_err(|e| anyhow!("Failed to create empty data set: {}", e))?;

        Ok(DataSetSummary::from(created))
    }

    pub async fn bulk_upload_data_sets(
        &self,
        project_id: i32,
        uploads: Vec<BulkDataSetUpload>,
    ) -> Result<Vec<DataSetSummary>> {
        let mut results = Vec::new();

        for upload in uploads {
            let created = self
                .data_set_service
                .create_with_auto_detect(
                    project_id,
                    upload.name.clone(),
                    upload.description.clone(),
                    upload.filename.clone(),
                    upload.file_bytes.clone(),
                )
                .await
                .map_err(|e| anyhow!("Failed to import data set {}: {}", upload.filename, e))?;

            results.push(DataSetSummary::from(created));
        }

        Ok(results)
    }

    pub async fn update_data_set(&self, request: DataSetUpdateRequest) -> Result<DataSetSummary> {
        let DataSetUpdateRequest {
            id,
            name,
            description,
            new_file,
        } = request;

        let (mut model, had_new_file) = if let Some(file) = new_file {
            let updated = self
                .data_set_service
                .update_file(id, file.filename, file.file_bytes)
                .await
                .map_err(|e| anyhow!("Failed to update data set file {}: {}", id, e))?;
            (updated, true)
        } else {
            let updated = self
                .data_set_service
                .update(id, name.clone(), description.clone())
                .await
                .map_err(|e| anyhow!("Failed to update data set {}: {}", id, e))?;
            (updated, false)
        };

        if had_new_file && (name.is_some() || description.is_some()) {
            model = self
                .data_set_service
                .update(id, name, description)
                .await
                .map_err(|e| anyhow!("Failed to update metadata for data set {}: {}", id, e))?;
        }

        Ok(DataSetSummary::from(model))
    }

    pub async fn update_data_set_graph_json(
        &self,
        id: i32,
        graph_json: String,
    ) -> Result<DataSetSummary> {
        let model = self
            .data_set_service
            .update_graph_data(id, graph_json)
            .await
            .map_err(|e| anyhow!("Failed to update graph data for data set {}: {}", id, e))?;

        Ok(DataSetSummary::from(model))
    }

    pub async fn reprocess_data_set(&self, id: i32) -> Result<DataSetSummary> {
        let model = self
            .data_set_service
            .reprocess(id)
            .await
            .map_err(|e| anyhow!("Failed to reprocess data set {}: {}", id, e))?;

        Ok(DataSetSummary::from(model))
    }

    pub async fn validate_data_set(&self, id: i32) -> Result<DataSetValidationSummary> {
        self.data_set_service
            .validate(id)
            .await
            .map_err(|e| anyhow!("Failed to validate data set {}: {}", id, e))
    }

    pub async fn validate_graph(&self, graph_id: i32) -> Result<GraphValidationSummary> {
        self.graph_service
            .validate_graph(graph_id)
            .await
            .map_err(|e| anyhow!("Failed to validate graph {}: {}", graph_id, e))
    }

    pub async fn delete_data_set(&self, id: i32) -> Result<()> {
        self.data_set_service
            .delete(id)
            .await
            .map_err(|e| anyhow!("Failed to delete data set {}: {}", id, e))
    }

    pub async fn merge_data_sets(
        &self,
        project_id: i32,
        data_set_ids: Vec<i32>,
        name: String,
        sum_weights: bool,
        delete_merged: bool,
    ) -> Result<DataSetSummary> {
        if data_set_ids.len() < 2 {
            return Err(anyhow!("At least 2 data sets are required for merging"));
        }

        // Load all datasets
        let models = data_sets::Entity::find()
            .filter(data_sets::Column::Id.is_in(data_set_ids.clone()))
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load data sets for merging: {}", e))?;

        if models.len() != data_set_ids.len() {
            return Err(anyhow!(
                "Some data sets were not found or don't belong to project {}",
                project_id
            ));
        }

        // Merge graph JSON data
        let merged_json = self.merge_graph_json_data(&models, sum_weights)?;

        // Create new dataset with merged data
        let summary = self
            .create_empty_data_set(DataSetEmptyCreateRequest {
                project_id,
                name,
                description: Some(format!("Merged from {} data sets", data_set_ids.len())),
            })
            .await?;

        // Update the new dataset with merged graph data
        let summary = self
            .data_set_service
            .update_graph_data(summary.id, merged_json)
            .await
            .map_err(|e| anyhow!("Failed to update merged data set: {}", e))?;

        // Delete source datasets if requested
        if delete_merged {
            for id in &data_set_ids {
                self.data_set_service.delete(*id).await.ok();
            }
        }

        Ok(DataSetSummary::from(summary))
    }

    fn merge_graph_json_data(
        &self,
        models: &[data_sets::Model],
        sum_weights: bool,
    ) -> Result<String> {
        #[derive(Deserialize, Serialize, Default)]
        struct GraphData {
            #[serde(default)]
            nodes: Vec<Value>,
            #[serde(default)]
            edges: Vec<Value>,
            #[serde(default)]
            layers: Vec<Value>,
        }

        let mut merged = GraphData::default();
        let mut node_map: HashMap<String, Value> = HashMap::new();
        let mut edge_map: HashMap<String, Value> = HashMap::new();
        let mut layer_map: HashMap<String, Value> = HashMap::new();

        for model in models {
            let graph: GraphData = serde_json::from_str(&model.graph_json).unwrap_or_default();

            // Merge nodes
            for node in graph.nodes {
                let id = node
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if id.is_empty() {
                    merged.nodes.push(node);
                    continue;
                }
                if let Some(existing) = node_map.get_mut(&id) {
                    if sum_weights {
                        if let (Some(existing_weight), Some(new_weight)) = (
                            existing.get("weight").and_then(|v| v.as_f64()),
                            node.get("weight").and_then(|v| v.as_f64()),
                        ) {
                            if let Some(obj) = existing.as_object_mut() {
                                obj.insert(
                                    "weight".to_string(),
                                    json!(existing_weight + new_weight),
                                );
                            }
                        }
                    }
                } else {
                    node_map.insert(id, node);
                }
            }

            // Merge edges
            for edge in graph.edges {
                let source = edge
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let target = edge
                    .get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let key = format!("{}:{}", source, target);
                if source.is_empty() || target.is_empty() {
                    merged.edges.push(edge);
                    continue;
                }
                if let Some(existing) = edge_map.get_mut(&key) {
                    if sum_weights {
                        if let (Some(existing_weight), Some(new_weight)) = (
                            existing.get("weight").and_then(|v| v.as_f64()),
                            edge.get("weight").and_then(|v| v.as_f64()),
                        ) {
                            if let Some(obj) = existing.as_object_mut() {
                                obj.insert(
                                    "weight".to_string(),
                                    json!(existing_weight + new_weight),
                                );
                            }
                        }
                    }
                } else {
                    edge_map.insert(key, edge);
                }
            }

            // Merge layers
            for layer in graph.layers {
                let id = layer
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if id.is_empty() {
                    merged.layers.push(layer);
                    continue;
                }
                layer_map.entry(id).or_insert(layer);
            }
        }

        merged.nodes.extend(node_map.into_values());
        merged.edges.extend(edge_map.into_values());
        merged.layers.extend(layer_map.into_values());

        serde_json::to_string(&merged)
            .map_err(|e| anyhow!("Failed to serialize merged data: {}", e))
    }

    pub async fn export_data_sets(
        &self,
        request: DataSetExportRequest,
    ) -> Result<DataSetExportResult> {
        let DataSetExportRequest {
            project_id,
            data_set_ids,
            format,
        } = request;

        let matching_count = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .filter(data_sets::Column::Id.is_in(data_set_ids.clone()))
            .count(&self.db)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to verify data sets for project {}: {}",
                    project_id,
                    e
                )
            })?;

        if matching_count != data_set_ids.len() as u64 {
            return Err(anyhow!(
                "Export request included data sets outside project {}",
                project_id
            ));
        }

        let bytes = match format {
            DataSetExportFormat::Xlsx => self
                .data_set_bulk_service
                .export_to_xlsx(&data_set_ids)
                .await
                .map_err(|e| anyhow!("Failed to export datasets to XLSX: {}", e))?,
            DataSetExportFormat::Ods => self
                .data_set_bulk_service
                .export_to_ods(&data_set_ids)
                .await
                .map_err(|e| anyhow!("Failed to export datasets to ODS: {}", e))?,
        };

        let filename = format!(
            "datasets_export_{}.{}",
            chrono::Utc::now().timestamp(),
            format.extension()
        );

        Ok(DataSetExportResult {
            data: bytes,
            filename,
            format,
        })
    }

    pub async fn import_data_sets(
        &self,
        request: DataSetImportRequest,
    ) -> Result<DataSetImportOutcome> {
        let result = match request.format {
            DataSetImportFormat::Xlsx => self
                .data_set_bulk_service
                .import_from_xlsx(request.project_id, &request.file_bytes)
                .await
                .map_err(|e| anyhow!("Failed to import datasets from XLSX: {}", e))?,
            DataSetImportFormat::Ods => self
                .data_set_bulk_service
                .import_from_ods(request.project_id, &request.file_bytes)
                .await
                .map_err(|e| anyhow!("Failed to import datasets from ODS: {}", e))?,
        };

        if result.imported_ids.is_empty() {
            return Ok(DataSetImportOutcome {
                data_sets: Vec::new(),
                created_count: result.created_count,
                updated_count: result.updated_count,
            });
        }

        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let models = data_sets::Entity::find()
            .filter(data_sets::Column::Id.is_in(result.imported_ids.clone()))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load imported datasets: {}", e))?;

        Ok(DataSetImportOutcome {
            data_sets: models.into_iter().map(DataSetSummary::from).collect(),
            created_count: result.created_count,
            updated_count: result.updated_count,
        })
    }

    pub async fn export_project_as_template(
        &self,
        project_id: i32,
    ) -> Result<library_items::Model> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", project_id, e))?
            .ok_or_else(|| anyhow!("Project {} not found", project_id))?;

        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?
            .ok_or_else(|| anyhow!("Project {} has no plan to export", project_id))?;

        let snapshot = self
            .load_plan_dag(project_id, None)
            .await?
            .ok_or_else(|| anyhow!("Project {} has no DAG to export", project_id))?;

        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load data sets for template: {}", e))?;

        let (dataset_records, dataset_graphs) = analyze_data_sets(&data_sets)?;
        let dataset_index = DatasetBundleIndex {
            datasets: dataset_records.clone(),
        };

        let manifest = ProjectBundleManifest {
            manifest_version: "1.0".to_string(),
            bundle_type: ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
            created_with: format!("layercake-{}", env!("CARGO_PKG_VERSION")),
            project_format_version: 1,
            generated_at: chrono::Utc::now(),
            source_project_id: project.id,
            plan_name: plan.name.clone(),
        };

        let project_record = ProjectRecord {
            name: project.name.clone(),
            description: project.description.clone(),
            tags: serde_json::from_str(&project.tags).unwrap_or_default(),
        };

        let dag_bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|e| anyhow!("Failed to encode DAG snapshot: {}", e))?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| anyhow!("Failed to encode template manifest: {}", e))?;
        let project_bytes = serde_json::to_vec_pretty(&project_record)
            .map_err(|e| anyhow!("Failed to encode project metadata: {}", e))?;
        let dataset_index_bytes = serde_json::to_vec_pretty(&dataset_index)
            .map_err(|e| anyhow!("Failed to encode dataset index: {}", e))?;
        let metadata_bytes = serde_json::to_vec_pretty(&json!({
            "layercakeProjectFormatVersion": 1
        }))
        .map_err(|e| anyhow!("Failed to encode metadata.json: {}", e))?;

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            write_bundle_common_files(
                &mut zip,
                &manifest_bytes,
                &metadata_bytes,
                &project_bytes,
                &dag_bytes,
                &dataset_index_bytes,
            )?;
            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

            for descriptor in &dataset_records {
                if let Some(graph_json) = dataset_graphs.get(&descriptor.original_id) {
                    let path = format!("datasets/{}", descriptor.filename);
                    zip.start_file(path, options)
                        .map_err(|e| anyhow!("Failed to add dataset file: {}", e))?;
                    zip.write_all(graph_json.as_bytes())
                        .map_err(|e| anyhow!("Failed to write dataset file: {}", e))?;
                }
            }

            zip.finish()
                .map_err(|e| anyhow!("Failed to finalize template archive: {}", e))?;
        }

        let zip_bytes = cursor.into_inner();
        let service = LibraryItemService::new(self.db.clone());
        let tags = serde_json::from_str(&project.tags).unwrap_or_default();

        let metadata = json!({
            "projectId": project.id,
            "planId": plan.id,
            "nodeCount": snapshot.nodes.len(),
            "edgeCount": snapshot.edges.len(),
            "datasetCount": dataset_records.len(),
            "manifestVersion": manifest.manifest_version,
            "projectFormatVersion": manifest.project_format_version
        });

        let item = service
            .create_binary_item(
                ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
                format!("{} Template", project.name),
                project.description.clone(),
                tags,
                metadata,
                Some("application/zip".to_string()),
                zip_bytes,
            )
            .await
            .map_err(|e| anyhow!("Failed to persist template: {}", e))?;

        Ok(item)
    }

    pub async fn export_project_archive(
        &self,
        project_id: i32,
        include_knowledge_base: bool,
    ) -> Result<ProjectArchiveFile> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", project_id, e))?
            .ok_or_else(|| anyhow!("Project {} not found", project_id))?;

        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?
            .ok_or_else(|| anyhow!("Project {} has no plan to export", project_id))?;

        let snapshot = self
            .load_plan_dag(project_id, None)
            .await?
            .ok_or_else(|| anyhow!("Project {} has no DAG to export", project_id))?;

        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load data sets for export: {}", e))?;

        let (dataset_records, dataset_graphs) = analyze_data_sets(&data_sets)?;
        let dataset_index = DatasetBundleIndex {
            datasets: dataset_records.clone(),
        };

        let manifest = ProjectBundleManifest {
            manifest_version: "1.0".to_string(),
            bundle_type: "project_archive".to_string(),
            created_with: format!("layercake-{}", env!("CARGO_PKG_VERSION")),
            project_format_version: 1,
            generated_at: chrono::Utc::now(),
            source_project_id: project.id,
            plan_name: plan.name.clone(),
        };

        let project_record = ProjectRecord {
            name: project.name.clone(),
            description: project.description.clone(),
            tags: serde_json::from_str(&project.tags).unwrap_or_default(),
        };

        let dag_bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|e| anyhow!("Failed to encode DAG snapshot: {}", e))?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| anyhow!("Failed to encode project export manifest: {}", e))?;
        let project_bytes = serde_json::to_vec_pretty(&project_record)
            .map_err(|e| anyhow!("Failed to encode project metadata: {}", e))?;
        let dataset_index_bytes = serde_json::to_vec_pretty(&dataset_index)
            .map_err(|e| anyhow!("Failed to encode dataset index: {}", e))?;
        let metadata_bytes = serde_json::to_vec_pretty(&json!({
            "layercakeProjectFormatVersion": 1
        }))
        .map_err(|e| anyhow!("Failed to encode metadata.json: {}", e))?;

        let mut additional_assets = self.collect_plan_assets(project_id).await?;
        if let Some(asset) = self.collect_stories_asset(project_id).await? {
            additional_assets.push(asset);
        }
        if let Some(asset) = self.collect_palette_asset(project_id).await? {
            additional_assets.push(asset);
        }
        if include_knowledge_base {
            if let Some((index_asset, mut file_assets)) =
                self.collect_knowledge_base_assets(project_id).await?
            {
                additional_assets.push(index_asset);
                additional_assets.append(&mut file_assets);
            }
        }

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            write_bundle_common_files(
                &mut zip,
                &manifest_bytes,
                &metadata_bytes,
                &project_bytes,
                &dag_bytes,
                &dataset_index_bytes,
            )?;

            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            for descriptor in &dataset_records {
                if let Some(graph_json) = dataset_graphs.get(&descriptor.original_id) {
                    let path = format!("datasets/{}", descriptor.filename);
                    zip.start_file(path, options)
                        .map_err(|e| anyhow!("Failed to add dataset file: {}", e))?;
                    zip.write_all(graph_json.as_bytes())
                        .map_err(|e| anyhow!("Failed to write dataset file: {}", e))?;
                }
            }

            for asset in additional_assets {
                zip.start_file(asset.path.clone(), options)
                    .map_err(|e| anyhow!("Failed to add {}: {}", asset.path, e))?;
                zip.write_all(&asset.bytes)
                    .map_err(|e| anyhow!("Failed to write {}: {}", asset.path, e))?;
            }

            zip.finish()
                .map_err(|e| anyhow!("Failed to finalize project archive: {}", e))?;
        }

        let filename = format!(
            "{}-project-export.zip",
            sanitize_dataset_filename(&project.name)
        );

        Ok(ProjectArchiveFile {
            filename,
            bytes: cursor.into_inner(),
        })
    }

    pub async fn import_project_archive(
        &self,
        archive_bytes: Vec<u8>,
        project_name: Option<String>,
    ) -> Result<ProjectSummary> {
        let mut archive = ZipArchive::new(Cursor::new(archive_bytes))
            .map_err(|e| anyhow!("Failed to read project archive: {}", e))?;

        let manifest: ProjectBundleManifest = read_template_json(&mut archive, "manifest.json")?;
        let project_record: ProjectRecord = read_template_json(&mut archive, "project.json")?;
        let dag_snapshot: PlanDagSnapshot = read_template_json(&mut archive, "dag.json")?;
        let dataset_index: DatasetBundleIndex =
            read_template_json(&mut archive, "datasets/index.json")?;
        let plans_index: Option<PlansIndex> =
            try_read_template_json(&mut archive, "plans/index.json")?;
        let stories_export: Option<StoriesExport> =
            try_read_template_json(&mut archive, "stories/stories.json")?;
        let palette_export: Option<PaletteExport> =
            try_read_template_json(&mut archive, "layers/palette.json")?;
        let knowledge_base_index: Option<KnowledgeBaseIndex> =
            try_read_template_json(&mut archive, "kb/index.json")?;
        let mut kb_file_blobs: HashMap<String, Vec<u8>> = HashMap::new();
        if let Some(index) = &knowledge_base_index {
            for file_entry in &index.files {
                let bytes = read_zip_file_bytes(&mut archive, &file_entry.blob_path)?;
                kb_file_blobs.insert(file_entry.id.clone(), bytes);
            }
        }

        let tags = project_record.tags.clone();
        let desired_name = project_name.unwrap_or(project_record.name.clone());
        let now = Utc::now();

        let project = projects::ActiveModel {
            name: Set(desired_name.clone()),
            description: Set(project_record.description.clone()),
            tags: Set(serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| anyhow!("Failed to create project from archive: {}", e))?;

        let dataset_service = DataSetService::new(self.db.clone());
        let mut id_map = HashMap::new();

        for descriptor in &dataset_index.datasets {
            let dataset_path = format!("datasets/{}", descriptor.filename);
            let file_bytes = {
                let mut dataset_file = archive
                    .by_name(&dataset_path)
                    .map_err(|e| anyhow!("Missing dataset file {}: {}", descriptor.filename, e))?;
                let mut bytes = Vec::new();
                dataset_file.read_to_end(&mut bytes).map_err(|e| {
                    anyhow!("Failed to read dataset {}: {}", descriptor.filename, e)
                })?;
                bytes
            };
            let file_format = DataSetFileFormat::from_str(&descriptor.file_format)
                .unwrap_or(DataSetFileFormat::Json);

            let dataset = dataset_service
                .create_from_file(
                    project.id,
                    descriptor.name.clone(),
                    descriptor.description.clone(),
                    descriptor.filename.clone(),
                    file_format,
                    file_bytes,
                    None,
                )
                .await
                .map_err(|e| anyhow!("Failed to import dataset {}: {}", descriptor.name, e))?;

            id_map.insert(descriptor.original_id, dataset.id);
        }

        if let Some(palette) = palette_export {
            self.import_palette_from_export(project.id, palette, &id_map)
                .await?;
        }

        if let Some(index) = plans_index {
            self.import_plans_from_export(project.id, index, &mut archive, &id_map)
                .await?;
        } else {
            let plan = self
                .create_plan(PlanCreateRequest {
                    project_id: project.id,
                    name: if manifest.plan_name.trim().is_empty() {
                        format!("{} Plan", desired_name)
                    } else {
                        manifest.plan_name.clone()
                    },
                    description: None,
                    tags: Some(vec![]),
                    yaml_content: "".to_string(),
                    dependencies: None,
                    status: Some("draft".to_string()),
                })
                .await?;

            insert_plan_dag_from_snapshot(&self.db, plan.id, &dag_snapshot, &id_map)
                .await
                .map_err(|e| anyhow!("Failed to recreate plan DAG: {}", e))?;
        }

        if let Some(story_bundle) = stories_export {
            self.import_stories_from_export(project.id, story_bundle, &id_map)
                .await?;
        }

        if let Some(kb_index) = knowledge_base_index {
            self.import_knowledge_base_from_export(project.id, kb_index, kb_file_blobs)
                .await?;
        }

        Ok(ProjectSummary::from(project))
    }

    pub async fn create_project_from_library(
        &self,
        library_item_id: i32,
        project_name: Option<String>,
    ) -> Result<ProjectSummary> {
        let service = LibraryItemService::new(self.db.clone());
        let item = service
            .get(library_item_id)
            .await
            .map_err(|e| anyhow!("Failed to load library item {}: {}", library_item_id, e))?
            .ok_or_else(|| anyhow!("Library item {} not found", library_item_id))?;

        if item.item_type != ITEM_TYPE_PROJECT && item.item_type != ITEM_TYPE_PROJECT_TEMPLATE {
            return Err(anyhow!(
                "Library item {} is type {}, expected project or project_template",
                library_item_id,
                item.item_type
            ));
        }

        let mut archive = ZipArchive::new(Cursor::new(item.content_blob.clone())).map_err(|e| {
            anyhow!(
                "Failed to read template archive for library item {}: {}",
                library_item_id,
                e
            )
        })?;

        let manifest: ProjectBundleManifest = read_template_json(&mut archive, "manifest.json")?;
        let project_record: ProjectRecord = read_template_json(&mut archive, "project.json")?;
        let dag_snapshot: PlanDagSnapshot = read_template_json(&mut archive, "dag.json")?;
        let dataset_index: DatasetBundleIndex =
            read_template_json(&mut archive, "datasets/index.json")?;

        let tags = project_record.tags.clone();
        let desired_name = project_name.unwrap_or(project_record.name.clone());
        let now = Utc::now();

        let project = projects::ActiveModel {
            name: Set(desired_name.clone()),
            description: Set(project_record.description.clone()),
            tags: Set(serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| anyhow!("Failed to create project from template: {}", e))?;

        let plan = self
            .create_plan(PlanCreateRequest {
                project_id: project.id,
                name: if manifest.plan_name.trim().is_empty() {
                    format!("{} Plan", desired_name)
                } else {
                    manifest.plan_name.clone()
                },
                description: None,
                tags: Some(vec![]),
                yaml_content: "".to_string(),
                dependencies: None,
                status: Some("draft".to_string()),
            })
            .await?;

        let dataset_service = DataSetService::new(self.db.clone());
        let mut id_map = HashMap::new();

        let is_template = item.item_type == ITEM_TYPE_PROJECT_TEMPLATE;

        for descriptor in &dataset_index.datasets {
            let dataset = if is_template {
                // Templates should not carry data rows forward; create empty datasets using the schema metadata.
                dataset_service
                    .create_empty(
                        project.id,
                        descriptor.name.clone(),
                        descriptor.description.clone(),
                    )
                    .await
            } else {
                let dataset_path = format!("datasets/{}", descriptor.filename);
                let file_bytes = {
                    let mut dataset_file = archive.by_name(&dataset_path).map_err(|e| {
                        anyhow!("Missing dataset file {}: {}", descriptor.filename, e)
                    })?;
                    let mut bytes = Vec::new();
                    dataset_file.read_to_end(&mut bytes).map_err(|e| {
                        anyhow!("Failed to read dataset {}: {}", descriptor.filename, e)
                    })?;
                    bytes
                };
                let file_format = DataSetFileFormat::from_str(&descriptor.file_format)
                    .unwrap_or(DataSetFileFormat::Json);

                dataset_service
                    .create_from_file(
                        project.id,
                        descriptor.name.clone(),
                        descriptor.description.clone(),
                        descriptor.filename.clone(),
                        file_format,
                        file_bytes,
                        None,
                    )
                    .await
            }
            .map_err(|e| anyhow!("Failed to import dataset {}: {}", descriptor.name, e))?;

            id_map.insert(descriptor.original_id, dataset.id);
        }

        insert_plan_dag_from_snapshot(&self.db, plan.id, &dag_snapshot, &id_map)
            .await
            .map_err(|e| anyhow!("Failed to recreate plan DAG: {}", e))?;

        Ok(ProjectSummary::from(project))
    }

    // ----- Plan DAG helpers -------------------------------------------------
    pub async fn load_plan_dag(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> Result<Option<PlanDagSnapshot>> {
        let project = match projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", project_id, e))?
        {
            Some(project) => project,
            None => return Ok(None),
        };

        let plan = match plan_id {
            Some(plan_id) => Some(self.resolve_plan_model(project_id, Some(plan_id)).await?),
            None => self
                .plan_service
                .get_default_plan(project_id)
                .await
                .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?,
        };

        if let Some(plan) = plan {
            let mut nodes = self
                .plan_dag_service
                .get_nodes(project_id, Some(plan.id))
                .await
                .map_err(|e| anyhow!("Failed to load Plan DAG nodes: {}", e))?;
            let edges = self
                .plan_dag_service
                .get_edges(project_id, Some(plan.id))
                .await
                .map_err(|e| anyhow!("Failed to load Plan DAG edges: {}", e))?;

            for idx in 0..nodes.len() {
                let node_type = nodes[idx].node_type;
                let node_id = nodes[idx].id.clone();

                match node_type {
                    PlanDagNodeType::DataSet => {
                        if let Ok(config) =
                            serde_json::from_str::<serde_json::Value>(&nodes[idx].config)
                        {
                            if let Some(data_set_id) = config
                                .get("dataSetId")
                                .and_then(|v| v.as_i64())
                                .map(|v| v as i32)
                            {
                                if let Some(data_set) = data_sets::Entity::find_by_id(data_set_id)
                                    .one(&self.db)
                                    .await
                                    .map_err(|e| {
                                        anyhow!("Failed to load data set {}: {}", data_set_id, e)
                                    })?
                                {
                                    let execution_state = match data_set.status.as_str() {
                                        "active" => "completed",
                                        "processing" => "processing",
                                        "error" => "error",
                                        _ => "not_started",
                                    }
                                    .to_string();

                                    nodes[idx].dataset_execution = Some(DataSetExecutionMetadata {
                                        data_set_id: data_set.id,
                                        filename: data_set.filename.clone(),
                                        status: data_set.status.clone(),
                                        processed_at: data_set.processed_at.map(|d| d.to_rfc3339()),
                                        execution_state,
                                        error_message: data_set.error_message.clone(),
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
                                annotations: graph.annotations.clone(),
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
        plan_id: Option<i32>,
        request: PlanDagNodeRequest,
    ) -> Result<PlanDagNode> {
        let existing_nodes = self
            .plan_dag_service
            .get_nodes(project_id, plan_id)
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
                plan_id,
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
        plan_id: Option<i32>,
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
                plan_id,
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
        plan_id: Option<i32>,
        node_id: String,
    ) -> Result<PlanDagNode> {
        self.plan_dag_service
            .delete_node(project_id, plan_id, node_id)
            .await
    }

    pub async fn move_plan_dag_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        position: Position,
    ) -> Result<PlanDagNode> {
        self.plan_dag_service
            .move_node(project_id, plan_id, node_id, position)
            .await
    }

    pub async fn batch_move_plan_dag_nodes(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
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
            .batch_move_nodes(project_id, plan_id, updates)
            .await
    }

    pub async fn create_plan_dag_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        request: PlanDagEdgeRequest,
    ) -> Result<PlanDagEdge> {
        let edge_id = generate_edge_id(&request.source, &request.target);
        let metadata_json = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Invalid edge metadata: {}", e))?;

        self.plan_dag_service
            .create_edge(
                project_id,
                plan_id,
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
        plan_id: Option<i32>,
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
            .update_edge(project_id, plan_id, edge_id, metadata_json)
            .await
    }

    pub async fn delete_plan_dag_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
    ) -> Result<PlanDagEdge> {
        self.plan_dag_service
            .delete_edge(project_id, plan_id, edge_id)
            .await
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
        alias: Option<String>,
        properties: Option<Value>,
    ) -> Result<LayerDto> {
        use crate::database::entities::graph_layers::Entity as Layers;

        let old_layer = Layers::find_by_id(layer_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load layer {}: {}", layer_id, e))?;

        let updated_layer = self
            .graph_service
            .update_layer_properties(layer_id, name.clone(), alias.clone(), properties.clone())
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

            if old_layer.alias != updated_layer.alias {
                let _ = self
                    .graph_edit_service
                    .create_edit(
                        old_layer.graph_id,
                        "layer".to_string(),
                        old_layer.layer_id.clone(),
                        "update".to_string(),
                        Some("alias".to_string()),
                        old_layer.alias.as_ref().map(|value| json!(value)),
                        updated_layer.alias.as_ref().map(|value| json!(value)),
                        None,
                        true,
                    )
                    .await;
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
                layer_update.alias,
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
        render_config: Option<RenderConfig>,
        max_rows: Option<usize>,
    ) -> Result<String> {
        let graph = self
            .graph_service
            .build_graph_from_dag_graph(graph_id)
            .await
            .map_err(|e| anyhow!("Failed to load graph {}: {}", graph_id, e))?;

        let content = self
            .export_service
            .export_to_string(&graph, &format, render_config)
            .map_err(|e| anyhow!("Failed to render graph export: {}", e))?;

        Ok(apply_preview_limit(content, format, max_rows))
    }

    pub async fn preview_sequence_export(
        &self,
        project_id: i32,
        story_id: i32,
        render_target: SequenceArtefactRenderTarget,
        render_config: SequenceRenderConfigResolved,
    ) -> Result<String> {
        let base_context = build_story_context(&self.db, project_id, story_id)
            .await
            .map_err(|e| anyhow!("Failed to build story context: {}", e))?;

        let context = apply_render_config(&base_context, render_config);

        let rendered = match render_target {
            SequenceArtefactRenderTarget::MermaidSequence => to_mermaid_sequence::render(&context)
                .map_err(|e| anyhow!("Failed to render Mermaid sequence: {}", e))?,
            SequenceArtefactRenderTarget::PlantUmlSequence => {
                to_plantuml_sequence::render(&context)
                    .map_err(|e| anyhow!("Failed to render PlantUML sequence: {}", e))?
            }
        };

        Ok(rendered)
    }

    async fn collect_plan_assets(&self, project_id: i32) -> Result<Vec<ExportAsset>> {
        let plans = self.plan_service.list_plans(project_id).await?;
        if plans.is_empty() {
            return Ok(Vec::new());
        }

        let mut assets = Vec::new();
        let mut index = PlansIndex::default();

        for plan in plans {
            if let Some(snapshot) = self.load_plan_dag(project_id, Some(plan.id)).await? {
                let filename = format!("plan_{}.json", plan.id);
                let metadata = ExportedPlanMetadata {
                    original_id: plan.id,
                    name: plan.name.clone(),
                    description: plan.description.clone(),
                    tags: serde_json::from_str(&plan.tags).unwrap_or_default(),
                    yaml_content: plan.yaml_content.clone(),
                    dependencies: plan
                        .dependencies
                        .as_ref()
                        .and_then(|value| serde_json::from_str::<Vec<i32>>(value).ok()),
                    status: plan.status.clone(),
                    version: plan.version,
                    created_at: plan.created_at,
                    updated_at: plan.updated_at,
                };
                let payload = ExportedPlanFile {
                    metadata,
                    dag: snapshot,
                };
                let bytes = serde_json::to_vec_pretty(&payload)
                    .map_err(|e| anyhow!("Failed to encode plan {}: {}", plan.id, e))?;
                assets.push(ExportAsset {
                    path: format!("plans/{}", filename),
                    bytes,
                });
                index.plans.push(PlanIndexEntry {
                    original_id: plan.id,
                    filename,
                    name: plan.name,
                });
            }
        }

        if !index.plans.is_empty() {
            let bytes = serde_json::to_vec_pretty(&index)
                .map_err(|e| anyhow!("Failed to encode plan index: {}", e))?;
            assets.push(ExportAsset {
                path: "plans/index.json".into(),
                bytes,
            });
        }

        Ok(assets)
    }

    async fn collect_stories_asset(&self, project_id: i32) -> Result<Option<ExportAsset>> {
        let stories = stories::Entity::find()
            .filter(stories::Column::ProjectId.eq(project_id))
            .order_by_asc(stories::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load stories for export: {}", e))?;

        if stories.is_empty() {
            return Ok(None);
        }

        let story_ids: Vec<i32> = stories.iter().map(|s| s.id).collect();
        let sequence_rows = if story_ids.is_empty() {
            Vec::new()
        } else {
            sequences::Entity::find()
                .filter(sequences::Column::StoryId.is_in(story_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to load sequences for export: {}", e))?
        };

        let mut sequence_map: HashMap<i32, Vec<sequences::Model>> = HashMap::new();
        for sequence in sequence_rows {
            sequence_map
                .entry(sequence.story_id)
                .or_default()
                .push(sequence);
        }

        let mut payload = StoriesExport::default();
        for story in stories {
            let tags = serde_json::from_str(&story.tags).unwrap_or_default();
            let enabled_dataset_ids =
                serde_json::from_str(&story.enabled_dataset_ids).unwrap_or_default();
            let layer_config = serde_json::from_str(&story.layer_config).unwrap_or(Value::Null);

            let sequences = sequence_map
                .remove(&story.id)
                .unwrap_or_default()
                .into_iter()
                .map(|sequence| {
                    let enabled_sequence_dataset_ids =
                        serde_json::from_str(&sequence.enabled_dataset_ids).unwrap_or_default();
                    let edge_order = serde_json::from_str(&sequence.edge_order).unwrap_or_default();

                    ExportedSequence {
                        original_id: sequence.id,
                        name: sequence.name,
                        description: sequence.description,
                        enabled_dataset_ids: enabled_sequence_dataset_ids,
                        edge_order,
                        created_at: sequence.created_at,
                        updated_at: sequence.updated_at,
                    }
                })
                .collect();

            payload.stories.push(ExportedStory {
                original_id: story.id,
                name: story.name,
                description: story.description,
                tags,
                enabled_dataset_ids,
                layer_config,
                sequences,
                created_at: story.created_at,
                updated_at: story.updated_at,
            });
        }

        let bytes = serde_json::to_vec_pretty(&payload)
            .map_err(|e| anyhow!("Failed to encode stories export: {}", e))?;
        Ok(Some(ExportAsset {
            path: "stories/stories.json".into(),
            bytes,
        }))
    }

    async fn collect_palette_asset(&self, project_id: i32) -> Result<Option<ExportAsset>> {
        let layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .order_by_asc(project_layers::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project layers: {}", e))?;

        if layers.is_empty() {
            return Ok(None);
        }

        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load layer aliases: {}", e))?;

        let export = PaletteExport {
            layers: layers
                .into_iter()
                .map(|layer| ExportedLayer {
                    original_id: layer.id,
                    layer_id: layer.layer_id,
                    name: layer.name,
                    background_color: layer.background_color,
                    text_color: layer.text_color,
                    border_color: layer.border_color,
                    alias: layer.alias,
                    source_dataset_id: layer.source_dataset_id,
                    enabled: layer.enabled,
                    created_at: layer.created_at,
                    updated_at: layer.updated_at,
                })
                .collect(),
            aliases: aliases
                .into_iter()
                .map(|alias| ExportedLayerAlias {
                    alias_layer_id: alias.alias_layer_id,
                    target_layer_original_id: alias.target_layer_id,
                })
                .collect(),
        };

        let bytes = serde_json::to_vec_pretty(&export)
            .map_err(|e| anyhow!("Failed to encode layer palette export: {}", e))?;
        Ok(Some(ExportAsset {
            path: "layers/palette.json".into(),
            bytes,
        }))
    }

    async fn collect_knowledge_base_assets(
        &self,
        project_id: i32,
    ) -> Result<Option<(ExportAsset, Vec<ExportAsset>)>> {
        let files = kb_files::Entity::find()
            .filter(kb_files::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load knowledge base files: {}", e))?;

        if files.is_empty() {
            return Ok(None);
        }

        let file_ids: Vec<Uuid> = files.iter().map(|file| file.id).collect();
        let file_tags = if file_ids.is_empty() {
            Vec::new()
        } else {
            kb_file_tags::Entity::find()
                .filter(kb_file_tags::Column::FileId.is_in(file_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to load file tags: {}", e))?
        };

        let tag_ids: Vec<Uuid> = file_tags.iter().map(|row| row.tag_id).collect();
        let tags = if tag_ids.is_empty() {
            Vec::new()
        } else {
            kb_tags::Entity::find()
                .filter(kb_tags::Column::Id.is_in(tag_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to load tags: {}", e))?
        };

        let documents = kb_docs::Entity::find()
            .filter(kb_docs::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load knowledge base documents: {}", e))?;

        let vector_states = kb_vector_state::Entity::find()
            .filter(kb_vector_state::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load knowledge base vector state: {}", e))?;

        let mut tag_lookup = HashMap::new();
        for tag in tags {
            tag_lookup.insert(
                tag.id,
                KnowledgeBaseTagEntry {
                    id: tag.id.to_string(),
                    name: tag.name,
                    scope: tag.scope,
                    color: tag.color,
                    created_at: tag.created_at,
                },
            );
        }

        let mut file_tag_map: HashMap<Uuid, Vec<String>> = HashMap::new();
        let mut file_tag_entries = Vec::new();
        for link in file_tags {
            file_tag_map
                .entry(link.file_id)
                .or_default()
                .push(link.tag_id.to_string());
            file_tag_entries.push(KnowledgeBaseFileTagEntry {
                file_id: link.file_id.to_string(),
                tag_id: link.tag_id.to_string(),
            });
        }

        let mut index = KnowledgeBaseIndex::default();
        index.tags = tag_lookup.values().cloned().collect();
        index.file_tags = file_tag_entries;

        let mut binary_assets = Vec::new();
        for file in files {
            let sanitized_name = sanitize_dataset_filename(&file.filename);
            let blob_path = format!("kb/files/{}/{}", file.id, sanitized_name);
            binary_assets.push(ExportAsset {
                path: blob_path.clone(),
                bytes: file.blob.clone(),
            });

            index.files.push(KnowledgeBaseFileEntry {
                id: file.id.to_string(),
                filename: file.filename,
                media_type: file.media_type,
                size_bytes: file.size_bytes,
                checksum: file.checksum,
                created_at: file.created_at,
                indexed: file.indexed,
                blob_path,
                tag_ids: file_tag_map.remove(&file.id).unwrap_or_default(),
            });
        }

        index.documents = documents
            .into_iter()
            .map(|doc| KnowledgeBaseDocumentEntry {
                id: doc.id.to_string(),
                file_id: doc.file_id.map(|id| id.to_string()),
                chunk_id: doc.chunk_id,
                media_type: doc.media_type,
                chunk_text: doc.chunk_text,
                metadata: doc.metadata,
                embedding_model: doc.embedding_model,
                embedding: doc
                    .embedding
                    .map(|bytes| general_purpose::STANDARD.encode(bytes)),
                created_at: doc.created_at,
            })
            .collect();

        index.vector_states = vector_states
            .into_iter()
            .map(|state| KnowledgeBaseVectorStateEntry {
                id: state.id.to_string(),
                status: state.status,
                last_indexed_at: state.last_indexed_at,
                last_error: state.last_error,
                config: state.config,
                embedding_provider: state.embedding_provider,
                embedding_model: state.embedding_model,
                updated_at: state.updated_at,
            })
            .collect();

        let index_bytes = serde_json::to_vec_pretty(&index)
            .map_err(|e| anyhow!("Failed to encode knowledge base index: {}", e))?;
        let index_asset = ExportAsset {
            path: "kb/index.json".into(),
            bytes: index_bytes,
        };

        Ok(Some((index_asset, binary_assets)))
    }

    async fn import_plans_from_export(
        &self,
        project_id: i32,
        plans_index: PlansIndex,
        archive: &mut ZipArchive<Cursor<Vec<u8>>>,
        dataset_map: &HashMap<i32, i32>,
    ) -> Result<()> {
        if plans_index.plans.is_empty() {
            return Ok(());
        }

        for entry in plans_index.plans {
            let path = format!("plans/{}", entry.filename);
            let plan_file: ExportedPlanFile = read_template_json(archive, &path)?;
            let plan = self
                .create_plan(PlanCreateRequest {
                    project_id,
                    name: plan_file.metadata.name.clone(),
                    description: plan_file.metadata.description.clone(),
                    tags: Some(plan_file.metadata.tags.clone()),
                    yaml_content: plan_file.metadata.yaml_content.clone(),
                    dependencies: plan_file.metadata.dependencies.clone(),
                    status: Some(plan_file.metadata.status.clone()),
                })
                .await?;

            insert_plan_dag_from_snapshot(&self.db, plan.id, &plan_file.dag, dataset_map)
                .await
                .map_err(|e| anyhow!("Failed to recreate plan {} DAG: {}", plan.id, e))?;
        }

        Ok(())
    }

    async fn import_stories_from_export(
        &self,
        project_id: i32,
        export: StoriesExport,
        dataset_map: &HashMap<i32, i32>,
    ) -> Result<()> {
        if export.stories.is_empty() {
            return Ok(());
        }

        for story in export.stories {
            let mapped_dataset_ids: Vec<i32> = story
                .enabled_dataset_ids
                .into_iter()
                .filter_map(|id| dataset_map.get(&id).copied())
                .collect();

            let story_model = stories::ActiveModel {
                id: NotSet,
                project_id: Set(project_id),
                name: Set(story.name),
                description: Set(story.description),
                tags: Set(serde_json::to_string(&story.tags).unwrap_or_else(|_| "[]".into())),
                enabled_dataset_ids: Set(
                    serde_json::to_string(&mapped_dataset_ids).unwrap_or_else(|_| "[]".into())
                ),
                layer_config: Set(
                    serde_json::to_string(&story.layer_config).unwrap_or_else(|_| "{}".into())
                ),
                created_at: Set(story.created_at),
                updated_at: Set(story.updated_at),
            }
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to insert story: {}", e))?;

            for sequence in story.sequences {
                let mapped_sequence_ids: Vec<i32> = sequence
                    .enabled_dataset_ids
                    .into_iter()
                    .filter_map(|id| dataset_map.get(&id).copied())
                    .collect();
                let mapped_edge_order: Vec<_> = sequence
                    .edge_order
                    .into_iter()
                    .filter_map(|mut ref_entry| {
                        dataset_map
                            .get(&ref_entry.dataset_id)
                            .copied()
                            .map(|new_id| {
                                ref_entry.dataset_id = new_id;
                                ref_entry
                            })
                    })
                    .collect();

                let sequence_model =
                    sequences::ActiveModel {
                        id: NotSet,
                        story_id: Set(story_model.id),
                        name: Set(sequence.name),
                        description: Set(sequence.description),
                        enabled_dataset_ids: Set(serde_json::to_string(&mapped_sequence_ids)
                            .unwrap_or_else(|_| "[]".into())),
                        edge_order: Set(serde_json::to_string(&mapped_edge_order)
                            .unwrap_or_else(|_| "[]".into())),
                        created_at: Set(sequence.created_at),
                        updated_at: Set(sequence.updated_at),
                    };

                sequence_model
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to insert sequence: {}", e))?;
            }
        }

        Ok(())
    }

    async fn import_palette_from_export(
        &self,
        project_id: i32,
        export: PaletteExport,
        dataset_map: &HashMap<i32, i32>,
    ) -> Result<()> {
        if export.layers.is_empty() {
            return Ok(());
        }

        let mut layer_id_map = HashMap::new();
        for layer in export.layers {
            let inserted = project_layers::ActiveModel {
                id: NotSet,
                project_id: Set(project_id),
                layer_id: Set(layer.layer_id),
                name: Set(layer.name),
                background_color: Set(layer.background_color),
                text_color: Set(layer.text_color),
                border_color: Set(layer.border_color),
                alias: Set(layer.alias),
                source_dataset_id: Set(layer
                    .source_dataset_id
                    .and_then(|id| dataset_map.get(&id).copied())),
                enabled: Set(layer.enabled),
                created_at: Set(layer.created_at),
                updated_at: Set(layer.updated_at),
            }
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to insert project layer: {}", e))?;

            layer_id_map.insert(layer.original_id, inserted.id);
        }

        for alias in export.aliases {
            if let Some(target_id) = layer_id_map.get(&alias.target_layer_original_id) {
                let alias_model = layer_aliases::ActiveModel {
                    id: NotSet,
                    project_id: Set(project_id),
                    alias_layer_id: Set(alias.alias_layer_id),
                    target_layer_id: Set(*target_id),
                    created_at: Set(Utc::now()),
                };
                alias_model
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to insert layer alias: {}", e))?;
            }
        }

        Ok(())
    }

    async fn import_knowledge_base_from_export(
        &self,
        project_id: i32,
        index: KnowledgeBaseIndex,
        blobs: HashMap<String, Vec<u8>>,
    ) -> Result<()> {
        if index.files.is_empty() {
            return Ok(());
        }

        let mut tag_id_map = HashMap::new();
        for tag in index.tags {
            let tag_id = Uuid::parse_str(&tag.id)
                .map_err(|e| anyhow!("Invalid tag id {}: {}", tag.id, e))?;
            tag_id_map.insert(tag.id.clone(), tag_id);
            if kb_tags::Entity::find_by_id(tag_id)
                .one(&self.db)
                .await?
                .is_none()
            {
                let model = kb_tags::ActiveModel {
                    id: Set(tag_id),
                    name: Set(tag.name),
                    scope: Set(tag.scope),
                    color: Set(tag.color),
                    created_at: Set(tag.created_at),
                };
                model
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to insert knowledge base tag: {}", e))?;
            }
        }

        let mut file_id_map = HashMap::new();
        for file in index.files {
            let file_id = Uuid::parse_str(&file.id)
                .map_err(|e| anyhow!("Invalid file id {}: {}", file.id, e))?;
            let blob = blobs
                .get(&file.id)
                .cloned()
                .ok_or_else(|| anyhow!("Missing blob for knowledge base file {}", file.id))?;
            let model = kb_files::ActiveModel {
                id: Set(file_id),
                project_id: Set(project_id),
                filename: Set(file.filename),
                media_type: Set(file.media_type),
                size_bytes: Set(file.size_bytes),
                blob: Set(blob),
                checksum: Set(file.checksum),
                created_by: Set(None),
                created_at: Set(file.created_at),
                indexed: Set(file.indexed),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert knowledge base file: {}", e))?;
            file_id_map.insert(file.id, file_id);
        }

        for link in index.file_tags {
            let file_id = file_id_map
                .get(&link.file_id)
                .copied()
                .ok_or_else(|| anyhow!("Unknown file id {} in file_tags", link.file_id))?;
            let tag_id = tag_id_map
                .get(&link.tag_id)
                .copied()
                .ok_or_else(|| anyhow!("Unknown tag id {} in file_tags", link.tag_id))?;

            let model = kb_file_tags::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_id: Set(file_id),
                tag_id: Set(tag_id),
                created_at: Set(Utc::now()),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert file tag mapping: {}", e))?;
        }

        for document in index.documents {
            let doc_id = Uuid::parse_str(&document.id)
                .map_err(|e| anyhow!("Invalid document id {}: {}", document.id, e))?;
            let file_id = match document.file_id {
                Some(ref id) => Some(
                    file_id_map
                        .get(id)
                        .copied()
                        .ok_or_else(|| anyhow!("Unknown file id {} in documents", id))?,
                ),
                None => None,
            };
            let embedding = match document.embedding {
                Some(ref encoded) => {
                    Some(general_purpose::STANDARD.decode(encoded).map_err(|e| {
                        anyhow!("Failed to decode embedding for {}: {}", document.id, e)
                    })?)
                }
                None => None,
            };

            let model = kb_docs::ActiveModel {
                id: Set(doc_id),
                project_id: Set(project_id),
                file_id: Set(file_id),
                chunk_id: Set(document.chunk_id),
                media_type: Set(document.media_type),
                chunk_text: Set(document.chunk_text),
                metadata: Set(document.metadata),
                embedding_model: Set(document.embedding_model),
                embedding: Set(embedding),
                created_at: Set(document.created_at),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert knowledge base document: {}", e))?;
        }

        for state in index.vector_states {
            let state_id = Uuid::parse_str(&state.id)
                .map_err(|e| anyhow!("Invalid vector state id {}: {}", state.id, e))?;
            let model = kb_vector_state::ActiveModel {
                id: Set(state_id),
                project_id: Set(project_id),
                status: Set(state.status),
                last_indexed_at: Set(state.last_indexed_at),
                last_error: Set(state.last_error),
                config: Set(state.config),
                updated_at: Set(state.updated_at),
                embedding_provider: Set(state.embedding_provider),
                embedding_model: Set(state.embedding_model),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert vector index state: {}", e))?;
        }

        Ok(())
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
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
struct ExportAsset {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanIndexEntry {
    original_id: i32,
    filename: String,
    name: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PlansIndex {
    plans: Vec<PlanIndexEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedPlanMetadata {
    original_id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    yaml_content: String,
    dependencies: Option<Vec<i32>>,
    status: String,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedPlanFile {
    metadata: ExportedPlanMetadata,
    dag: PlanDagSnapshot,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedSequence {
    original_id: i32,
    name: String,
    description: Option<String>,
    enabled_dataset_ids: Vec<i32>,
    edge_order: Vec<crate::graphql::types::sequence::SequenceEdgeRef>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedStory {
    original_id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    enabled_dataset_ids: Vec<i32>,
    layer_config: Value,
    sequences: Vec<ExportedSequence>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct StoriesExport {
    stories: Vec<ExportedStory>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedLayer {
    original_id: i32,
    layer_id: String,
    name: String,
    background_color: String,
    text_color: String,
    border_color: String,
    alias: Option<String>,
    source_dataset_id: Option<i32>,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedLayerAlias {
    alias_layer_id: String,
    target_layer_original_id: i32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PaletteExport {
    layers: Vec<ExportedLayer>,
    aliases: Vec<ExportedLayerAlias>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseFileEntry {
    id: String,
    filename: String,
    media_type: String,
    size_bytes: i64,
    checksum: String,
    created_at: DateTime<Utc>,
    indexed: bool,
    blob_path: String,
    tag_ids: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseTagEntry {
    id: String,
    name: String,
    scope: String,
    color: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseFileTagEntry {
    file_id: String,
    tag_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseDocumentEntry {
    id: String,
    file_id: Option<String>,
    chunk_id: String,
    media_type: String,
    chunk_text: String,
    metadata: Option<Value>,
    embedding_model: Option<String>,
    embedding: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseVectorStateEntry {
    id: String,
    status: String,
    last_indexed_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    config: Option<Value>,
    embedding_provider: Option<String>,
    embedding_model: Option<String>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseIndex {
    files: Vec<KnowledgeBaseFileEntry>,
    tags: Vec<KnowledgeBaseTagEntry>,
    file_tags: Vec<KnowledgeBaseFileTagEntry>,
    documents: Vec<KnowledgeBaseDocumentEntry>,
    vector_states: Vec<KnowledgeBaseVectorStateEntry>,
}

#[derive(Clone)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub description_is_set: bool,
    pub tags: Option<Vec<String>>,
}

impl ProjectUpdate {
    pub fn new(
        name: Option<String>,
        description: Option<String>,
        description_is_set: bool,
        tags: Option<Vec<String>>,
    ) -> Self {
        Self {
            name,
            description,
            description_is_set,
            tags,
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
}

impl From<data_sets::Model> for DataSetSummary {
    fn from(model: data_sets::Model) -> Self {
        let (node_count, edge_count, layer_count) = summarize_graph_counts(&model.graph_json);
        let has_layers = layer_count.unwrap_or(0) > 0;
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
    pub attrs: Option<Value>,
    pub belongs_to: Option<String>,
}

#[derive(Clone)]
pub struct GraphLayerUpdateRequest {
    pub id: i32,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub properties: Option<Value>,
}

fn node_type_prefix(node_type: &PlanDagNodeType) -> &'static str {
    match node_type {
        PlanDagNodeType::DataSet => "dataset",
        PlanDagNodeType::Graph => "graph",
        PlanDagNodeType::Transform => "transform",
        PlanDagNodeType::Filter => "filter",
        PlanDagNodeType::Merge => "merge",
        PlanDagNodeType::GraphArtefact => "graphartefact",
        PlanDagNodeType::TreeArtefact => "treeartefact",
        PlanDagNodeType::Story => "story",
        PlanDagNodeType::SequenceArtefact => "sequenceartefact",
    }
}

fn node_type_storage_name(node_type: &PlanDagNodeType) -> &'static str {
    match node_type {
        PlanDagNodeType::DataSet => "DataSetNode",
        PlanDagNodeType::Graph => "GraphNode",
        PlanDagNodeType::Transform => "TransformNode",
        PlanDagNodeType::Filter => "FilterNode",
        PlanDagNodeType::Merge => "MergeNode",
        PlanDagNodeType::GraphArtefact => "GraphArtefactNode",
        PlanDagNodeType::TreeArtefact => "TreeArtefactNode",
        PlanDagNodeType::Story => "StoryNode",
        PlanDagNodeType::SequenceArtefact => "SequenceArtefactNode",
    }
}

fn generate_node_id(
    node_type: &PlanDagNodeType,
    _existing_nodes: &[PlanDagNode],
) -> Result<String> {
    // Generate a globally unique ID using UUID to prevent collisions across projects/plans
    // Format: <node_type_prefix>_<uuid>
    let prefix = node_type_prefix(node_type);
    let uuid = Uuid::new_v4().simple().to_string();

    // Use first 12 characters of UUID for readability
    let short_uuid = uuid.chars().take(12).collect::<String>();

    Ok(format!("{}_{}", prefix, short_uuid))
}

fn generate_edge_id(_source: &str, _target: &str) -> String {
    // Generate a globally unique ID using UUID to prevent collisions
    // Format: edge_<uuid>
    let uuid = Uuid::new_v4().simple().to_string();

    // Use first 12 characters of UUID for readability while maintaining uniqueness
    let short_uuid = uuid.chars().take(12).collect::<String>();

    format!("edge_{}", short_uuid)
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetBundleDescriptor {
    pub original_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_format: String,
    pub node_count: Option<usize>,
    pub edge_count: Option<usize>,
    pub layer_count: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetBundleIndex {
    pub datasets: Vec<DatasetBundleDescriptor>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectBundleManifest {
    pub manifest_version: String,
    pub bundle_type: String,
    pub created_with: String,
    pub project_format_version: u32,
    pub generated_at: DateTime<Utc>,
    pub source_project_id: i32,
    pub plan_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRecord {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

fn analyze_data_sets(
    data_sets: &[data_sets::Model],
) -> Result<(Vec<DatasetBundleDescriptor>, HashMap<i32, String>)> {
    let mut descriptors = Vec::new();
    let mut tables = HashMap::new();

    for data_set in data_sets {
        let (node_count, edge_count, layer_count) = summarize_graph_counts(&data_set.graph_json);
        let descriptor = DatasetBundleDescriptor {
            original_id: data_set.id,
            name: data_set.name.clone(),
            description: data_set.description.clone(),
            filename: format!(
                "{}_{}.json",
                sanitize_dataset_filename(&data_set.name),
                data_set.id
            ),
            file_format: "json".to_string(),
            node_count,
            edge_count,
            layer_count,
        };
        tables.insert(data_set.id, data_set.graph_json.clone());
        descriptors.push(descriptor);
    }

    Ok((descriptors, tables))
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

fn write_bundle_common_files<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    manifest_bytes: &[u8],
    metadata_bytes: &[u8],
    project_bytes: &[u8],
    dag_bytes: &[u8],
    dataset_index_bytes: &[u8],
) -> Result<()> {
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("manifest.json", options)
        .map_err(|e| anyhow!("Failed to add manifest.json: {}", e))?;
    zip.write_all(manifest_bytes)
        .map_err(|e| anyhow!("Failed to write manifest.json: {}", e))?;

    zip.start_file("metadata.json", options)
        .map_err(|e| anyhow!("Failed to add metadata.json: {}", e))?;
    zip.write_all(metadata_bytes)
        .map_err(|e| anyhow!("Failed to write metadata.json: {}", e))?;

    zip.start_file("project.json", options)
        .map_err(|e| anyhow!("Failed to add project.json: {}", e))?;
    zip.write_all(project_bytes)
        .map_err(|e| anyhow!("Failed to write project.json: {}", e))?;

    zip.start_file("dag.json", options)
        .map_err(|e| anyhow!("Failed to add dag.json: {}", e))?;
    zip.write_all(dag_bytes)
        .map_err(|e| anyhow!("Failed to write dag.json: {}", e))?;

    zip.start_file("datasets/index.json", options)
        .map_err(|e| anyhow!("Failed to add datasets/index.json: {}", e))?;
    zip.write_all(dataset_index_bytes)
        .map_err(|e| anyhow!("Failed to write datasets/index.json: {}", e))?;

    Ok(())
}

fn sanitize_dataset_filename(name: &str) -> String {
    let filtered: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = filtered.trim_matches('_');
    if trimmed.is_empty() {
        "dataset".to_string()
    } else {
        trimmed.to_string()
    }
}

fn read_template_json<T: DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    path: &str,
) -> Result<T> {
    let mut file = archive
        .by_name(path)
        .map_err(|e| anyhow!("Template archive missing {}: {}", path, e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| anyhow!("Failed to read {}: {}", path, e))?;
    serde_json::from_slice(&buffer).map_err(|e| anyhow!("Failed to parse {}: {}", path, e))
}

fn try_read_template_json<T: DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    path: &str,
) -> Result<Option<T>> {
    match archive.by_name(path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| anyhow!("Failed to read {}: {}", path, e))?;
            let parsed = serde_json::from_slice(&buffer)
                .map_err(|e| anyhow!("Failed to parse {}: {}", path, e))?;
            Ok(Some(parsed))
        }
        Err(ZipError::FileNotFound) => Ok(None),
        Err(e) => Err(anyhow!("Template archive missing {}: {}", path, e)),
    }
}

fn read_zip_file_bytes(archive: &mut ZipArchive<Cursor<Vec<u8>>>, path: &str) -> Result<Vec<u8>> {
    let mut file = archive
        .by_name(path)
        .map_err(|e| anyhow!("Archive missing {}: {}", path, e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| anyhow!("Failed to read {}: {}", path, e))?;
    Ok(buffer)
}

async fn insert_plan_dag_from_snapshot(
    db: &DatabaseConnection,
    plan_id: i32,
    snapshot: &PlanDagSnapshot,
    dataset_id_map: &HashMap<i32, i32>,
) -> Result<()> {
    let now = Utc::now();

    // Remap node and edge IDs to avoid collisions with existing records
    let mut node_id_map: HashMap<String, String> = HashMap::new();
    let mut edge_id_map: HashMap<String, String> = HashMap::new();

    let mut allocate_node_id = |old_id: &str| -> String {
        node_id_map
            .entry(old_id.to_string())
            .or_insert_with(|| format!("node_{}", Uuid::new_v4().simple()))
            .clone()
    };

    let mut allocate_edge_id = |old_id: &str| -> String {
        edge_id_map
            .entry(old_id.to_string())
            .or_insert_with(|| format!("edge_{}", Uuid::new_v4().simple()))
            .clone()
    };

    for node in &snapshot.nodes {
        let mut config_value: Value = serde_json::from_str(&node.config)
            .map_err(|e| anyhow!("Invalid node config JSON: {}", e))?;

        if let Some(old_id) = config_value
            .get("dataSetId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
        {
            if let Some(new_id) = dataset_id_map.get(&old_id) {
                if let Some(obj) = config_value.as_object_mut() {
                    obj.insert("dataSetId".to_string(), json!(new_id));
                }
            }
        }

        let metadata_json = serde_json::to_string(&node.metadata)
            .map_err(|e| anyhow!("Failed to encode node metadata: {}", e))?;
        let config_json = serde_json::to_string(&config_value)
            .map_err(|e| anyhow!("Failed to encode node config: {}", e))?;

        let new_id = allocate_node_id(&node.id);

        plan_dag_nodes::ActiveModel {
            id: Set(new_id.clone()),
            plan_id: Set(plan_id),
            node_type: Set(node_type_storage_name(&node.node_type).to_string()),
            position_x: Set(node.position.x),
            position_y: Set(node.position.y),
            source_position: Set(node.source_position.clone()),
            target_position: Set(node.target_position.clone()),
            metadata_json: Set(metadata_json),
            config_json: Set(config_json),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("Failed to insert plan node {}: {}", new_id, e))?;
    }

    for edge in &snapshot.edges {
        let metadata_json = serde_json::to_string(&edge.metadata)
            .map_err(|e| anyhow!("Failed to encode edge metadata: {}", e))?;

        let new_id = allocate_edge_id(&edge.id);
        let source = node_id_map
            .get(&edge.source)
            .cloned()
            .unwrap_or_else(|| edge.source.clone());
        let target = node_id_map
            .get(&edge.target)
            .cloned()
            .unwrap_or_else(|| edge.target.clone());

        plan_dag_edges::ActiveModel {
            id: Set(new_id.clone()),
            plan_id: Set(plan_id),
            source_node_id: Set(source),
            target_node_id: Set(target),
            metadata_json: Set(metadata_json),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("Failed to insert plan edge {}: {}", new_id, e))?;
    }

    Ok(())
}
