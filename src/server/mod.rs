pub mod app;
pub mod handlers;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum MigrateDirection {
    Up,
    Down,
    Fresh,
}

use crate::database::{connection::*, migrations::Migrator};
use anyhow::Result;
use sea_orm_migration::prelude::*;
use tracing::info;

pub async fn start_server(port: u16, database_path: &str, cors_origin: Option<&str>) -> Result<()> {
    let database_url = get_database_url(Some(database_path));
    let db = establish_connection(&database_url).await?;
    
    // Run migrations
    Migrator::up(&db, None).await?;
    info!("Database migrations completed");

    let app = app::create_app(db, cors_origin).await?;
    
    // Log all HTTP routes
    log_routes();
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server running on http://0.0.0.0:{}", port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn log_routes() {
    info!("HTTP Routes:");
    info!("  GET    /health");
    info!("  GET    /docs                                   # Swagger UI");
    info!("  GET    /api-docs/openapi.json                 # OpenAPI spec");
    info!("  GET    /api/v1/projects");
    info!("  POST   /api/v1/projects");
    info!("  GET    /api/v1/projects/:id");
    info!("  PUT    /api/v1/projects/:id");
    info!("  DELETE /api/v1/projects/:id");
    info!("  GET    /api/v1/projects/:id/plans");
    info!("  POST   /api/v1/projects/:id/plans");
    info!("  GET    /api/v1/projects/:id/plans/:plan_id");
    info!("  PUT    /api/v1/projects/:id/plans/:plan_id");
    info!("  DELETE /api/v1/projects/:id/plans/:plan_id");
    info!("  POST   /api/v1/projects/:id/plans/:plan_id/execute");
    info!("  GET    /api/v1/projects/:id/nodes");
    info!("  POST   /api/v1/projects/:id/nodes");
    info!("  DELETE /api/v1/projects/:id/nodes");
    info!("  GET    /api/v1/projects/:id/edges");
    info!("  POST   /api/v1/projects/:id/edges");
    info!("  DELETE /api/v1/projects/:id/edges");
    info!("  GET    /api/v1/projects/:id/layers");
    info!("  POST   /api/v1/projects/:id/layers");
    info!("  DELETE /api/v1/projects/:id/layers");
    info!("  POST   /api/v1/projects/:id/import/csv");
    info!("  GET    /api/v1/projects/:id/export/:format");
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