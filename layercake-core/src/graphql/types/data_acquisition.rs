use async_graphql::{Enum, InputObject, SimpleObject, Upload};
use chrono::{DateTime, Utc};

#[derive(SimpleObject, Clone)]
pub struct KnowledgeBaseStatus {
    pub project_id: i32,
    pub file_count: i64,
    pub chunk_count: i64,
    pub status: String,
    pub last_indexed_at: Option<DateTime<Utc>>,
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(SimpleObject, Clone)]
pub struct ProjectFile {
    pub id: String,
    pub filename: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub indexed: bool,
}

#[derive(SimpleObject, Clone)]
pub struct TagView {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub color: Option<String>,
}

#[derive(InputObject)]
pub struct IngestFileInput {
    pub project_id: i32,
    pub filename: String,
    pub media_type: String,
    pub file: Upload,
    #[graphql(default)]
    pub tags: Vec<String>,
    #[graphql(default = false)]
    pub index_immediately: bool,
}

#[derive(SimpleObject)]
pub struct FileIngestionPayload {
    pub file_id: String,
    pub checksum: String,
    pub chunk_count: i32,
    pub indexed: bool,
}

#[derive(InputObject)]
pub struct UpdateIngestedFileInput {
    pub project_id: i32,
    pub file_id: String,
    pub filename: Option<String>,
    #[graphql(default)]
    pub tags: Vec<String>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum KnowledgeBaseAction {
    Rebuild,
    Clear,
}

#[derive(InputObject)]
pub struct KnowledgeBaseCommandInput {
    pub project_id: i32,
    pub action: KnowledgeBaseAction,
}

#[derive(InputObject)]
pub struct DatasetGenerationInput {
    pub project_id: i32,
    pub prompt: String,
    #[graphql(default)]
    pub tag_names: Vec<String>,
}

#[derive(SimpleObject)]
pub struct DatasetGenerationPayload {
    pub dataset_yaml: Option<String>,
}

#[derive(InputObject)]
pub struct DeleteFileInput {
    pub project_id: i32,
    pub file_id: String,
}

#[derive(InputObject)]
pub struct ToggleFileIndexInput {
    pub project_id: i32,
    pub file_id: String,
    pub indexed: bool,
}

#[derive(InputObject)]
pub struct GetFileContentInput {
    pub project_id: i32,
    pub file_id: String,
}

#[derive(SimpleObject)]
pub struct FileContentPayload {
    pub filename: String,
    pub media_type: String,
    pub content: String, // Base64 encoded
}
