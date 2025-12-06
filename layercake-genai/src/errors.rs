use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataAcquisitionError {
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    #[error("ingestion failed: {0}")]
    Ingestion(anyhow::Error),
    #[error("embedding failure: {0}")]
    Embedding(anyhow::Error),
    #[error("vector store error: {0}")]
    VectorStore(anyhow::Error),
    #[error("tag error: {0}")]
    Tag(anyhow::Error),
    #[error("dataset generation failed: {0}")]
    Dataset(anyhow::Error),
    #[error("validation failed for {field}: {message}")]
    Validation { field: String, message: String },
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, DataAcquisitionError>;
