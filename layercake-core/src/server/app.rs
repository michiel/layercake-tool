use anyhow::{anyhow, Result};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::DatabaseConnection;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[cfg(feature = "graphql")]
use crate::collaboration::{CollaborationCoordinator, CoordinatorHandle};
#[cfg(feature = "graphql")]
use crate::graphql::{
    mutations::Mutation, queries::Query, subscriptions::Subscription, GraphQLContext, GraphQLSchema,
};
#[cfg(feature = "graphql")]
use crate::server::websocket::websocket_handler;
#[cfg(feature = "graphql")]
use crate::services::{ExportService, GraphService, ImportService, PlanDagService};
#[cfg(feature = "graphql")]
use crate::{console::chat::ChatConfig, graphql::chat_manager::ChatManager};
#[cfg(feature = "graphql")]
use async_graphql::{
    parser::types::{DocumentOperations, OperationType, Selection},
    Request, Response as GraphQLResponse, Schema,
};
#[cfg(feature = "graphql")]
use std::sync::Arc;

use super::handlers::health;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)] // Reserved for future REST endpoints or middleware
    pub db: DatabaseConnection,
    #[cfg(feature = "graphql")]
    pub graphql_schema: GraphQLSchema,
    #[cfg(feature = "graphql")]
    pub coordinator_handle: CoordinatorHandle,
}

