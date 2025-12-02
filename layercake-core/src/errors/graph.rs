//! Graph-related error types
//!
//! This module provides structured error types for graph operations,
//! including node and edge management, validation, and cycles.
//!
//! # Examples
//!
//! ```rust
//! use layercake::errors::GraphError;
//!
//! // Create a not found error
//! let err = GraphError::NotFound(42);
//!
//! // Create a validation error
//! let err = GraphError::Validation("Invalid node configuration".to_string());
//!
//! // Create a cycle detection error
//! let err = GraphError::CycleDetected("A -> B -> C -> A".to_string());
//! ```

#![allow(dead_code)]

use thiserror::Error;

/// Graph-related errors
#[derive(Error, Debug)]
pub enum GraphError {
    /// Graph not found by ID
    #[error("Graph {0} not found")]
    NotFound(i32),

    /// Invalid node reference
    #[error("Invalid node reference: {0}")]
    InvalidNode(String),

    /// Invalid edge reference
    #[error("Invalid edge reference: {0}")]
    InvalidEdge(String),

    /// Cycle detected in graph
    #[error("Cycle detected in graph: {0}")]
    CycleDetected(String),

    /// Invalid layer reference
    #[error("Invalid layer: {0}")]
    InvalidLayer(String),

    /// Node already exists
    #[error("Node '{0}' already exists")]
    NodeAlreadyExists(String),

    /// Edge already exists
    #[error("Edge {from} -> {to} already exists")]
    EdgeAlreadyExists {
        /// Source node identifier
        from: String,
        /// Target node identifier
        to: String,
    },

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Validation failed
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Graph edit operation failed
    #[error("Graph edit failed: {0}")]
    EditFailed(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid graph structure
    #[error("Invalid graph structure: {0}")]
    InvalidStructure(String),

    /// Node not found by identifier
    #[error("Node '{0}' not found")]
    NodeNotFound(String),

    /// Edge not found
    #[error("Edge {from} -> {to} not found")]
    EdgeNotFound {
        /// Source node identifier
        from: String,
        /// Target node identifier
        to: String,
    },

    /// Layer not found
    #[error("Layer '{0}' not found")]
    LayerNotFound(String),

    /// Invalid import format
    #[error("Invalid import format: {0}")]
    InvalidImportFormat(String),

    /// Export operation failed
    #[error("Export failed: {0}")]
    ExportFailed(String),
}

impl GraphError {
    /// Check if this is a client error (400-series)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            GraphError::InvalidNode(_)
                | GraphError::InvalidEdge(_)
                | GraphError::InvalidLayer(_)
                | GraphError::Validation(_)
                | GraphError::MissingField(_)
                | GraphError::InvalidStructure(_)
                | GraphError::NodeAlreadyExists(_)
                | GraphError::EdgeAlreadyExists { .. }
                | GraphError::CycleDetected(_)
                | GraphError::InvalidImportFormat(_)
        )
    }

    /// Check if this is a not found error (404)
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            GraphError::NotFound(_)
                | GraphError::NodeNotFound(_)
                | GraphError::EdgeNotFound { .. }
                | GraphError::LayerNotFound(_)
        )
    }

    /// Get error code for GraphQL/API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            GraphError::NotFound(_)
            | GraphError::NodeNotFound(_)
            | GraphError::EdgeNotFound { .. }
            | GraphError::LayerNotFound(_) => "NOT_FOUND",
            GraphError::InvalidNode(_)
            | GraphError::InvalidEdge(_)
            | GraphError::InvalidLayer(_)
            | GraphError::Validation(_)
            | GraphError::MissingField(_)
            | GraphError::InvalidStructure(_)
            | GraphError::InvalidImportFormat(_) => "VALIDATION_FAILED",
            GraphError::NodeAlreadyExists(_) | GraphError::EdgeAlreadyExists { .. } => "CONFLICT",
            GraphError::CycleDetected(_) => "CYCLE_DETECTED",
            GraphError::Database(_) => "DATABASE_ERROR",
            GraphError::EditFailed(_) | GraphError::ExportFailed(_) => "OPERATION_FAILED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = GraphError::NotFound(42);
        assert_eq!(err.to_string(), "Graph 42 not found");
        assert!(err.is_not_found());
        assert_eq!(err.error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_node_already_exists() {
        let err = GraphError::NodeAlreadyExists("node-1".to_string());
        assert_eq!(err.to_string(), "Node 'node-1' already exists");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "CONFLICT");
    }

    #[test]
    fn test_edge_already_exists() {
        let err = GraphError::EdgeAlreadyExists {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        assert_eq!(err.to_string(), "Edge A -> B already exists");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "CONFLICT");
    }

    #[test]
    fn test_cycle_detected() {
        let err = GraphError::CycleDetected("A -> B -> C -> A".to_string());
        assert_eq!(err.to_string(), "Cycle detected in graph: A -> B -> C -> A");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "CYCLE_DETECTED");
    }

    #[test]
    fn test_validation_error() {
        let err = GraphError::Validation("Invalid configuration".to_string());
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "VALIDATION_FAILED");
    }
}
