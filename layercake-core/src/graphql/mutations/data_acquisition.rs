use std::io::Read;

use anyhow::Error as AnyError;
use async_graphql::{Context, Object, Result};

use layercake_data_acquisition::dataset_generation::DatasetGenerationRequest;
use layercake_data_acquisition::errors::DataAcquisitionError;
use layercake_data_acquisition::services::{
    FileIngestionRequest, KnowledgeBaseCommand, UpdateIngestedFileRequest,
};
use uuid::Uuid;

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    DatasetGenerationInput, DatasetGenerationPayload, FileIngestionPayload, IngestFileInput,
    KnowledgeBaseAction, KnowledgeBaseCommandInput, ProjectFile, UpdateIngestedFileInput,
};

#[derive(Default)]
pub struct DataAcquisitionMutation;

#[Object]
impl DataAcquisitionMutation {
    async fn ingest_file(
        &self,
        ctx: &Context<'_>,
        input: IngestFileInput,
    ) -> Result<FileIngestionPayload> {
        let context = ctx.data::<GraphQLContext>()?;
        if input.filename.trim().is_empty() {
            return Err(StructuredError::validation(
                "filename",
                "Filename cannot be empty",
            ));
        }
        if input.media_type.trim().is_empty() {
            return Err(StructuredError::validation(
                "mediaType",
                "Media type cannot be empty",
            ));
        }

        let upload = input
            .file
            .value(ctx)
            .map_err(|e| StructuredError::validation("file", format!("Invalid upload: {}", e)))?;
        let size_hint = upload.size().unwrap_or(0) as usize;
        let mut reader = upload.into_read();
        let mut bytes = Vec::with_capacity(size_hint);
        reader.read_to_end(&mut bytes).map_err(|e| {
            StructuredError::service("file", format!("Failed to read upload: {}", e))
        })?;

        let request = FileIngestionRequest {
            project_id: input.project_id,
            uploader_user_id: None,
            filename: input.filename.clone(),
            media_type: input.media_type.clone(),
            tags: input.tags.clone(),
            index_immediately: input.index_immediately,
        };

        let result = context
            .app
            .data_acquisition_service()
            .ingest_bytes(request, bytes)
            .await
            .map_err(|e| map_data_acquisition_error("DataAcquisitionService::ingest_bytes", e))?;

        Ok(FileIngestionPayload {
            file_id: result.file_id.to_string(),
            checksum: result.checksum,
            chunk_count: result.chunk_count as i32,
            indexed: result.indexed,
        })
    }

    async fn run_knowledge_base_command(
        &self,
        ctx: &Context<'_>,
        input: KnowledgeBaseCommandInput,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let command = match input.action {
            KnowledgeBaseAction::Rebuild => KnowledgeBaseCommand::RebuildProject {
                project_id: input.project_id,
            },
            KnowledgeBaseAction::Clear => KnowledgeBaseCommand::ClearProject {
                project_id: input.project_id,
            },
        };

        context
            .app
            .data_acquisition_service()
            .execute_kb_command(command)
            .await
            .map_err(|e| {
                map_data_acquisition_error("DataAcquisitionService::execute_kb_command", e)
            })?;

        Ok(true)
    }

    async fn generate_dataset_from_prompt(
        &self,
        ctx: &Context<'_>,
        input: DatasetGenerationInput,
    ) -> Result<DatasetGenerationPayload> {
        let context = ctx.data::<GraphQLContext>()?;
        let request = DatasetGenerationRequest {
            project_id: input.project_id,
            prompt: input.prompt.clone(),
            tag_names: input.tag_names.clone(),
        };

        let output = context
            .app
            .data_acquisition_service()
            .dataset_from_prompt(request)
            .await
            .map_err(|e| {
                StructuredError::service("DataAcquisitionService::dataset_from_prompt", e)
            })?;

        Ok(DatasetGenerationPayload {
            dataset_yaml: output,
        })
    }

