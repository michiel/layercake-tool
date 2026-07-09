use sea_orm::Statement;
use sea_orm_migration::prelude::*;

/// Rebuild `graph_edits` without its foreign key to the (now dropped) `graphs`
/// table.
///
/// `graph_edits` was created with `FOREIGN KEY (graph_id) REFERENCES graphs(id)`.
/// A later migration dropped the legacy `graphs` table but left this FK in
/// place. With SQLite foreign-key enforcement enabled, every insert into
/// `graph_edits` then fails with "no such table: main.graphs", which silently
/// breaks the entire graph-edit tracking / replay feature. SQLite cannot drop a
/// foreign key in place, so we rebuild the table (preserving rows) and recreate
/// its indexes, including the existing unique (graph_id, sequence_number)
/// constraint.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        let stmts = [
            // New table with identical columns but NO foreign key to graphs.
            r#"
            CREATE TABLE graph_edits_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                graph_id INTEGER NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                operation TEXT NOT NULL,
                field_name TEXT,
                old_value JSON,
                new_value JSON,
                sequence_number INTEGER NOT NULL,
                applied BOOLEAN NOT NULL DEFAULT 0,
                created_at TIMESTAMP NOT NULL,
                created_by INTEGER
            )
            "#,
            // Preserve any existing rows.
            r#"
            INSERT INTO graph_edits_new
                (id, graph_id, target_type, target_id, operation, field_name,
                 old_value, new_value, sequence_number, applied, created_at, created_by)
            SELECT
                id, graph_id, target_type, target_id, operation, field_name,
                old_value, new_value, sequence_number, applied, created_at, created_by
            FROM graph_edits
            "#,
            "DROP TABLE graph_edits",
            "ALTER TABLE graph_edits_new RENAME TO graph_edits",
            // Recreate indexes (names match the original migration).
            "CREATE INDEX idx_graph_edits_graph_id ON graph_edits(graph_id)",
            "CREATE INDEX idx_graph_edits_target ON graph_edits(graph_id, target_type, target_id)",
            "CREATE INDEX idx_graph_edits_sequence ON graph_edits(graph_id, sequence_number)",
            "CREATE UNIQUE INDEX uq_graph_edits_graph_sequence ON graph_edits(graph_id, sequence_number)",
        ];

        for stmt in stmts {
            db.execute(Statement::from_string(backend, stmt.to_string()))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Non-reversible: the `graphs` table this FK referenced no longer
        // exists, so we cannot restore the original foreign key.
        Ok(())
    }
}
