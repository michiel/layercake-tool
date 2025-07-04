use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::DatabaseConnection;
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use anyhow::Result;

#[cfg(feature = "server")]
use utoipa::OpenApi;
#[cfg(feature = "server")]
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "graphql")]
use std::sync::Arc;
#[cfg(feature = "graphql")]
use async_graphql::{Schema, Request, Response as GraphQLResponse};
#[cfg(feature = "graphql")]
use crate::graphql::{GraphQLContext, GraphQLSchema, queries::Query, mutations::Mutation};
#[cfg(feature = "graphql")]
use crate::services::{ImportService, ExportService, GraphService};


use super::handlers::{health, projects, plans, graph_data};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    #[cfg(feature = "graphql")]
    pub graphql_schema: GraphQLSchema,
}

#[cfg(feature = "server")]
#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        projects::list_projects,
        projects::create_project,
        projects::get_project,
        projects::update_project,
        projects::delete_project,
    ),
    components(
        schemas(
            crate::database::entities::projects::Model,
            crate::server::handlers::projects::CreateProjectRequest,
            crate::server::handlers::projects::UpdateProjectRequest,
        )
    ),
    info(
        title = "Layercake API",
        version = "1.0.0",
        description = "Graph visualization and transformation tool API"
    ),
    servers(
        (url = "/", description = "Local server")
    )
)]
struct ApiDoc;

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
            async_graphql::EmptySubscription,
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
        .route("/health", get(health::health_check))
        
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        
        // OpenAPI documentation
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));

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

    // Add static file serving for frontend assets
    app = app
        .route("/", get(serve_frontend_html))
        .nest_service("/static", ServeDir::new("frontend/dist"))
        .fallback_service(ServeDir::new("frontend/dist"));

    let app = app
        // Add middleware
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(state);

    Ok(app)
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Project routes
        .route("/projects", get(projects::list_projects))
        .route("/projects", post(projects::create_project))
        .route("/projects/:id", get(projects::get_project))
        .route("/projects/:id", put(projects::update_project))
        .route("/projects/:id", delete(projects::delete_project))
        
        // Plan routes
        .route("/projects/:id/plans", get(plans::list_plans))
        .route("/projects/:id/plans", post(plans::create_plan))
        .route("/projects/:id/plans/:plan_id", get(plans::get_plan))
        .route("/projects/:id/plans/:plan_id", put(plans::update_plan))
        .route("/projects/:id/plans/:plan_id", delete(plans::delete_plan))
        .route("/projects/:id/plans/:plan_id/execute", post(plans::execute_plan))
        
        // Graph data routes
        .route("/projects/:id/nodes", get(graph_data::list_nodes))
        .route("/projects/:id/nodes", post(graph_data::create_nodes))
        .route("/projects/:id/nodes", delete(graph_data::delete_nodes))
        
        .route("/projects/:id/edges", get(graph_data::list_edges))
        .route("/projects/:id/edges", post(graph_data::create_edges))
        .route("/projects/:id/edges", delete(graph_data::delete_edges))
        
        .route("/projects/:id/layers", get(graph_data::list_layers))
        .route("/projects/:id/layers", post(graph_data::create_layers))
        .route("/projects/:id/layers", delete(graph_data::delete_layers))
        
        .route("/projects/:id/import/csv", post(graph_data::import_csv))
        .route("/projects/:id/export/:format", get(graph_data::export_graph))
}

#[cfg(feature = "graphql")]
async fn graphql_handler(
    State(state): State<AppState>,
    req: axum::extract::Json<Request>,
) -> axum::response::Json<GraphQLResponse> {
    let response = state.graphql_schema.execute(req.0).await;
    axum::response::Json(response)
}

#[cfg(feature = "graphql")]
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

/// Serve the main frontend HTML shell
async fn serve_frontend_html() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Layercake</title>
    <script>
        window.LAYERCAKE_CONFIG = {
            cdnBase: 'https://cdn.jsdelivr.net/gh/OWNER/REPO@main-build',
            fallback: '/static',
            apiBase: ''
        };
        
        async function loadApp() {
            try {
                const versionResp = await fetch(`${window.LAYERCAKE_CONFIG.cdnBase}/version.json`);
                const {version} = await versionResp.json();
                
                // Load CSS and JS with version-based cache busting
                loadAsset('link', `${window.LAYERCAKE_CONFIG.cdnBase}/style.css?v=${version}`);
                loadAsset('script', `${window.LAYERCAKE_CONFIG.cdnBase}/script.js?v=${version}`);
            } catch (error) {
                console.warn('CDN assets failed, falling back to local:', error);
                // Fallback to local assets
                loadAsset('link', '/static/style.css');
                loadAsset('script', '/static/script.js');
            }
        }
        
        function loadAsset(type, src) {
            if (type === 'link') {
                const link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = src;
                document.head.appendChild(link);
            } else if (type === 'script') {
                const script = document.createElement('script');
                script.src = src;
                script.defer = true;
                document.head.appendChild(script);
            }
        }
        
        document.addEventListener('DOMContentLoaded', loadApp);
    </script>
</head>
<body>
    <div id="root">
        <div style="padding: 20px; text-align: center; font-family: system-ui, sans-serif;">
            <h2>Loading Layercake...</h2>
            <p>Graph visualization and transformation tool</p>
            <div style="margin-top: 20px;">
                <div class="loading-spinner" style="
                    border: 3px solid #f3f3f3;
                    border-top: 3px solid #3498db;
                    border-radius: 50%;
                    width: 30px;
                    height: 30px;
                    animation: spin 1s linear infinite;
                    margin: 0 auto;
                "></div>
            </div>
        </div>
    </div>
    <style>
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
    </style>
</body>
</html>"#;
    
    axum::response::Html(html)
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