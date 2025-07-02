use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(database_url).await
}

pub fn get_database_url(database_path: Option<&str>) -> String {
    match database_path {
        Some(path) if path == ":memory:" => "sqlite::memory:".to_string(),
        Some(path) => format!("sqlite:{}", path),
        None => "sqlite:layercake.db".to_string(),
    }
}