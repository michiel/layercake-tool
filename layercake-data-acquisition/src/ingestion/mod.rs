use async_trait::async_trait;
use serde_json::Value;
use tokio::io::AsyncRead;

use crate::config::DataAcquisitionConfig;
use crate::errors::{DataAcquisitionError, Result};

pub mod parsers;

/// Core document chunk metadata ready for embedding.
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub chunk_id: String,
    pub text: String,
    pub metadata: Value,
}

/// Result of parsing a file into chunks and enriched metadata.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub media_type: String,
    pub filename: String,
    pub chunks: Vec<DocumentChunk>,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub document: ParsedDocument,
    pub raw_bytes: Vec<u8>,
}

/// Trait implemented by each filetype parser.
#[async_trait]
pub trait DocumentParser: Send + Sync {
    async fn parse(
        &self,
        bytes: &[u8],
        filename: &str,
        media_type: &str,
        config: &DataAcquisitionConfig,
    ) -> Result<ParsedDocument>;
}

/// Thin wrapper responsible for dispatching bytes to the correct parser.
pub struct IngestionService {
    config: DataAcquisitionConfig,
    parsers: Vec<(String, Box<dyn DocumentParser>)>,
}

impl IngestionService {
    pub fn new(config: DataAcquisitionConfig) -> Self {
        Self {
            config,
            parsers: Vec::new(),
        }
    }

    pub fn with_parser<T>(mut self, media_type: &str, parser: T) -> Self
    where
        T: DocumentParser + 'static,
    {
        self.parsers
            .push((media_type.to_string(), Box::new(parser)));
        self
    }

    pub fn register_parser<T>(&mut self, media_type: &str, parser: T)
    where
        T: DocumentParser + 'static,
    {
        self.parsers
            .push((media_type.to_string(), Box::new(parser)));
    }

    pub fn register_boxed_parser(&mut self, media_type: String, parser: Box<dyn DocumentParser>) {
        self.parsers.push((media_type, parser));
    }

    pub async fn parse_stream<R>(
        &self,
        media_type: &str,
        filename: &str,
        mut reader: R,
    ) -> Result<ParsedFile>
    where
        R: AsyncRead + Unpin + Send,
    {
        use tokio::io::AsyncReadExt;

        let mut buf = Vec::new();
        reader
            .read_to_end(&mut buf)
            .await
            .map_err(|err| DataAcquisitionError::Ingestion(err.into()))?;
        let parser = self
            .parsers
            .iter()
            .find(|(mt, _)| mt == media_type)
            .map(|(_, parser)| parser.as_ref())
            .ok_or_else(|| DataAcquisitionError::UnsupportedMediaType(media_type.to_string()))?;

        let document = parser
            .parse(&buf, filename, media_type, &self.config)
            .await?;
        Ok(ParsedFile {
            document,
            raw_bytes: buf,
        })
    }

    pub async fn parse_bytes(
        &self,
        media_type: &str,
        filename: &str,
        bytes: &[u8],
    ) -> Result<ParsedDocument> {
        let parser = self
            .parsers
            .iter()
            .find(|(mt, _)| mt == media_type)
            .map(|(_, parser)| parser.as_ref())
            .ok_or_else(|| DataAcquisitionError::UnsupportedMediaType(media_type.to_string()))?;

        parser
            .parse(bytes, filename, media_type, &self.config)
            .await
    }
}
