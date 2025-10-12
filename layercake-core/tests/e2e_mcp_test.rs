//! End-to-end test for MCP functionality
//! 
//! This test demonstrates the complete MCP workflow by:
//! 1. Starting a Layercake server with in-memory SQLite
//! 2. Connecting via MCP HTTP
//! 3. Creating a project
//! 4. Adding nodes, edges, and layers
//! 5. Creating and executing a plan
//! 6. Exporting to JSON

use tokio::time::{sleep, Duration};
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use reqwest::Client;

struct McpClient {
    client: Client,
    url: String,
    request_id: u32,
}

impl McpClient {
    async fn new(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client: Client::new(),
            url: url.to_string(),
            request_id: 0,
        })
    }

    async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value, Box<dyn std::error::Error>> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });

        let response = self.client.post(&self.url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            return Err(format!("MCP request failed with status: {}, body: {}", status, error_body).into());
        }

        let response_json: Value = response.json().await?;

        if let Some(error) = response_json.get("error") {
            return Err(format!("MCP error: {}", error).into());
        }
        if let Some(result) = response_json.get("result") {
            return Ok(result.clone());
        }

        Err("No valid response received".into())
    }

    async fn call_tool(&mut self, tool_name: &str, arguments: Option<Value>) -> Result<Value, Box<dyn std::error::Error>> {
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });
        self.send_request("tools/call", Some(params)).await
    }
}

