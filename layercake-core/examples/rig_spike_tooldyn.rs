/// Test rig ToolDyn for dynamic tool registration
///
/// This demonstrates wrapping MCP-style dynamic tools for use with rig.
/// Run with: OPENAI_API_KEY=... cargo run --example rig_spike_tooldyn
use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::{Prompt, ToolDefinition};
use rig::providers::openai;
use rig::tool::{Tool, ToolDyn};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Example dynamic tool that mimics MCP tool behavior
/// In practice, this would wrap an actual MCP tool
struct DynamicMcpTool {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

impl DynamicMcpTool {
    fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    fn with_parameters(mut self, params: serde_json::Value) -> Self {
        self.parameters = params;
        self
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Dynamic tool error: {0}")]
struct DynamicToolError(String);

// Implement Tool trait for compile-time known tool
#[derive(Deserialize, Serialize)]
struct Calculator;

#[derive(Deserialize)]
struct AddArgs {
    x: i32,
    y: i32,
}

impl Tool for Calculator {
    const NAME: &'static str = "add";

    type Error = DynamicToolError;
    type Args = AddArgs;
    type Output = i32;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "add".to_string(),
            description: "Add two numbers".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "x": {"type": "number", "description": "First number"},
                    "y": {"type": "number", "description": "Second number"}
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("   🔨 Calculator tool called: add({}, {})", args.x, args.y);
        Ok(args.x + args.y)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rig ToolDyn Test ===\n");

    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set");
        println!("   Set OPENAI_API_KEY to run this test\n");
        return Ok(());
    }

    println!("🔧 Test: Static Tool (Tool trait)");
    let openai_client = openai::Client::from_env();

    // Test static tool with Tool trait
    let agent = openai_client
        .agent("gpt-4o-mini")
        .preamble("You are a calculator assistant")
        .tool(Calculator)
        .build();

    let response = agent.prompt("What is 15 + 27?").await?;
    println!("   Response: {}", response);
    println!("   ✅ Static Tool trait works!\n");

    // TODO: Test ToolDyn - requires implementing ToolDyn manually or using rmcp feature
    println!("🔧 TODO: ToolDyn Test");
    println!("   Options for dynamic tools:");
    println!("   1. Use `rmcp` feature flag (built-in MCP support)");
    println!("   2. Use dynamic_tools() with ToolSet for RAG-based tool selection");
    println!("   3. Implement ToolDyn trait manually for custom dynamic dispatch\n");

    println!("   ℹ️  For MCP integration, rig has `rmcp` feature flag");
    println!("   ℹ️  See rag_dynamic_tools.rs example for ToolSet approach");

    Ok(())
}
