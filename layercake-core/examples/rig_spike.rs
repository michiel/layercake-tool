/// Spike: Test rig-core basic functionality
///
/// This example validates:
/// - Basic rig client initialization
/// - Chat completion
/// - Streaming API
/// - Tool calling
/// - Error handling
///
/// Run with: cargo run --example rig_spike
///
/// Set OPENAI_API_KEY environment variable for testing

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rig Spike: Basic Integration Test ===\n");

    // Test 1: Basic completion (requires OPENAI_API_KEY)
    if std::env::var("OPENAI_API_KEY").is_ok() {
        test_basic_completion().await?;
        test_streaming().await?;
        test_tool_calling().await?;
    } else {
        println!("⚠️  OPENAI_API_KEY not set, skipping live tests");
        println!("   Set OPENAI_API_KEY to run integration tests\n");
    }

    // Test 2: API exploration
    test_api_structure().await?;

    println!("\n=== Spike Complete ===");
    Ok(())
}

async fn test_basic_completion() -> Result<()> {
    println!("📝 Test 1: Basic Completion");

    // This will help us understand the actual rig API
    use rig_core::prelude::*;

    let client = rig_core::providers::openai::Client::from_env();

    // Test model initialization
    let model = client.model("gpt-4o-mini").build();

    // Test basic prompt
    let response = model.prompt("What is 2+2? Answer with just the number.").await?;

    println!("   Response: {}", response);
    println!("   ✅ Basic completion works\n");

    Ok(())
}

async fn test_streaming() -> Result<()> {
    println!("📡 Test 2: Streaming");

    use rig_core::prelude::*;

    let client = rig_core::providers::openai::Client::from_env();
    let model = client.model("gpt-4o-mini").build();

    // Test streaming - we need to understand the streaming API
    println!("   Testing streaming API...");

    // Note: This will likely need adjustment based on actual rig API
    // We're discovering the API here
    let response = model.prompt("Count from 1 to 3").await?;
    println!("   Streamed response: {}", response);
    println!("   ⚠️  Need to investigate actual streaming API\n");

    Ok(())
}

async fn test_tool_calling() -> Result<()> {
    println!("🔧 Test 3: Tool Calling");

    use rig_core::prelude::*;
    use rig_core::tool::Tool;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    // Define a simple test tool
    #[derive(Deserialize, Serialize)]
    struct Calculator;

    #[derive(Deserialize)]
    struct AddArgs {
        x: i32,
        y: i32,
    }

    #[async_trait::async_trait]
    impl Tool for Calculator {
        const NAME: &'static str = "add";

        type Error = anyhow::Error;
        type Args = AddArgs;
        type Output = i32;

        async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
            rig_core::completion::ToolDefinition {
                name: "add".to_string(),
                description: "Add two numbers".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "x": {"type": "number"},
                        "y": {"type": "number"}
                    },
                    "required": ["x", "y"]
                }),
            }
        }

        async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
            println!("   🔨 Tool called: add({}, {})", args.x, args.y);
            Ok(args.x + args.y)
        }
    }

    let client = rig_core::providers::openai::Client::from_env();

    // Test agent with tools
    let agent = client
        .agent("gpt-4o-mini")
        .preamble("You are a calculator assistant")
        .tool(Calculator)
        .build();

    let response = agent.prompt("What is 15 + 27?").await?;
    println!("   Response: {}", response);
    println!("   ✅ Tool calling works\n");

    Ok(())
}

async fn test_api_structure() -> Result<()> {
    println!("🔍 Test 4: API Structure Exploration");

    println!("   Checking rig-core module structure...");

    // Document what we learn about the API
    println!("   - rig_core::prelude exports main types");
    println!("   - rig_core::providers::{openai, anthropic, gemini, ollama}");
    println!("   - rig_core::tool::Tool trait");
    println!("   - rig_core::completion types");

    // Key questions to answer:
    println!("\n   Key findings needed:");
    println!("   ❓ How does streaming work? (token-by-token or full response?)");
    println!("   ❓ Can Tool::NAME be dynamic? (likely not - const requirement)");
    println!("   ❓ How to handle multiple tool calls in one turn?");
    println!("   ❓ What error types does rig return?");
    println!("   ❓ How to customize request timeout?");

    println!("\n   ✅ API exploration notes recorded\n");

    Ok(())
}
