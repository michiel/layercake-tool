use sea_orm::{Database, DatabaseConnection, DbErr};
use crate::database::migrations::Migrator;
use sea_orm_migration::prelude::*;

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(database_url).await
}

pub async fn setup_database(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}

pub fn get_database_url(database_path: Option<&str>) -> String {
    match database_path {
        Some(path) if path == ":memory:" => "sqlite::memory:".to_string(),
        Some(path) => format!("sqlite://{}?mode=rwc", path),
        None => "sqlite://layercake.db?mode=rwc".to_string(),
    }
}