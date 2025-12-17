use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        match backend {
            sea_orm::DatabaseBackend::Sqlite => {
                // SQLite doesn't support dropping FK constraints, so recreate table

                // 1. Drop existing indexes (they'll be recreated after table rename)
                let _ = db
                    .execute_unprepared("DROP INDEX IF EXISTS idx_edges_graph_external_unique")
                    .await;
                let _ = db
                    .execute_unprepared("DROP INDEX IF EXISTS idx_edges_graph")
                    .await;
                let _ = db
                    .execute_unprepared("DROP INDEX IF EXISTS idx_edges_source")
                    .await;
                let _ = db
                    .execute_unprepared("DROP INDEX IF EXISTS idx_edges_target")
                    .await;
                let _ = db
                    .execute_unprepared("DROP INDEX IF EXISTS idx_edges_source_target")
                    .await;
                let _ = db
                    .execute_unprepared("DROP INDEX IF EXISTS idx_edges_layer")
                    .await;

                // 2. Create new table without node FK constraints
                db.execute_unprepared(
                    r#"
                    CREATE TABLE graph_data_edges_new (
                        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                        graph_data_id INTEGER NOT NULL,
                        external_id TEXT NOT NULL,
                        source TEXT NOT NULL,
                        target TEXT NOT NULL,
                        label TEXT NULL,
                        layer TEXT NULL,
                        weight REAL NULL,
                        comment TEXT NULL,
                        source_dataset_id INTEGER NULL,
                        attributes TEXT NULL,
                        created_at TEXT NOT NULL,
                        FOREIGN KEY (graph_data_id) REFERENCES graph_data(id) ON DELETE CASCADE
                    )
                    "#,
                )
                .await?;

                // 3. Copy data from old table
                db.execute_unprepared(
                    r#"
                    INSERT INTO graph_data_edges_new
                    SELECT id, graph_data_id, external_id, source, target, label,
                           layer, weight, comment, source_dataset_id, attributes, created_at
                    FROM graph_data_edges
                    "#,
                )
                .await?;

                // 4. Drop old table
                db.execute_unprepared("DROP TABLE graph_data_edges").await?;

                // 5. Rename new table
                db.execute_unprepared(
                    "ALTER TABLE graph_data_edges_new RENAME TO graph_data_edges",
                )
                .await?;

                // 6. Recreate indexes
                db.execute_unprepared(
                    "CREATE UNIQUE INDEX idx_edges_graph_external_unique ON graph_data_edges(graph_data_id, external_id)"
                )
                .await?;

                db.execute_unprepared(
                    "CREATE INDEX idx_edges_graph ON graph_data_edges(graph_data_id)",
                )
                .await?;

                db.execute_unprepared(
                    "CREATE INDEX idx_edges_source ON graph_data_edges(graph_data_id, source)",
                )
                .await?;

                db.execute_unprepared(
                    "CREATE INDEX idx_edges_target ON graph_data_edges(graph_data_id, target)",
                )
                .await?;

                db.execute_unprepared(
                    "CREATE INDEX idx_edges_source_target ON graph_data_edges(source, target)",
                )
                .await?;

                db.execute_unprepared("CREATE INDEX idx_edges_layer ON graph_data_edges(layer)")
                    .await?;
            }
            sea_orm::DatabaseBackend::Postgres => {
                // Postgres can drop constraints directly
                db.execute_unprepared(
                    "ALTER TABLE graph_data_edges DROP CONSTRAINT IF EXISTS fk_graph_data_edges_source"
                )
                .await?;

                db.execute_unprepared(
                    "ALTER TABLE graph_data_edges DROP CONSTRAINT IF EXISTS fk_graph_data_edges_target"
                )
                .await?;
            }
            sea_orm::DatabaseBackend::MySql => {
                // MySQL syntax
                db.execute_unprepared(
                    "ALTER TABLE graph_data_edges DROP FOREIGN KEY fk_graph_data_edges_source",
                )
                .await?;

                db.execute_unprepared(
                    "ALTER TABLE graph_data_edges DROP FOREIGN KEY fk_graph_data_edges_target",
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Downgrade not supported - would require re-adding FK constraints
        // which could fail if data violates constraints
        let db = manager.get_connection();

        db.execute_unprepared(
            "-- Downgrade not supported: Cannot safely re-add FK constraints that may be violated",
        )
        .await?;

        Ok(())
    }
}
