use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);

    // Configure connection pool settings optimised for SQLite
    // SQLite benefits plateau around 10-20 connections due to write serialisation
    opt.max_connections(20) // Reduced from 100 - optimal for SQLite
        .min_connections(5)
        .connect_timeout(Duration::from_secs(5)) // Reduced from 8s
        .acquire_timeout(Duration::from_secs(5)) // Reduced from 8s
        .idle_timeout(Duration::from_secs(300)) // 5 minutes
        .max_lifetime(Duration::from_secs(3600)) // 1 hour
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    Database::connect(opt).await
}

pub fn get_database_url(database_path: Option<&str>) -> String {
    match database_path {
        Some(path) if path == ":memory:" => "sqlite::memory:".to_string(),
        Some(path) => format!("sqlite://{}?mode=rwc", path),
        None => "sqlite://layercake.db?mode=rwc".to_string(),
    }
}
