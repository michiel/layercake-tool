//! Import and export error types
//!
//! This module provides structured error types for import and export operations,
//! including format handling, template rendering, and data transformation.
//!
//! # Examples
//!
//! ```rust
//! use layercake::errors::ImportExportError;
//!
//! // Create an import error
//! let err = ImportExportError::ImportFailed("Invalid CSV format".to_string());
//!
//! // Create an export error
//! let err = ImportExportError::ExportFailed("Template not found".to_string());
//!
//! // Create a format error
//! let err = ImportExportError::UnsupportedFormat("application/xml".to_string());
//! ```

#![allow(dead_code)]

use thiserror::Error;

/// Import and export operation errors
#[derive(Error, Debug)]
pub enum ImportExportError {
    /// Import operation failed
    #[error("Import failed: {0}")]
    ImportFailed(String),

    /// Export operation failed
    #[error("Export failed: {0}")]
    ExportFailed(String),

    /// Unsupported format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Parsing error
    #[error("Parsing error: {0}")]
    ParseError(String),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// CSV parsing/writing error
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    /// Template rendering failed
    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    /// Invalid graph data
    #[error("Invalid graph data: {0}")]
    InvalidGraphData(String),

    /// Missing required data
    #[error("Missing required data: {0}")]
    MissingData(String),

    /// Invalid file format
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    YamlError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Template not found
    #[error("Template '{0}' not found")]
    TemplateNotFound(String),

    /// Invalid template syntax
    #[error("Invalid template syntax: {0}")]
    InvalidTemplate(String),

    /// Data transformation error
    #[error("Data transformation failed: {0}")]
    TransformationFailed(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

impl ImportExportError {
    /// Check if this is a client error (400-series)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            ImportExportError::UnsupportedFormat(_)
                | ImportExportError::ParseError(_)
                | ImportExportError::InvalidGraphData(_)
                | ImportExportError::MissingData(_)
                | ImportExportError::InvalidFormat(_)
                | ImportExportError::YamlError(_)
                | ImportExportError::InvalidConfiguration(_)
                | ImportExportError::InvalidTemplate(_)
                | ImportExportError::ValidationFailed(_)
        )
    }

    /// Check if this is a not found error (404)
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            ImportExportError::FileNotFound(_) | ImportExportError::TemplateNotFound(_)
        )
    }

    /// Check if this is a server error (500-series)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            ImportExportError::ImportFailed(_)
                | ImportExportError::ExportFailed(_)
                | ImportExportError::TemplateError(_)
                | ImportExportError::TransformationFailed(_)
                | ImportExportError::Database(_)
                | ImportExportError::Io(_)
        )
    }

    /// Get error code for GraphQL/API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            ImportExportError::ImportFailed(_) => "IMPORT_FAILED",
            ImportExportError::ExportFailed(_) => "EXPORT_FAILED",
            ImportExportError::UnsupportedFormat(_) => "UNSUPPORTED_FORMAT",
            ImportExportError::ParseError(_) => "PARSE_ERROR",
            ImportExportError::SerializationError(_) => "SERIALIZATION_ERROR",
            ImportExportError::CsvError(_) => "CSV_ERROR",
            ImportExportError::TemplateError(_) => "TEMPLATE_ERROR",
            ImportExportError::InvalidGraphData(_) => "INVALID_GRAPH_DATA",
            ImportExportError::MissingData(_) => "MISSING_DATA",
            ImportExportError::InvalidFormat(_) => "INVALID_FORMAT",
            ImportExportError::FileNotFound(_) => "FILE_NOT_FOUND",
            ImportExportError::Io(_) => "IO_ERROR",
            ImportExportError::YamlError(_) => "YAML_ERROR",
            ImportExportError::InvalidConfiguration(_) => "INVALID_CONFIGURATION",
            ImportExportError::TemplateNotFound(_) => "TEMPLATE_NOT_FOUND",
            ImportExportError::InvalidTemplate(_) => "INVALID_TEMPLATE",
            ImportExportError::TransformationFailed(_) => "TRANSFORMATION_FAILED",
            ImportExportError::EncodingError(_) => "ENCODING_ERROR",
            ImportExportError::ValidationFailed(_) => "VALIDATION_FAILED",
            ImportExportError::Database(_) => "DATABASE_ERROR",
        }
    }
}

// Implement conversion from serde_yaml::Error
impl From<serde_yaml::Error> for ImportExportError {
    fn from(err: serde_yaml::Error) -> Self {
        ImportExportError::YamlError(err.to_string())
    }
}

// Implement conversion from handlebars::RenderError
#[cfg(feature = "graphql")]
impl From<handlebars::RenderError> for ImportExportError {
    fn from(err: handlebars::RenderError) -> Self {
        ImportExportError::TemplateError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_failed() {
        let err = ImportExportError::ImportFailed("Invalid format".to_string());
        assert_eq!(err.to_string(), "Import failed: Invalid format");
        assert!(err.is_server_error());
        assert_eq!(err.error_code(), "IMPORT_FAILED");
    }

    #[test]
    fn test_export_failed() {
        let err = ImportExportError::ExportFailed("Template error".to_string());
        assert_eq!(err.to_string(), "Export failed: Template error");
        assert!(err.is_server_error());
        assert_eq!(err.error_code(), "EXPORT_FAILED");
    }

    #[test]
    fn test_unsupported_format() {
        let err = ImportExportError::UnsupportedFormat("application/xml".to_string());
        assert_eq!(err.to_string(), "Unsupported format: application/xml");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "UNSUPPORTED_FORMAT");
    }

    #[test]
    fn test_template_not_found() {
        let err = ImportExportError::TemplateNotFound("custom-template".to_string());
        assert_eq!(err.to_string(), "Template 'custom-template' not found");
        assert!(err.is_not_found());
        assert_eq!(err.error_code(), "TEMPLATE_NOT_FOUND");
    }

    #[test]
    fn test_invalid_graph_data() {
        let err = ImportExportError::InvalidGraphData("Missing nodes".to_string());
        assert_eq!(err.to_string(), "Invalid graph data: Missing nodes");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "INVALID_GRAPH_DATA");
    }

    #[test]
    fn test_yaml_error() {
        let err = ImportExportError::YamlError("Invalid YAML syntax".to_string());
        assert_eq!(err.to_string(), "YAML error: Invalid YAML syntax");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "YAML_ERROR");
    }

    #[test]
    fn test_serialization_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = ImportExportError::from(json_err);
        // SerializationError from serde_json is neither client nor server error in our categorization
        // It's a parsing error that could be either depending on context
        assert_eq!(err.error_code(), "SERIALIZATION_ERROR");
    }
}
