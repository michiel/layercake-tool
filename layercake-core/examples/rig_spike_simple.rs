/// Simplified rig spike - test basic functionality
///
/// Run with: OPENAI_API_KEY=... cargo run --example rig_spike_simple
use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rig Basic Test ===\n");

    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set");
        println!("   Set OPENAI_API_KEY to run this test\n");
        return Ok(());
    }

    // Test 1: Agent (simplest API)
    println!("🤖 Test: Agent with Prompt");
    let openai_client = openai::Client::from_env();
    let gpt4 = openai_client.agent("gpt-4o-mini").build();

    let response = gpt4
        .prompt("What is 2+2? Answer with just the number.")
        .await?;

    println!("   Response: {}", response);
    println!("   ✅ Works!\n");

    println!("=== Test Passed ===");
    Ok(())
}
