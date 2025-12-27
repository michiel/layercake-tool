use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{AppContext, DataSetSummary, DataSetValidationSummary, GraphValidationSummary};
use super::{BulkDataSetUpload, DataSetEmptyCreateRequest, DataSetFileCreateRequest};
use super::{DataSetExportFormat, DataSetExportRequest, DataSetExportResult, DataSetUpdateRequest};
use super::{DataSetImportFormat, DataSetImportOutcome, DataSetImportRequest};
use crate::database::entities::data_sets;
use crate::auth::Actor;
use crate::errors::{CoreError, CoreResult};

impl AppContext {
    pub async fn list_data_sets(&self, project_id: i32) -> CoreResult<Vec<DataSetSummary>> {
        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .order_by_asc(data_sets::Column::Name)
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!(
                    "Failed to list data sets for project {}: {}",
                    project_id, e
                ))
            })?;

        Ok(data_sets.into_iter().map(DataSetSummary::from).collect())
    }

    pub async fn available_data_sets(
        &self,
        project_id: i32,
    ) -> CoreResult<Vec<DataSetSummary>> {
        self.list_data_sets(project_id).await
    }

    pub async fn get_data_set(&self, id: i32) -> CoreResult<Option<DataSetSummary>> {
        let data_set = data_sets::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load data set {}: {}", id, e))
            })?;

        Ok(data_set.map(DataSetSummary::from))
    }

    pub async fn create_data_set_from_file(
        &self,
        actor: &Actor,
        request: DataSetFileCreateRequest,
    ) -> CoreResult<DataSetSummary> {
        let DataSetFileCreateRequest {
            project_id,
            name,
            description,
            filename,
            file_format,
            tabular_data_type,
            file_bytes,
        } = request;
        self.authorize_project_write(actor, project_id).await?;

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
            .await?;

        Ok(DataSetSummary::from(created))
    }

    pub async fn create_empty_data_set(
        &self,
        actor: &Actor,
        request: DataSetEmptyCreateRequest,
    ) -> CoreResult<DataSetSummary> {
        let DataSetEmptyCreateRequest {
            project_id,
            name,
            description,
        } = request;
        self.authorize_project_write(actor, project_id).await?;

        let created = self
            .data_set_service
            .create_empty(project_id, name, description)
            .await?;

        Ok(DataSetSummary::from(created))
    }

    pub async fn bulk_upload_data_sets(
        &self,
        actor: &Actor,
        project_id: i32,
        uploads: Vec<BulkDataSetUpload>,
    ) -> CoreResult<Vec<DataSetSummary>> {
        self.authorize_project_write(actor, project_id).await?;
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
                .await?;

            results.push(DataSetSummary::from(created));
        }

        Ok(results)
    }

    pub async fn update_data_set(
        &self,
        actor: &Actor,
        request: DataSetUpdateRequest,
    ) -> CoreResult<DataSetSummary> {
        let DataSetUpdateRequest {
            id,
            name,
            description,
            new_file,
        } = request;
        let project_id = self.project_id_for_data_set(id).await?;
        self.authorize_project_write(actor, project_id).await?;

        let (mut model, had_new_file) = if let Some(file) = new_file {
            let updated = self
                .data_set_service
                .update_file(id, file.filename, file.file_bytes)
                .await?;
            (updated, true)
        } else {
            let updated = self
                .data_set_service
                .update(id, name.clone(), description.clone())
                .await?;
            (updated, false)
        };

        if had_new_file && (name.is_some() || description.is_some()) {
            model = self
                .data_set_service
                .update(id, name, description)
                .await?;
        }

        Ok(DataSetSummary::from(model))
    }

    pub async fn update_data_set_graph_json(
        &self,
        actor: &Actor,
        id: i32,
        graph_json: String,
    ) -> CoreResult<DataSetSummary> {
        self.authorize_data_set_write(actor, id).await?;
        let model = self
            .data_set_service
            .update_graph_data(id, graph_json)
            .await?;

        Ok(DataSetSummary::from(model))
    }

    pub async fn reprocess_data_set(
        &self,
        actor: &Actor,
        id: i32,
    ) -> CoreResult<DataSetSummary> {
        self.authorize_data_set_write(actor, id).await?;
        let model = self
            .data_set_service
            .reprocess(id)
            .await?;

        Ok(DataSetSummary::from(model))
    }

    pub async fn validate_data_set(&self, id: i32) -> CoreResult<DataSetValidationSummary> {
        self.data_set_service
            .validate(id)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to validate data set {}: {}", id, e))
            })
    }

    pub async fn validate_graph(&self, graph_id: i32) -> CoreResult<GraphValidationSummary> {
        self.graph_service
            .validate_graph(graph_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to validate graph {}: {}", graph_id, e)))
    }

    pub async fn delete_data_set(&self, actor: &Actor, id: i32) -> CoreResult<()> {
        self.authorize_data_set_write(actor, id).await?;
        self.data_set_service.delete(id).await
    }

    pub async fn merge_data_sets(
        &self,
        actor: &Actor,
        project_id: i32,
        data_set_ids: Vec<i32>,
        name: String,
        sum_weights: bool,
        delete_merged: bool,
    ) -> CoreResult<DataSetSummary> {
        self.authorize_project_write(actor, project_id).await?;
        if data_set_ids.len() < 2 {
            return Err(CoreError::validation(
                "At least 2 data sets are required for merging",
            ));
        }

        // Load all datasets
        let models = data_sets::Entity::find()
            .filter(data_sets::Column::Id.is_in(data_set_ids.clone()))
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load data sets for merging: {}", e)))?;

        if models.len() != data_set_ids.len() {
            return Err(CoreError::validation(format!(
                "Some data sets were not found or don't belong to project {}",
                project_id
            )));
        }

        // Merge graph JSON data
        let merged_json = self.merge_graph_json_data(&models, sum_weights)?;

        // Create new dataset with merged data
        let summary = self
            .create_empty_data_set(
                actor,
                DataSetEmptyCreateRequest {
                    project_id,
                    name,
                    description: Some(format!(
                        "Merged from {} data sets",
                        data_set_ids.len()
                    )),
                },
            )
            .await?;

        // Update the new dataset with merged graph data
        let summary = self
            .data_set_service
            .update_graph_data(summary.id, merged_json)
            .await?;

        // Delete source datasets if requested
        if delete_merged {
            for id in &data_set_ids {
                let _ = self.delete_data_set(actor, *id).await;
            }
        }

        Ok(DataSetSummary::from(summary))
    }

    fn merge_graph_json_data(
        &self,
        models: &[data_sets::Model],
        sum_weights: bool,
    ) -> CoreResult<String> {
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
            .map_err(|e| CoreError::internal(format!("Failed to serialize merged data: {}", e)))
    }

    pub async fn export_data_sets(
        &self,
        actor: &Actor,
        request: DataSetExportRequest,
    ) -> CoreResult<DataSetExportResult> {
        let DataSetExportRequest {
            project_id,
            data_set_ids,
            format,
        } = request;
        self.authorize_project_read(actor, project_id).await?;

        let matching_count = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .filter(data_sets::Column::Id.is_in(data_set_ids.clone()))
            .count(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!(
                    "Failed to verify data sets for project {}: {}",
                    project_id, e
                ))
            })?;

        if matching_count != data_set_ids.len() as u64 {
            return Err(CoreError::validation(format!(
                "Export request included data sets outside project {}",
                project_id
            )));
        }

        let bytes = match format {
            DataSetExportFormat::Xlsx => self
                .data_set_bulk_service
                .export_to_xlsx(&data_set_ids)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to export datasets to XLSX: {}", e))
                })?,
            DataSetExportFormat::Ods => self
                .data_set_bulk_service
                .export_to_ods(&data_set_ids)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to export datasets to ODS: {}", e))
                })?,
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
        actor: &Actor,
        request: DataSetImportRequest,
    ) -> CoreResult<DataSetImportOutcome> {
        self.authorize_project_write(actor, request.project_id).await?;
        let result = match request.format {
            DataSetImportFormat::Xlsx => self
                .data_set_bulk_service
                .import_from_xlsx(request.project_id, &request.file_bytes)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to import datasets from XLSX: {}", e))
                })?,
            DataSetImportFormat::Ods => self
                .data_set_bulk_service
                .import_from_ods(request.project_id, &request.file_bytes)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to import datasets from ODS: {}", e))
                })?,
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
            .map_err(|e| {
                CoreError::internal(format!("Failed to load imported datasets: {}", e))
            })?;

        Ok(DataSetImportOutcome {
            data_sets: models.into_iter().map(DataSetSummary::from).collect(),
            created_count: result.created_count,
            updated_count: result.updated_count,
        })
    }
}
