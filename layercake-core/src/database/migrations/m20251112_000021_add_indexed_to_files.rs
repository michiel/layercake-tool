use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Add indexed column with default value true
        db.execute_unprepared(
            "ALTER TABLE files ADD COLUMN indexed BOOLEAN NOT NULL DEFAULT 1"
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // SQLite doesn't support DROP COLUMN in older versions
        // Create a new table without the indexed column
        db.execute_unprepared(
            r#"
            CREATE TABLE files_backup (
                id TEXT PRIMARY KEY NOT NULL,
                project_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                media_type TEXT NOT NULL,
                size_bytes BIGINT NOT NULL,
                blob BLOB NOT NULL,
                checksum TEXT NOT NULL,
                created_by INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            )
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            INSERT INTO files_backup
            SELECT id, project_id, filename, media_type, size_bytes, blob, checksum, created_by, created_at
            FROM files
            "#
        )
        .await?;

        db.execute_unprepared("DROP TABLE files").await?;

        db.execute_unprepared("ALTER TABLE files_backup RENAME TO files").await?;

        Ok(())
    }
}
