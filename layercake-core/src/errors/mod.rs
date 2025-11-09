//! Domain-specific error types for layercake-core
//!
//! This module provides structured error types for different domains in the application,
//! making error handling more consistent, debuggable, and user-friendly.
//!
//! # Error Categories
//!
//! - **GraphError**: Graph operations (nodes, edges, layers, validation)
//! - **PlanError**: Plan and DAG execution errors
//! - **DataSetError**: Data source operations (import, export, validation)
//! - **AuthError**: Authentication and authorisation errors
//! - **ImportExportError**: Import/export format handling and transformation
//!
//! # GraphQL Integration
//!
//! When the `graphql` feature is enabled, error types can be converted to GraphQL errors
//! with structured error codes and extensions using the `ToGraphQLError` trait.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use layercake::errors::{GraphError, PlanError};
//!
//! // Create a graph error
//! let err = GraphError::NotFound(42);
//!
//! // Create a plan error
//! let err = PlanError::ExecutionFailed {
//!     node: "transform-1".to_string(),
//!     reason: "Invalid configuration".to_string(),
//! };
//!
//! // Check error categories
//! assert!(err.is_execution_error());
//! ```
//!
//! ## GraphQL Conversion (with graphql feature)
//!
//! ```rust,ignore
//! use layercake::errors::{GraphError, ToGraphQLError};
//!
//! let err = GraphError::NotFound(42);
//! let graphql_err = err.to_graphql_error();
//! ```
//!
//! ## Using in Functions
//!
//! ```rust
//! use layercake::errors::GraphError;
//!
//! fn find_node(id: &str) -> Result<String, GraphError> {
//!     if id.is_empty() {
//!         return Err(GraphError::InvalidNode("Node ID cannot be empty".to_string()));
//!     }
//!     Ok(id.to_string())
//! }
//! ```

pub mod auth;
pub mod common;
pub mod data_set;
pub mod graph;
pub mod import_export;
pub mod plan;

// Re-export all error types
pub use auth::AuthError;
pub use data_set::DataSetError;
pub use graph::GraphError;
pub use import_export::ImportExportError;
pub use plan::PlanError;

// Re-export common utilities
#[cfg(feature = "graphql")]
pub use common::{anyhow_to_graphql, ResultExt, ToGraphQLError};

/// Result type alias for graph operations
pub type GraphResult<T> = Result<T, GraphError>;

/// Result type alias for plan operations
pub type PlanResult<T> = Result<T, PlanError>;

/// Result type alias for data set operations
pub type DataSetResult<T> = Result<T, DataSetError>;

/// Result type alias for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Result type alias for import/export operations
pub type ImportExportResult<T> = Result<T, ImportExportError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_result_alias() {
        let result: GraphResult<i32> = Err(GraphError::NotFound(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_plan_result_alias() {
        let result: PlanResult<String> = Err(PlanError::NotFound(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_data_set_result_alias() {
        let result: DataSetResult<()> = Err(DataSetError::NotFound(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_result_alias() {
        let result: AuthResult<()> = Err(AuthError::InvalidCredentials);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_export_result_alias() {
        let result: ImportExportResult<()> =
            Err(ImportExportError::ImportFailed("Test".to_string()));
        assert!(result.is_err());
    }
}
