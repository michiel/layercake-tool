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

        // Every statement is written to be safe to re-run: SeaORM's SQLite DDL is
        // not reliably transactional, so a mid-migration failure can leave the
        // rebuild partially applied. Making each step idempotent lets the
        // migration converge to the canonical end state on retry.
        let stmts = [
            // Clean up any leftover scratch table from a prior partial run.
            "DROP TABLE IF EXISTS graph_edits_new",
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
            // Preserve existing rows. INSERT OR IGNORE so a re-run (where
            // graph_edits already has the new shape) does not error on the copy.
            r#"
            INSERT OR IGNORE INTO graph_edits_new
                (id, graph_id, target_type, target_id, operation, field_name,
                 old_value, new_value, sequence_number, applied, created_at, created_by)
            SELECT
                id, graph_id, target_type, target_id, operation, field_name,
                old_value, new_value, sequence_number, applied, created_at, created_by
            FROM graph_edits
            "#,
            "DROP TABLE graph_edits",
            "ALTER TABLE graph_edits_new RENAME TO graph_edits",
            // Recreate indexes idempotently. SQLite carries an index's definition
            // across a table rebuild in some paths, so CREATE ... IF NOT EXISTS
            // is required — it is a no-op if the index already exists and creates
            // it otherwise, converging to the same state on fresh or retried runs.
            "CREATE INDEX IF NOT EXISTS idx_graph_edits_graph_id ON graph_edits(graph_id)",
            "CREATE INDEX IF NOT EXISTS idx_graph_edits_target ON graph_edits(graph_id, target_type, target_id)",
            "CREATE INDEX IF NOT EXISTS idx_graph_edits_sequence ON graph_edits(graph_id, sequence_number)",
            "CREATE UNIQUE INDEX IF NOT EXISTS uq_graph_edits_graph_sequence ON graph_edits(graph_id, sequence_number)",
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
