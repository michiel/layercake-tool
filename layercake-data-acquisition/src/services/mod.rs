use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use rig::providers::{ollama, openai};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use sha2::{Digest, Sha256};
use tokio::io::AsyncRead;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::config::{DataAcquisitionConfig, EmbeddingProviderConfig};
use crate::dataset_generation::{DatasetGenerationRequest, DatasetGenerator};
use crate::embeddings::EmbeddingService;
use crate::entities::{file_tags, files, kb_documents, tags, vector_index_state};
use crate::errors::DataAcquisitionError;
use crate::ingestion::{self, IngestionService, ParsedDocument};
use crate::tagging::TagScope;
use crate::vector_store::{SqliteVectorStore, VectorSearchResult};

#[derive(Clone)]
struct EmbeddingSignature {
    provider: String,
    model: String,
}

pub struct DataAcquisitionService {
    db: DatabaseConnection,
    ingestion: Arc<IngestionService>,
    embeddings: Option<EmbeddingService>,
    embedding_signature: Option<EmbeddingSignature>,
    desired_embedding_provider: Option<String>,
    dataset_generator: Option<DatasetGenerator>,
    vector_store: SqliteVectorStore,
}

#[derive(Debug, Clone)]
pub struct FileIngestionRequest {
    pub project_id: i32,
    pub uploader_user_id: Option<i32>,
    pub filename: String,
    pub media_type: String,
    pub tags: Vec<String>,
    pub index_immediately: bool,
}

#[derive(Debug, Clone)]
pub struct FileIngestionResult {
    pub file_id: Uuid,
    pub checksum: String,
    pub chunk_count: usize,
    pub indexed: bool,
}

#[derive(Debug, Clone)]
pub struct KnowledgeBaseStatus {
    pub project_id: i32,
    pub file_count: i64,
    pub chunk_count: i64,
    pub last_indexed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(Debug, Clone)]
pub enum KnowledgeBaseCommand {
    RebuildProject { project_id: i32 },
    ClearProject { project_id: i32 },
}

#[derive(Debug, Clone)]
pub struct StoredFile {
    pub id: Uuid,
    pub project_id: i32,
    pub filename: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateIngestedFileRequest {
    pub project_id: i32,
    pub file_id: Uuid,
    pub filename: Option<String>,
    pub tags: Vec<String>,
}

impl DataAcquisitionService {
    pub fn new(
        db: DatabaseConnection,
        provider_hint: Option<String>,
        provider_config: EmbeddingProviderConfig,
    ) -> Self {
        let config = DataAcquisitionConfig::default();

        let mut ingestion = IngestionService::new(config.clone());
        for (media_type, parser) in ingestion::parsers::default_parsers() {
            ingestion.register_boxed_parser(media_type, parser);
        }

        let normalized_provider = provider_hint
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());

