use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
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

#[cfg(feature = "mcp")]
use crate::mcp::McpServer;
#[cfg(feature = "mcp")]
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
#[cfg(feature = "mcp")]
use axum::response::Response;

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
        db,
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
        app = app
            .route("/mcp", get(mcp_discovery))          // Discovery endpoint for clients
            .route("/mcp/ws", get(mcp_websocket_handler)) // WebSocket endpoint
            .route("/mcp/rpc", post(mcp_http_handler))    // HTTP JSON-RPC endpoint
            .route("/mcp/tools/list", get(mcp_tools_list))
            .route("/mcp/tools/call", post(mcp_tools_call));
    }

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

#[cfg(feature = "mcp")]
async fn mcp_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_mcp_socket(socket, state.db))
}

#[cfg(feature = "mcp")]
async fn handle_mcp_socket(socket: WebSocket, db: sea_orm::DatabaseConnection) {
    use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
    use crate::mcp::handlers;
    use futures_util::{SinkExt, StreamExt};
    use tracing::{debug, error};

    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                debug!("MCP received: {}", text);
                
                // Parse JSON-RPC request
                match serde_json::from_str::<JsonRpcRequest>(&text) {
                    Ok(request) => {
                        // Handle the request
                        let response = handlers::handle_request(request, &db).await;
                        
                        // Send response
                        let response_text = serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#.to_string());
                        
                        debug!("MCP sending: {}", response_text);
                        
                        if let Err(e) = sender.send(axum::extract::ws::Message::Text(response_text)).await {
                            error!("Failed to send MCP response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse MCP JSON-RPC request: {}", e);
                        
                        // Send error response
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None,
                            result: None,
                            error: Some(crate::mcp::protocol::JsonRpcError {
                                code: -32700,
                                message: "Parse error".to_string(),
                                data: Some(serde_json::Value::String(e.to_string())),
                            }),
                        };
                        
                        let response_text = serde_json::to_string(&error_response)
                            .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}"#.to_string());
                        
                        if let Err(e) = sender.send(axum::extract::ws::Message::Text(response_text)).await {
                            error!("Failed to send MCP error response: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                debug!("MCP WebSocket connection closed");
                break;
            }
            Ok(axum::extract::ws::Message::Ping(data)) => {
                debug!("MCP ping received, sending pong");
                if let Err(e) = sender.send(axum::extract::ws::Message::Pong(data)).await {
                    error!("Failed to send MCP pong: {}", e);
                    break;
                }
            }
            Ok(_) => {
                // Ignore other message types
            }
            Err(e) => {
                error!("MCP WebSocket error: {}", e);
                break;
            }
        }
    }

    debug!("MCP WebSocket connection ended");
}

#[cfg(feature = "mcp")]
async fn mcp_discovery(
    State(_state): State<AppState>,
) -> impl axum::response::IntoResponse {
    use serde_json::json;
    
    let discovery_info = json!({
        "name": "Layercake MCP Server",
        "version": "0.1.7",
        "description": "Graph visualization and transformation MCP server",
        "protocol": {
            "version": "2024-11-05",
            "transports": [
                {
                    "type": "http",
                    "uri": "/mcp/rpc",
                    "method": "POST"
                },
                {
                    "type": "websocket", 
                    "uri": "/mcp/ws"
                }
            ]
        },
        "capabilities": {
            "tools": {
                "listChanged": true
            },
            "resources": {
                "listChanged": true,
                "subscribe": false
            },
            "prompts": {
                "listChanged": true
            }
        },
        "endpoints": {
            "tools": "/mcp/tools/list",
            "call": "/mcp/tools/call",
            "rpc": "/mcp/rpc",
            "websocket": "/mcp/ws"
        }
    });
    
    axum::response::Json(discovery_info)
}

#[cfg(feature = "mcp")]
async fn mcp_http_handler(
    State(state): State<AppState>,
    axum::extract::Json(request): axum::extract::Json<crate::mcp::protocol::JsonRpcRequest>,
) -> impl axum::response::IntoResponse {
    use crate::mcp::handlers;
    
    let response = handlers::handle_request(request, &state.db).await;
    axum::response::Json(response)
}

#[cfg(feature = "mcp")]
async fn mcp_tools_list(
    State(_state): State<AppState>,
) -> impl axum::response::IntoResponse {
    use crate::mcp::tools;
    use serde_json::json;
    
    let mut all_tools = Vec::new();
    all_tools.extend(tools::projects::get_project_tools());
    all_tools.extend(tools::plans::get_plan_tools());
    all_tools.extend(tools::graph_data::get_graph_data_tools());
    
    let response = json!({
        "tools": all_tools
    });
    
    axum::response::Json(response)
}

#[cfg(feature = "mcp")]
async fn mcp_tools_call(
    State(state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    use crate::mcp::handlers;
    use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
    use serde_json::json;
    
    // Extract tool name and arguments from the payload
    let tool_name = payload.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    let arguments = payload.get("arguments").cloned();
    
    // Create a JSON-RPC request for the tool call
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": tool_name,
            "arguments": arguments
        })),
    };
    
    let response = handlers::handle_request(request, &state.db).await;
    axum::response::Json(response)
}