// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod server;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::{error, info};

use server::ServerHandle;

// Tauri command to get application info
#[tauri::command]
async fn get_app_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "name": "Layercake",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Interactive graph transformation and visualization tool"
    }))
}

// Tauri command to get server connection info
#[tauri::command]
async fn get_server_info(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let guard = state.server_handle.read().await;
    if let Some(handle) = guard.as_ref() {
        Ok(serde_json::json!({
            "port": handle.port,
            "secret": handle.secret,
            "url": format!("http://localhost:{}", handle.port)
        }))
    } else {
        Err("Server not started".to_string())
    }
}

// Tauri command to check backend server status
#[tauri::command]
async fn check_server_status(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let guard = state.server_handle.read().await;
    if let Some(handle) = guard.as_ref() {
        Ok(server::check_server_health(handle.port).await)
    } else {
        Ok(false)
    }
}

// State management for the desktop application
struct AppState {
    server_handle: Arc<RwLock<Option<ServerHandle>>>,
    database_path: Arc<RwLock<PathBuf>>,
    database_dir: Arc<RwLock<PathBuf>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for desktop application
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Layercake desktop application");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            get_server_info,
            check_server_status,
            commands::database::get_database_path,
            commands::database::get_database_info,
            commands::database::reinitialize_database,
            commands::database::show_database_location,
        ])
        .setup(|app| {
            info!("Tauri application setup starting");

            // Get app data directory
            let app_handle = app.handle();
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data directory: {}", e))?;

            // Create app data directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Failed to create app data directory: {}", e))?;

            // Set database path
            let database_dir = app_data_dir;
            let database_path = database_dir.join("layercake.db");
            let database_path_str = database_path.to_string_lossy().to_string();

            info!("Database path: {}", database_path_str);

            // Create app state
            let app_state = AppState {
                server_handle: Arc::new(RwLock::new(None)),
                database_path: Arc::new(RwLock::new(database_path.clone())),
                database_dir: Arc::new(RwLock::new(database_dir.clone())),
            };

            // Clone state for setup
            let server_handle = app_state.server_handle.clone();
            let db_path = database_path.clone();

            // Start embedded server with dynamic port allocation
            tauri::async_runtime::spawn(async move {
                match server::start_embedded_server(db_path.to_string_lossy().to_string()).await {
                    Ok(handle) => {
                        let port = handle.port;
                        info!(
                            "Embedded server started successfully on port {} with secret authentication",
                            port
                        );
                        let mut guard = server_handle.write().await;
                        *guard = Some(handle);
                    }
                    Err(e) => {
                        error!("Failed to start embedded server: {}", e);
                    }
                }
            });

            // Store state in app
            app.manage(app_state);

            // Set up window event handlers
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "Failed to get main window".to_string())?;

            // Clone server handle for cleanup
            let server_handle_cleanup = app_handle.state::<AppState>().server_handle.clone();

            // Handle window close event
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    info!("Application closing, shutting down server");

                    // Shutdown server gracefully
                    tauri::async_runtime::block_on(async {
                        let mut guard = server_handle_cleanup.write().await;
                        if let Some(handle) = guard.take() {
                            if let Err(e) = handle.shutdown().await {
                                error!("Error shutting down server: {}", e);
                            }
                        }
                    });
                }
            });

            info!("Tauri application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
