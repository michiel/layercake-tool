use async_graphql::*;
use base64::Engine;

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{CreateLibrarySourceInput, DataSource, ImportLibrarySourcesInput, LibrarySource, SeedLibrarySourcesResult, UpdateLibrarySourceInput};
use crate::services::library_source_service::LibrarySourceService;

#[derive(Default)]
pub struct LibrarySourceMutation;

#[Object]
impl LibrarySourceMutation {
async fn create_library_source(
    &self,
    ctx: &Context<'_>,
    input: CreateLibrarySourceInput,
) -> Result<LibrarySource> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = LibrarySourceService::new(context.db.clone());

    let CreateLibrarySourceInput {
        name,
        description,
        filename,
        file_content,
        file_format,
        data_type,
    } = input;

    use base64::Engine;
    let file_bytes = base64::engine::general_purpose::STANDARD
        .decode(&file_content)
        .map_err(|e| {
            StructuredError::bad_request(format!("Failed to decode base64 file content: {}", e))
        })?;

    let file_format: crate::database::entities::data_sources::FileFormat = file_format.into();
    let data_type: crate::database::entities::data_sources::DataType = data_type.into();

    let model = service
        .create_from_file(
            name,
            description,
            filename,
            file_format,
            data_type,
            file_bytes,
        )
        .await
        .map_err(|e| StructuredError::service("LibrarySourceService::create_from_file", e))?;

    Ok(LibrarySource::from(model))
}

/// Update an existing LibrarySource metadata and optionally replace its file
async fn update_library_source(
    &self,
    ctx: &Context<'_>,
    id: i32,
    input: UpdateLibrarySourceInput,
) -> Result<LibrarySource> {
    if input.file_content.is_none() && input.filename.is_some() {
        return Err(StructuredError::validation(
            "filename",
            "filename can only be changed when fileContent is provided",
        ));
    }

    let context = ctx.data::<GraphQLContext>()?;
    let service = LibrarySourceService::new(context.db.clone());

    let mut current = if let Some(file_content) = &input.file_content {
        use base64::Engine;
        let file_bytes = base64::engine::general_purpose::STANDARD
            .decode(file_content)
            .map_err(|e| {
                StructuredError::bad_request(format!(
                    "Failed to decode base64 file content: {}",
                    e
                ))
            })?;

        let filename = if let Some(filename) = &input.filename {
            filename.clone()
        } else {
            service
                .get_by_id(id)
                .await
                .map_err(|e| StructuredError::service("LibrarySourceService::get_by_id", e))?
                .ok_or_else(|| StructuredError::not_found("LibrarySource", id))?
                .filename
        };

        service
            .update_file(id, filename, file_bytes)
            .await
            .map_err(|e| StructuredError::service("LibrarySourceService::update_file", e))?
    } else {
        service
            .get_by_id(id)
            .await
            .map_err(|e| StructuredError::service("LibrarySourceService::get_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("LibrarySource", id))?
    };

    if input.name.is_some() || input.description.is_some() {
        current = service
            .update(id, input.name.clone(), input.description.clone())
            .await
            .map_err(|e| StructuredError::service("LibrarySourceService::update", e))?;
    }

    Ok(LibrarySource::from(current))
}

/// Delete a LibrarySource
async fn delete_library_source(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = LibrarySourceService::new(context.db.clone());

    service
        .delete(id)
        .await
        .map_err(|e| StructuredError::service("LibrarySourceService::delete", e))?;

    Ok(true)
}

/// Reprocess the stored file for a LibrarySource
async fn reprocess_library_source(&self, ctx: &Context<'_>, id: i32) -> Result<LibrarySource> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = LibrarySourceService::new(context.db.clone());

    let model = service
        .reprocess(id)
        .await
        .map_err(|e| StructuredError::service("LibrarySourceService::reprocess", e))?;

    Ok(LibrarySource::from(model))
}

/// Import one or more LibrarySources into a project as project-scoped DataSources
async fn import_library_sources(
    &self,
    ctx: &Context<'_>,
    input: ImportLibrarySourcesInput,
) -> Result<Vec<DataSource>> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = LibrarySourceService::new(context.db.clone());

    let models = service
        .import_many_into_project(input.project_id, &input.library_source_ids)
        .await
        .map_err(|e| {
            StructuredError::service("LibrarySourceService::import_many_into_project", e)
        })?;

    Ok(models.into_iter().map(DataSource::from).collect())
}

/// Seed the shared library with the canonical GitHub resources bundle
async fn seed_library_sources(&self, ctx: &Context<'_>) -> Result<SeedLibrarySourcesResult> {
    let context = ctx.data::<GraphQLContext>()?;
    let service = LibrarySourceService::new(context.db.clone());

    let result = service.seed_from_github_library().await.map_err(|e| {
        StructuredError::service("LibrarySourceService::seed_from_github_library", e)
    })?;

    Ok(SeedLibrarySourcesResult::from(result))
}
}
