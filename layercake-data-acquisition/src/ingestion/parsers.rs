use std::io::{Cursor, Read};

use async_trait::async_trait;
use calamine::{open_workbook_auto_from_rs, Reader};
use csv::ReaderBuilder;
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use serde_json::json;
use tokio::task;
use zip::read::ZipArchive;

use crate::config::DataAcquisitionConfig;
use crate::errors::{DataAcquisitionError, Result};
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
            Box::new(TextParser) as Box<dyn DocumentParser>,
        ),
        (
            "application/pdf".to_string(),
            Box::new(PdfParser) as Box<dyn DocumentParser>,
        ),
        (
            "text/markdown".to_string(),
            Box::new(MarkdownParser) as Box<dyn DocumentParser>,
        ),
        (
            "text/csv".to_string(),
            Box::new(CsvParser) as Box<dyn DocumentParser>,
        ),
        (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            Box::new(SpreadsheetParser) as Box<dyn DocumentParser>,
        ),
        (
            "application/vnd.oasis.opendocument.spreadsheet".to_string(),
            Box::new(SpreadsheetParser) as Box<dyn DocumentParser>,
        ),
        (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            Box::new(DocxParser) as Box<dyn DocumentParser>,
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

#[derive(Default)]
pub struct MarkdownParser;

#[async_trait]
impl DocumentParser for MarkdownParser {
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
pub struct CsvParser;

#[async_trait]
impl DocumentParser for CsvParser {
    async fn parse(
        &self,
        bytes: &[u8],
        filename: &str,
        media_type: &str,
        config: &DataAcquisitionConfig,
    ) -> Result<ParsedDocument> {
        let csv_bytes = bytes.to_vec();
        let text = task::spawn_blocking(move || parse_csv(csv_bytes))
            .await
            .map_err(|err| DataAcquisitionError::Ingestion(err.into()))??;

        Ok(build_parsed_document(filename, media_type, &text, config))
    }
}

#[derive(Default)]
pub struct SpreadsheetParser;

#[async_trait]
impl DocumentParser for SpreadsheetParser {
    async fn parse(
        &self,
        bytes: &[u8],
        filename: &str,
        media_type: &str,
        config: &DataAcquisitionConfig,
    ) -> Result<ParsedDocument> {
        let spreadsheet_bytes = bytes.to_vec();
        let text = task::spawn_blocking(move || parse_spreadsheet(spreadsheet_bytes))
            .await
            .map_err(|err| DataAcquisitionError::Ingestion(err.into()))??;

        Ok(build_parsed_document(filename, media_type, &text, config))
    }
}

#[derive(Default)]
pub struct DocxParser;

#[async_trait]
impl DocumentParser for DocxParser {
    async fn parse(
        &self,
        bytes: &[u8],
        filename: &str,
        media_type: &str,
        config: &DataAcquisitionConfig,
    ) -> Result<ParsedDocument> {
        let docx_bytes = bytes.to_vec();
        let text = task::spawn_blocking(move || parse_docx(docx_bytes))
            .await
            .map_err(|err| DataAcquisitionError::Ingestion(err.into()))??;

        Ok(build_parsed_document(filename, media_type, &text, config))
    }
}

fn parse_csv(bytes: Vec<u8>) -> Result<String> {
    let mut reader = ReaderBuilder::new().from_reader(bytes.as_slice());
    let mut buffer = String::new();

    if let Ok(headers) = reader
        .headers()
        .map_err(|err| DataAcquisitionError::Ingestion(err.into()))
    {
        buffer.push_str(&headers.iter().collect::<Vec<_>>().join(","));
        buffer.push('\n');
    }

    for record in reader.records() {
        let record = record.map_err(|err| DataAcquisitionError::Ingestion(err.into()))?;
        buffer.push_str(&record.iter().collect::<Vec<_>>().join(","));
        buffer.push('\n');
    }

    Ok(buffer)
}

fn parse_spreadsheet(bytes: Vec<u8>) -> Result<String> {
    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|err| DataAcquisitionError::Ingestion(err.into()))?;

    let mut output = String::new();
    for sheet in workbook.sheet_names().to_owned() {
        if let Ok(range) = workbook.worksheet_range(&sheet) {
            output.push_str(&format!("Sheet: {}\n", sheet));
            for row in range.rows() {
                let values: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
                output.push_str(&values.join(","));
                output.push('\n');
            }
            output.push('\n');
        }
    }

    if output.trim().is_empty() {
        Err(DataAcquisitionError::Ingestion(anyhow::anyhow!(
            "No readable cells found in spreadsheet"
        )))
    } else {
        Ok(output)
    }
}

fn parse_docx(bytes: Vec<u8>) -> Result<String> {
    let cursor = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(cursor).map_err(|err| DataAcquisitionError::Ingestion(err.into()))?;

    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|err| DataAcquisitionError::Ingestion(err.into()))?
        .read_to_string(&mut xml)
        .map_err(|err| DataAcquisitionError::Ingestion(err.into()))?;

    let mut reader = XmlReader::from_str(&xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut text = String::new();
    let mut last_event_was_paragraph = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                last_event_was_paragraph = true;
            }
            Ok(Event::Text(e)) => {
                let content = e
                    .unescape()
                    .map_err(|err| DataAcquisitionError::Ingestion(err.into()))?;
                if last_event_was_paragraph && content.trim().is_empty() {
                    // Ignore empty text nodes created by paragraph boundaries
                } else {
                    text.push_str(content.as_ref());
                    text.push(' ');
                }
                last_event_was_paragraph = false;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(err) => {
                return Err(DataAcquisitionError::Ingestion(err.into()));
            }
        }
    }

    Ok(text)
}
