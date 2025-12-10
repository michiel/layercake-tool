use sea_orm_migration::prelude::*;

/// Migration to copy existing datasets/graphs into unified graph_data tables.
///
/// This is additive and non-destructive: old tables remain in place. It can be
/// rerun safely because inserts target the same IDs (+ offset for graphs);
/// callsites should ensure it is executed only once per environment.
#[derive(DeriveMigrationName)]
pub struct Migration;

const GRAPH_ID_OFFSET: i64 = 1_000_000;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // 0. Offset graph_edits references (only once)
        db.execute(Statement::from_string(
            backend,
            r#"
            UPDATE graph_edits
            SET graph_id = graph_id + $1
            WHERE graph_id < $1;
            "#,
        ))
        .bind(GRAPH_ID_OFFSET)
        .await?;

        // 1. Migrate datasets into graph_data (source_type = 'dataset')
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT OR IGNORE INTO graph_data (
                id, project_id, name, source_type, dag_node_id,
                file_format, origin, filename, blob, file_size, processed_at,
                status, node_count, edge_count, error_message, metadata, annotations,
                created_at, updated_at
            )
            SELECT
                id, project_id, name, 'dataset', NULL,
                file_format, origin, filename, blob, file_size, processed_at,
                status, 0, 0, error_message, metadata,
                json(annotations),  -- normalize to JSON array
                created_at, updated_at
            FROM data_sets;
            "#,
        ))
        .await?;

        // 2. Migrate dataset nodes into graph_data_nodes (external_id = original id)
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT OR IGNORE INTO graph_data_nodes (
                graph_data_id, external_id, label, layer, weight, is_partition,
                belongs_to, comment, source_dataset_id, attributes, created_at
            )
            SELECT
                dataset_id, id, label, layer, CAST(weight AS REAL), is_partition,
                belongs_to, comment, dataset_id, attributes, COALESCE(created_at, CURRENT_TIMESTAMP)
            FROM dataset_graph_nodes;
            "#,
        ))
        .await?;

        // 3. Migrate dataset edges into graph_data_edges (external_id = original id)
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT OR IGNORE INTO graph_data_edges (
                graph_data_id, external_id, source, target, label, layer, weight,
                comment, source_dataset_id, attributes, created_at
            )
            SELECT
                dataset_id, id, source, target, label, layer, CAST(weight AS REAL),
                comment, dataset_id, attributes, COALESCE(created_at, CURRENT_TIMESTAMP)
            FROM dataset_graph_edges;
            "#,
        ))
        .await?;

        // 4. Migrate computed graphs into graph_data (source_type = 'computed')
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT OR IGNORE INTO graph_data (
                id, project_id, name, source_type, dag_node_id,
                file_format, origin, filename, blob, file_size, processed_at,
                source_hash, computed_date,
                last_edit_sequence, has_pending_edits, last_replay_at,
                status, node_count, edge_count, error_message, json(annotations), metadata,
                created_at, updated_at
            )
            SELECT
                (id + $1), project_id, name, 'computed', node_id,
                NULL, NULL, NULL, NULL, NULL, NULL,
                source_hash, computed_date,
                last_edit_sequence, has_pending_edits, last_replay_at,
                CASE
                    WHEN execution_state = 'Completed' THEN 'active'
                    WHEN execution_state = 'Error' THEN 'error'
                    ELSE 'processing'
                END,
                node_count, edge_count, error_message, annotations, metadata,
                created_at, updated_at
            FROM graphs;
            "#,
        ))
        .bind(GRAPH_ID_OFFSET)
        .await?;

        // 5. Migrate computed graph nodes into graph_data_nodes
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT OR IGNORE INTO graph_data_nodes (
                graph_data_id, external_id, label, layer, weight, is_partition,
                belongs_to, comment, source_dataset_id, attributes, created_at
            )
            SELECT
                (graph_id + $1), id, label, layer, weight, is_partition,
                belongs_to, comment, dataset_id, attrs, COALESCE(created_at, CURRENT_TIMESTAMP)
            FROM graph_nodes;
            "#,
        ))
        .bind(GRAPH_ID_OFFSET)
        .await?;

        // 6. Migrate computed graph edges into graph_data_edges
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT OR IGNORE INTO graph_data_edges (
                graph_data_id, external_id, source, target, label, layer, weight,
                comment, source_dataset_id, attributes, created_at
            )
            SELECT
                (graph_id + $1), id, source, target, label, layer, weight,
                comment, dataset_id, attrs, COALESCE(created_at, CURRENT_TIMESTAMP)
            FROM graph_edges;
            "#,
        ))
        .bind(GRAPH_ID_OFFSET)
        .await?;

        // 7. Recompute node/edge counts in graph_data
        db.execute(Statement::from_string(
            backend,
            r#"
            UPDATE graph_data
            SET node_count = (
                SELECT COUNT(*) FROM graph_data_nodes WHERE graph_data_id = graph_data.id
            ),
            edge_count = (
                SELECT COUNT(*) FROM graph_data_edges WHERE graph_data_id = graph_data.id
            );
            "#,
        ))
        .await?;

        // 8. Capture validation counts
        db.execute(Statement::from_string(
            backend,
            r#"
            CREATE TABLE IF NOT EXISTS graph_data_migration_validation (
                check_name TEXT NOT NULL,
                old_count INTEGER,
                new_count INTEGER,
                delta INTEGER,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        ))
        .await?;

        // Dataset count comparison
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT INTO graph_data_migration_validation (check_name, old_count, new_count, delta)
            SELECT 'datasets', (SELECT COUNT(*) FROM data_sets), (SELECT COUNT(*) FROM graph_data WHERE source_type = 'dataset'),
                   (SELECT COUNT(*) FROM data_sets) - (SELECT COUNT(*) FROM graph_data WHERE source_type = 'dataset');
            "#,
        ))
        .await?;

        // Graph count comparison
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT INTO graph_data_migration_validation (check_name, old_count, new_count, delta)
            SELECT 'graphs', (SELECT COUNT(*) FROM graphs), (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed'),
                   (SELECT COUNT(*) FROM graphs) - (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed');
            "#,
        ))
        .await?;

        // Node count comparison
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT INTO graph_data_migration_validation (check_name, old_count, new_count, delta)
            SELECT 'nodes',
                   (SELECT COUNT(*) FROM dataset_graph_nodes) + (SELECT COUNT(*) FROM graph_nodes),
                   (SELECT COUNT(*) FROM graph_data_nodes),
                   ((SELECT COUNT(*) FROM dataset_graph_nodes) + (SELECT COUNT(*) FROM graph_nodes)) - (SELECT COUNT(*) FROM graph_data_nodes);
            "#,
        ))
        .await?;

        // Edge count comparison
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT INTO graph_data_migration_validation (check_name, old_count, new_count, delta)
            SELECT 'edges',
                   (SELECT COUNT(*) FROM dataset_graph_edges) + (SELECT COUNT(*) FROM graph_edges),
                   (SELECT COUNT(*) FROM graph_data_edges),
                   ((SELECT COUNT(*) FROM dataset_graph_edges) + (SELECT COUNT(*) FROM graph_edges)) - (SELECT COUNT(*) FROM graph_data_edges);
            "#,
        ))
        .await?;

        // Orphaned edge references (edges whose source/target nodes do not exist)
        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT INTO graph_data_migration_validation (check_name, old_count, new_count, delta)
            SELECT 'orphaned_edge_source',
                   NULL,
                   (SELECT COUNT(*) FROM graph_data_edges e WHERE NOT EXISTS (
                        SELECT 1 FROM graph_data_nodes n
                        WHERE n.graph_data_id = e.graph_data_id AND n.external_id = e.source
                   )),
                   NULL;
            "#,
        ))
        .await?;

        db.execute(Statement::from_string(
            backend,
            r#"
            INSERT INTO graph_data_migration_validation (check_name, old_count, new_count, delta)
            SELECT 'orphaned_edge_target',
                   NULL,
                   (SELECT COUNT(*) FROM graph_data_edges e WHERE NOT EXISTS (
                        SELECT 1 FROM graph_data_nodes n
                        WHERE n.graph_data_id = e.graph_data_id AND n.external_id = e.target
                   )),
                   NULL;
            "#,
        ))
        .await?;

        // 9. Reseed sequences/autoincrement to max(id)
        match backend {
            sea_orm::DatabaseBackend::Sqlite => {
                for table in &["graph_data", "graph_data_nodes", "graph_data_edges"] {
                    let stmt = format!(
                        "UPDATE sqlite_sequence SET seq = (SELECT IFNULL(MAX(id), 0) FROM {tbl}) WHERE name = '{tbl}';",
                        tbl = table
                    );
                    db.execute(Statement::from_string(backend, stmt)).await?;
                }
            }
            sea_orm::DatabaseBackend::Postgres => {
                db.execute(Statement::from_string(
                    backend,
                    "SELECT setval(pg_get_serial_sequence('graph_data','id'), GREATEST((SELECT MAX(id) FROM graph_data), 1), true);",
                ))
                .await?;
                db.execute(Statement::from_string(
                    backend,
                    "SELECT setval(pg_get_serial_sequence('graph_data_nodes','id'), GREATEST((SELECT MAX(id) FROM graph_data_nodes), 1), true);",
                ))
                .await?;
                db.execute(Statement::from_string(
                    backend,
                    "SELECT setval(pg_get_serial_sequence('graph_data_edges','id'), GREATEST((SELECT MAX(id) FROM graph_data_edges), 1), true);",
                ))
                .await?;
            }
            _ => { /* no-op for other backends */ }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // Best-effort cleanup of migrated rows; keeps schema intact.
        db.execute(Statement::from_string(
            backend,
            "DELETE FROM graph_data_edges;",
        ))
        .await?;
        db.execute(Statement::from_string(
            backend,
            "DELETE FROM graph_data_nodes;",
        ))
        .await?;
        db.execute(Statement::from_string(
            backend,
            "DELETE FROM graph_data;",
        ))
        .await?;
        Ok(())
    }
}
