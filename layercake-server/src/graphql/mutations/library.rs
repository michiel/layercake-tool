use async_graphql::*;
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::path::PathBuf;

use layercake_core::database::entities::common_types::FileFormat as EntityFileFormat;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    DataSet, ExportProjectArchivePayload, LibraryItem, LibraryItemType, Project,
    UpdateLibraryItemInput, UploadLibraryItemInput,
};
use layercake_core::services::library_item_service::{
    infer_data_type, DatasetMetadata, LibraryItemService, SeedLibraryResult, ITEM_TYPE_DATASET,
    ITEM_TYPE_PROJECT, ITEM_TYPE_PROJECT_TEMPLATE, ITEM_TYPE_PROMPT,
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
        let actor = context.actor_for_request(ctx).await;
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

                service
                    .create_dataset_item(
                        &actor,
                        input.name,
                        input.description,
                        tags,
                        input.file_name,
                        file_format.into(),
                        input.tabular_data_type.map(Into::into),
                        input.content_type,
                        file_bytes,
                    )
                    .await
            }
            LibraryItemType::Project
            | LibraryItemType::ProjectTemplate
            | LibraryItemType::Prompt => {
                let metadata = json!({
                    "filename": input.file_name,
                    "uploadSource": "graphql",
                    "contentType": input.content_type,
                });

                let item_type = match input.item_type {
                    LibraryItemType::Project => ITEM_TYPE_PROJECT.to_string(),
                    LibraryItemType::ProjectTemplate => ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
                    LibraryItemType::Prompt => ITEM_TYPE_PROMPT.to_string(),
                    _ => ITEM_TYPE_DATASET.to_string(),
                };

                service
                    .create_binary_item(
                        &actor,
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
        .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(LibraryItem::from(item))
    }

    /// Delete a library item
    async fn delete_library_item(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = LibraryItemService::new(context.db.clone());

        service
            .delete(&actor, id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

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
        let actor = context.actor_for_request(ctx).await;
        let service = LibraryItemService::new(context.db.clone());

        let updated = service
            .update_fields(&actor, id, input.name, input.description, input.tags)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(LibraryItem::from(updated))
    }

    /// Import one or more dataset-type library items into a project
    async fn import_library_datasets(
        &self,
        ctx: &Context<'_>,
        input: ImportLibraryDatasetsInput,
    ) -> Result<Vec<DataSet>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = LibraryItemService::new(context.db.clone());

        let models = service
            .import_many_datasets(&actor, input.project_id, &input.library_item_ids)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(models.into_iter().map(DataSet::from).collect())
    }

    /// Export an existing project as a reusable template stored in the library
    async fn export_project_as_template(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
    ) -> Result<LibraryItem> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let item = context
            .app
            .export_project_as_template(&actor, project_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(LibraryItem::from(item))
    }

    /// Export a project archive for direct download
    async fn export_project_archive(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
        #[graphql(name = "includeKnowledgeBase")] include_knowledge_base: Option<bool>,
    ) -> Result<ExportProjectArchivePayload> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let archive = context
            .app
            .export_project_archive(&actor, project_id, include_knowledge_base.unwrap_or(false))
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        let file_content = general_purpose::STANDARD.encode(archive.bytes);
        Ok(ExportProjectArchivePayload {
            filename: archive.filename,
            file_content,
        })
    }

    /// Import a project from a zip archive
    async fn import_project_archive(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "fileContent")] file_content: String,
        name: Option<String>,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let archive_bytes = general_purpose::STANDARD
            .decode(&file_content)
            .map_err(|e| {
                StructuredError::validation(
                    "fileContent",
                    format!("Invalid base64 file content: {}", e),
                )
            })?;

        let project = context
            .app
            .import_project_archive(&actor, archive_bytes, name)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Project::from(project))
    }

    /// Export a project archive directly to a filesystem directory
    async fn export_project_to_directory(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
        path: String,
        #[graphql(name = "includeKnowledgeBase")] include_knowledge_base: Option<bool>,
        #[graphql(name = "keepConnection")] keep_connection: Option<bool>,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let target = PathBuf::from(&path);

        context
            .app
            .export_project_to_directory(
                &actor,
                project_id,
                &target,
                include_knowledge_base.unwrap_or(false),
                keep_connection.unwrap_or(false),
            )
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(true)
    }

    /// Import a project from a filesystem directory matching the exported bundle layout
    async fn import_project_from_directory(
        &self,
        ctx: &Context<'_>,
        path: String,
        name: Option<String>,
        #[graphql(name = "keepConnection")] keep_connection: Option<bool>,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let source = PathBuf::from(&path);

        let project = context
            .app
            .import_project_from_directory(&actor, &source, name, keep_connection.unwrap_or(false))
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Project::from(project))
    }

    /// Re-import a project using its stored connection path
    async fn reimport_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let project = context
            .app
            .reimport_project_from_connection(&actor, project_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Project::from(project))
    }

    /// Re-export a project to its stored connection path
    async fn reexport_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
        #[graphql(name = "includeKnowledgeBase")] include_knowledge_base: Option<bool>,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .reexport_project_to_connection(&actor, project_id, include_knowledge_base.unwrap_or(false))
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(true)
    }

    /// Reset a project by exporting and re-importing it
    /// This recreates the project with fresh IDs while preserving all data
    async fn reset_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "projectId")] project_id: i32,
        #[graphql(name = "includeKnowledgeBase")] include_knowledge_base: Option<bool>,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        // Export the project
        let archive = context
            .app
            .export_project_archive(&actor, project_id, include_knowledge_base.unwrap_or(false))
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        // Get project name before deletion
        let projects = context
            .app
            .list_projects()
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;
        let project_name = projects
            .into_iter()
            .find(|p| p.id == project_id)
            .map(|p| p.name)
            .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

        // Delete the old project
        context
            .app
            .delete_project(&actor, project_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        // Re-import the project
        let new_project = context
            .app
            .import_project_archive(&actor, archive.bytes, Some(project_name))
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Project::from(new_project))
    }

    /// Create a new project from a stored library item (project or template)
    async fn create_project_from_library(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "libraryItemId")] library_item_id: i32,
        name: Option<String>,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let project = context
            .app
            .create_project_from_library(&actor, library_item_id, name)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Project::from(project))
    }

    /// Seed the dataset library from the bundled resources
    async fn seed_library_items(&self, ctx: &Context<'_>) -> Result<SeedLibraryItemsResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = LibraryItemService::new(context.db.clone());

        let result = service
            .seed_from_repository(&actor)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(SeedLibraryItemsResult::from(result))
    }

    /// Re-detect data type for a library dataset by analyzing file content
    #[graphql(name = "redetectLibraryDatasetType")]
    async fn redetect_library_dataset_type(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<LibraryItem> {
        let context = ctx.data::<GraphQLContext>()?;
        let _actor = context.actor_for_request(ctx).await;
        let service = LibraryItemService::new(context.db.clone());

        // Get the library item
        let item = service
            .get(id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?
            .ok_or_else(|| StructuredError::not_found("Library item", id.to_string()))?;

        // Verify it's a dataset
        if item.item_type != ITEM_TYPE_DATASET {
            return Err(StructuredError::bad_request(format!(
                "Cannot re-detect type for non-dataset item (type: {})",
                item.item_type
            )));
        }

        // Parse existing metadata
        let mut metadata =
            serde_json::from_str::<DatasetMetadata>(&item.metadata).unwrap_or_default();

        // Parse format from metadata
        let file_format = metadata
            .format
            .parse::<EntityFileFormat>()
            .or_else(|_| {
                EntityFileFormat::from_extension(&metadata.filename)
                    .ok_or_else(|| anyhow::anyhow!("Cannot determine file format"))
            })
            .map_err(|e| StructuredError::bad_request(format!("Invalid file format: {}", e)))?;

        // Infer data type from content
        let inferred_type = infer_data_type(&metadata.filename, &file_format, &item.content_blob)
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        // Update metadata with new data type
        metadata.data_type = inferred_type.as_ref().to_string();

        // Update the library item metadata
        let updated_metadata_json = serde_json::to_string(&metadata).map_err(|e| {
            StructuredError::internal(format!("Failed to serialize metadata: {}", e))
        })?;

        use layercake_core::database::entities::library_items;
        use sea_orm::{ActiveModelTrait, Set};

        let mut active: library_items::ActiveModel = item.into();
        active.metadata = Set(updated_metadata_json);
        active.updated_at = Set(chrono::Utc::now());

        let updated = active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("update library item metadata", e))?;

        Ok(LibraryItem::from(updated))
    }
}
