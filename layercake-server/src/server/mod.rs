pub mod app;
pub mod handlers;
pub mod middleware;

pub mod websocket;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum MigrateDirection {
    Up,
    Down,
    Fresh,
}

use anyhow::Result;
use layercake_core::database::{connection::*, migrations::Migrator};
use sea_orm_migration::prelude::*;
use tracing::{info, warn};

pub async fn start_server(port: u16, database_path: &str, cors_origin: Option<&str>) -> Result<()> {
    let database_url = get_database_url(Some(database_path));
    let db = establish_connection(&database_url).await?;

    // Run migrations
    Migrator::up(&db, None).await?;
    info!("Database migrations completed");

    // Reconcile any graph_data rows left in the transitional Processing state
    // by a previously interrupted execution so they are not silently stuck.
    match layercake_core::services::GraphDataService::new(db.clone())
        .reconcile_interrupted_processing()
        .await
    {
        Ok(0) => {}
        Ok(n) => info!("Reconciled {} interrupted graph execution(s) at startup", n),
        Err(e) => warn!("Failed to reconcile interrupted graph executions: {}", e),
    }

    let app = app::create_app(db, cors_origin).await?;

    // Log all HTTP routes dynamically
    log_routes(port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server running on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

fn log_routes(port: u16) {
    info!("API Endpoints:");
    info!("  /health                     - Health check");

    #[cfg(feature = "graphql")]
    {
        info!("  /graphql                    - GraphQL API & Playground");
        info!("  /ws/collaboration/:project_id - WebSocket collaboration endpoint");
    }

    let _ = port;
}

pub async fn migrate_database(database_path: &str, direction: MigrateDirection) -> Result<()> {
    let database_url = get_database_url(Some(database_path));
    let db = establish_connection(&database_url).await?;

    match direction {
        MigrateDirection::Up => {
            info!("Running migrations up");
            Migrator::up(&db, None).await?;
        }
        MigrateDirection::Down => {
            info!("Running migrations down");
            Migrator::down(&db, None).await?;
        }
        MigrateDirection::Fresh => {
            info!("Running fresh migrations (down then up)");
            Migrator::down(&db, None).await?;
            Migrator::up(&db, None).await?;
        }
    }

    info!("Database migration completed");
    Ok(())
}
