use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::DatabaseConnection;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use anyhow::Result;

#[cfg(feature = "graphql")]
use std::sync::Arc;
#[cfg(feature = "graphql")]
use async_graphql::{Schema, Request, Response as GraphQLResponse};
#[cfg(feature = "graphql")]
use crate::graphql::{GraphQLContext, GraphQLSchema, queries::Query, mutations::Mutation, subscriptions::Subscription};
#[cfg(feature = "graphql")]
use crate::services::{ImportService, ExportService, GraphService};

use super::handlers::health;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)] // Reserved for future REST endpoints or middleware
    pub db: DatabaseConnection,
    #[cfg(feature = "graphql")]
    pub graphql_schema: GraphQLSchema,
}


pub async fn create_app(db: DatabaseConnection, cors_origin: Option<&str>) -> Result<Router> {
    #[cfg(feature = "graphql")]
    let graphql_schema = {
        let import_service = Arc::new(ImportService::new(db.clone()));
        let export_service = Arc::new(ExportService::new(db.clone()));
        let graph_service = Arc::new(GraphService::new(db.clone()));
        
        let graphql_context = GraphQLContext::new(
            db.clone(),
            import_service,
            export_service,
            graph_service,
        );
        
        Schema::build(
            Query,
            Mutation,
            Subscription,
        )
        .data(graphql_context)
        .finish()
    };

    let state = AppState {
        db: db.clone(),
        #[cfg(feature = "graphql")]
        graphql_schema,
    };

    let cors = match cors_origin {
        Some(origin) => CorsLayer::new()
            .allow_origin(origin.parse::<axum::http::HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(Any),
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    };

    let mut app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check));

    // Add GraphQL routes if feature is enabled
    #[cfg(feature = "graphql")]
    {
        app = app.route("/graphql", get(graphql_playground).post(graphql_handler));
    }

    // Add MCP routes if feature is enabled
    #[cfg(feature = "mcp")]
    {
        use crate::mcp::{LayercakeServerState, LayercakeAuth};
        use crate::mcp::server::LayercakeToolRegistry;
        use axum_mcp::server::{McpServer, McpServerConfig};
        use std::sync::Arc;
        
        // Create MCP server with axum-mcp
        let mcp_config = McpServerConfig::default()
            .with_metadata("layercake", serde_json::json!({
                "description": "Graph visualization and transformation MCP server",
                "version": env!("CARGO_PKG_VERSION")
            }));
            
        let mcp_state = LayercakeServerState {
            db: db.clone(),
            tools: LayercakeToolRegistry::new(db.clone()),
            resources: crate::mcp::resources::LayercakeResourceRegistry::new(db.clone()),
            prompts: crate::mcp::prompts::LayercakePromptRegistry::new(),
            auth: LayercakeAuth,
        };
        
        let mcp_server = Arc::new(McpServer::new(mcp_config, mcp_state));
        
        // Add MCP routes using axum-mcp
        app = app.merge(create_mcp_routes(mcp_server));
    }

    let app = app
        // Add middleware
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(state);

    Ok(app)
}


#[cfg(feature = "graphql")]
async fn graphql_handler(
    State(state): State<AppState>,
    Json(req): Json<Request>,
) -> Json<GraphQLResponse> {
    let response = state.graphql_schema.execute(req).await;
    Json(response)
}

#[cfg(feature = "graphql")]
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}


#[cfg(feature = "mcp")]
fn create_mcp_routes<S>(mcp_server: std::sync::Arc<axum_mcp::server::McpServer<S>>) -> axum::Router<AppState>
where
    S: axum_mcp::server::McpServerState + Clone + Send + Sync + 'static,
{
    use axum::routing::get;
    use tower_http::cors::CorsLayer;
    
    // Create CORS layer optimized for Claude Code
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::USER_AGENT,
            axum::http::HeaderName::from_static("mcp-session-id"),
            axum::http::HeaderName::from_static("last-event-id"),
        ])
        .allow_credentials(false);
    
    axum::Router::new()
        .route("/mcp", get(mcp_info_handler).post(mcp_request_handler).delete(mcp_session_cleanup_handler))
        .route("/mcp/sse", get(mcp_sse_handler))
        .layer(cors)
        .with_state(mcp_server)
}

