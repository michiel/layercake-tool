use async_graphql::*;
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    DataSet, ExportProjectArchivePayload, LibraryItem, LibraryItemType, Project,
    UpdateLibraryItemInput, UploadLibraryItemInput,
};
use crate::services::library_item_service::{
    LibraryItemService, SeedLibraryResult, ITEM_TYPE_DATASET, ITEM_TYPE_PROJECT,
    ITEM_TYPE_PROJECT_TEMPLATE,
};

#[derive(InputObject)]
pub struct ImportLibraryDatasetsInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "libraryItemIds")]
    pub library_item_ids: Vec<i32>,
}

#[derive(SimpleObject)]
pub struct SeedLibraryItemsResult {
    #[graphql(name = "totalRemoteFiles")]
    pub total_remote_files: i32,
    #[graphql(name = "createdCount")]
    pub created_count: i32,
    #[graphql(name = "skippedCount")]
    pub skipped_count: i32,
    #[graphql(name = "failedFiles")]
    pub failed_files: Vec<String>,
}

impl From<SeedLibraryResult> for SeedLibraryItemsResult {
    fn from(result: SeedLibraryResult) -> Self {
        Self {
            total_remote_files: result.total_remote_files as i32,
            created_count: result.created_count as i32,
            skipped_count: result.skipped_count as i32,
            failed_files: result.failed_files,
        }
    }
}

#[derive(Default)]
pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    /// Upload a new library item (dataset, project, or template)
    async fn upload_library_item(
        &self,
        ctx: &Context<'_>,
        input: UploadLibraryItemInput,
    ) -> Result<LibraryItem> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());
        let tags = input.tags.unwrap_or_default();

        let file_bytes = general_purpose::STANDARD
            .decode(input.file_content.as_bytes())
            .map_err(|e| {
                StructuredError::bad_request(format!("Failed to decode base64 file content: {}", e))
            })?;

        let item = match input.item_type {
            LibraryItemType::Dataset => {
                let file_format = input.file_format.ok_or_else(|| {
                    StructuredError::bad_request("fileFormat is required for dataset uploads")
                })?;

                let data_type = input.data_type.ok_or_else(|| {
                    StructuredError::bad_request("dataType is required for dataset uploads")
                })?;

                service
                    .create_dataset_item(
                        input.name,
                        input.description,
                        tags,
                        input.file_name,
                        file_format.into(),
                        data_type.into(),
                        input.content_type,
                        file_bytes,
                    )
                    .await
            }
            LibraryItemType::Project | LibraryItemType::ProjectTemplate => {
                let metadata = json!({
                    "filename": input.file_name,
                    "uploadSource": "graphql",
                    "contentType": input.content_type,
                });

                let item_type = match input.item_type {
                    LibraryItemType::Project => ITEM_TYPE_PROJECT.to_string(),
                    LibraryItemType::ProjectTemplate => ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
                    _ => ITEM_TYPE_DATASET.to_string(),
                };

                service
                    .create_binary_item(
                        item_type,
                        input.name,
                        input.description,
                        tags,
                        metadata,
                        input.content_type,
                        file_bytes,
                    )
                    .await
            }
        }
        .map_err(|e| StructuredError::service("LibraryItemService::upload", e))?;

        Ok(LibraryItem::from(item))
    }

    /// Delete a library item
    async fn delete_library_item(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());

        service
            .delete(id)
            .await
            .map_err(|e| StructuredError::service("LibraryItemService::delete", e))?;

        Ok(true)
    }

    /// Update a library item's basic metadata (name, description, tags)
    async fn update_library_item(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateLibraryItemInput,
    ) -> Result<LibraryItem> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());

        let updated = service
            .update_fields(id, input.name, input.description, input.tags)
            .await
            .map_err(|e| StructuredError::service("LibraryItemService::update_fields", e))?;

        Ok(LibraryItem::from(updated))
    }

    /// Import one or more dataset-type library items into a project
    async fn import_library_datasets(
        &self,
        ctx: &Context<'_>,
        input: ImportLibraryDatasetsInput,
    ) -> Result<Vec<DataSet>> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());

        let models = service
            .import_many_datasets(input.project_id, &input.library_item_ids)
            .await
            .map_err(|e| StructuredError::service("LibraryItemService::import_many_datasets", e))?;

        Ok(models.into_iter().map(DataSet::from).collect())
    }

    /// Export an existing project as a reusable template stored in the library
    async fn export_project_as_template(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
    ) -> Result<LibraryItem> {
        let context = ctx.data::<GraphQLContext>()?;
        let item = context
            .app
            .export_project_as_template(project_id)
            .await
            .map_err(|e| StructuredError::service("AppContext::export_project_as_template", e))?;

        Ok(LibraryItem::from(item))
    }

    /// Export a project archive for direct download
    async fn export_project_archive(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
    ) -> Result<ExportProjectArchivePayload> {
        let context = ctx.data::<GraphQLContext>()?;
        let archive = context
            .app
            .export_project_archive(project_id)
            .await
            .map_err(|e| StructuredError::service("AppContext::export_project_archive", e))?;

        let file_content = general_purpose::STANDARD.encode(archive.bytes);
        Ok(ExportProjectArchivePayload {
            filename: archive.filename,
            file_content,
        })
    }

    /// Create a new project from a stored library item (project or template)
    async fn create_project_from_library(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "libraryItemId")] library_item_id: i32,
        name: Option<String>,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = context
            .app
            .create_project_from_library(library_item_id, name)
            .await
            .map_err(|e| StructuredError::service("AppContext::create_project_from_library", e))?;

        Ok(Project::from(project))
    }

    /// Seed the dataset library from the bundled resources
    async fn seed_library_items(&self, ctx: &Context<'_>) -> Result<SeedLibraryItemsResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = LibraryItemService::new(context.db.clone());

        let result = service
            .seed_from_repository()
            .await
            .map_err(|e| StructuredError::service("LibraryItemService::seed_from_repository", e))?;

        Ok(SeedLibraryItemsResult::from(result))
    }
}
