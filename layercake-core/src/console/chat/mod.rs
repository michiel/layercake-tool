#![cfg(feature = "console")]

mod config;
mod mcp_bridge;
mod providers;
mod session;

pub use config::ChatConfig;
pub(crate) use mcp_bridge::McpBridge;
pub use providers::ChatProvider;
pub use session::{ChatEvent, ChatSession};