        let (embeddings, signature, dataset_generator) = match normalized_provider.as_deref() {
            Some("openai") => {
                let api_key = provider_config
                    .openai_api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok());
                match api_key.filter(|value| !value.is_empty()) {
                    Some(api_key) => {
                        let base_url = provider_config
                            .openai_base_url
                            .clone()
                            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
                            .filter(|value| !value.is_empty());
                        let mut builder = openai::Client::builder(api_key.as_str());
                        if let Some(ref url) = base_url {
                            builder = builder.base_url(url);
                        }
                        let client = builder.build();
                        let model = provider_config
                            .openai_model
                            .clone()
                            .or_else(|| std::env::var("LAYERCAKE_OPENAI_EMBEDDING_MODEL").ok())
                            .filter(|value| !value.is_empty())
                            .unwrap_or_else(|| "text-embedding-3-large".to_string());
                        let embedding_service =
                            EmbeddingService::openai(client.clone(), model.clone());
                        (
                            Some(embedding_service),
                            Some(EmbeddingSignature {
                                provider: "openai".to_string(),
                                model: model.clone(),
                            }),
                            Some(DatasetGenerator::new(client)),
                        )
                    }
                    None => {
                        warn!(
                            "OPENAI_API_KEY is not configured; knowledge base embeddings are disabled"
                        );
                        (None, None, None)
                    }
                }
            }
            Some("ollama") => {
                let model = provider_config
                    .ollama_model
                    .clone()
                    .or_else(|| std::env::var("LAYERCAKE_OLLAMA_EMBEDDING_MODEL").ok())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "nomic-embed-text:v1.5".to_string());
                let base_url = provider_config
                    .ollama_base_url
                    .clone()
                    .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
                    .or_else(|| std::env::var("OLLAMA_API_BASE_URL").ok())
                    .filter(|value| !value.is_empty());
                let client = if let Some(ref url) = base_url {
                    ollama::Client::builder().base_url(url).build()
                } else {
                    ollama::Client::new()
                };
                let embedding_service = EmbeddingService::ollama(client, model.clone());
                (
                    Some(embedding_service),
                    Some(EmbeddingSignature {
                        provider: "ollama".to_string(),
                        model,
                    }),
                    None,
                )
            }
            Some(provider) => {
                info!(
                    "Embedding provider '{}' is not supported for knowledge base embeddings",
                    provider
                );
                (None, None, None)
            }
            None => (None, None, None),
        };

        Self {
            db: db.clone(),
            ingestion: Arc::new(ingestion),
            embeddings,
            embedding_signature: signature,
            desired_embedding_provider: normalized_provider,
            dataset_generator,
            vector_store: SqliteVectorStore::new(db),
        }
    }

    #[instrument(skip(self, reader))]
    pub async fn ingest_file<R>(
        &self,
        request: FileIngestionRequest,
        reader: R,
    ) -> Result<FileIngestionResult>
    where
        R: AsyncRead + Unpin + Send,
    {
        let parsed_file = self
            .ingestion
            .parse_stream(&request.media_type, &request.filename, reader)
            .await?;

        self.ingest_parsed(request, parsed_file.document, parsed_file.raw_bytes)
            .await
    }

    pub async fn ingest_bytes(
        &self,
        request: FileIngestionRequest,
        bytes: Vec<u8>,
    ) -> Result<FileIngestionResult> {
        let parsed = self
            .ingestion
            .parse_bytes(&request.media_type, &request.filename, &bytes)
            .await?;
        self.ingest_parsed(request, parsed, bytes).await
    }

    async fn persist_file(
        &self,
        request: &FileIngestionRequest,
        bytes: &[u8],
        checksum: &str,
    ) -> Result<Uuid> {
        let txn = self.db.begin().await?;
        let file_id = Uuid::new_v4();
        let record = files::ActiveModel {
            id: Set(file_id),
            project_id: Set(request.project_id),
            filename: Set(request.filename.clone()),
            media_type: Set(request.media_type.clone()),
            size_bytes: Set(bytes.len() as i64),
            blob: Set(bytes.to_vec()),
            checksum: Set(checksum.to_string()),
            created_by: Set(request.uploader_user_id),
            created_at: Set(chrono::Utc::now()),
            indexed: Set(request.index_immediately),
        };
        record.insert(&txn).await?;

        let tag_ids = self
            .ensure_tags(&txn, &request.tags, TagScope::File)
            .await?;
        for tag_id in tag_ids {
            file_tags::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_id: Set(file_id),
                tag_id: Set(tag_id),
                created_at: Set(chrono::Utc::now()),
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        Ok(file_id)
    }

    async fn ingest_parsed(
        &self,
        request: FileIngestionRequest,
        parsed: ParsedDocument,
        bytes: Vec<u8>,
    ) -> Result<FileIngestionResult> {
        let checksum = format!("{:x}", Sha256::digest(&bytes));
        let chunk_count = parsed.chunks.len();
        let signature_for_index = if request.index_immediately {
            Some(
                self.ensure_embedding_signature(request.project_id, true)
                    .await?,
            )
        } else {
            None
        };

        let file_id = self
            .persist_file(&request, &bytes, &checksum)
            .await
            .map_err(|err| DataAcquisitionError::Ingestion(err))?;

        let mut indexed = false;
        if request.index_immediately {
            self.index_parsed_document(request.project_id, file_id, &parsed, signature_for_index)
                .await?;
            indexed = true;
        }

        Ok(FileIngestionResult {
            file_id,
            checksum,
            chunk_count,
            indexed,
        })
    }

    async fn ensure_tags<C>(
        &self,
        conn: &C,
        tags_to_apply: &[String],
        scope: TagScope,
    ) -> Result<Vec<Uuid>>
    where
        C: ConnectionTrait,
    {
        let mut ids = Vec::with_capacity(tags_to_apply.len());
        let mut seen = HashSet::with_capacity(tags_to_apply.len());

        for name in tags_to_apply {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            let normalized = trimmed.to_ascii_lowercase();
            if !seen.insert(normalized) {
                continue;
            }

            let existing = tags::Entity::find()
                .filter(tags::Column::Name.eq(trimmed))
                .filter(tags::Column::Scope.eq(scope.as_str()))
                .one(conn)
                .await?;

            if let Some(model) = existing {
                ids.push(model.id);
                continue;
            }

            let tag_id = Uuid::new_v4();
            tags::ActiveModel {
                id: Set(tag_id),
                name: Set(trimmed.to_string()),
                scope: Set(scope.as_str().to_string()),
                color: Set(None),
                created_at: Set(chrono::Utc::now()),
            }
            .insert(conn)
            .await?;
            ids.push(tag_id);
        }

        Ok(ids)
    }

    async fn index_parsed_document(
        &self,
        project_id: i32,
        file_id: Uuid,
        parsed: &ParsedDocument,
        signature_override: Option<EmbeddingSignature>,
    ) -> Result<()> {
        let signature = if let Some(sig) = signature_override {
            sig
        } else {
            self.ensure_embedding_signature(project_id, true).await?
        };

        let service = self
            .embeddings
            .as_ref()
            .ok_or_else(|| DataAcquisitionError::Validation {
                field: "embeddingProvider".into(),
                message: self.embedding_not_configured_message(),
            })?;
        let embeddings = service.embed_chunks(&parsed.chunks).await?;

        self.vector_store
            .add_embeddings(
                project_id,
                file_id,
                &parsed.media_type,
                &signature.model,
                &embeddings,
            )
            .await?;

        self.update_vector_state(
            project_id,
            "ready",
            None,
            Some(chrono::Utc::now()),
            Some(signature.provider),
            Some(signature.model),
        )
        .await
    }

    pub async fn execute_kb_command(&self, command: KnowledgeBaseCommand) -> Result<()> {
        match command {
            KnowledgeBaseCommand::ClearProject { project_id } => {
                kb_documents::Entity::delete_many()
                    .filter(kb_documents::Column::ProjectId.eq(project_id))
                    .exec(&self.db)
                    .await?;
                self.update_vector_state(project_id, "empty", None, None, None, None)
                    .await?;
            }
            KnowledgeBaseCommand::RebuildProject { project_id } => {
                let signature = self.ensure_embedding_signature(project_id, false).await?;
                kb_documents::Entity::delete_many()
                    .filter(kb_documents::Column::ProjectId.eq(project_id))
                    .exec(&self.db)
                    .await?;
                self.update_vector_state(
                    project_id,
                    "rebuilding",
                    None,
                    None,
                    Some(signature.provider.clone()),
                    Some(signature.model.clone()),
                )
                .await?;
                let files = files::Entity::find()
                    .filter(files::Column::ProjectId.eq(project_id))
                    .filter(files::Column::Indexed.eq(true))
                    .order_by_desc(files::Column::CreatedAt)
                    .all(&self.db)
                    .await?;

                let rebuild_result: Result<()> = async {
                    for file in files {
                        let parsed = self
                            .ingestion
                            .parse_bytes(&file.media_type, &file.filename, &file.blob)
                            .await?;
                        self.index_parsed_document(
                            project_id,
                            file.id,
                            &parsed,
                            Some(signature.clone()),
                        )
                        .await?;
                    }
                    Ok(())
                }
                .await;

                match rebuild_result {
                    Ok(_) => {
                        self.update_vector_state(
                            project_id,
                            "ready",
                            None,
                            Some(chrono::Utc::now()),
                            Some(signature.provider.clone()),
                            Some(signature.model.clone()),
                        )
                        .await?;
                    }
                    Err(err) => {
                        self.update_vector_state(
                            project_id,
                            "error",
                            Some(err.to_string()),
                            None,
                            Some(signature.provider.clone()),
                            Some(signature.model.clone()),
                        )
                        .await?;
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn knowledge_base_status(&self, project_id: i32) -> Result<KnowledgeBaseStatus> {
        let file_count = files::Entity::find()
            .filter(files::Column::ProjectId.eq(project_id))
            .filter(files::Column::Indexed.eq(true))
            .count(&self.db)
            .await?;
        let chunk_count = kb_documents::Entity::find()
            .filter(kb_documents::Column::ProjectId.eq(project_id))
            .count(&self.db)
            .await?;
        let state = vector_index_state::Entity::find()
            .filter(vector_index_state::Column::ProjectId.eq(project_id))
            .order_by_desc(vector_index_state::Column::UpdatedAt)
            .one(&self.db)
            .await?;

        Ok(KnowledgeBaseStatus {
            project_id,
            file_count: file_count as i64,
            chunk_count: chunk_count as i64,
            last_indexed_at: state.as_ref().and_then(|s| s.last_indexed_at),
            status: state
                .as_ref()
                .map(|s| s.status.clone())
                .unwrap_or_else(|| "uninitialized".to_string()),
            embedding_provider: state.as_ref().and_then(|s| s.embedding_provider.clone()),
            embedding_model: state.as_ref().and_then(|s| s.embedding_model.clone()),
        })
    }

    async fn update_vector_state(
        &self,
        project_id: i32,
        status: &str,
        last_error: Option<String>,
        last_indexed_at: Option<chrono::DateTime<chrono::Utc>>,
        embedding_provider: Option<String>,
        embedding_model: Option<String>,
    ) -> Result<()> {
        let existing = vector_index_state::Entity::find()
            .filter(vector_index_state::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await?;

        if let Some(model) = existing {
            let mut active: vector_index_state::ActiveModel = model.into();
            active.status = Set(status.to_string());
            active.last_error = Set(last_error.clone());
            active.updated_at = Set(chrono::Utc::now());
            active.last_indexed_at = Set(last_indexed_at);
            active.embedding_provider = Set(embedding_provider.clone());
            active.embedding_model = Set(embedding_model.clone());
            active.update(&self.db).await?;
        } else {
            vector_index_state::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                status: Set(status.to_string()),
                last_indexed_at: Set(last_indexed_at),
                last_error: Set(last_error),
                config: Set(None),
                updated_at: Set(chrono::Utc::now()),
                embedding_provider: Set(embedding_provider),
                embedding_model: Set(embedding_model),
            }
            .insert(&self.db)
            .await?;
        }

        Ok(())
    }

    fn current_embedding_signature(&self) -> Option<EmbeddingSignature> {
        self.embedding_signature.as_ref().cloned()
    }

    async fn ensure_embedding_signature(
        &self,
        project_id: i32,
        enforce_match: bool,
    ) -> Result<EmbeddingSignature> {
        if let Some(signature) = self.current_embedding_signature() {
            if enforce_match {
                self.ensure_signature_matches_state(project_id, &signature)
                    .await?;
            }
            return Ok(signature);
        }

        let message = self.embedding_not_configured_message();

        Err(DataAcquisitionError::Validation {
            field: "embeddingProvider".into(),
            message,
        }
        .into())
    }

    async fn ensure_signature_matches_state(
        &self,
        project_id: i32,
        signature: &EmbeddingSignature,
    ) -> Result<()> {
        if let Some(state) = vector_index_state::Entity::find()
            .filter(vector_index_state::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await?
        {
            if let Some(provider) = state.embedding_provider {
                if provider != signature.provider {
                    return Err(DataAcquisitionError::Validation {
                        field: "embeddingProvider".into(),
                        message: format!(
                            "Knowledge base currently uses provider '{}'. Clear the knowledge base before switching to '{}'.",
                            provider, signature.provider
                        ),
                    }
                    .into());
                }
            }
            if let Some(model) = state.embedding_model {
                if model != signature.model {
                    return Err(DataAcquisitionError::Validation {
                        field: "embeddingModel".into(),
                        message: format!(
                            "Knowledge base currently uses model '{}'. Clear the knowledge base before switching to '{}'.",
                            model, signature.model
                        ),
                    }
                    .into());
                }
            }
        }

        Ok(())
    }

    fn embedding_not_configured_message(&self) -> String {
        match self.desired_embedding_provider.as_deref() {
            Some("openai") => {
                "Embedding provider 'openai' is configured, but no OpenAI API key was found. Configure OPENAI_API_KEY or update system settings before indexing.".to_string()
            }
            Some("ollama") => "Embedding provider 'ollama' is configured, but the Ollama embedding service is unavailable. Ensure the Ollama base URL and embedding model are configured in system settings."
                .to_string(),
            Some(provider) => format!(
                "Embedding provider '{}' is not supported for knowledge base embeddings yet.",
                provider
            ),
            None => "Embedding provider not configured. Update system settings before indexing."
                .to_string(),
        }
    }

    pub async fn dataset_from_prompt(
        &self,
        request: DatasetGenerationRequest,
    ) -> Result<Option<String>> {
        let generator = match &self.dataset_generator {
            Some(gen) => gen,
            None => return Ok(None),
        };

        let output = generator.run(request).await?;
        Ok(Some(output))
    }

    pub async fn search_context(
        &self,
        project_id: i32,
        embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        self.vector_store
            .similarity_search(project_id, embedding, top_k)
            .await
    }

    pub async fn list_files(&self, project_id: i32) -> Result<Vec<StoredFile>> {
        let files = files::Entity::find()
            .filter(files::Column::ProjectId.eq(project_id))
            .order_by_desc(files::Column::CreatedAt)
            .all(&self.db)
            .await?;

        let file_ids: Vec<Uuid> = files.iter().map(|f| f.id).collect();
        let mut tag_map: HashMap<Uuid, Vec<String>> = HashMap::new();

        if !file_ids.is_empty() {
            let file_tag_rows = file_tags::Entity::find()
                .filter(file_tags::Column::FileId.is_in(file_ids.clone()))
                .all(&self.db)
                .await?;

            let tag_ids: Vec<Uuid> = file_tag_rows.iter().map(|row| row.tag_id).collect();
            let tag_models = if tag_ids.is_empty() {
                Vec::new()
            } else {
                tags::Entity::find()
                    .filter(tags::Column::Id.is_in(tag_ids.clone()))
                    .all(&self.db)
                    .await?
            };
            let mut tag_lookup = HashMap::new();
            for tag in tag_models {
                tag_lookup.insert(tag.id, tag.name);
            }

            for row in file_tag_rows {
                if let Some(name) = tag_lookup.get(&row.tag_id) {
                    tag_map
                        .entry(row.file_id)
                        .or_default()
                        .push(name.to_string());
                }
            }
        }

        let mut stored = Vec::with_capacity(files.len());
        for file in files {
            let mut tags = tag_map.remove(&file.id).unwrap_or_default();
            tags.sort();
            stored.push(StoredFile {
                id: file.id,
                project_id: file.project_id,
                filename: file.filename,
                media_type: file.media_type,
                size_bytes: file.size_bytes,
                checksum: file.checksum,
                created_at: file.created_at,
                tags,
            });
        }

        Ok(stored)
    }

    pub async fn update_ingested_file(
        &self,
        request: UpdateIngestedFileRequest,
    ) -> Result<StoredFile> {
        let txn = self.db.begin().await?;
        let file = files::Entity::find_by_id(request.file_id)
            .one(&txn)
            .await?
            .ok_or_else(|| DataAcquisitionError::NotFound(format!("File {}", request.file_id)))?;

        if file.project_id != request.project_id {
            return Err(DataAcquisitionError::Validation {
                field: "projectId".into(),
                message: "File belongs to a different project".into(),
            }
            .into());
        }

        if request
            .filename
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
        {
            return Err(DataAcquisitionError::Validation {
                field: "filename".into(),
                message: "Filename cannot be empty".into(),
            }
            .into());
        }

        let mut active: files::ActiveModel = file.clone().into();
        if let Some(name) = request.filename {
            active.filename = Set(name);
        }
        let updated = active.update(&txn).await?;

        file_tags::Entity::delete_many()
            .filter(file_tags::Column::FileId.eq(request.file_id))
            .exec(&txn)
            .await?;

        let tag_ids = self
            .ensure_tags(&txn, &request.tags, TagScope::File)
            .await?;
        for tag_id in tag_ids {
            file_tags::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_id: Set(request.file_id),
                tag_id: Set(tag_id),
                created_at: Set(chrono::Utc::now()),
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        let mut tags = request.tags.clone();
        tags.sort();

        Ok(StoredFile {
            id: updated.id,
            project_id: updated.project_id,
            filename: updated.filename,
            media_type: updated.media_type,
            size_bytes: updated.size_bytes,
            checksum: updated.checksum,
            created_at: updated.created_at,
            tags,
        })
    }

    pub async fn delete_file(&self, project_id: i32, file_id: Uuid) -> Result<()> {
        let file = files::Entity::find_by_id(file_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DataAcquisitionError::NotFound(format!("File {}", file_id)))?;

        if file.project_id != project_id {
            return Err(DataAcquisitionError::Validation {
                field: "projectId".into(),
                message: "File belongs to a different project".into(),
            }
            .into());
        }

        // Delete embeddings from vector store
        kb_documents::Entity::delete_many()
            .filter(kb_documents::Column::ProjectId.eq(project_id))
            .filter(kb_documents::Column::FileId.eq(file_id))
            .exec(&self.db)
            .await?;

        // Delete file tags
        file_tags::Entity::delete_many()
            .filter(file_tags::Column::FileId.eq(file_id))
            .exec(&self.db)
            .await?;

        // Delete the file itself
        files::Entity::delete_by_id(file_id).exec(&self.db).await?;

        Ok(())
    }

    pub async fn toggle_file_index(
        &self,
        project_id: i32,
        file_id: Uuid,
        indexed: bool,
    ) -> Result<()> {
        let file = files::Entity::find_by_id(file_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DataAcquisitionError::NotFound(format!("File {}", file_id)))?;

        if file.project_id != project_id {
            return Err(DataAcquisitionError::Validation {
                field: "projectId".into(),
                message: "File belongs to a different project".into(),
            }
            .into());
        }

        // Update the indexed field in the database
        let mut active: files::ActiveModel = file.clone().into();
        active.indexed = Set(indexed);
        active.update(&self.db).await?;

        if indexed {
            // Add to index
            let parsed = self
                .ingestion
                .parse_bytes(&file.media_type, &file.filename, &file.blob)
                .await?;
            self.index_parsed_document(project_id, file_id, &parsed, None)
                .await?;
        } else {
            // Remove from index
            kb_documents::Entity::delete_many()
                .filter(kb_documents::Column::ProjectId.eq(project_id))
                .filter(kb_documents::Column::FileId.eq(file_id))
                .exec(&self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn get_file_content(&self, project_id: i32, file_id: Uuid) -> Result<Vec<u8>> {
        let file = files::Entity::find_by_id(file_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DataAcquisitionError::NotFound(format!("File {}", file_id)))?;

        if file.project_id != project_id {
            return Err(DataAcquisitionError::Validation {
                field: "projectId".into(),
                message: "File belongs to a different project".into(),
            }
            .into());
        }

        Ok(file.blob)
    }

    pub async fn is_file_indexed(&self, project_id: i32, file_id: Uuid) -> Result<bool> {
        let file = files::Entity::find_by_id(file_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DataAcquisitionError::NotFound(format!("File {}", file_id)))?;

        if file.project_id != project_id {
            return Err(DataAcquisitionError::Validation {
                field: "projectId".into(),
                message: "File belongs to a different project".into(),
            }
            .into());
        }

        Ok(file.indexed)
    }

    /// Get reference to the embedding service (for RAG use)
    pub fn embeddings(&self) -> Option<&EmbeddingService> {
        self.embeddings.as_ref()
    }
}
