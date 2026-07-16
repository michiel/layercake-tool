pub mod app;
pub mod handlers;
pub mod middleware;
pub mod static_assets;

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

pub async fn start_server(
    host: &str,
    port: u16,
    database_path: &str,
    cors_origin: Option<&str>,
    open_browser: bool,
) -> Result<()> {
    // Warn loudly before creating a brand-new database file, and always report
    // the absolute location. Running `serve --database layercake.db` from the
    // wrong directory otherwise silently creates a stray empty DB in the cwd
    // (and migrations then populate it), which looks like "my data vanished".
    if database_path != ":memory:" {
        let path = std::path::Path::new(database_path);
        let creating = !path.exists();
        let absolute = std::path::absolute(path)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| database_path.to_string());
        if creating {
            warn!(
                "Database file '{}' does not exist; creating a new empty database at {}. \
                 If you expected existing data, stop and re-run from the correct directory \
                 or pass an absolute --database path.",
                database_path, absolute
            );
        } else {
            info!("Using database at {}", absolute);
        }
    }

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

    // Report an absolute database path so tools that resolve it from a
    // different working directory (e.g. `layercake doctor --port N`) open the
    // right file. The file exists by now (migrations ran above), so
    // canonicalize succeeds; fall back to the raw value for `:memory:` or edge
    // cases.
    let absolute_database_path = if database_path == ":memory:" {
        database_path.to_string()
    } else {
        std::fs::canonicalize(database_path)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| database_path.to_string())
    };

    let app = app::create_app(db, cors_origin, absolute_database_path).await?;

    // Log all HTTP routes dynamically
    log_routes(port);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    // For the browser URL, present loopback for wildcard binds so the link is clickable.
    let display_host = if host == "0.0.0.0" || host == "::" {
        "127.0.0.1"
    } else {
        host
    };
    let url = format!("http://{}:{}", display_host, port);
    info!("Server running on {}", url);

    if open_browser {
        open_in_browser(&url);
    }

    axum::serve(listener, app).await?;

    Ok(())
}

/// Best-effort launch of the default browser at `url`. Failures are logged, not fatal.
fn open_in_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let cmd = ("xdg-open", vec![url]);
    #[cfg(target_os = "macos")]
    let cmd = ("open", vec![url]);
    #[cfg(target_os = "windows")]
    let cmd = ("cmd", vec!["/C", "start", "", url]);

    let (program, args) = cmd;
    match std::process::Command::new(program).args(&args).spawn() {
        Ok(_) => info!("Opened {} in the default browser", url),
        Err(e) => warn!("Could not open browser ({}): visit {} manually", e, url),
    }
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
