use anyhow::Result;
use rand::Rng;
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::info;

use layercake_core::database::connection::{establish_connection, get_database_url};
use layercake_core::database::migrations::Migrator;
use layercake_core::server::app::create_app;
use sea_orm_migration::prelude::*;

pub struct ServerHandle {
    pub port: u16,
    pub secret: String,
    pub handle: JoinHandle<Result<()>>,
}

impl ServerHandle {
    /// Gracefully shutdown the server
    pub async fn shutdown(self) -> Result<()> {
        info!("Shutting down embedded server");
        // The server will be dropped when the handle is aborted
        self.handle.abort();
        Ok(())
    }
}

/// Generate a cryptographically secure random secret for authentication
fn generate_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const SECRET_LEN: usize = 32;

    let mut rng = rand::thread_rng();
    (0..SECRET_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Start the embedded server on a background task with dynamic port allocation
/// Uses port 0 to let the OS pick an available port automatically (Tauri v2 best practice)
pub async fn start_embedded_server(database_path: String) -> Result<ServerHandle> {
    info!("Starting embedded server with dynamic port allocation");

    // Generate a shared secret for authentication
    let secret = generate_secret();
    info!("Generated server secret");

    // Establish database connection
    let database_url = get_database_url(Some(&database_path));
    let db = establish_connection(&database_url).await?;

    // Run migrations
    info!("Running database migrations");
    Migrator::up(&db, None).await?;
    info!("Database migrations completed");

    // Create the Axum app with CORS allowing tauri:// protocol
    let app = create_app(db, Some("tauri://localhost")).await?;

    // Use port 0 for dynamic allocation - OS will pick an available port
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));

    // Create a channel to send the actual port back
    let (port_tx, port_rx) = oneshot::channel();

    // Spawn the server on a background task
    let handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind server: {}", e))?;

        // Get the actual port that was assigned
        let actual_addr = listener
            .local_addr()
            .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;

        info!("Server successfully bound to {}", actual_addr);

        // Send the port back to the main task
        let _ = port_tx.send(actual_addr.port());

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    });

    // Wait for the actual port to be assigned
    let port = port_rx
        .await
        .map_err(|e| anyhow::anyhow!("Failed to receive port: {}", e))?;

    info!("Embedded server started on port {}", port);

    Ok(ServerHandle {
        port,
        secret,
        handle,
    })
}

/// Check if the server is healthy by making a health check request
pub async fn check_server_health(port: u16) -> bool {
    let url = format!("http://localhost:{}/health", port);
    match reqwest::get(&url).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
