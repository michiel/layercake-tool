/// Simplified rig spike - test basic functionality
///
/// Run with: OPENAI_API_KEY=... cargo run --example rig_spike_simple

use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rig Basic Test ===\n");

    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set");
        println!("   Set OPENAI_API_KEY to run this test\n");
        return Ok(());
    }

    // Test 1: Basic completion
    println!("📝 Test: Basic Completion");
    let client = providers::openai::Client::from_env();
    let model = client.completion_model("gpt-4o-mini");

    let response = model.prompt("What is 2+2? Answer with just the number.").await?;
    println!("   Response: {}", response);
    println!("   ✅ Works!\n");

    // Test 2: Agent
    println!("🤖 Test: Agent");
    let agent = client
        .agent("gpt-4o-mini")
        .preamble("You are a helpful assistant")
        .build();

    let response = agent.prompt("Say hello").await?;
    println!("   Response: {}", response);
    println!("   ✅ Works!\n");

    println!("=== All Tests Passed ===");
    Ok(())
}
