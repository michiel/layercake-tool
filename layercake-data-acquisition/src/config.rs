use std::time::Duration;

/// Configuration knobs for data acquisition flows. Defaults favor offline-friendly
/// behavior and small-batch embedding runs suitable for SQLite-backed storage.
#[derive(Debug, Clone)]
pub struct DataAcquisitionConfig {
    /// Maximum number of characters per extracted chunk before splitting.
    pub max_chunk_chars: usize,
    /// Overlap characters retained between chunk splits.
    pub chunk_overlap_chars: usize,
    /// Maximum documents to embed per batch call.
    pub embedding_batch_size: usize,
    /// Optional timeout applied to long-running ingestion tasks.
    pub ingestion_timeout: Option<Duration>,
}

impl Default for DataAcquisitionConfig {
    fn default() -> Self {
        Self {
            max_chunk_chars: 2_048,
            chunk_overlap_chars: 128,
            embedding_batch_size: 8,
            ingestion_timeout: Some(Duration::from_secs(300)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmbeddingProviderConfig {
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub openai_model: Option<String>,
    pub ollama_base_url: Option<String>,
    pub ollama_api_key: Option<String>,
    pub ollama_model: Option<String>,
}

impl EmbeddingProviderConfig {
    pub fn from_env() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_base_url: std::env::var("OPENAI_BASE_URL").ok(),
            openai_model: std::env::var("LAYERCAKE_OPENAI_EMBEDDING_MODEL").ok(),
            ollama_base_url: std::env::var("OLLAMA_BASE_URL")
                .ok()
                .or_else(|| std::env::var("OLLAMA_API_BASE_URL").ok()),
            ollama_api_key: std::env::var("OLLAMA_API_KEY").ok(),
            ollama_model: std::env::var("LAYERCAKE_OLLAMA_EMBEDDING_MODEL").ok(),
        }
    }
}
