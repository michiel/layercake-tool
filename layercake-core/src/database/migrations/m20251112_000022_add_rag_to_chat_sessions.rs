use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Add RAG configuration columns with sensible defaults
        db.execute_unprepared(
            "ALTER TABLE chat_sessions ADD COLUMN enable_rag BOOLEAN NOT NULL DEFAULT 1",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE chat_sessions ADD COLUMN rag_top_k INTEGER NOT NULL DEFAULT 5",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE chat_sessions ADD COLUMN rag_threshold REAL NOT NULL DEFAULT 0.7",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE chat_sessions ADD COLUMN include_citations BOOLEAN NOT NULL DEFAULT 1",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // SQLite doesn't support DROP COLUMN easily
        // Create a new table without the RAG columns
        db.execute_unprepared(
            r#"
            CREATE TABLE chat_sessions_backup (
                id INTEGER PRIMARY KEY NOT NULL,
                session_id TEXT NOT NULL UNIQUE,
                project_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                title TEXT,
                provider TEXT NOT NULL,
                model_name TEXT NOT NULL,
                system_prompt TEXT,
                is_archived BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_activity_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            INSERT INTO chat_sessions_backup
            SELECT id, session_id, project_id, user_id, title, provider, model_name,
                   system_prompt, is_archived, created_at, updated_at, last_activity_at
            FROM chat_sessions
            "#,
        )
        .await?;

        db.execute_unprepared("DROP TABLE chat_sessions").await?;

        db.execute_unprepared("ALTER TABLE chat_sessions_backup RENAME TO chat_sessions")
            .await?;

        // Recreate indices
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_chat_sessions_project ON chat_sessions(project_id)",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_chat_sessions_user ON chat_sessions(user_id)",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_chat_sessions_activity ON chat_sessions(last_activity_at)"
        )
        .await?;

        Ok(())
    }
}
