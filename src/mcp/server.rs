//! MCP server implementation using WebSocket

use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::mcp::handlers;
use sea_orm::DatabaseConnection;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use tracing::{info, error, debug};

/// MCP server that handles WebSocket connections
pub struct McpServer {
    db: DatabaseConnection,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Start the MCP server on the given address
    pub async fn run(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(&addr).await?;
        info!("MCP server listening on {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            debug!("New connection from {}", peer_addr);
            let db = self.db.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, db).await {
                    error!("Error handling connection from {}: {}", peer_addr, e);
                }
            });
        }

        Ok(())
    }
}

/// Handle a single WebSocket connection
async fn handle_connection(stream: TcpStream, db: DatabaseConnection) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        let msg = msg?;
        
        match msg {
            Message::Text(text) => {
                debug!("Received message: {}", text);
                
                // Parse JSON-RPC request
                match serde_json::from_str::<JsonRpcRequest>(&text) {
                    Ok(request) => {
                        // Handle the request
                        let response = handlers::handle_request(request, &db).await;
                        
                        // Send response
                        let response_text = serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#.to_string());
                        
                        debug!("Sending response: {}", response_text);
                        
                        if let Err(e) = ws_sender.send(Message::Text(response_text)).await {
                            error!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request: {}", e);
                        
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
                        
                        if let Err(e) = ws_sender.send(Message::Text(response_text)).await {
                            error!("Failed to send error response: {}", e);
                            break;
                        }
                    }
                }
            }
            Message::Binary(_) => {
                error!("Binary messages not supported");
            }
            Message::Close(_) => {
                info!("Connection closed by client");
                break;
            }
            Message::Ping(data) => {
                debug!("Received ping, sending pong");
                if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                    error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Message::Pong(_) => {
                debug!("Received pong");
            }
            Message::Frame(_) => {
                // Internal frame, ignore
            }
        }
    }

    info!("Connection closed");
    Ok(())
}