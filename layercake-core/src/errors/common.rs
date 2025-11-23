//! Common error utilities and GraphQL conversion
//!
//! This module provides utilities for converting domain errors to GraphQL errors
//! with structured error codes and extensions.
//!
//! # Examples
//!
//! ```rust
//! use layercake::errors::GraphError;
//! use layercake::errors::common::ToGraphQLError;
//!
//! let err = GraphError::NotFound(42);
//! let graphql_err = err.to_graphql_error();
//! ```

#[cfg(feature = "graphql")]
use async_graphql::{Error as GraphQLError, ErrorExtensions};

use super::*;

/// Convert domain errors to GraphQL errors with error codes
#[cfg(feature = "graphql")]
pub trait ToGraphQLError {
    /// Convert to GraphQL error with structured extensions
    fn to_graphql_error(&self) -> GraphQLError;
}

#[cfg(feature = "graphql")]
impl ToGraphQLError for GraphError {
    fn to_graphql_error(&self) -> GraphQLError {
        let code = self.error_code();
        let message = self.to_string();

        GraphQLError::new(message).extend_with(|_, e| {
            e.set("code", code);

            // Add additional context based on error type
            match self {
                GraphError::NotFound(id) => {
                    e.set("graphId", *id);
                }
                GraphError::NodeNotFound(node) => {
                    e.set("nodeId", node);
                }
                GraphError::EdgeAlreadyExists { from, to } => {
                    e.set("from", from);
                    e.set("to", to);
                }
                GraphError::EdgeNotFound { from, to } => {
                    e.set("from", from);
                    e.set("to", to);
                }
                GraphError::LayerNotFound(layer) => {
                    e.set("layer", layer);
                }
                _ => {}
            }
        })
    }
}

#[cfg(feature = "graphql")]
impl ToGraphQLError for PlanError {
    fn to_graphql_error(&self) -> GraphQLError {
        let code = self.error_code();
        let message = self.to_string();

        GraphQLError::new(message).extend_with(|_, e| {
            e.set("code", code);

            // Add additional context based on error type
            match self {
                PlanError::NotFound(id) => {
                    e.set("planId", *id);
                }
                PlanError::NodeNotFound(node) => {
                    e.set("nodeId", node);
                }
                PlanError::EdgeNotFound { from, to } => {
                    e.set("from", from);
                    e.set("to", to);
                }
                PlanError::ExecutionFailed { node, reason } => {
                    e.set("nodeId", node);
                    e.set("reason", reason);
                }
                PlanError::EdgeAlreadyExists { from, to } => {
                    e.set("from", from);
                    e.set("to", to);
                }
                _ => {}
            }
        })
    }
}

#[cfg(feature = "graphql")]
impl ToGraphQLError for DataSetError {
    fn to_graphql_error(&self) -> GraphQLError {
        let code = self.error_code();
        let message = self.to_string();

        GraphQLError::new(message).extend_with(|_, e| {
            e.set("code", code);

            // Add additional context based on error type
            match self {
                DataSetError::NotFound(id) => {
                    e.set("dataSetId", *id);
                }
                DataSetError::UnsupportedFormat(format) => {
                    e.set("format", format);
                }
                DataSetError::FileNotFound(path) => {
                    e.set("path", path);
                }
                _ => {}
            }
        })
    }
}

#[cfg(feature = "graphql")]
impl ToGraphQLError for AuthError {
    fn to_graphql_error(&self) -> GraphQLError {
        let code = self.error_code();
        let message = self.to_string();

        GraphQLError::new(message).extend_with(|_, e| {
            e.set("code", code);

            // Add HTTP status code for auth errors
            e.set("statusCode", self.http_status_code());

            // Add additional context based on error type
            match self {
                AuthError::InvalidEmail(email) => {
                    e.set("email", email);
                }
                AuthError::InvalidUsername(username) => {
                    e.set("username", username);
                }
                AuthError::InvalidRole(role) => {
                    e.set("role", role);
                }
                _ => {}
            }
        })
    }
}

