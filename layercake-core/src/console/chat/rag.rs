use anyhow::Result;
use layercake_data_acquisition::vector_store::VectorSearchResult;

/// Context built from retrieved document chunks for RAG
#[derive(Debug, Clone)]
pub struct RagContext {
    pub chunks: Vec<RagChunk>,
    pub total_tokens: usize,
}

/// A single retrieved document chunk with metadata
#[derive(Debug, Clone)]
pub struct RagChunk {
    pub content: String,
    pub source: String,
    pub score: f32,
    pub file_id: String,
}

impl RagContext {
    /// Build a formatted context string for inclusion in the LLM prompt
    pub fn to_context_string(&self) -> String {
        if self.chunks.is_empty() {
            return String::new();
        }

        let mut context = String::new();
        context.push_str("# Knowledge Base Context\n\n");
        context.push_str("The following information has been retrieved from the project's knowledge base ");
        context.push_str("to help answer the user's question:\n\n");

        for (i, chunk) in self.chunks.iter().enumerate() {
            context.push_str(&format!(
                "## Document {} (relevance: {:.1}%)\n",
                i + 1,
                chunk.score * 100.0
            ));
            context.push_str(&format!("**Source:** {}\n\n", chunk.source));
            context.push_str(&chunk.content);
            context.push_str("\n\n---\n\n");
        }

        context
    }

    /// Get formatted citations for the response footer
    pub fn get_citations(&self) -> Vec<String> {
        self.chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| format!("[{}] {}", i + 1, chunk.source))
            .collect()
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// Builder for constructing RAG context from search results
pub struct RagContextBuilder {
    results: Vec<VectorSearchResult>,
    threshold: f32,
    max_tokens: usize,
}

impl RagContextBuilder {
    /// Create a new builder with threshold and token budget
    ///
    /// # Arguments
    /// * `threshold` - Minimum similarity score (0.0-1.0) for chunk inclusion
    /// * `max_tokens` - Maximum tokens to include (approximate)
    pub fn new(threshold: f32, max_tokens: usize) -> Self {
        Self {
            results: Vec::new(),
            threshold: threshold.clamp(0.0, 1.0),
            max_tokens,
        }
    }

    /// Add search results to be processed
    pub fn add_results(mut self, results: Vec<VectorSearchResult>) -> Self {
        self.results = results;
        self
    }

    /// Build the RAG context, filtering by threshold and respecting token budget
    pub fn build(self) -> RagContext {
        let mut chunks = Vec::new();
        let mut total_tokens = 0;

        for result in self.results {
            // Filter by threshold
            if result.score < self.threshold {
                tracing::debug!(
                    score = result.score,
                    threshold = self.threshold,
                    "Skipping chunk below threshold"
                );
                continue;
            }

            // Estimate tokens (rough: 1 token ≈ 4 characters)
            let estimated_tokens = result.content.len() / 4;

            if total_tokens + estimated_tokens > self.max_tokens {
                tracing::debug!(
                    current_tokens = total_tokens,
                    estimated_tokens = estimated_tokens,
                    max_tokens = self.max_tokens,
                    "Reached token budget, stopping"
                );
                break;
            }

            chunks.push(RagChunk {
                content: result.content.clone(),
                source: result
                    .filename
                    .clone()
                    .unwrap_or_else(|| "Unknown source".to_string()),
                score: result.score,
                file_id: result.file_id.to_string(),
            });

            total_tokens += estimated_tokens;
        }

        tracing::info!(
            chunks_count = chunks.len(),
            total_tokens = total_tokens,
            "Built RAG context"
        );

        RagContext {
            chunks,
            total_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_result(score: f32, content: &str) -> VectorSearchResult {
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("test.txt".to_string()),
            content: content.to_string(),
            score,
        }
    }

    #[test]
    fn test_threshold_filtering() {
        let results = vec![
            create_test_result(0.9, "High relevance"),
            create_test_result(0.5, "Low relevance"),
            create_test_result(0.8, "Medium-high relevance"),
        ];

        let context = RagContextBuilder::new(0.7, 10000)
            .add_results(results)
            .build();

        assert_eq!(context.chunks.len(), 2);
        assert_eq!(context.chunks[0].content, "High relevance");
        assert_eq!(context.chunks[1].content, "Medium-high relevance");
    }

    #[test]
    fn test_token_budget() {
        // Create results that would exceed budget
        let large_content = "a".repeat(5000); // ~1250 tokens
        let results = vec![
            create_test_result(0.9, &large_content),
            create_test_result(0.85, &large_content),
            create_test_result(0.8, &large_content),
        ];

        let context = RagContextBuilder::new(0.0, 2000).add_results(results).build();

        // Should only include first chunk due to token budget
        assert_eq!(context.chunks.len(), 1);
        assert!(context.total_tokens <= 2000);
    }

    #[test]
    fn test_citations() {
        let results = vec![
            create_test_result(0.9, "Content 1"),
            create_test_result(0.8, "Content 2"),
        ];

        let context = RagContextBuilder::new(0.0, 10000)
            .add_results(results)
            .build();

        let citations = context.get_citations();
        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0], "[1] test.txt");
        assert_eq!(citations[1], "[2] test.txt");
    }

    #[test]
    fn test_empty_context() {
        let results = vec![
            create_test_result(0.3, "Too low"),
            create_test_result(0.2, "Also too low"),
        ];

        let context = RagContextBuilder::new(0.7, 10000)
            .add_results(results)
            .build();

        assert!(context.is_empty());
        assert_eq!(context.to_context_string(), "");
    }

    #[test]
    fn test_context_string_format() {
        let results = vec![create_test_result(0.9, "Test content")];

        let context = RagContextBuilder::new(0.0, 10000)
            .add_results(results)
            .build();

        let context_str = context.to_context_string();
        assert!(context_str.contains("Knowledge Base Context"));
        assert!(context_str.contains("Document 1 (relevance: 90.0%)"));
        assert!(context_str.contains("**Source:** test.txt"));
        assert!(context_str.contains("Test content"));
    }
}
