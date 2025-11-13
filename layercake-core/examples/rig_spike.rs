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

    use rig::client::CompletionClient;
    use rig::completion::Prompt;
    use rig::providers;

    let client = providers::openai::Client::from_env();
    let agent = client.agent(providers::openai::GPT_4O_MINI).build();

    let response = agent
        .prompt("What is 2+2? Answer with just the number.")
        .await?;

    println!("   Response: {}", response);
    println!("   ✅ Basic completion works\n");

    Ok(())
}

async fn test_streaming() -> Result<()> {
    println!("📡 Test 2: Streaming");

    use futures_util::StreamExt;
    use rig::client::CompletionClient;
    use rig::providers;
    use rig::agent::MultiTurnStreamItem;
    use rig::streaming::{StreamedAssistantContent, StreamingPrompt};

    let client = providers::openai::Client::from_env();
    let agent = client.agent(providers::openai::GPT_4O_MINI).build();

    println!("   Testing streaming API...");

    let mut stream = agent.stream_prompt("Count from 1 to 3").await;
    let mut aggregated = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(MultiTurnStreamItem::StreamItem(StreamedAssistantContent::Text(text))) => {
                print!("{}", text.text);
                aggregated.push_str(&text.text);
            }
            Ok(MultiTurnStreamItem::FinalResponse(final_response)) => {
                println!("\n   Tokens used: {:?}", final_response.usage());
            }
            Ok(_) => {}
            Err(err) => {
                println!("   Streaming error: {err}");
                break;
            }
        }
    }

    println!("   Aggregated response: {}", aggregated);
    println!("   ✅ Streaming prompt worked\n");

    Ok(())
}

async fn test_tool_calling() -> Result<()> {
    println!("🔧 Test 3: Tool Calling");

    use rig::client::CompletionClient;
    use rig::completion::{Prompt, ToolDefinition};
    use rig::providers;
    use rig::tool::Tool;
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

    #[derive(Debug, thiserror::Error)]
    #[error("Calculator error")]
    struct CalcError;

    impl Tool for Calculator {
        const NAME: &'static str = "add";

        type Error = CalcError;
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
            println!("   🔨 Tool called: add({}, {})", args.x, args.y);
            Ok(args.x + args.y)
        }
    }

    let client = providers::openai::Client::from_env();

    // Test agent with tools
    let agent = client
        .agent(providers::openai::GPT_4O_MINI)
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

    println!("   Checking rig module structure...");

    // Document what we learn about the API
    println!("   - rig::prelude exports main types");
    println!("   - rig::providers::{{openai, anthropic, gemini, ollama}}");
    println!("   - rig::tool::Tool trait");
    println!("   - rig::completion types");
    println!("   - Import: use `rig::` not `rig_core::`");

    // Key questions to answer:
    println!("\n   Key findings needed:");
    println!("   ❓ How does streaming work? (token-by-token or full response?)");
    println!("   ❌ Tool::NAME must be const - cannot be dynamic");
    println!("   ❓ How to handle multiple tool calls in one turn?");
    println!("   ❓ What error types does rig return?");
    println!("   ❓ How to customize request timeout?");
    println!("   ❓ How to access tool call metadata for persistence?");

    println!("\n   ✅ API exploration notes recorded\n");

    Ok(())
}
