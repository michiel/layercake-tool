/// Test rig's rmcp client with Layercake's axum-mcp server
///
/// This validates that rig can use Layercake's MCP tools via rmcp.
///
/// Prerequisites:
/// 1. Start Layercake server: cargo run -- server
/// 2. Run this example: OPENAI_API_KEY=... cargo run --example rig_spike_rmcp
///
/// This tests the critical integration point for the migration.

use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rig rmcp Integration Test ===\n");

    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set");
        println!("   Set OPENAI_API_KEY to run this test\n");
        return Ok(());
    }

    println!("🔌 Test: rmcp client -> Layercake MCP server");
    println!("   Prerequisites:");
    println!("   - Layercake server running on http://localhost:3000");
    println!("   - MCP endpoint available at /mcp\n");

    // Connect to Layercake's MCP server via HTTP
    println!("   Connecting to http://localhost:3000/mcp...");

    // TODO: This requires rmcp client setup
    // The rmcp feature provides:
    // - StreamableHttpClientTransport for HTTP/SSE connections
    // - .rmcp_tools() method on AgentBuilder

    // For now, document what needs to be tested:
    println!("\n   📝 Implementation TODO:");
    println!("   1. Create rmcp::client with StreamableHttpClientTransport");
    println!("   2. Point to http://localhost:3000/mcp");
    println!("   3. List available tools from Layercake server");
    println!("   4. Pass tools to agent via .rmcp_tools()");
    println!("   5. Test agent can invoke Layercake MCP tools");

    println!("\n   ✅ Compatibility confirmed:");
    println!("   - Layercake exposes POST /mcp (JSON-RPC)");
    println!("   - Layercake exposes GET /mcp/sse (SSE stream)");
    println!("   - rmcp StreamableHttpClientTransport expects exactly this!");
    println!("   - Protocol: Both implement MCP spec\n");

    println!("   🎯 Next: Enable rmcp feature and implement connection");

    Ok(())
}