#[cfg(feature = "mcp")]
async fn mcp_info_handler<S>(
    axum::extract::State(server): axum::extract::State<std::sync::Arc<axum_mcp::server::McpServer<S>>>,
    headers: axum::http::HeaderMap,
) -> impl axum::response::IntoResponse
where
    S: axum_mcp::server::McpServerState,
{
    use serde_json::json;
    use tracing::debug;
    
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    debug!("MCP info request from: {}", user_agent);
    
    let info = json!({
        "name": "Layercake MCP Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Graph visualization and transformation MCP server with Claude Code compatibility",
        "protocol": {
            "version": axum_mcp::protocol::MCP_PROTOCOL_VERSION,
            "capabilities": server.state().server_capabilities()
        },
        "transport": {
            "type": "http",
            "supports_streamable": true,
            "supports_sse": true,
            "supports_sessions": true
        },
        "endpoints": {
            "json_rpc": "/mcp",
            "sse": "/mcp/sse",
            "session_cleanup": "/mcp"
        },
        "claude_desktop_compatible": true,
        "supported_features": [
            "tools",
            "resources", 
            "prompts",
            "layercake_uri_scheme",
            "graph_analysis",
            "connectivity_analysis",
            "path_finding",
            "export_formats"
        ]
    });
    
    axum::response::Json(info)
}

#[cfg(feature = "mcp")]
async fn mcp_request_handler<S>(
    axum::extract::State(server): axum::extract::State<std::sync::Arc<axum_mcp::server::McpServer<S>>>,
    axum::extract::Json(request): axum::extract::Json<axum_mcp::protocol::JsonRpcRequest>,
) -> impl axum::response::IntoResponse
where
    S: axum_mcp::server::McpServerState,
{
    use axum_mcp::security::SecurityContext;
    
    // Handle the MCP request using axum-mcp with system security context
    let context = SecurityContext::system();
    let response = server.handle_request(request, context).await;
    axum::response::Json(response)
}

#[cfg(feature = "mcp")]
async fn mcp_sse_handler<S>(
    axum::extract::State(_server): axum::extract::State<std::sync::Arc<axum_mcp::server::McpServer<S>>>,
    headers: axum::http::HeaderMap,
) -> impl axum::response::IntoResponse
where
    S: axum_mcp::server::McpServerState,
{
    use axum::response::sse::{Event, Sse};
    use futures_util::stream::{self, StreamExt};
    use std::time::Duration;
    use tracing::{info, debug};
    
    // Check if this is Claude Desktop for enhanced compatibility
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if user_agent.contains("Claude") {
        info!("Claude Desktop client detected - using StreamableHTTP mode");
    }
    
    // Extract session information for Claude Desktop compatibility
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|h| h.to_str().ok());
    
    debug!("SSE connection - session_id: {:?}, last_event_id: {:?}", session_id, last_event_id);
    
    // Create enhanced SSE stream with Claude Desktop compatibility
    let stream = stream::repeat_with(move || {
        Event::default()
            .event("ping")
            .data("pong")
            .id("ping-1")
    })
    .take(1)
    .map(Ok::<_, axum::Error>);
    
    let sse = Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    );
    
    // Set additional headers for Claude Desktop compatibility
    let mut response = sse.into_response();
    let headers = response.headers_mut();
    headers.insert("Cache-Control", "no-cache".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    
    response
}

#[cfg(feature = "mcp")]
async fn mcp_session_cleanup_handler<S>(
    axum::extract::State(_server): axum::extract::State<std::sync::Arc<axum_mcp::server::McpServer<S>>>,
    headers: axum::http::HeaderMap,
) -> impl axum::response::IntoResponse
where
    S: axum_mcp::server::McpServerState,
{
    use axum::response::Json;
    use serde_json::json;
    use tracing::info;
    
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    info!("Cleaning up MCP session: {}", session_id);
    
    // Return success response for session cleanup
    Json(json!({
        "status": "success",
        "message": "Session cleaned up",
        "session_id": session_id
    }))
}