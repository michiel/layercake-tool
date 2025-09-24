// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::{info};

// Tauri command to get application info
#[tauri::command]
async fn get_app_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "name": "Layercake",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Interactive graph transformation and visualization tool"
    }))
}

// Tauri command to check backend server status
#[tauri::command]
async fn check_server_status(url: String) -> Result<bool, String> {
    // Simple health check to backend server
    match reqwest::get(&format!("{}/health", url)).await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

// State management for the desktop application
#[derive(Default)]
struct AppState {
    server_url: Arc<RwLock<Option<String>>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for desktop application
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Layercake desktop application");

    let app_state = AppState::default();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            check_server_status
        ])
        .setup(|app| {
            info!("Tauri application setup complete");

            // Set up window event handlers
            let window = app.get_webview_window("main")
                .ok_or_else(|| "Failed to get main window".to_string())?;

            // Handle window close event
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    info!("Application closing");
                    // Perform any cleanup here if needed
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}