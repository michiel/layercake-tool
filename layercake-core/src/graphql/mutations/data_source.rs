use async_graphql::*;
use base64::Engine;
use sea_orm::{ActiveModelTrait, EntityTrait};

use crate::app_context::{
    BulkDataSourceUpload, DataSourceEmptyCreateRequest, DataSourceExportFormat,
    DataSourceExportRequest, DataSourceFileCreateRequest, DataSourceFileReplacement,
    DataSourceImportFormat, DataSourceImportRequest, DataSourceUpdateRequest,
};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    BulkUploadDataSourceInput, CreateDataSourceInput, CreateEmptyDataSourceInput, DataSource,
    ExportDataSourcesInput, ExportDataSourcesResult, ImportDataSourcesInput,
    ImportDataSourcesResult, UpdateDataSourceInput,
};

#[derive(Default)]
pub struct DataSourceMutation;

#[Object]
impl DataSourceMutation {
    /// Create a new DataSource from uploaded file
    async fn create_data_source_from_file(
        &self,
        ctx: &Context<'_>,
        input: CreateDataSourceInput,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;

        use base64::Engine;
        let file_bytes = base64::engine::general_purpose::STANDARD
            .decode(&input.file_content)
            .map_err(|e| {
                StructuredError::bad_request(format!("Failed to decode base64 file content: {}", e))
            })?;

        let summary = context
            .app
            .create_data_source_from_file(DataSourceFileCreateRequest {
                project_id: input.project_id,
                name: input.name,
                description: input.description,
                filename: input.filename,
                file_format: input.file_format.into(),
                data_type: input.data_type.into(),
                file_bytes,
            })
            .await
            .map_err(|e| StructuredError::service("AppContext::create_data_source_from_file", e))?;

        Ok(DataSource::from(summary))
    }

    /// Create a new empty DataSource (without file upload)
    async fn create_empty_data_source(
        &self,
        ctx: &Context<'_>,
        input: CreateEmptyDataSourceInput,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .create_empty_data_source(DataSourceEmptyCreateRequest {
                project_id: input.project_id,
                name: input.name,
                description: input.description,
                data_type: input.data_type.into(),
            })
            .await
            .map_err(|e| StructuredError::service("AppContext::create_empty_data_source", e))?;

        Ok(DataSource::from(summary))
    }

    /// Bulk upload multiple DataSources with automatic file type detection
    async fn bulk_upload_data_sources(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        files: Vec<BulkUploadDataSourceInput>,
    ) -> Result<Vec<DataSource>> {
        let context = ctx.data::<GraphQLContext>()?;

        use base64::Engine;
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

            uploads.push(BulkDataSourceUpload {
                name: file_input.name,
                description: file_input.description,
                filename: file_input.filename,
                file_bytes,
            });
        }

        let summaries = context
            .app
            .bulk_upload_data_sources(project_id, uploads)
            .await
            .map_err(|e| StructuredError::service("AppContext::bulk_upload_data_sources", e))?;

        Ok(summaries.into_iter().map(DataSource::from).collect())
    }

    /// Update DataSource metadata
    async fn update_data_source(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateDataSourceInput,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;

        use base64::Engine;
        let new_file = if let Some(file_content_b64) = &input.file_content {
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(file_content_b64)
                .map_err(|e| {
                    StructuredError::bad_request(format!(
                        "Failed to decode base64 file content: {}",
                        e
                    ))
                })?;

            Some(DataSourceFileReplacement {
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
            .update_data_source(DataSourceUpdateRequest {
                id,
                name: input.name,
                description: input.description,
                new_file,
            })
            .await
            .map_err(|e| StructuredError::service("AppContext::update_data_source", e))?;

        Ok(DataSource::from(summary))
    }

    /// Delete DataSource
    async fn delete_data_source(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        context
            .app
            .delete_data_source(id)
            .await
            .map_err(|e| StructuredError::service("AppContext::delete_data_source", e))?;

        Ok(true)
    }

    /// Reprocess existing DataSource file
    async fn reprocess_data_source(&self, ctx: &Context<'_>, id: i32) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .reprocess_data_source(id)
            .await
            .map_err(|e| StructuredError::service("AppContext::reprocess_data_source", e))?;

        Ok(DataSource::from(summary))
    }

    /// Update DataSource graph data directly
    async fn update_data_source_graph_data(
        &self,
        ctx: &Context<'_>,
        id: i32,
        graph_json: String,
    ) -> Result<DataSource> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .update_data_source_graph_json(id, graph_json)
            .await
            .map_err(|e| {
                StructuredError::service("AppContext::update_data_source_graph_json", e)
            })?;

        Ok(DataSource::from(summary))
    }

    /// Export data sources as spreadsheet (XLSX or ODS)
    async fn export_data_sources(
        &self,
        ctx: &Context<'_>,
        input: ExportDataSourcesInput,
    ) -> Result<ExportDataSourcesResult> {
        let context = ctx.data::<GraphQLContext>()?;

        let format = match input.format {
            crate::graphql::types::SpreadsheetFormat::XLSX => DataSourceExportFormat::Xlsx,
            crate::graphql::types::SpreadsheetFormat::ODS => DataSourceExportFormat::Ods,
        };

        let exported = context
            .app
            .export_data_sources(DataSourceExportRequest {
                project_id: input.project_id,
                data_source_ids: input.data_source_ids,
                format,
            })
            .await
            .map_err(|e| StructuredError::service("AppContext::export_data_sources", e))?;

        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(&exported.data);

        Ok(ExportDataSourcesResult {
            file_content: encoded,
            filename: exported.filename,
            format: exported.format.extension().to_string(),
        })
    }

    /// Import data sources from spreadsheet (XLSX or ODS)
    async fn import_data_sources(
        &self,
        ctx: &Context<'_>,
        input: ImportDataSourcesInput,
    ) -> Result<ImportDataSourcesResult> {
        let context = ctx.data::<GraphQLContext>()?;

        use base64::{engine::general_purpose, Engine as _};
        let file_bytes = general_purpose::STANDARD
            .decode(&input.file_content)
            .map_err(|e| StructuredError::bad_request(format!("Invalid base64 content: {}", e)))?;

        let format = DataSourceImportFormat::from_filename(&input.filename).ok_or_else(|| {
            StructuredError::bad_request("Only XLSX and ODS formats are supported for import")
        })?;

        let outcome = context
            .app
            .import_data_sources(DataSourceImportRequest {
                project_id: input.project_id,
                format,
                file_bytes,
            })
            .await
            .map_err(|e| StructuredError::service("AppContext::import_data_sources", e))?;

        Ok(ImportDataSourcesResult {
            data_sources: outcome
                .data_sources
                .into_iter()
                .map(DataSource::from)
                .collect(),
            created_count: outcome.created_count,
            updated_count: outcome.updated_count,
        })
    }
}
