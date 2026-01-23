use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create plan_dag_annotations table
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            r#"
            CREATE TABLE plan_dag_annotations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                plan_id INTEGER NOT NULL,
                target_id TEXT NOT NULL,
                target_type TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
            )
            "#
            .to_string(),
        ))
        .await?;

        // Create indices for efficient queries
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "CREATE INDEX idx_annotations_target ON plan_dag_annotations(target_id, target_type)"
                .to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "CREATE INDEX idx_annotations_key ON plan_dag_annotations(key)".to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "CREATE INDEX idx_annotations_plan ON plan_dag_annotations(plan_id)".to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop the table (indices will be dropped automatically)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DROP TABLE IF EXISTS plan_dag_annotations".to_string(),
        ))
        .await?;

        Ok(())
    }
}