pub async fn create_app(db: DatabaseConnection, cors_origin: Option<&str>) -> Result<Router> {
    #[cfg(feature = "graphql")]
    let (graphql_schema, coordinator_handle) = {
        let import_service = Arc::new(ImportService::new(db.clone()));
        let export_service = Arc::new(ExportService::new(db.clone()));
        let graph_service = Arc::new(GraphService::new(db.clone()));
        let plan_dag_service = Arc::new(PlanDagService::new(db.clone()));

        // Initialize actor-based collaboration coordinator
        let coordinator_handle = CollaborationCoordinator::spawn();

        // Spawn background task to cleanup idle broadcast channels
        tokio::spawn(async move {
            use crate::graphql::subscriptions::{
                COLLABORATION_EVENTS, DELTA_EVENTS, EXECUTION_STATUS_EVENTS,
            };
            use std::time::Duration;

            let mut interval = tokio::time::interval(Duration::from_secs(60));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                let cleaned_collab = COLLABORATION_EVENTS.cleanup_idle().await;
                let cleaned_delta = DELTA_EVENTS.cleanup_idle().await;
                let cleaned_exec = EXECUTION_STATUS_EVENTS.cleanup_idle().await;
                let total_cleaned = cleaned_collab + cleaned_delta + cleaned_exec;

                if total_cleaned > 0 {
                    tracing::info!(
                        "Cleaned {} idle broadcast channels (collaboration: {}, delta: {}, execution: {})",
                        total_cleaned, cleaned_collab, cleaned_delta, cleaned_exec
                    );
                }

                // Log channel statistics for monitoring
                let collab_count = COLLABORATION_EVENTS.channel_count().await;
                let delta_count = DELTA_EVENTS.channel_count().await;
                let exec_count = EXECUTION_STATUS_EVENTS.channel_count().await;

                if collab_count + delta_count + exec_count > 0 {
                    tracing::debug!(
                        "Active broadcast channels - collaboration: {}, delta: {}, execution: {}",
                        collab_count,
                        delta_count,
                        exec_count
                    );
                }
            }
        });

        let chat_config: Arc<ChatConfig> = Arc::new(ChatConfig::load(&db).await?);
        let chat_manager = Arc::new(ChatManager::new());

        let graphql_context = GraphQLContext::new(
            db.clone(),
            import_service,
            export_service,
            graph_service,
            plan_dag_service,
            chat_config.clone(),
            chat_manager.clone(),
        );

        let schema = Schema::build(Query, Mutation, Subscription)
            .data(graphql_context)
            .finish();

        (schema, coordinator_handle)
    };

    let state = AppState {
        db: db.clone(),
        #[cfg(feature = "graphql")]
        graphql_schema,
        #[cfg(feature = "graphql")]
        coordinator_handle,
    };

    let cors = match cors_origin {
        Some(origin) => CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<axum::http::HeaderValue>()
                    .map_err(|e| anyhow!("Invalid CORS origin: {}", e))?,
            )
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers(Any)
            .allow_credentials(false),
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers(Any)
            .allow_credentials(false),
    };

    let mut app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check));

    // Add GraphQL routes if feature is enabled
    #[cfg(feature = "graphql")]
    {
        app = app
            .route(
                "/graphql",
                get(graphql_playground)
                    .post(graphql_handler)
                    .options(|| async { axum::http::StatusCode::OK }),
            )
            .route("/graphql/ws", get(graphql_ws_handler))
            .route("/ws/collaboration", get(websocket_handler));
    }

    // Add MCP routes if feature is enabled
    #[cfg(feature = "mcp")]
    {
        use crate::mcp::server::LayercakeToolRegistry;
        use crate::mcp::{LayercakeAuth, LayercakeServerState};
        use axum_mcp::server::{McpServer, McpServerConfig};
        use std::sync::Arc;

        // Create MCP server with axum-mcp
        let mcp_config = McpServerConfig::default().with_metadata(
            "layercake",
            serde_json::json!({
                "description": "Graph visualization and transformation MCP server",
                "version": env!("CARGO_PKG_VERSION")
            }),
        );

        let mcp_state = LayercakeServerState {
            db: db.clone(),
            tools: LayercakeToolRegistry::new(db.clone()),
            resources: crate::mcp::resources::LayercakeResourceRegistry::new(db.clone()),
            prompts: crate::mcp::prompts::LayercakePromptRegistry::new(),
            auth: LayercakeAuth::new(),
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
    tracing::debug!("GraphQL request received");
    let mut req = req;
    let mutation_log = capture_mutation_log_info(&mut req);
    let response = state.graphql_schema.execute(req).await;

    if let Some(info) = mutation_log {
        let has_errors = !response.errors.is_empty();
        let status = if has_errors { "ERROR" } else { "OK" };

        if has_errors {
            let error_summary = response
                .errors
                .iter()
                .map(|err| err.message.as_str())
                .collect::<Vec<_>>()
                .join(" | ");
            tracing::info!(
                target: "graphql::mutation",
                "mutation={} status={} params={} error={}",
                info.field_name,
                status,
                info.params_json,
                error_summary
            );
        } else {
            tracing::info!(
                target: "graphql::mutation",
                "mutation={} status={} params={}",
                info.field_name,
                status,
                info.params_json
            );
        }
    }

    tracing::debug!("GraphQL request completed");
    Json(response)
}

#[cfg(feature = "graphql")]
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

#[cfg(feature = "graphql")]
struct MutationLogInfo {
    field_name: String,
    params_json: String,
}

#[cfg(feature = "graphql")]
fn capture_mutation_log_info(req: &mut Request) -> Option<MutationLogInfo> {
    let operation_name_owned = req.operation_name.clone();
    let doc = req.parsed_query().ok()?;
    let operation_name = operation_name_owned.as_deref();

    let op = match &doc.operations {
        DocumentOperations::Single(operation) => operation,
        DocumentOperations::Multiple(map) => {
            let name = operation_name?;
            map.get(name)?
        }
    };

    if op.node.ty != OperationType::Mutation {
        return None;
    }

    let field_name =
        op.node
            .selection_set
            .node
            .items
            .iter()
            .find_map(|selection| match &selection.node {
                Selection::Field(field) => Some(field.node.name.node.to_string()),
                _ => None,
            })?;

    let params_json = serde_json::json!({
        "operationName": operation_name_owned,
        "variables": req.variables.clone(),
    })
    .to_string();

    Some(MutationLogInfo {
        field_name,
        params_json,
    })
}

#[cfg(feature = "graphql")]
async fn graphql_ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.protocols(["graphql-transport-ws", "graphql-ws"])
        .on_upgrade(move |socket| async move {
            handle_graphql_ws(socket, state.graphql_schema).await;
        })
}

