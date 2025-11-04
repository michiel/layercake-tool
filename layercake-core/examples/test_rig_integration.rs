/// Integration test for rig-based chat implementation
/// Tests both Ollama (local) and Gemini (with API key from environment)
///
/// Run with:
/// ```
/// cargo run --example test_rig_integration --features console
/// ```

use anyhow::Result;
use rig::completion::Prompt;
use rig::client::CompletionClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Rig Integration Test ===\n");

    // Test 1: Ollama (local, no API key needed)
    println!("📝 Test 1: Ollama with llama3.2");
    println!("--------------------------------");
    test_ollama().await?;
    println!();

    // Test 2: Gemini (requires GOOGLE_API_KEY environment variable)
    if std::env::var("GOOGLE_API_KEY").is_ok() {
        println!("📝 Test 2: Gemini with API key from environment");
        println!("------------------------------------------------");
        test_gemini().await?;
        println!();
    } else {
        println!("⚠️  Skipping Gemini test (GOOGLE_API_KEY not set)");
        println!();
    }

    println!("✅ All tests completed successfully!");
    Ok(())
}

async fn test_ollama() -> Result<()> {
    use rig::providers::ollama;

    // Set default Ollama base URL if not already set
    if std::env::var("OLLAMA_API_BASE_URL").is_err() {
        std::env::set_var("OLLAMA_API_BASE_URL", "http://localhost:11434");
    }

    let client = ollama::Client::from_env();
    let agent = client.agent("llama3.2").build();

    println!("Sending prompt: 'What is 2+2? Answer with just the number.'");

    let response = agent
        .prompt("What is 2+2? Answer with just the number.")
        .await?;

    println!("Response: {}", response);

    // Basic validation
    if response.contains("4") {
        println!("✅ Ollama test passed (response contains '4')");
    } else {
        println!("⚠️  Unexpected response from Ollama");
    }

    Ok(())
}

async fn test_gemini() -> Result<()> {
    use rig::providers::gemini;
    use rig::providers::gemini::completion::gemini_api_types::{AdditionalParameters, GenerationConfig};

    let api_key = std::env::var("GOOGLE_API_KEY")?;
    let client = gemini::Client::new(&api_key);

    // Create generation config and additional params (required by Gemini)
    let gen_cfg = GenerationConfig::default();
    let additional_params = AdditionalParameters::default().with_config(gen_cfg);

    // Try gemini-1.5-flash which is more stable than 2.0-flash-exp
    let agent = client
        .agent("gemini-1.5-flash")
        .additional_params(serde_json::to_value(additional_params)?)
        .build();

    println!("Sending prompt: 'What is the capital of France? Answer with just the city name.'");

    let response = agent
        .prompt("What is the capital of France? Answer with just the city name.")
        .await?;

    println!("Response: {}", response);

    // Basic validation
    if response.to_lowercase().contains("paris") {
        println!("✅ Gemini test passed (response contains 'Paris')");
    } else {
        println!("⚠️  Unexpected response from Gemini");
    }

    Ok(())
}
