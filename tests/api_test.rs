//! API integration tests
//! 
//! Tests for REST API endpoints, GraphQL queries, and MCP tools

use anyhow::Result;
use axum::http::{StatusCode, HeaderName, HeaderValue};
use axum_test::TestServer;
use layercake::database::connection::setup_database;
use layercake::server::app::create_app;
use sea_orm::Database;
use serde_json::{json, Value};
use tempfile::NamedTempFile;

/// Create a test server with in-memory database
async fn setup_test_server() -> Result<TestServer> {
    let temp_file = NamedTempFile::new()?;
    let db_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());
    
    let db = Database::connect(&db_url).await?;
    setup_database(&db).await?;
    
    let app = create_app(db, Some("*")).await?;
    let server = TestServer::new(app)?;
    
    Ok(server)
}

#[tokio::test]
async fn test_health_endpoint() -> Result<()> {
    let server = setup_test_server().await?;
    
    let response = server.get("/health").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let body: Value = response.json();
    assert_eq!(body["service"], "layercake-server");
    assert_eq!(body["status"], "healthy");
    assert!(body["version"].is_string());
    
    Ok(())
}

#[tokio::test]
async fn test_projects_crud_api() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Test POST /api/v1/projects (create)
    let create_payload = json!({
        "name": "Test API Project",
        "description": "Created via API test"
    });
    
    let response = server
        .post("/api/v1/projects")
        .json(&create_payload)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let project: Value = response.json();
    let project_id = project["id"].as_i64().unwrap();
    assert_eq!(project["name"], "Test API Project");
    assert_eq!(project["description"], "Created via API test");
    
    // Test GET /api/v1/projects (list)
    let response = server.get("/api/v1/projects").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let projects: Vec<Value> = response.json();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["id"], project_id);
    
    // Test GET /api/v1/projects/{id} (get single)
    let response = server
        .get(&format!("/api/v1/projects/{}", project_id))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let fetched_project: Value = response.json();
    assert_eq!(fetched_project["id"], project_id);
    assert_eq!(fetched_project["name"], "Test API Project");
    
    // Test PUT /api/v1/projects/{id} (update)
    let update_payload = json!({
        "name": "Updated API Project",
        "description": "Updated via API test"
    });
    
    let response = server
        .put(&format!("/api/v1/projects/{}", project_id))
        .json(&update_payload)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let updated_project: Value = response.json();
    assert_eq!(updated_project["name"], "Updated API Project");
    assert_eq!(updated_project["description"], "Updated via API test");
    
    // Test DELETE /api/v1/projects/{id}
    let response = server
        .delete(&format!("/api/v1/projects/{}", project_id))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
    
    // Verify deletion
    let response = server
        .get(&format!("/api/v1/projects/{}", project_id))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    
    Ok(())
}

#[tokio::test]
async fn test_plans_api_with_json_content() -> Result<()> {
    let server = setup_test_server().await?;
    
    // First create a project
    let project_payload = json!({
        "name": "Plan Test Project",
        "description": "For testing plan APIs"
    });
    
    let response = server
        .post("/api/v1/projects")
        .json(&project_payload)
        .await;
    
    let project: Value = response.json();
    let project_id = project["id"].as_i64().unwrap();
    
    // Create a plan with JSON content
    let plan_json = json!({
        "meta": {
            "name": "API Test Plan",
            "description": "Created via API test"
        },
        "import": {
            "profiles": [
                {
                    "filename": "test_nodes.csv",
                    "filetype": "Nodes"
                }
            ]
        },
        "export": {
            "profiles": [
                {
                    "filename": "test_output.json",
                    "exporter": "JSON"
                }
            ]
        }
    });
    
    let plan_payload = json!({
        "name": "API Test Plan",
        "plan_content": serde_json::to_string_pretty(&plan_json)?,
        "dependencies": []
    });
    
    let response = server
        .post(&format!("/api/v1/projects/{}/plans", project_id))
        .json(&plan_payload)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let plan: Value = response.json();
    let plan_id = plan["id"].as_i64().unwrap();
    assert_eq!(plan["name"], "API Test Plan");
    assert_eq!(plan["plan_format"], "json");
    assert_eq!(plan["plan_schema_version"], "1.0.0");
    
    // Test plan content parsing
    let plan_content: Value = serde_json::from_str(plan["plan_content"].as_str().unwrap())?;
    assert_eq!(plan_content["meta"]["name"], "API Test Plan");
    assert_eq!(plan_content["import"]["profiles"][0]["filename"], "test_nodes.csv");
    
    // Test plan execution endpoint
    let response = server
        .post(&format!("/api/v1/projects/{}/plans/{}/execute", project_id, plan_id))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let execution_result: Value = response.json();
    assert!(execution_result["status"].is_string());
    assert!(execution_result["plan_id"].is_number());
    
    Ok(())
}

#[cfg(feature = "graphql")]
#[tokio::test]
async fn test_graphql_endpoint() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Test GraphQL query
    let query = json!({
        "query": r#"
            query {
                projects {
                    id
                    name
                    description
                }
            }
        "#
    });
    
    let response = server
        .post("/graphql")
        .json(&query)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let result: Value = response.json();
    assert!(result["data"].is_object());
    assert!(result["data"]["projects"].is_array());
    
    Ok(())
}

#[tokio::test]
async fn test_static_file_serving() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Test frontend HTML shell
    let response = server.get("/").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let html = response.text();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Layercake"));
    assert!(html.contains("window.LAYERCAKE_CONFIG"));
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let server = setup_test_server().await?;
    
    // Test 404 for non-existent project
    let response = server.get("/api/v1/projects/99999").await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    
    // Test invalid JSON payload
    let response = server
        .post("/api/v1/projects")
        .json(&json!({"invalid": "data"}))
        .await;
    
    // Should return a client error status
    assert!(response.status_code().is_client_error());
    
    Ok(())
}

#[tokio::test]
async fn test_cors_headers() -> Result<()> {
    let server = setup_test_server().await?;
    
    let response = server
        .get("/health")
        .add_header(
            HeaderName::from_static("origin"), 
            HeaderValue::from_static("http://localhost:3001")
        )
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    // CORS headers should be present
    let headers = response.headers();
    assert!(headers.get("access-control-allow-origin").is_some());
    
    Ok(())
}