#[cfg(feature = "graphql")]
async fn handle_graphql_ws(
    socket: axum::extract::ws::WebSocket,
    schema: crate::graphql::GraphQLSchema,
) {
    use futures_util::{SinkExt, StreamExt};

    tracing::info!("GraphQL WebSocket connection established");
    let (mut sink, mut stream) = socket.split();

    while let Some(Ok(msg)) = stream.next().await {
        tracing::debug!("WebSocket message received: {:?}", msg);
        if let axum::extract::ws::Message::Text(text) = msg {
            tracing::debug!("WebSocket text: {}", text);
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&text) {
                let msg_type = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
                tracing::info!("GraphQL WS message type: {}", msg_type);

                match msg_type {
                    "connection_init" => {
                        tracing::info!("GraphQL WS connection_init");
                        let ack = serde_json::json!({"type": "connection_ack"});
                        let _ = sink
                            .send(axum::extract::ws::Message::Text(ack.to_string().into()))
                            .await;
                    }
                    "subscribe" => {
                        tracing::info!("GraphQL WS subscribe: {:?}", payload);
                        if let Some(id) = payload.get("id").and_then(|i| i.as_str()) {
                            if let Some(query_payload) = payload.get("payload") {
                                if let Ok(request) = serde_json::from_value::<async_graphql::Request>(
                                    query_payload.clone(),
                                ) {
                                    let mut response_stream = schema.execute_stream(request);

                                    // Send subscription responses as they arrive
                                    while let Some(response) = response_stream.next().await {
                                        let next_msg = serde_json::json!({
                                            "id": id,
                                            "type": "next",
                                            "payload": response
                                        });
                                        if sink
                                            .send(axum::extract::ws::Message::Text(
                                                next_msg.to_string().into(),
                                            ))
                                            .await
                                            .is_err()
                                        {
                                            return;
                                        }
                                    }

                                    // Send complete when subscription ends
                                    let complete_msg =
                                        serde_json::json!({"id": id, "type": "complete"});
                                    let _ = sink
                                        .send(axum::extract::ws::Message::Text(
                                            complete_msg.to_string().into(),
                                        ))
                                        .await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(feature = "mcp")]
fn create_mcp_routes<S>(
    mcp_server: std::sync::Arc<axum_mcp::server::McpServer<S>>,
) -> axum::Router<AppState>
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
        .route(
            "/mcp",
            get(mcp_info_handler)
                .post(mcp_request_handler)
                .delete(mcp_session_cleanup_handler),
        )
        .route("/mcp/sse", get(mcp_sse_handler))
        .layer(cors)
        .with_state(mcp_server)
}

#[cfg(feature = "mcp")]
async fn mcp_info_handler<S>(
    axum::extract::State(server): axum::extract::State<
        std::sync::Arc<axum_mcp::server::McpServer<S>>,
    >,
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
    axum::extract::State(server): axum::extract::State<
        std::sync::Arc<axum_mcp::server::McpServer<S>>,
    >,
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
    axum::extract::State(_server): axum::extract::State<
        std::sync::Arc<axum_mcp::server::McpServer<S>>,
    >,
    headers: axum::http::HeaderMap,
) -> impl axum::response::IntoResponse
where
    S: axum_mcp::server::McpServerState,
{
    use axum::response::sse::{Event, Sse};
    use futures_util::stream::{self, StreamExt};
    use std::time::Duration;
    use tracing::{debug, info};

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

    let last_event_id = headers.get("last-event-id").and_then(|h| h.to_str().ok());

    debug!(
        "SSE connection - session_id: {:?}, last_event_id: {:?}",
        session_id, last_event_id
    );

    // Create enhanced SSE stream with Claude Desktop compatibility
    let stream =
        stream::repeat_with(move || Event::default().event("ping").data("pong").id("ping-1"))
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
    // These are static strings that should always parse correctly
    if let Ok(cache_control) = "no-cache".parse() {
        headers.insert("Cache-Control", cache_control);
    }
    if let Ok(connection) = "keep-alive".parse() {
        headers.insert("Connection", connection);
    }
    if let Ok(access_control) = "*".parse() {
        headers.insert("Access-Control-Allow-Origin", access_control);
    }

    response
}

#[cfg(feature = "mcp")]
async fn mcp_session_cleanup_handler<S>(
    axum::extract::State(_server): axum::extract::State<
        std::sync::Arc<axum_mcp::server::McpServer<S>>,
    >,
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
