use axum::{
    extract::{ws::WebSocket, Query, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::server::app::AppState;
use super::{
    session::SessionManager,
    types::{ClientMessage, ServerMessage},
};

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub project_id: i32,
    #[serde(default)]
    pub token: Option<String>, // JWT token for authentication
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(app_state): State<AppState>,
) -> Response {
    // TODO: Validate JWT token here
    // For now, we'll skip authentication validation

    info!(
        "WebSocket connection request for project_id: {}",
        params.project_id
    );

    ws.on_upgrade(move |socket| handle_socket(socket, params.project_id, app_state.session_manager))
}

async fn handle_socket(socket: WebSocket, project_id: i32, session_manager: Arc<SessionManager>) {
    let (mut sender, mut receiver) = socket.split();

    // Create a channel for this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn a task to handle outgoing messages
    let tx_clone = tx.clone();
    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if let Err(e) = sender.send(axum::extract::ws::Message::Text(json)).await {
                        error!("Failed to send WebSocket message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                }
            }
        }
    });

    let mut user_id: Option<String> = None;
    let mut rate_limiter = RateLimiter::new(20, std::time::Duration::from_secs(1)); // 20 messages per second

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                // Apply rate limiting
                if !rate_limiter.allow() {
                    warn!("Rate limit exceeded for project {}", project_id);
                    let error_msg = ServerMessage::Error {
                        message: "Rate limit exceeded".to_string(),
                    };
                    if let Err(_) = tx.send(error_msg) {
                        break;
                    }
                    continue;
                }

                // Parse and handle client message
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        if let Err(e) = handle_client_message(
                            client_msg,
                            project_id,
                            &session_manager,
                            &tx,
                            &mut user_id,
                        ).await {
                            error!("Error handling client message: {}", e);
                            let error_msg = ServerMessage::Error {
                                message: format!("Error processing message: {}", e),
                            };
                            if let Err(_) = tx.send(error_msg) {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse client message: {}", e);
                        let error_msg = ServerMessage::Error {
                            message: "Invalid message format".to_string(),
                        };
                        if let Err(_) = tx.send(error_msg) {
                            break;
                        }
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                info!("WebSocket connection closed for project {}", project_id);
                break;
            }
            Ok(axum::extract::ws::Message::Ping(data)) => {
                // Respond to ping with pong - use tx channel instead of direct sender
                let pong_msg = ServerMessage::Error {
                    message: "pong".to_string(),
                };
                if let Err(_) = tx.send(pong_msg) {
                    break;
                }
            }
            Ok(_) => {
                // Ignore other message types
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up on disconnect
    if let Some(uid) = user_id {
        if let Err(e) = session_manager.leave_project_session(project_id, &uid) {
            error!("Error during cleanup: {}", e);
        }
        info!("User {} disconnected from project {}", uid, project_id);
    }

    // Cancel sender task
    sender_task.abort();
}

async fn handle_client_message(
    message: ClientMessage,
    project_id: i32,
    session_manager: &SessionManager,
    tx: &mpsc::UnboundedSender<ServerMessage>,
    current_user_id: &mut Option<String>,
) -> Result<(), String> {
    match message {
        ClientMessage::JoinSession { data } => {
            // Validate input
            if data.user_id.trim().is_empty() || data.user_name.trim().is_empty() {
                return Err("User ID and name cannot be empty".to_string());
            }

            // Join the project session
            session_manager.join_project_session(
                project_id,
                data.user_id.clone(),
                data.user_name,
                data.avatar_color,
                tx.clone(),
            )?;

            *current_user_id = Some(data.user_id.clone());

            // If joining a specific document, join that too
            if let Some(doc_id) = data.document_id {
                // For now, assume it's a canvas document type
                // TODO: Get document type from database or client
                session_manager.join_document(
                    project_id,
                    &data.user_id,
                    doc_id,
                    super::types::DocumentType::Canvas,
                )?;
            }

            info!("User {} joined project {}", data.user_id, project_id);
        }

        ClientMessage::CursorUpdate { data } => {
            if let Some(user_id) = current_user_id {
                // Validate cursor position values
                if !validate_cursor_position(&data.position) {
                    return Err("Invalid cursor position values".to_string());
                }

                session_manager.update_cursor_position(
                    project_id,
                    user_id,
                    &data.document_id,
                    data.position,
                    data.selected_node_id,
                )?;
            } else {
                return Err("Must join session before updating cursor".to_string());
            }
        }

        ClientMessage::SwitchDocument { data } => {
            if let Some(user_id) = current_user_id {
                session_manager.join_document(
                    project_id,
                    user_id,
                    data.document_id,
                    data.document_type,
                )?;
            } else {
                return Err("Must join session before switching documents".to_string());
            }
        }

        ClientMessage::LeaveSession { data: _ } => {
            if let Some(user_id) = current_user_id {
                session_manager.leave_project_session(project_id, user_id)?;
                *current_user_id = None;
            }
        }
    }

    Ok(())
}

fn validate_cursor_position(position: &super::types::CursorPosition) -> bool {
    use super::types::CursorPosition;

    match position {
        CursorPosition::Canvas { x, y, zoom } => {
            x.is_finite() && y.is_finite() && zoom.map_or(true, |z| z.is_finite() && z > 0.0)
        }
        CursorPosition::Spreadsheet { row, column, .. } => {
            *row >= 0 && *column >= 0
        }
        CursorPosition::ThreeD { x, y, z, rotation, scale, .. } => {
            x.is_finite() && y.is_finite() && z.is_finite()
                && rotation.map_or(true, |(rx, ry, rz)| rx.is_finite() && ry.is_finite() && rz.is_finite())
                && scale.map_or(true, |s| s.is_finite() && s > 0.0)
        }
        CursorPosition::Timeline { timestamp, track } => {
            *timestamp >= 0 && track.map_or(true, |t| t >= 0)
        }
        CursorPosition::CodeEditor { line, column, .. } => {
            *line >= 0 && *column >= 0
        }
    }
}

/// Simple rate limiter for WebSocket messages
struct RateLimiter {
    max_requests: u32,
    window_duration: std::time::Duration,
    requests: std::collections::VecDeque<std::time::Instant>,
}

impl RateLimiter {
    fn new(max_requests: u32, window_duration: std::time::Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            requests: std::collections::VecDeque::new(),
        }
    }

    fn allow(&mut self) -> bool {
        let now = std::time::Instant::now();

        // Remove old requests outside the window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window_duration {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we can allow this request
        if self.requests.len() < self.max_requests as usize {
            self.requests.push_back(now);
            true
        } else {
            false
        }
    }
}