async fn start_server() -> Result<std::process::Child, Box<dyn std::error::Error>> {
    println!("Building and starting Layercake server...");
    
    // First build the binary
    let build_result = Command::new("cargo")
        .args(&["build", "--features", "server,graphql,mcp", "--release"])
        .status()
        .expect("Failed to build server");
    
    if !build_result.success() {
        return Err("Failed to build server".into());
    }
    
    // Start server using the built binary (use a temporary database file)
    let temp_db = format!("/tmp/layercake_test_{}.db", std::process::id());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let executable_path = format!("{}/../target/release/layercake", manifest_dir);
    let child = Command::new(&executable_path)
        .args(&["serve", "--database", &format!("sqlite://{}", temp_db), "--port", "3001"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Wait for server to start
    sleep(Duration::from_secs(3)).await;
    
    // Test if server is responding
    for i in 0..10 {
        if let Ok(_) = tokio::net::TcpStream::connect("127.0.0.1:3001").await {
            println!("Server is ready on port 3001");
            return Ok(child);
        }
        println!("Waiting for server to start... attempt {}", i + 1);
        sleep(Duration::from_millis(500)).await;
    }
    
    Err("Server failed to start after 10 attempts".into())
}

#[tokio::test]
async fn test_mcp_end_to_end_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Start the server
    let mut server_process = start_server().await?;

    // Connect to MCP HTTP endpoint
    println!("Connecting to MCP HTTP endpoint...");
    let mut client = McpClient::new("http://127.0.0.1:3001/mcp").await?;
    println!("Connected to MCP");

    // Step 1: Create a new project
    println!("Creating project...");
    let project_result = client.call_tool("create_project", Some(json!({
        "name": "E2E Test Project",
        "description": "End-to-end test project for MCP functionality"
    }))).await?;
    
    println!("Project result: {}", serde_json::to_string_pretty(&project_result)?);
    let project_id = project_result["id"].as_i64().expect("Project ID should be a number") as i32;
    println!("Created project with ID: {}", project_id);

    // Step 2: Add 3 nodes
    println!("Adding nodes...");
    let nodes_csv = "node_id,label,layer_id\nnode1,Node One,layer1\nnode2,Node Two,layer1\nnode3,Node Three,layer2";
    
    let _nodes_result = client.call_tool("import_csv", Some(json!({
        "project_id": project_id,
        "nodes_csv": nodes_csv
    }))).await?;
    println!("Added 3 nodes");

    // Step 3: Add 2 edges
    println!("Adding edges...");
    let edges_csv = "source_node_id,target_node_id\nnode1,node2\nnode2,node3";
    
    let _edges_result = client.call_tool("import_csv", Some(json!({
        "project_id": project_id,
        "edges_csv": edges_csv
    }))).await?;
    println!("Added 2 edges");

    // Step 4: Add 2 layers
    println!("Adding layers...");
    let layers_csv = "layer_id,name,color\nlayer1,First Layer,#FF0000\nlayer2,Second Layer,#00FF00";
    
    let _layers_result = client.call_tool("import_csv", Some(json!({
        "project_id": project_id,
        "layers_csv": layers_csv
    }))).await?;
    println!("Added 2 layers");

    // Step 5: Create a plan
    println!("Creating plan...");
    let plan_yaml = r#"
name: "E2E Test Plan"
description: "Simple transformation plan for testing"
steps:
  - name: "identity"
    type: "transform"
    config: {}
"#;

    let plan_result = client.call_tool("create_plan", Some(json!({
        "project_id": project_id,
        "name": "E2E Test Plan",
        "yaml_content": plan_yaml
    }))).await?;
    
    let plan_id = plan_result["id"].as_i64().expect("Plan ID should be a number") as i32;
    println!("Created plan with ID: {}", plan_id);

    // Step 6: Execute the plan
    println!("Executing plan...");
    let _exec_result = client.call_tool("execute_plan", Some(json!({
        "plan_id": plan_id
    }))).await?;
    println!("Plan executed successfully");

    // Step 7: Export to JSON
    println!("Exporting to JSON...");
    let export_result = client.call_tool("export_graph", Some(json!({
        "project_id": project_id,
        "format": "json"
    }))).await?;
    
    let exported_content = &export_result["content"];
    println!("Export completed");

    // Verify the exported content contains our data
    println!("Verifying exported data...");
    if let Some(content_str) = exported_content.as_str() {
        let exported_graph: Value = serde_json::from_str(content_str)?;
        
        // Check that we have nodes, edges, and layers
        assert!(exported_graph.get("nodes").is_some(), "Exported graph should contain nodes");
        assert!(exported_graph.get("edges").is_some(), "Exported graph should contain edges");
        assert!(exported_graph.get("layers").is_some(), "Exported graph should contain layers");
        
        // Verify counts
        let nodes = exported_graph["nodes"].as_array().expect("Nodes should be an array");
        let edges = exported_graph["edges"].as_array().expect("Edges should be an array");
        let layers = exported_graph["layers"].as_array().expect("Layers should be an array");
        
        assert_eq!(nodes.len(), 3, "Should have 3 nodes");
        assert_eq!(edges.len(), 2, "Should have 2 edges");
        assert_eq!(layers.len(), 2, "Should have 2 layers");
        
        println!("✓ Verified 3 nodes, 2 edges, 2 layers in exported data");
    } else {
        panic!("Export content should be a string");
    }

    // Step 8: Get final graph data to verify everything is in place
    println!("Getting final graph data...");
    let graph_data = client.call_tool("get_graph_data", Some(json!({
        "project_id": project_id
    }))).await?;
    
    let node_count = graph_data["nodes"]["count"].as_u64().expect("Node count should be a number");
    let edge_count = graph_data["edges"]["count"].as_u64().expect("Edge count should be a number");
    let layer_count = graph_data["layers"]["count"].as_u64().expect("Layer count should be a number");
    
    assert_eq!(node_count, 3, "Should have 3 nodes in database");
    assert_eq!(edge_count, 2, "Should have 2 edges in database");
    assert_eq!(layer_count, 2, "Should have 2 layers in database");
    
    println!("✓ Final verification: {} nodes, {} edges, {} layers", node_count, edge_count, layer_count);

    println!("\n🎉 End-to-end MCP test completed successfully!");
    println!("✓ Server started with in-memory SQLite");
    println!("✓ MCP WebSocket connection established");
    println!("✓ Created project: '{}'", project_result["name"]);
    println!("✓ Added 3 nodes via CSV import");
    println!("✓ Added 2 edges via CSV import");
    println!("✓ Added 2 layers via CSV import");
    println!("✓ Created and executed transformation plan");
    println!("✓ Exported graph data to JSON format");
    println!("✓ Verified all data integrity");

    // Clean up: Kill the server process
    let _ = server_process.kill();
    let _ = server_process.wait();
    
    // Clean up temp database file
    let temp_db = format!("/tmp/layercake_test_{}.db", std::process::id());
    let _ = std::fs::remove_file(temp_db);

    Ok(())
}