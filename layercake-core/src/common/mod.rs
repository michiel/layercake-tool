//! Common utilities shared across GraphQL and MCP APIs
//!
//! This module provides shared functionality for error handling, validation,
//! and other cross-cutting concerns used by both API surfaces.

pub mod db_errors;
pub mod error_helpers;
pub mod handlebars;

// Re-export commonly used items for convenience
pub use handlebars::{get_handlebars, write_string_to_file};
