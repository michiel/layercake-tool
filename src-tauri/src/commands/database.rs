use tauri::{AppHandle, State};
use tracing::{error, info};

use crate::AppState;

/// Get the database path
#[tauri::command]
pub async fn get_database_path(state: State<'_, AppState>) -> Result<String, String> {
    let path = state.database_path.read().await;
    Ok(path.to_string_lossy().to_string())
}

/// Get database information
#[tauri::command]
pub async fn get_database_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db_path = state.database_path.read().await;
    let db_dir = state.database_dir.read().await;

    // Get file size if file exists
    let size = match std::fs::metadata(&*db_path) {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };

    // Convert to MB for display
    let size_mb = size as f64 / 1_048_576.0;

    Ok(serde_json::json!({
        "path": db_path.to_string_lossy().to_string(),
        "directory": db_dir.to_string_lossy().to_string(),
        "size_bytes": size,
        "size_mb": format!("{:.2}", size_mb),
        "exists": db_path.exists(),
    }))
}

/// Reinitialize the database (drops all data and recreates tables)
#[tauri::command]
pub async fn reinitialize_database(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Reinitializing database");

    // Step 1: Shut down the server
    {
        let mut server_guard = state.server_handle.write().await;
        if let Some(handle) = server_guard.take() {
            info!("Shutting down server for database reinitialization");
            handle
                .shutdown()
                .await
                .map_err(|e| format!("Failed to shutdown server: {}", e))?;
        }
    }

    // Step 2: Delete the database file
    let db_path = state.database_path.read().await.clone();
    info!("Deleting database file: {}", db_path.to_string_lossy());

    if db_path.exists() {
        std::fs::remove_file(&db_path)
            .map_err(|e| format!("Failed to delete database file: {}", e))?;
    }

    // Step 3: Restart the server (which will recreate the database and run migrations)
    info!("Restarting server with fresh database");
    match crate::server::start_embedded_server(db_path.to_string_lossy().to_string()).await {
        Ok(handle) => {
            let port = handle.port;
            info!(
                "Server restarted successfully with fresh database on port {}",
                port
            );
            let mut server_guard = state.server_handle.write().await;
            *server_guard = Some(handle);
            Ok("Database reinitialized successfully".to_string())
        }
        Err(e) => {
            error!("Failed to restart server: {}", e);
            Err(format!("Failed to restart server: {}", e))
        }
    }
}

/// Show database location in file manager
/// Note: For now, this just returns the directory path
/// The frontend can display this or use it as needed
#[tauri::command]
pub async fn show_database_location(state: State<'_, AppState>) -> Result<String, String> {
    let dir = state.database_dir.read().await;
    Ok(dir.to_string_lossy().to_string())
}
