use anyhow::{Context, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

use crate::embeddings::EmbeddingChunk;
use crate::entities::{files, kb_documents};

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub file_id: Uuid,
    pub filename: Option<String>,
    pub content: String,
    pub score: f32,
}

pub struct SqliteVectorStore {
    db: DatabaseConnection,
}

impl SqliteVectorStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn add_embeddings(
        &self,
        project_id: i32,
        file_id: Uuid,
        media_type: &str,
        embedding_model: &str,
        embeddings: &[EmbeddingChunk],
    ) -> Result<()> {
        if embeddings.is_empty() {
            return Ok(());
        }

        for chunk in embeddings {
            let record = kb_documents::ActiveModel {
                id: Set(Uuid::new_v4()),
                project_id: Set(project_id),
                file_id: Set(Some(file_id)),
                chunk_id: Set(chunk.chunk_id.clone()),
                media_type: Set(media_type.to_string()),
                chunk_text: Set(chunk.text.clone()),
                metadata: Set(Some(chunk.metadata.clone())),
                embedding_model: Set(Some(embedding_model.to_string())),
                embedding: Set(Some(Self::serialize_embedding(&chunk.embedding))),
                created_at: Set(Utc::now()),
            };
            record
                .insert(&self.db)
                .await
                .context("failed to persist embedding chunk")?;
        }

        Ok(())
    }

    pub async fn similarity_search(
        &self,
        project_id: i32,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        // Load kb_documents with optional file join
        let docs_with_files = kb_documents::Entity::find()
            .filter(kb_documents::Column::ProjectId.eq(project_id))
            .find_also_related(files::Entity)
            .order_by_desc(kb_documents::Column::CreatedAt)
            .all(&self.db)
            .await
            .context("failed to load kb documents")?;

        let mut scored: Vec<VectorSearchResult> = docs_with_files
            .into_iter()
            .filter_map(|(doc, file)| {
                let embedding_bytes = doc.embedding?;
                let embedding = Self::deserialize_embedding(&embedding_bytes).ok()?;
                let score = cosine_similarity(query_embedding, &embedding);

                // Get file_id from doc, fall back to unknown UUID if missing
                let file_id = doc.file_id.unwrap_or_else(Uuid::nil);
                let filename = file.map(|f| f.filename);

                Some(VectorSearchResult {
                    file_id,
                    filename,
                    content: doc.chunk_text,
                    score,
                })
            })
            .collect();

        scored.sort_by(|a, b| b.score.total_cmp(&a.score));
        scored.truncate(top_k);
        Ok(scored)
    }

    fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
        embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    fn deserialize_embedding(bytes: &[u8]) -> Result<Vec<f32>> {
        if bytes.len() % 4 != 0 {
            anyhow::bail!("invalid embedding byte length");
        }

        Ok(bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}
