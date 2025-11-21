use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        if backend == sea_orm::DatabaseBackend::Sqlite {
            db.execute(Statement::from_string(
                backend,
                "ALTER TABLE plans ADD COLUMN description TEXT".to_string(),
            ))
            .await?;

            db.execute(Statement::from_string(
                backend,
                "ALTER TABLE plans ADD COLUMN tags TEXT NOT NULL DEFAULT '[]'".to_string(),
            ))
            .await?;

            Ok(())
        } else {
            manager
                .alter_table(
                    Table::alter()
                        .table(Plans::Table)
                        .add_column(ColumnDef::new(Plans::Description).text())
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Plans::Table)
                        .add_column(ColumnDef::new(Plans::Tags).text().not_null().default("[]"))
                        .to_owned(),
                )
                .await
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        if backend == sea_orm::DatabaseBackend::Sqlite {
            db.execute(Statement::from_string(
                backend,
                r#"
                CREATE TABLE plans_backup (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    yaml_content TEXT NOT NULL,
                    dependencies TEXT,
                    status TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )
                "#
                .to_string(),
            ))
            .await?;

            db.execute(Statement::from_string(
                backend,
                r#"
                INSERT INTO plans_backup (
                    id, project_id, name, yaml_content, dependencies, status, version, created_at, updated_at
                )
                SELECT id, project_id, name, yaml_content, dependencies, status, version, created_at, updated_at
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
                "ALTER TABLE plans_backup RENAME TO plans".to_string(),
            ))
            .await?;

            Ok(())
        } else {
            manager
                .alter_table(
                    Table::alter()
                        .table(Plans::Table)
                        .drop_column(Plans::Description)
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Plans::Table)
                        .drop_column(Plans::Tags)
                        .to_owned(),
                )
                .await
        }
    }
}

#[derive(DeriveIden)]
enum Plans {
    Table,
    Description,
    Tags,
}
