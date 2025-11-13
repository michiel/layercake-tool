use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Add origin column with default value 'file_upload'
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TABLE data_sets ADD COLUMN origin TEXT NOT NULL DEFAULT 'file_upload'"
                .to_string(),
        ))
        .await?;

        // Update existing rows based on characteristics:
        // - If file_size is 0 or very small, likely manual_edit
        // - Otherwise, file_upload (default already set)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE data_sets SET origin = 'manual_edit' WHERE file_size = 0".to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Note: SQLite doesn't support DROP COLUMN in older versions
        // This creates a new table without the origin column and copies data
        let db = manager.get_connection();

        // For SQLite, we need to recreate the table without the origin column
        // This is a simplified rollback - in production you might want to preserve more data
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE TABLE data_sets_backup (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                filename TEXT NOT NULL,
                blob BLOB NOT NULL,
                graph_json TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'processing',
                error_message TEXT,
                file_size BIGINT NOT NULL,
                processed_at TEXT,
                file_format TEXT NOT NULL DEFAULT 'csv',
                data_type TEXT NOT NULL DEFAULT 'nodes',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            )
            "#
            .to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            INSERT INTO data_sets_backup
            SELECT id, project_id, name, description, filename, blob, graph_json,
                   status, error_message, file_size, processed_at, file_format,
                   data_type, created_at, updated_at
            FROM data_sets
            "#
            .to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP TABLE data_sets".to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "ALTER TABLE data_sets_backup RENAME TO data_sets".to_string(),
        ))
        .await?;

        Ok(())
    }
}