    async fn update_ingested_file(
        &self,
        ctx: &Context<'_>,
        input: UpdateIngestedFileInput,
    ) -> Result<ProjectFile> {
        let context = ctx.data::<GraphQLContext>()?;
        let file_id = Uuid::parse_str(&input.file_id)
            .map_err(|e| StructuredError::validation("file_id", format!("Invalid UUID: {}", e)))?;

        let request = UpdateIngestedFileRequest {
            project_id: input.project_id,
            file_id,
            filename: input.filename.clone(),
            tags: input.tags.clone(),
        };

        let updated = context
            .app
            .data_acquisition_service()
            .update_ingested_file(request)
            .await
            .map_err(|e| {
                map_data_acquisition_error("DataAcquisitionService::update_ingested_file", e)
            })?;

        let indexed = context
            .app
            .data_acquisition_service()
            .is_file_indexed(input.project_id, file_id)
            .await
            .unwrap_or(false);

        Ok(ProjectFile {
            id: updated.id.to_string(),
            filename: updated.filename,
            media_type: updated.media_type,
            size_bytes: updated.size_bytes,
            checksum: updated.checksum,
            created_at: updated.created_at,
            tags: updated.tags,
            indexed,
        })
    }

    async fn delete_file(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::DeleteFileInput,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let file_id = Uuid::parse_str(&input.file_id)
            .map_err(|e| StructuredError::validation("file_id", format!("Invalid UUID: {}", e)))?;

        context
            .app
            .data_acquisition_service()
            .delete_file(input.project_id, file_id)
            .await
            .map_err(|e| map_data_acquisition_error("DataAcquisitionService::delete_file", e))?;

        Ok(true)
    }

    async fn toggle_file_index(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::ToggleFileIndexInput,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let file_id = Uuid::parse_str(&input.file_id)
            .map_err(|e| StructuredError::validation("file_id", format!("Invalid UUID: {}", e)))?;

        context
            .app
            .data_acquisition_service()
            .toggle_file_index(input.project_id, file_id, input.indexed)
            .await
            .map_err(|e| {
                map_data_acquisition_error("DataAcquisitionService::toggle_file_index", e)
            })?;

        Ok(true)
    }

    async fn get_file_content(
        &self,
        ctx: &Context<'_>,
        input: crate::graphql::types::GetFileContentInput,
    ) -> Result<crate::graphql::types::FileContentPayload> {
        let context = ctx.data::<GraphQLContext>()?;
        let file_id = Uuid::parse_str(&input.file_id)
            .map_err(|e| StructuredError::validation("file_id", format!("Invalid UUID: {}", e)))?;

        let content = context
            .app
            .data_acquisition_service()
            .get_file_content(input.project_id, file_id)
            .await
            .map_err(|e| {
                map_data_acquisition_error("DataAcquisitionService::get_file_content", e)
            })?;

        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(&content);

        // Get file metadata for filename and media_type
        let files = context
            .app
            .data_acquisition_service()
            .list_files(input.project_id)
            .await
            .map_err(|e| map_data_acquisition_error("DataAcquisitionService::list_files", e))?;

        let file = files
            .into_iter()
            .find(|f| f.id.to_string() == file_id.to_string())
            .ok_or_else(|| StructuredError::not_found("File", file_id))?;

        Ok(crate::graphql::types::FileContentPayload {
            filename: file.filename,
            media_type: file.media_type,
            content: encoded,
        })
    }
}

fn map_data_acquisition_error(action: &str, error: AnyError) -> async_graphql::Error {
    if let Some(data_error) = error.downcast_ref::<DataAcquisitionError>() {
        match data_error {
            DataAcquisitionError::UnsupportedMediaType(media_type) => {
                return StructuredError::validation(
                    "mediaType",
                    format!("Unsupported media type '{}'", media_type),
                );
            }
            DataAcquisitionError::Validation { field, message } => {
                return StructuredError::validation(field, message.clone());
            }
            DataAcquisitionError::NotFound(message) => {
                return StructuredError::not_found_msg(message.clone());
            }
            DataAcquisitionError::Ingestion(details) => {
                return StructuredError::bad_request(format!("Ingestion failed: {}", details));
            }
            DataAcquisitionError::Embedding(details) => {
                return StructuredError::service("Embedding", details);
            }
            DataAcquisitionError::VectorStore(details) => {
                return StructuredError::service("VectorStore", details);
            }
            DataAcquisitionError::Tag(details) => {
                return StructuredError::service("TagService", details);
            }
            DataAcquisitionError::Dataset(details) => {
                return StructuredError::service("DatasetGenerator", details);
            }
            _ => {}
        }
    }

    StructuredError::service(action, error)
}
