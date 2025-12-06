/// Test rig streaming functionality
///
/// Run with: OPENAI_API_KEY=... cargo run --example rig_spike_streaming
use anyhow::Result;
use rig::agent::stream_to_stdout;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::openai;
use rig::streaming::StreamingPrompt;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rig Streaming Test ===\n");

    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set");
        println!("   Set OPENAI_API_KEY to run this test\n");
        return Ok(());
    }

    println!("🔄 Test: Streaming completion");
    let openai_client = openai::Client::from_env();
    let agent = openai_client.agent("gpt-4o-mini").build();

    // Test streaming using stream_prompt() + stream_to_stdout()
    println!("   Streaming response:");
    print!("   ");

    let mut stream = agent.stream_prompt("Count from 1 to 5").await;

    // Use rig's built-in stream_to_stdout helper
    let result = stream_to_stdout(&mut stream).await?;

    println!("\n");
    println!("   Token usage: {:?}", result.usage());
    println!("   Full response: {}", result.response());
    println!("   ✅ Streaming works!\n");

    Ok(())
}
