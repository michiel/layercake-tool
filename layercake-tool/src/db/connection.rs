// src/db/connection.rs
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::path::Path;
use crate::db::migrate;

pub async fn establish_connection(db_path: &str) -> Result<DatabaseConnection, DbErr> {
    // Ensure directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DbErr::Custom(format!("Failed to create database directory: {}", e))
            })?;
        }
    }

    // Connect with SQLite
    let db_url = if db_path == ":memory:" {
        "sqlite::memory:".to_string()
    } else {
        format!("sqlite:{}?mode=rwc", db_path)
    };
    
    let conn = Database::connect(db_url).await?;
    
    // Run migrations
    migrate::run_migrations(&conn).await.map_err(|e| {
        DbErr::Custom(format!("Migration failed: {}", e))
    })?;
    
    Ok(conn)
}
