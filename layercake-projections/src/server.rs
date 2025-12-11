use axum::{extract::State, routing::{get, get_service}, Router};
use async_graphql::Schema;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse as AxumGraphQLResponse};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info, warn};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc;

use crate::graphql::{ProjectionsSchema, ProjectionMutation, ProjectionQuery, ProjectionSubscription};

#[derive(Clone)]
pub struct ProjectionsRouterState {
    pub projections_schema: ProjectionsSchema,
}

pub fn router(schema: ProjectionsSchema) -> Router<ProjectionsRouterState> {
    let static_assets = get_service(
        ServeDir::new("projections-frontend/dist")
            .fallback(ServeFile::new("projections-frontend/dist/index.html")),
    );

    Router::new()
        .route(
            "/graphql",
            get(graphql_playground)
                .post(graphql_handler)
                .options(|| async { axum::http::StatusCode::OK }),
        )
        .route("/graphql/ws", get(graphql_ws_handler))
        .nest_service("/", static_assets)
        .with_state(ProjectionsRouterState {
            projections_schema: schema,
        })
}

async fn graphql_handler(
    State(state): State<ProjectionsRouterState>,
    request: GraphQLRequest,
) -> AxumGraphQLResponse {
    let req = request.into_inner();
    let response = state.projections_schema.execute(req).await;
    AxumGraphQLResponse(response.into())
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")
            .subscription_endpoint("/graphql/ws"),
    ))
}

async fn graphql_ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    State(state): State<ProjectionsRouterState>,
) -> impl axum::response::IntoResponse {
    ws.protocols(["graphql-transport-ws", "graphql-ws"])
        .on_upgrade(move |socket| async move {
            handle_graphql_ws(socket, state.projections_schema).await;
        })
}

async fn handle_graphql_ws<Q, M, S>(
    socket: axum::extract::ws::WebSocket,
    schema: Schema<Q, M, S>,
) where
    Q: async_graphql::ObjectType + Send + Sync + 'static,
    M: async_graphql::ObjectType + Send + Sync + 'static,
    S: async_graphql::SubscriptionType + Send + Sync + 'static,
{
    info!("GraphQL WebSocket connection established");
    let (mut sink, mut stream) = socket.split();
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<axum::extract::ws::Message>();

    loop {
        tokio::select! {
            Some(Ok(msg)) = stream.next() => {
        debug!("WebSocket message received: {:?}", msg);
        if let axum::extract::ws::Message::Text(text) = msg {
            debug!("WebSocket text: {}", text);
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&text) {
                let msg_type = payload.get("type").and_then(|t| t.as_str()).unwrap_or("");
                info!("GraphQL WS message type: {}", msg_type);

                match msg_type {
                    "connection_init" => {
                        info!("GraphQL WS connection_init");
                        let ack = serde_json::json!({"type": "connection_ack"});
                        let _ = sink
                            .send(axum::extract::ws::Message::Text(ack.to_string().into()))
                            .await;
                    }
                    "ping" => {
                        let pong = serde_json::json!({"type": "pong"});
                        let _ = sink
                            .send(axum::extract::ws::Message::Text(pong.to_string().into()))
                            .await;
                    }
                    "subscribe" => {
                        info!("GraphQL WS subscribe: {:?}", payload);
                        if let Some(id) = payload.get("id").and_then(|i| i.as_str()) {
                            if let Some(query_payload) = payload.get("payload") {
                                if let Ok(request) = serde_json::from_value::<async_graphql::Request>(
                                    query_payload.clone(),
                                ) {
                                    let schema_clone = schema.clone();
                                    let msg_tx_clone = msg_tx.clone();
                                    let id_owned = id.to_string();

                                    tokio::spawn(async move {
                                        let mut response_stream = schema_clone.execute_stream(request);
                                        while let Some(response) = response_stream.next().await {
                                            let next_msg = serde_json::json!({
                                                "id": id_owned,
                                                "type": "next",
                                                "payload": response
                                            });
                                            if msg_tx_clone
                                                .send(axum::extract::ws::Message::Text(
                                                    next_msg.to_string().into(),
                                                ))
                                                .is_err()
                                            {
                                                warn!("Failed to send subscription response for id: {}", id_owned);
                                                return;
                                            }
                                        }

                                        let complete_msg =
                                            serde_json::json!({"id": id_owned, "type": "complete"});
                                        let _ = msg_tx_clone
                                            .send(axum::extract::ws::Message::Text(
                                                complete_msg.to_string().into(),
                                            ));
                                    });
                                }
                            }
                        }
                    }
                    other => {
                        warn!("Unknown GraphQL WS message type: {}, payload: {:?}", other, payload);
                    }
                }
            }
        }
            }
            Some(msg) = msg_rx.recv() => {
                if sink.send(msg).await.is_err() {
                    warn!("Failed to send message to WebSocket, connection closed");
                    break;
                }
            }
            else => {
                info!("WebSocket connection closed");
                break;
            }
        }
    }
}

pub fn build_schema(
    context: crate::graphql::ProjectionSchemaContext,
) -> ProjectionsSchema {
    Schema::build(
        ProjectionQuery::default(),
        ProjectionMutation,
        ProjectionSubscription,
    )
    .data(context)
    .finish()
}
