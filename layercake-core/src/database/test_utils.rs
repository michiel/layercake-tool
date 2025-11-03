#[cfg(test)]
use sea_orm::{Database, DatabaseConnection};

#[cfg(test)]
pub async fn setup_test_db() -> DatabaseConnection {
    // Create an in-memory SQLite database for testing
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    use sea_orm_migration::MigratorTrait;
    crate::database::migrations::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    db
}
