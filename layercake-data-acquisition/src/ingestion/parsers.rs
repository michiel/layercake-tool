use async_trait::async_trait;
use serde_json::json;
use tokio::task;

use crate::config::DataAcquisitionConfig;
use crate::errors::Result;
use crate::ingestion::{DocumentChunk, DocumentParser, ParsedDocument};

#[derive(Default)]
pub struct TextParser;

#[async_trait]
impl DocumentParser for TextParser {
    async fn parse(
        &self,
        bytes: &[u8],
        filename: &str,
        media_type: &str,
        config: &DataAcquisitionConfig,
    ) -> Result<ParsedDocument> {
        let text = String::from_utf8_lossy(bytes).to_string();
        Ok(build_parsed_document(filename, media_type, &text, config))
    }
}

#[derive(Default)]
pub struct PdfParser;

#[async_trait]
impl DocumentParser for PdfParser {
    async fn parse(
        &self,
        bytes: &[u8],
        filename: &str,
        media_type: &str,
        config: &DataAcquisitionConfig,
    ) -> Result<ParsedDocument> {
        let pdf_bytes = bytes.to_vec();
        let text = task::spawn_blocking(move || pdf_extract::extract_text_from_mem(&pdf_bytes))
            .await
            .map_err(|err| crate::errors::DataAcquisitionError::Ingestion(err.into()))?
            .map_err(|err| crate::errors::DataAcquisitionError::Ingestion(err.into()))?;

        Ok(build_parsed_document(filename, media_type, &text, config))
    }
}

pub fn default_parsers() -> Vec<(String, Box<dyn DocumentParser>)> {
    vec![
        (
            "text/plain".to_string(),
            Box::new(TextParser::default()) as Box<dyn DocumentParser>,
        ),
        (
            "application/pdf".to_string(),
            Box::new(PdfParser::default()) as Box<dyn DocumentParser>,
        ),
    ]
}

fn build_parsed_document(
    filename: &str,
    media_type: &str,
    text: &str,
    config: &DataAcquisitionConfig,
) -> ParsedDocument {
    let mut chunks = Vec::new();
    for (idx, chunk) in text
        .as_bytes()
        .chunks(config.max_chunk_chars)
        .map(|slice| String::from_utf8_lossy(slice).to_string())
        .enumerate()
    {
        chunks.push(DocumentChunk {
            chunk_id: format!("{}::{}", filename, idx),
            text: chunk,
            metadata: json!({ "chunk_index": idx }),
        });
    }

    ParsedDocument {
        media_type: media_type.to_string(),
        filename: filename.to_string(),
        chunks,
        metadata: json!({ "source": filename }),
    }
}
