//! MCP (Model Context Protocol) implementation for Layercake
//! 
//! This module provides AI tool integration capabilities through the Model Context Protocol,
//! allowing AI assistants and other tools to interact with graph data.

#[cfg(feature = "mcp")]
pub mod server;
#[cfg(feature = "mcp")]
pub mod protocol;
#[cfg(feature = "mcp")]
pub mod tools;
#[cfg(feature = "mcp")]
pub mod resources;
#[cfg(feature = "mcp")]
pub mod prompts;
#[cfg(feature = "mcp")]
pub mod handlers;

#[cfg(feature = "mcp")]
pub use server::McpServer;
#[cfg(feature = "mcp")]
pub use protocol::McpProtocol;