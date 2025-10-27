//! Common utilities shared across GraphQL and MCP APIs
//!
//! This module provides shared functionality for error handling, validation,
//! and other cross-cutting concerns used by both API surfaces.

pub mod db_errors;
pub mod error_helpers;

// Re-export commonly used items for convenience
pub use db_errors::{format_db_error, format_db_error_detailed, DbErrorKind};
pub use error_helpers::{
    auth, context_error, db_error_msg, not_found_msg, not_found_simple, service_error_msg,
    validation,
};
