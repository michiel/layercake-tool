//! Data source error types
//!
//! This module provides structured error types for data source operations,
//! including file imports, format detection, and validation.
//!
//! # Examples
//!
//! ```rust
//! use layercake::errors::DataSourceError;
//!
//! // Create a not found error
//! let err = DataSourceError::NotFound(42);
//!
//! // Create an unsupported format error
//! let err = DataSourceError::UnsupportedFormat("application/xml".to_string());
//!
//! // Create a CSV parsing error
//! let err = DataSourceError::InvalidCsv("Missing header row".to_string());
//! ```

use thiserror::Error;

/// Data source operation errors
#[derive(Error, Debug)]
pub enum DataSourceError {
    /// Data source not found by ID
    #[error("Data source {0} not found")]
    NotFound(i32),

    /// Unsupported file format
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    /// Invalid CSV format or parsing error
    #[error("Invalid CSV: {0}")]
    InvalidCsv(String),

    /// Invalid JSON format or parsing error
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    /// Invalid spreadsheet format or parsing error
    #[error("Invalid spreadsheet: {0}")]
    InvalidSpreadsheet(String),

    /// Import operation failed
    #[error("Import failed: {0}")]
    ImportFailed(String),

    /// Export operation failed
    #[error("Export failed: {0}")]
    ExportFailed(String),

    /// File type detection failed
    #[error("File type detection failed")]
    DetectionFailed,

    /// Data source validation failed
    #[error("Data source validation failed: {0}")]
    ValidationFailed(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// CSV parsing error
    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid file path
    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Empty file or data source
    #[error("Empty data source: {0}")]
    EmptySource(String),

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// Data source already exists
    #[error("Data source '{0}' already exists")]
    AlreadyExists(String),
}

impl DataSourceError {
    /// Check if this is a client error (400-series)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            DataSourceError::UnsupportedFormat(_)
                | DataSourceError::InvalidCsv(_)
                | DataSourceError::InvalidJson(_)
                | DataSourceError::InvalidSpreadsheet(_)
                | DataSourceError::ValidationFailed(_)
                | DataSourceError::MissingField(_)
                | DataSourceError::InvalidPath(_)
                | DataSourceError::EmptySource(_)
                | DataSourceError::InvalidFormat(_)
                | DataSourceError::AlreadyExists(_)
        )
    }

    /// Check if this is a not found error (404)
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            DataSourceError::NotFound(_) | DataSourceError::FileNotFound(_)
        )
    }

    /// Check if this is a server error (500-series)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            DataSourceError::Database(_)
                | DataSourceError::Io(_)
                | DataSourceError::ImportFailed(_)
                | DataSourceError::ExportFailed(_)
                | DataSourceError::DetectionFailed
        )
    }

    /// Get error code for GraphQL/API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            DataSourceError::NotFound(_) | DataSourceError::FileNotFound(_) => "NOT_FOUND",
            DataSourceError::UnsupportedFormat(_)
            | DataSourceError::InvalidCsv(_)
            | DataSourceError::InvalidJson(_)
            | DataSourceError::InvalidSpreadsheet(_)
            | DataSourceError::ValidationFailed(_)
            | DataSourceError::MissingField(_)
            | DataSourceError::InvalidPath(_)
            | DataSourceError::EmptySource(_)
            | DataSourceError::InvalidFormat(_)
            | DataSourceError::EncodingError(_) => "VALIDATION_FAILED",
            DataSourceError::AlreadyExists(_) => "CONFLICT",
            DataSourceError::Database(_) => "DATABASE_ERROR",
            DataSourceError::Io(_) => "IO_ERROR",
            DataSourceError::CsvError(_) => "CSV_ERROR",
            DataSourceError::JsonError(_) => "JSON_ERROR",
            DataSourceError::ImportFailed(_)
            | DataSourceError::ExportFailed(_)
            | DataSourceError::DetectionFailed => "OPERATION_FAILED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = DataSourceError::NotFound(42);
        assert_eq!(err.to_string(), "Data source 42 not found");
        assert!(err.is_not_found());
        assert_eq!(err.error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_unsupported_format() {
        let err = DataSourceError::UnsupportedFormat("application/xml".to_string());
        assert_eq!(err.to_string(), "Unsupported file format: application/xml");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "VALIDATION_FAILED");
    }

    #[test]
    fn test_invalid_csv() {
        let err = DataSourceError::InvalidCsv("Missing header row".to_string());
        assert_eq!(err.to_string(), "Invalid CSV: Missing header row");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "VALIDATION_FAILED");
    }

    #[test]
    fn test_import_failed() {
        let err = DataSourceError::ImportFailed("Network timeout".to_string());
        assert_eq!(err.to_string(), "Import failed: Network timeout");
        assert!(err.is_server_error());
        assert_eq!(err.error_code(), "OPERATION_FAILED");
    }

    #[test]
    fn test_file_not_found() {
        let err = DataSourceError::FileNotFound("/path/to/file.csv".to_string());
        assert_eq!(err.to_string(), "File not found: /path/to/file.csv");
        assert!(err.is_not_found());
        assert_eq!(err.error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_already_exists() {
        let err = DataSourceError::AlreadyExists("data-source-1".to_string());
        assert_eq!(
            err.to_string(),
            "Data source 'data-source-1' already exists"
        );
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "CONFLICT");
    }
}
