use anyhow::Result;
use rig::client::EmbeddingsClient;
use rig::embeddings::EmbeddingModel;
use rig::providers::{ollama, openai};
use serde_json::Value;

use crate::ingestion::DocumentChunk;

#[derive(Debug, Clone)]
pub struct EmbeddingChunk {
    pub chunk_id: String,
    pub embedding: Vec<f32>,
    pub text: String,
    pub metadata: Value,
}

enum EmbeddingBackend {
    OpenAi {
        client: openai::Client,
        model: String,
    },
    Ollama {
        client: ollama::Client,
        model: String,
    },
}

pub struct EmbeddingService {
    backend: EmbeddingBackend,
}

impl EmbeddingService {
    pub fn openai(client: openai::Client, model: impl Into<String>) -> Self {
        Self {
            backend: EmbeddingBackend::OpenAi {
                client,
                model: model.into(),
            },
        }
    }

    pub fn ollama(client: ollama::Client, model: impl Into<String>) -> Self {
        Self {
            backend: EmbeddingBackend::Ollama {
                client,
                model: model.into(),
            },
        }
    }

    pub fn provider_name(&self) -> &'static str {
        match self.backend {
            EmbeddingBackend::OpenAi { .. } => "openai",
            EmbeddingBackend::Ollama { .. } => "ollama",
        }
    }

    pub fn model_name(&self) -> &str {
        match &self.backend {
            EmbeddingBackend::OpenAi { model, .. } => model.as_str(),
            EmbeddingBackend::Ollama { model, .. } => model.as_str(),
        }
    }

    pub async fn embed_chunks(&self, chunks: &[DocumentChunk]) -> Result<Vec<EmbeddingChunk>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        tracing::info!(
            provider = self.provider_name(),
            model = self.model_name(),
            chunk_count = chunks.len(),
            avg_chunk_length = chunks.iter().map(|c| c.text.len()).sum::<usize>() / chunks.len(),
            "Embedding document chunks"
        );

        let vectors = match &self.backend {
            EmbeddingBackend::OpenAi { client, model } => client
                .embedding_model(model)
                .embed_texts(chunks.iter().map(|chunk| chunk.text.clone()))
                .await
                .map_err(|e| {
                    tracing::error!(
                        provider = "openai",
                        model = model,
                        chunk_count = chunks.len(),
                        error = %e,
                        "Failed to embed chunks"
                    );
                    e
                })?,
            EmbeddingBackend::Ollama { client, model } => {
                // Note: Ollama 0.9.x logs "cannot decode batches" warnings
                // These are harmless llama.cpp internals - embeddings work correctly
                //
                // nomic-embed-text models have 768 dimensions
                // TODO: Make embedding dimensions configurable per model
                client
                    .embedding_model_with_ndims(model, 768)
                    .embed_texts(chunks.iter().map(|chunk| chunk.text.clone()))
                    .await
                    .map_err(|e| {
                        tracing::error!(
                            provider = "ollama",
                            model = model,
                            chunk_count = chunks.len(),
                            error = %e,
                            "Failed to embed chunks"
                        );
                        e
                    })?
            }
        };

        tracing::info!(
            chunk_count = chunks.len(),
            "Successfully embedded all chunks"
        );

        Ok(chunks
            .iter()
            .zip(vectors.into_iter())
            .map(|(chunk, embedding)| EmbeddingChunk {
                chunk_id: chunk.chunk_id.clone(),
                embedding: embedding
                    .vec
                    .into_iter()
                    .map(|value| value as f32)
                    .collect(),
                text: chunk.text.clone(),
                metadata: chunk.metadata.clone(),
            })
            .collect())
    }

    /// Embed a single text query (for RAG search)
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        tracing::debug!(
            provider = self.provider_name(),
            model = self.model_name(),
            text_length = text.len(),
            "Embedding single text for RAG query"
        );

        let vectors = match &self.backend {
            EmbeddingBackend::OpenAi { client, model } => client
                .embedding_model(model)
                .embed_texts(vec![text.to_string()])
                .await
                .map_err(|e| {
                    tracing::error!(
                        provider = "openai",
                        model = model,
                        error = %e,
                        "Failed to embed text"
                    );
                    e
                })?,
            EmbeddingBackend::Ollama { client, model } => client
                .embedding_model_with_ndims(model, 768)
                .embed_texts(vec![text.to_string()])
                .await
                .map_err(|e| {
                    tracing::error!(
                        provider = "ollama",
                        model = model,
                        error = %e,
                        text_length = text.len(),
                        "Failed to embed text - this may be due to Ollama version or token limits"
                    );
                    e
                })?,
        };

        tracing::debug!("Successfully embedded text");

        Ok(vectors
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?
            .vec
            .into_iter()
            .map(|value| value as f32)
            .collect())
    }
}
