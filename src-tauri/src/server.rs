use anyhow::Result;
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tracing::info;

use layercake_core::database::connection::{get_database_url, establish_connection};
use layercake_core::database::migrations::Migrator;
use layercake_core::server::app::create_app;
use sea_orm_migration::prelude::*;

pub struct ServerHandle {
    pub port: u16,
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

/// Start the embedded server on a background task
pub async fn start_embedded_server(database_path: String, port: u16) -> Result<ServerHandle> {
    info!("Starting embedded server on port {}", port);

    // Establish database connection
    let database_url = get_database_url(Some(&database_path));
    let db = establish_connection(&database_url).await?;

    // Run migrations
    info!("Running database migrations");
    Migrator::up(&db, None).await?;
    info!("Database migrations completed");

    // Create the Axum app with CORS allowing tauri:// protocol
    let app = create_app(db, Some("tauri://localhost")).await?;

    // Spawn the server on a background task
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Embedded server listening on {}", addr);

    let handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind server: {}", e))?;

        info!("Server successfully bound to {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    });

    Ok(ServerHandle { port, handle })
}

/// Check if the server is healthy by making a health check request
pub async fn check_server_health(port: u16) -> bool {
    let url = format!("http://localhost:{}/health", port);
    match reqwest::get(&url).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