#[cfg(feature = "graphql")]
impl ToGraphQLError for ImportExportError {
    fn to_graphql_error(&self) -> GraphQLError {
        let code = self.error_code();
        let message = self.to_string();

        GraphQLError::new(message).extend_with(|_, e| {
            e.set("code", code);

            // Add additional context based on error type
            match self {
                ImportExportError::UnsupportedFormat(format) => {
                    e.set("format", format);
                }
                ImportExportError::TemplateNotFound(template) => {
                    e.set("template", template);
                }
                ImportExportError::FileNotFound(path) => {
                    e.set("path", path);
                }
                _ => {}
            }
        })
    }
}

/// Convert anyhow::Error to GraphQL error
#[cfg(feature = "graphql")]
pub fn anyhow_to_graphql(err: &anyhow::Error) -> GraphQLError {
    // Try to downcast to known error types
    if let Some(graph_err) = err.downcast_ref::<GraphError>() {
        return graph_err.to_graphql_error();
    }

    if let Some(plan_err) = err.downcast_ref::<PlanError>() {
        return plan_err.to_graphql_error();
    }

    if let Some(data_set_err) = err.downcast_ref::<DataSetError>() {
        return data_set_err.to_graphql_error();
    }

    if let Some(auth_err) = err.downcast_ref::<AuthError>() {
        return auth_err.to_graphql_error();
    }

    if let Some(import_export_err) = err.downcast_ref::<ImportExportError>() {
        return import_export_err.to_graphql_error();
    }

    // Fallback to generic error
    GraphQLError::new(err.to_string()).extend_with(|_, e| {
        e.set("code", "INTERNAL_ERROR");
    })
}

/// Extension trait for Result<T, E> to convert errors to GraphQL errors
#[cfg(feature = "graphql")]
pub trait ResultExt<T> {
    /// Convert error to GraphQL error
    fn to_graphql_result(self) -> Result<T, GraphQLError>;
}

#[cfg(feature = "graphql")]
impl<T> ResultExt<T> for Result<T, GraphError> {
    fn to_graphql_result(self) -> Result<T, GraphQLError> {
        self.map_err(|e| e.to_graphql_error())
    }
}

#[cfg(feature = "graphql")]
impl<T> ResultExt<T> for Result<T, PlanError> {
    fn to_graphql_result(self) -> Result<T, GraphQLError> {
        self.map_err(|e| e.to_graphql_error())
    }
}

#[cfg(feature = "graphql")]
impl<T> ResultExt<T> for Result<T, DataSetError> {
    fn to_graphql_result(self) -> Result<T, GraphQLError> {
        self.map_err(|e| e.to_graphql_error())
    }
}

#[cfg(feature = "graphql")]
impl<T> ResultExt<T> for Result<T, AuthError> {
    fn to_graphql_result(self) -> Result<T, GraphQLError> {
        self.map_err(|e| e.to_graphql_error())
    }
}

#[cfg(feature = "graphql")]
impl<T> ResultExt<T> for Result<T, ImportExportError> {
    fn to_graphql_result(self) -> Result<T, GraphQLError> {
        self.map_err(|e| e.to_graphql_error())
    }
}

#[cfg(feature = "graphql")]
impl<T> ResultExt<T> for Result<T, anyhow::Error> {
    fn to_graphql_result(self) -> Result<T, GraphQLError> {
        self.map_err(|e| anyhow_to_graphql(&e))
    }
}

#[cfg(all(test, feature = "graphql"))]
mod tests {
    use super::*;

    #[test]
    fn test_graph_error_to_graphql() {
        let err = GraphError::NotFound(42);
        let graphql_err = err.to_graphql_error();

        assert!(graphql_err.message.contains("Graph 42 not found"));
    }

    #[test]
    fn test_plan_error_to_graphql() {
        let err = PlanError::ExecutionFailed {
            node: "transform-1".to_string(),
            reason: "Invalid config".to_string(),
        };
        let graphql_err = err.to_graphql_error();

        assert!(graphql_err.message.contains("transform-1"));
        assert!(graphql_err.message.contains("Invalid config"));
    }

    #[test]
    fn test_auth_error_to_graphql() {
        let err = AuthError::InvalidCredentials;
        let graphql_err = err.to_graphql_error();

        assert!(graphql_err.message.contains("Invalid credentials"));
    }

    #[test]
    fn test_result_ext() {
        let result: Result<i32, GraphError> = Err(GraphError::NotFound(42));
        let graphql_result = result.to_graphql_result();

        assert!(graphql_result.is_err());
    }
}
