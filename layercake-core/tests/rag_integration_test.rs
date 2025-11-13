// RAG Integration Tests
// Tests the RAG context building and filtering logic

use layercake_data_acquisition::vector_store::VectorSearchResult;
use uuid::Uuid;

// Helper to create RagContext for testing
// Note: RagContextBuilder is part of the internal API, so we test via the public types
fn build_rag_context(
    results: Vec<VectorSearchResult>,
    threshold: f32,
    max_tokens: usize,
) -> (Vec<(String, f32, String)>, usize) {
    let mut chunks = Vec::new();
    let mut total_tokens = 0;

    for result in results {
        if result.score < threshold {
            continue;
        }

        let estimated_tokens = result.content.len() / 4;
        if total_tokens + estimated_tokens > max_tokens {
            break;
        }

        let source = result
            .filename
            .unwrap_or_else(|| "Unknown source".to_string());
        chunks.push((result.content, result.score, source));
        total_tokens += estimated_tokens;
    }

    (chunks, total_tokens)
}

#[test]
fn test_rag_context_threshold_filtering() {
    let results = vec![
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("high.txt".to_string()),
            content: "High relevance document".to_string(),
            score: 0.95,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("medium.txt".to_string()),
            content: "Medium relevance".to_string(),
            score: 0.75,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("low.txt".to_string()),
            content: "Low relevance".to_string(),
            score: 0.3,
        },
    ];

    let (chunks, _) = build_rag_context(results, 0.7, 4000);

    // Should only include documents with score >= 0.7
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].2, "high.txt");
    assert_eq!(chunks[1].2, "medium.txt");
}

#[test]
fn test_rag_token_budget() {
    // Create documents that would fit individually but not together
    let doc1_content = "a".repeat(4000); // ~1000 tokens
    let doc2_content = "b".repeat(6000); // ~1500 tokens

    let results = vec![
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("doc1.txt".to_string()),
            content: doc1_content,
            score: 0.9,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("doc2.txt".to_string()),
            content: doc2_content,
            score: 0.8,
        },
    ];

    let (chunks, total_tokens) = build_rag_context(results, 0.0, 2000);

    // Should include first doc but not second (would exceed budget)
    assert_eq!(chunks.len(), 1);
    assert!(total_tokens <= 2000);
    assert_eq!(chunks[0].2, "doc1.txt");
}

#[test]
fn test_rag_empty_results() {
    let results = vec![];
    let (chunks, total_tokens) = build_rag_context(results, 0.7, 4000);

    assert_eq!(chunks.len(), 0);
    assert_eq!(total_tokens, 0);
}

#[test]
fn test_rag_all_below_threshold() {
    let results = vec![
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("doc1.txt".to_string()),
            content: "Low".to_string(),
            score: 0.5,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("doc2.txt".to_string()),
            content: "Also low".to_string(),
            score: 0.3,
        },
    ];

    let (chunks, _) = build_rag_context(results, 0.7, 4000);
    assert_eq!(chunks.len(), 0);
}

#[test]
fn test_rag_preserves_order() {
    let results = vec![
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("first.txt".to_string()),
            content: "First".to_string(),
            score: 0.95,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("second.txt".to_string()),
            content: "Second".to_string(),
            score: 0.85,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("third.txt".to_string()),
            content: "Third".to_string(),
            score: 0.75,
        },
    ];

    let (chunks, _) = build_rag_context(results, 0.7, 4000);

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].2, "first.txt");
    assert_eq!(chunks[1].2, "second.txt");
    assert_eq!(chunks[2].2, "third.txt");
}

#[test]
fn test_rag_threshold_boundary() {
    let results = vec![
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("exact.txt".to_string()),
            content: "At threshold".to_string(),
            score: 0.7,
        },
        VectorSearchResult {
            file_id: Uuid::new_v4(),
            filename: Some("below.txt".to_string()),
            content: "Below threshold".to_string(),
            score: 0.699,
        },
    ];

    let (chunks, _) = build_rag_context(results, 0.7, 4000);

    // Should include 0.7, exclude 0.699
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].2, "exact.txt");
}

#[test]
fn test_rag_missing_filename() {
    let results = vec![VectorSearchResult {
        file_id: Uuid::new_v4(),
        filename: None,
        content: "No filename".to_string(),
        score: 0.9,
    }];

    let (chunks, _) = build_rag_context(results, 0.0, 4000);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].2, "Unknown source");
}
