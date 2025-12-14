use sea_orm::Statement;
use sea_orm_migration::prelude::*;

/// Migration to repair graphs created after the initial schema migration.
///
/// Issue: Graphs created after 2025-12-10 05:36:46 (when m20251210_000004 ran)
/// have metadata in graph_data but nodes/edges remain only in legacy tables.
///
/// This migration identifies such graphs and populates graph_data_nodes/edges
/// from the legacy graph_nodes/graph_edges tables.
#[derive(DeriveMigrationName)]
pub struct Migration;

// Migration ran at 2025-12-10 05:36:46 UTC
const MIGRATION_CUTOFF: &str = "2025-12-10 05:36:46";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // Log graphs that need repair
        let broken_graphs = db
            .query_one(Statement::from_string(
                backend,
                format!(
                    "
                SELECT COUNT(*) as count
                FROM graph_data gd
                WHERE gd.source_type = 'computed'
                  AND gd.created_at > '{cutoff}'
                  AND (
                      SELECT COUNT(*)
                      FROM graph_data_nodes
                      WHERE graph_data_id = gd.id
                  ) = 0
                  AND gd.node_count > 0;",
                    cutoff = MIGRATION_CUTOFF
                ),
            ))
            .await?;

        if let Some(row) = broken_graphs {
            let count: i64 = row.try_get("", "count")?;
            tracing::info!(
                "Found {} post-migration graphs with missing nodes/edges data",
                count
            );
        }

        // Migrate nodes for post-migration computed graphs
        // These graphs have entries in graph_data but no nodes/edges in new schema
        db.execute(Statement::from_string(
            backend,
            format!(
                "
            INSERT OR IGNORE INTO graph_data_nodes (
                graph_data_id, external_id, label, layer, weight, is_partition,
                belongs_to, comment, source_dataset_id, attributes, created_at
            )
            SELECT
                gd.id, gn.id, gn.label, gn.layer, gn.weight, gn.is_partition,
                gn.belongs_to, gn.comment, gn.dataset_id, gn.attrs,
                COALESCE(gn.created_at, CURRENT_TIMESTAMP)
            FROM graph_nodes gn
            INNER JOIN graphs g ON g.id = gn.graph_id
            INNER JOIN graph_data gd ON gd.id = g.id AND gd.source_type = 'computed'
            WHERE gd.created_at > '{cutoff}'
              AND NOT EXISTS (
                  SELECT 1 FROM graph_data_nodes
                  WHERE graph_data_id = gd.id AND external_id = gn.id
              );",
                cutoff = MIGRATION_CUTOFF
            ),
        ))
        .await?;

        // Migrate edges for post-migration computed graphs
        db.execute(Statement::from_string(
            backend,
            format!(
                "
            INSERT OR IGNORE INTO graph_data_edges (
                graph_data_id, external_id, source, target, label, layer, weight,
                comment, source_dataset_id, attributes, created_at
            )
            SELECT
                gd.id, ge.id, ge.source, ge.target, ge.label, ge.layer, ge.weight,
                ge.comment, ge.dataset_id, ge.attrs,
                COALESCE(ge.created_at, CURRENT_TIMESTAMP)
            FROM graph_edges ge
            INNER JOIN graphs g ON g.id = ge.graph_id
            INNER JOIN graph_data gd ON gd.id = g.id AND gd.source_type = 'computed'
            WHERE gd.created_at > '{cutoff}'
              AND NOT EXISTS (
                  SELECT 1 FROM graph_data_edges
                  WHERE graph_data_id = gd.id AND external_id = ge.id
              )
              -- Ensure both source and target nodes exist
              AND EXISTS (
                  SELECT 1 FROM graph_data_nodes n
                  WHERE n.graph_data_id = gd.id AND n.external_id = ge.source
              )
              AND EXISTS (
                  SELECT 1 FROM graph_data_nodes n
                  WHERE n.graph_data_id = gd.id AND n.external_id = ge.target
              );",
                cutoff = MIGRATION_CUTOFF
            ),
        ))
        .await?;

        // Update node and edge counts in graph_data to reflect actual data
        db.execute(Statement::from_string(
            backend,
            format!(
                "
            UPDATE graph_data
            SET node_count = (
                SELECT COUNT(*) FROM graph_data_nodes WHERE graph_data_id = graph_data.id
            ),
            edge_count = (
                SELECT COUNT(*) FROM graph_data_edges WHERE graph_data_id = graph_data.id
            )
            WHERE source_type = 'computed'
              AND created_at > '{}';",
                MIGRATION_CUTOFF
            ),
        ))
        .await?;

        // Log repair results
        let repair_summary = db
            .query_all(Statement::from_string(
                backend,
                format!(
                    "
                SELECT
                    gd.id,
                    gd.name,
                    gd.node_count,
                    gd.edge_count,
                    gd.created_at
                FROM graph_data gd
                WHERE gd.source_type = 'computed'
                  AND gd.created_at > '{}'
                ORDER BY gd.created_at;",
                    MIGRATION_CUTOFF
                ),
            ))
            .await?;

        tracing::info!("Repaired {} post-migration graphs", repair_summary.len());
        for row in repair_summary {
            let id: i32 = row.try_get("", "id")?;
            let name: String = row.try_get("", "name")?;
            let nodes: i32 = row.try_get("", "node_count")?;
            let edges: i32 = row.try_get("", "edge_count")?;
            tracing::info!(
                "  Graph {} '{}': {} nodes, {} edges",
                id,
                name,
                nodes,
                edges
            );
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();

        // Remove nodes/edges added by this migration
        db.execute(Statement::from_string(
            backend,
            format!(
                "
            DELETE FROM graph_data_edges
            WHERE graph_data_id IN (
                SELECT id FROM graph_data
                WHERE source_type = 'computed' AND created_at > '{}'
            );",
                MIGRATION_CUTOFF
            ),
        ))
        .await?;

        db.execute(Statement::from_string(
            backend,
            format!(
                "
            DELETE FROM graph_data_nodes
            WHERE graph_data_id IN (
                SELECT id FROM graph_data
                WHERE source_type = 'computed' AND created_at > '{}'
            );",
                MIGRATION_CUTOFF
            ),
        ))
        .await?;

        Ok(())
    }
}
