use sea_orm::{Database, DatabaseConnection, DbErr};

pub struct TestDb {
    url: String,
}

impl TestDb {
    pub fn new_in_memory() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
        }
    }

    pub fn new_file(path: impl Into<String>) -> Self {
        Self { url: path.into() }
    }

    pub async fn connect(&self) -> Result<DatabaseConnection, DbErr> {
        Database::connect(&self.url).await
    }
}
