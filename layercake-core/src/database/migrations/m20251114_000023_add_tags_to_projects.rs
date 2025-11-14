use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Add tags column as JSON array (stored as TEXT in SQLite)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TABLE projects ADD COLUMN tags TEXT DEFAULT '[]'".to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // SQLite doesn't support DROP COLUMN directly, so we recreate the table
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE TABLE projects_backup (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
            .to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO projects_backup
            SELECT id, name, description, created_at, updated_at
            FROM projects
            "#
            .to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP TABLE projects".to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TABLE projects_backup RENAME TO projects".to_string(),
        ))
        .await?;

        Ok(())
    }
}
