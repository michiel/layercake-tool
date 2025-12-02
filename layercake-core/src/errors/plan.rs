//! Plan and DAG execution error types
//!
//! This module provides structured error types for plan operations,
//! including DAG node management, execution, and validation.
//!
//! # Examples
//!
//! ```rust
//! use layercake::errors::PlanError;
//!
//! // Create a not found error
//! let err = PlanError::NotFound(42);
//!
//! // Create an execution error
//! let err = PlanError::ExecutionFailed {
//!     node: "transform-1".to_string(),
//!     reason: "Invalid configuration".to_string(),
//! };
//!
//! // Create a cycle detection error
//! let err = PlanError::CycleDetected("node-1 -> node-2 -> node-1".to_string());
//! ```

#![allow(dead_code)]

use thiserror::Error;

/// Plan and DAG execution errors
#[derive(Error, Debug)]
pub enum PlanError {
    /// Plan not found by ID
    #[error("Plan {0} not found")]
    NotFound(i32),

    /// DAG node not found
    #[error("DAG node '{0}' not found")]
    NodeNotFound(String),

    /// DAG edge not found
    #[error("DAG edge {from} -> {to} not found")]
    EdgeNotFound {
        /// Source node identifier
        from: String,
        /// Target node identifier
        to: String,
    },

    /// Invalid DAG configuration
    #[error("Invalid DAG configuration: {0}")]
    InvalidConfig(String),

    /// Execution failed at specific node
    #[error("Execution failed at node '{node}': {reason}")]
    ExecutionFailed {
        /// Node identifier where execution failed
        node: String,
        /// Reason for failure
        reason: String,
    },

    /// Plan is already executing
    #[error("Plan is already executing")]
    AlreadyExecuting,

    /// Plan execution was stopped
    #[error("Plan execution was stopped")]
    ExecutionStopped,

    /// Cycle detected in DAG
    #[error("Cycle detected in DAG: {0}")]
    CycleDetected(String),

    /// Missing required node configuration
    #[error("Missing required node configuration: {0}")]
    MissingConfiguration(String),

    /// Invalid node type
    #[error("Invalid node type: {0}")]
    InvalidNodeType(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Invalid YAML/JSON configuration
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    /// Node already exists in DAG
    #[error("Node '{0}' already exists in DAG")]
    NodeAlreadyExists(String),

    /// Edge already exists in DAG
    #[error("Edge {from} -> {to} already exists")]
    EdgeAlreadyExists {
        /// Source node identifier
        from: String,
        /// Target node identifier
        to: String,
    },

    /// Invalid dependency reference
    #[error("Invalid dependency reference: {0}")]
    InvalidDependency(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    Validation(String),

    /// Timeout during execution
    #[error("Execution timeout for node '{0}'")]
    ExecutionTimeout(String),

    /// Invalid node state
    #[error("Invalid node state: {0}")]
    InvalidState(String),
}

impl PlanError {
    /// Check if this is a client error (400-series)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            PlanError::InvalidConfig(_)
                | PlanError::CycleDetected(_)
                | PlanError::MissingConfiguration(_)
                | PlanError::InvalidNodeType(_)
                | PlanError::InvalidFormat(_)
                | PlanError::NodeAlreadyExists(_)
                | PlanError::EdgeAlreadyExists { .. }
                | PlanError::InvalidDependency(_)
                | PlanError::MissingField(_)
                | PlanError::Validation(_)
                | PlanError::InvalidState(_)
        )
    }

    /// Check if this is a not found error (404)
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            PlanError::NotFound(_) | PlanError::NodeNotFound(_) | PlanError::EdgeNotFound { .. }
        )
    }

    /// Check if this is an execution error
    pub fn is_execution_error(&self) -> bool {
        matches!(
            self,
            PlanError::ExecutionFailed { .. }
                | PlanError::AlreadyExecuting
                | PlanError::ExecutionStopped
                | PlanError::ExecutionTimeout(_)
        )
    }

    /// Get error code for GraphQL/API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            PlanError::NotFound(_)
            | PlanError::NodeNotFound(_)
            | PlanError::EdgeNotFound { .. } => "NOT_FOUND",
            PlanError::InvalidConfig(_)
            | PlanError::MissingConfiguration(_)
            | PlanError::InvalidNodeType(_)
            | PlanError::InvalidFormat(_)
            | PlanError::InvalidDependency(_)
            | PlanError::MissingField(_)
            | PlanError::Validation(_)
            | PlanError::InvalidState(_) => "VALIDATION_FAILED",
            PlanError::NodeAlreadyExists(_) | PlanError::EdgeAlreadyExists { .. } => "CONFLICT",
            PlanError::CycleDetected(_) => "CYCLE_DETECTED",
            PlanError::ExecutionFailed { .. }
            | PlanError::ExecutionStopped
            | PlanError::ExecutionTimeout(_) => "EXECUTION_FAILED",
            PlanError::AlreadyExecuting => "CONFLICT",
            PlanError::Database(_) => "DATABASE_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_not_found() {
        let err = PlanError::NotFound(42);
        assert_eq!(err.to_string(), "Plan 42 not found");
        assert!(err.is_not_found());
        assert_eq!(err.error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_node_not_found() {
        let err = PlanError::NodeNotFound("transform-1".to_string());
        assert_eq!(err.to_string(), "DAG node 'transform-1' not found");
        assert!(err.is_not_found());
        assert_eq!(err.error_code(), "NOT_FOUND");
    }

    #[test]
    fn test_execution_failed() {
        let err = PlanError::ExecutionFailed {
            node: "transform-1".to_string(),
            reason: "Invalid config".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Execution failed at node 'transform-1': Invalid config"
        );
        assert!(err.is_execution_error());
        assert_eq!(err.error_code(), "EXECUTION_FAILED");
    }

    #[test]
    fn test_cycle_detected() {
        let err = PlanError::CycleDetected("A -> B -> C -> A".to_string());
        assert_eq!(err.to_string(), "Cycle detected in DAG: A -> B -> C -> A");
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "CYCLE_DETECTED");
    }

    #[test]
    fn test_already_executing() {
        let err = PlanError::AlreadyExecuting;
        assert_eq!(err.to_string(), "Plan is already executing");
        assert!(err.is_execution_error());
        assert_eq!(err.error_code(), "CONFLICT");
    }

    #[test]
    fn test_invalid_config() {
        let err = PlanError::InvalidConfig("Missing required field".to_string());
        assert!(err.is_client_error());
        assert_eq!(err.error_code(), "VALIDATION_FAILED");
    }
}
