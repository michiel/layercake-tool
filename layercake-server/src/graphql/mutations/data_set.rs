use async_graphql::*;
use base64::Engine;

use layercake_core::app_context::{
    BulkDataSetUpload, DataSetEmptyCreateRequest, DataSetExportFormat, DataSetExportRequest,
    DataSetFileCreateRequest, DataSetFileReplacement, DataSetImportFormat, DataSetImportRequest,
    DataSetUpdateRequest,
};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    BulkUploadDataSetInput, CreateDataSetInput, CreateEmptyDataSetInput, DataSet,
    DataSetValidationResult, ExportDataSetsInput, ExportDataSetsResult, ImportDataSetsInput,
    ImportDataSetsResult, MergeDataSetsInput, UpdateDataSetInput,
};

#[derive(Default)]
pub struct DataSetMutation;

#[Object]
impl DataSetMutation {
    /// Create a new DataSet from uploaded file
    async fn create_data_set_from_file(
        &self,
        ctx: &Context<'_>,
        input: CreateDataSetInput,
    ) -> Result<DataSet> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let file_bytes = base64::engine::general_purpose::STANDARD
            .decode(&input.file_content)
            .map_err(|e| {
                StructuredError::bad_request(format!("Failed to decode base64 file content: {}", e))
            })?;

        let summary = context
            .app
            .create_data_set_from_file(&actor, DataSetFileCreateRequest {
                project_id: input.project_id,
                name: input.name,
                description: input.description,
                filename: input.filename,
                file_format: input.file_format.into(),
                tabular_data_type: input.tabular_data_type.map(Into::into),
                file_bytes,
            })
            .await
            .map_err(Error::from)?;

        Ok(DataSet::from(summary))
    }

    /// Create a new empty DataSet (without file upload)
    async fn create_empty_data_set(
        &self,
        ctx: &Context<'_>,
        input: CreateEmptyDataSetInput,
    ) -> Result<DataSet> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let summary = context
            .app
            .create_empty_data_set(&actor, DataSetEmptyCreateRequest {
                project_id: input.project_id,
                name: input.name,
                description: input.description,
            })
            .await
            .map_err(Error::from)?;

        Ok(DataSet::from(summary))
    }

    /// Bulk upload multiple DataSets with automatic file type detection
    async fn bulk_upload_data_sets(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        files: Vec<BulkUploadDataSetInput>,
    ) -> Result<Vec<DataSet>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let mut uploads = Vec::with_capacity(files.len());
        for file_input in files {
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(&file_input.file_content)
                .map_err(|e| {
                    StructuredError::bad_request(format!(
                        "Failed to decode base64 file content for {}: {}",
                        file_input.filename, e
                    ))
                })?;

            uploads.push(BulkDataSetUpload {
                name: file_input.name,
                description: file_input.description,
                filename: file_input.filename,
                file_bytes,
            });
        }

        let summaries = context
            .app
            .bulk_upload_data_sets(&actor, project_id, uploads)
            .await
            .map_err(Error::from)?;

        Ok(summaries.into_iter().map(DataSet::from).collect())
    }

    /// Update DataSet metadata
    async fn update_data_set(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateDataSetInput,
    ) -> Result<DataSet> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let new_file = if let Some(file_content_b64) = &input.file_content {
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(file_content_b64)
                .map_err(|e| {
                    StructuredError::bad_request(format!(
                        "Failed to decode base64 file content: {}",
                        e
                    ))
                })?;

            Some(DataSetFileReplacement {
                filename: input
                    .filename
                    .clone()
                    .unwrap_or_else(|| "updated_file".to_string()),
                file_bytes,
            })
        } else {
            None
        };

        let summary = context
            .app
            .update_data_set(&actor, DataSetUpdateRequest {
                id,
                name: input.name,
                description: input.description,
                new_file,
            })
            .await
            .map_err(Error::from)?;

        Ok(DataSet::from(summary))
    }

    /// Delete DataSet
    async fn delete_data_set(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .delete_data_set(&actor, id)
            .await
            .map_err(Error::from)?;

        Ok(true)
    }

    /// Reprocess existing DataSet file
    async fn reprocess_data_set(&self, ctx: &Context<'_>, id: i32) -> Result<DataSet> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let summary = context
            .app
            .reprocess_data_set(&actor, id)
            .await
            .map_err(Error::from)?;

        Ok(DataSet::from(summary))
    }

    /// Update DataSet graph data directly
    async fn update_data_set_graph_data(
        &self,
        ctx: &Context<'_>,
        id: i32,
        graph_json: String,
    ) -> Result<DataSet> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let summary = context
            .app
            .update_data_set_graph_json(&actor, id, graph_json)
            .await
            .map_err(Error::from)?;

        Ok(DataSet::from(summary))
    }

    /// Validate DataSet graph integrity
    async fn validate_data_set(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<DataSetValidationResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .validate_data_set(id)
            .await
            .map_err(Error::from)?;

        Ok(DataSetValidationResult::from(summary))
    }

    /// Export data sources as spreadsheet (XLSX or ODS)
    async fn export_data_sets(
        &self,
        ctx: &Context<'_>,
        input: ExportDataSetsInput,
    ) -> Result<ExportDataSetsResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let format = match input.format {
            crate::graphql::types::SpreadsheetFormat::XLSX => DataSetExportFormat::Xlsx,
            crate::graphql::types::SpreadsheetFormat::ODS => DataSetExportFormat::Ods,
        };

        let exported = context
            .app
            .export_data_sets(&actor, DataSetExportRequest {
                project_id: input.project_id,
                data_set_ids: input.data_set_ids,
                format,
            })
            .await
            .map_err(Error::from)?;

        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(&exported.data);

        Ok(ExportDataSetsResult {
            file_content: encoded,
            filename: exported.filename,
            format: exported.format.extension().to_string(),
        })
    }

    /// Import data sources from spreadsheet (XLSX or ODS)
    async fn import_data_sets(
        &self,
        ctx: &Context<'_>,
        input: ImportDataSetsInput,
    ) -> Result<ImportDataSetsResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        use base64::{engine::general_purpose, Engine as _};
        let file_bytes = general_purpose::STANDARD
            .decode(&input.file_content)
            .map_err(|e| StructuredError::bad_request(format!("Invalid base64 content: {}", e)))?;

        let format = DataSetImportFormat::from_filename(&input.filename).ok_or_else(|| {
            StructuredError::bad_request("Only XLSX and ODS formats are supported for import")
        })?;

        let outcome = context
            .app
            .import_data_sets(&actor, DataSetImportRequest {
                project_id: input.project_id,
                format,
                file_bytes,
            })
            .await
            .map_err(Error::from)?;

        Ok(ImportDataSetsResult {
            data_sets: outcome.data_sets.into_iter().map(DataSet::from).collect(),
            created_count: outcome.created_count,
            updated_count: outcome.updated_count,
        })
    }

    /// Merge multiple data sets into a single new data set
    async fn merge_data_sets(
        &self,
        ctx: &Context<'_>,
        input: MergeDataSetsInput,
    ) -> Result<DataSet> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let summary = context
            .app
            .merge_data_sets(
                &actor,
                input.project_id,
                input.data_set_ids,
                input.name,
                input.sum_weights,
                input.delete_merged,
            )
            .await
            .map_err(Error::from)?;

        Ok(DataSet::from(summary))
    }
}
