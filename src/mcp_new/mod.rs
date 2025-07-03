//! MCP implementation using axum-mcp framework
//! 
//! This module provides the MCP server implementation for Layercake,
//! using the axum-mcp framework for standardized MCP protocol support.

#[cfg(feature = "mcp")]
pub mod server;
#[cfg(feature = "mcp")]
pub mod tools;

#[cfg(feature = "mcp")]
pub use server::{LayercakeServerState, LayercakeAuth};