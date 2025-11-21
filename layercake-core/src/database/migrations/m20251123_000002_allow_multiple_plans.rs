use sea_orm::{DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        match backend {
            DatabaseBackend::Sqlite => recreate_plans_table_sqlite(db, backend, false).await,
            DatabaseBackend::Postgres => {
                db.execute(Statement::from_string(
                    backend,
                    "ALTER TABLE plans DROP CONSTRAINT IF EXISTS plans_project_id_key".to_string(),
                ))
                .await?;
                Ok(())
            }
            DatabaseBackend::MySql => {
                if let Err(_) = db
                    .execute(Statement::from_string(
                        backend,
                        "ALTER TABLE plans DROP INDEX plans_project_id".to_string(),
                    ))
                    .await
                {
                    db.execute(Statement::from_string(
                        backend,
                        "ALTER TABLE plans DROP INDEX project_id".to_string(),
                    ))
                    .await?;
                }
                Ok(())
            }
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        match backend {
            DatabaseBackend::Sqlite => recreate_plans_table_sqlite(db, backend, true).await,
            DatabaseBackend::Postgres => {
                db.execute(Statement::from_string(
                    backend,
                    "ALTER TABLE plans ADD CONSTRAINT plans_project_id_key UNIQUE(project_id)"
                        .to_string(),
                ))
                .await?;
                Ok(())
            }
            DatabaseBackend::MySql => {
                db.execute(Statement::from_string(
                    backend,
                    "ALTER TABLE plans ADD UNIQUE INDEX plans_project_id (project_id)".to_string(),
                ))
                .await?;
                Ok(())
            }
        }
    }
}

async fn recreate_plans_table_sqlite(
    db: &SchemaManagerConnection<'_>,
    backend: DatabaseBackend,
    with_unique: bool,
) -> Result<(), DbErr> {
    let unique_clause = if with_unique { "UNIQUE" } else { "" };

    db.execute(Statement::from_string(
        backend,
        "PRAGMA foreign_keys=OFF".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        backend,
        format!(
            r#"
            CREATE TABLE plans_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL {unique_clause},
                name TEXT NOT NULL,
                description TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                yaml_content TEXT NOT NULL,
                dependencies TEXT,
                status TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            )
            "#
        ),
    ))
    .await?;

    db.execute(Statement::from_string(
        backend,
        r#"
        INSERT INTO plans_new (
            id,
            project_id,
            name,
            description,
            tags,
            yaml_content,
            dependencies,
            status,
            version,
            created_at,
            updated_at
        )
        SELECT
            id,
            project_id,
            name,
            description,
            tags,
            yaml_content,
            dependencies,
            status,
            version,
            created_at,
            updated_at
        FROM plans
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        backend,
        "DROP TABLE plans".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        backend,
        "ALTER TABLE plans_new RENAME TO plans".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        backend,
        "PRAGMA foreign_keys=ON".to_string(),
    ))
    .await?;

    Ok(())
}
