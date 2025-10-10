use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};
use tracing::{debug, info, warn};
use crate::database::entities::graph_edits::{self, Entity as GraphEdits};
use super::graph_edit_applicator::{GraphEditApplicator, ApplyResult};

/// Service for managing graph edit operations
///
/// Handles creation, retrieval, and management of graph edits for change tracking
/// and replay functionality.
pub struct GraphEditService {
    db: DatabaseConnection,
}

impl GraphEditService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new graph edit record with auto-incrementing sequence number
    ///
    /// # Arguments
    /// * `graph_id` - ID of the graph being edited
    /// * `target_type` - Type of entity ('node', 'edge', 'layer')
    /// * `target_id` - ID of the target entity
    /// * `operation` - Operation type ('create', 'update', 'delete')
    /// * `field_name` - Field being updated (for update operations)
    /// * `old_value` - Previous value (for updates/deletes)
    /// * `new_value` - New value (for creates/updates)
    /// * `created_by` - Optional user ID
    pub async fn create_edit(
        &self,
        graph_id: i32,
        target_type: String,
        target_id: String,
        operation: String,
        field_name: Option<String>,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        created_by: Option<i32>,
    ) -> Result<graph_edits::Model> {
        // Get the next sequence number for this graph
        let next_sequence = self.get_next_sequence_number(graph_id).await?;

        // Create the edit record
        let edit = graph_edits::ActiveModel {
            id: Set(Default::default()),
            graph_id: Set(graph_id),
            target_type: Set(target_type),
            target_id: Set(target_id),
            operation: Set(operation),
            field_name: Set(field_name),
            old_value: Set(old_value),
            new_value: Set(new_value),
            sequence_number: Set(next_sequence),
            applied: Set(false),
            created_at: Set(chrono::Utc::now()),
            created_by: Set(created_by),
        };

        let edit = edit.insert(&self.db).await?;

        // Update the graph's last_edit_sequence and has_pending_edits
        self.update_graph_edit_metadata(graph_id, next_sequence).await?;

        Ok(edit)
    }

    /// Get the next sequence number for a graph
    async fn get_next_sequence_number(&self, graph_id: i32) -> Result<i32> {
        let last_edit = GraphEdits::find()
            .filter(graph_edits::Column::GraphId.eq(graph_id))
            .order_by_desc(graph_edits::Column::SequenceNumber)
            .one(&self.db)
            .await?;

        Ok(last_edit.map(|e| e.sequence_number + 1).unwrap_or(1))
    }

    /// Update graph metadata after creating an edit
    async fn update_graph_edit_metadata(&self, graph_id: i32, sequence_number: i32) -> Result<()> {
        use crate::database::entities::graphs::{self, Entity as Graphs};

        let graph = Graphs::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;

        let mut active_model: graphs::ActiveModel = graph.into();
        active_model.last_edit_sequence = Set(sequence_number);
        active_model.has_pending_edits = Set(true);
        active_model.updated_at = Set(chrono::Utc::now());

        active_model.update(&self.db).await?;
        Ok(())
    }

    /// Get all edits for a graph in sequence order
    ///
    /// # Arguments
    /// * `graph_id` - ID of the graph
    /// * `unapplied_only` - If true, only return edits where applied=false
    pub async fn get_edits_for_graph(
        &self,
        graph_id: i32,
        unapplied_only: bool,
    ) -> Result<Vec<graph_edits::Model>> {
        let mut query = GraphEdits::find()
            .filter(graph_edits::Column::GraphId.eq(graph_id));

        if unapplied_only {
            query = query.filter(graph_edits::Column::Applied.eq(false));
        }

        let edits = query
            .order_by_asc(graph_edits::Column::SequenceNumber)
            .all(&self.db)
            .await?;

        Ok(edits)
    }

    /// Mark an edit as applied
    pub async fn mark_edit_applied(&self, edit_id: i32) -> Result<()> {
        let edit = GraphEdits::find_by_id(edit_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Edit not found"))?;

        let mut active_model: graph_edits::ActiveModel = edit.into();
        active_model.applied = Set(true);
        active_model.update(&self.db).await?;

        Ok(())
    }

    /// Mark multiple edits as applied
    pub async fn mark_edits_applied(&self, edit_ids: Vec<i32>) -> Result<()> {
        for edit_id in edit_ids {
            self.mark_edit_applied(edit_id).await?;
        }
        Ok(())
    }

    /// Clear all edits for a graph
    pub async fn clear_graph_edits(&self, graph_id: i32) -> Result<u64> {
        use crate::database::entities::graphs::{self, Entity as Graphs};

        // Delete all edits
        let result = GraphEdits::delete_many()
            .filter(graph_edits::Column::GraphId.eq(graph_id))
            .exec(&self.db)
            .await?;

        // Reset graph metadata
        let graph = Graphs::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;

        let mut active_model: graphs::ActiveModel = graph.into();
        active_model.last_edit_sequence = Set(0);
        active_model.has_pending_edits = Set(false);
        active_model.last_replay_at = Set(None);
        active_model.updated_at = Set(chrono::Utc::now());

        active_model.update(&self.db).await?;

        Ok(result.rows_affected)
    }

    /// Update last_replay_at timestamp for a graph
    pub async fn mark_graph_replayed(&self, graph_id: i32) -> Result<()> {
        use crate::database::entities::graphs::{self, Entity as Graphs};

        let graph = Graphs::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;

        let mut active_model: graphs::ActiveModel = graph.into();
        active_model.last_replay_at = Set(Some(chrono::Utc::now()));
        active_model.updated_at = Set(chrono::Utc::now());

        active_model.update(&self.db).await?;
        Ok(())
    }

    /// Get edit count for a graph
    pub async fn get_edit_count(&self, graph_id: i32, unapplied_only: bool) -> Result<u64> {
        let mut query = GraphEdits::find()
            .filter(graph_edits::Column::GraphId.eq(graph_id));

        if unapplied_only {
            query = query.filter(graph_edits::Column::Applied.eq(false));
        }

        let count = query.count(&self.db).await?;
        Ok(count)
    }

    /// Replay all unapplied edits for a graph
    ///
    /// Returns a summary of the replay operation
    pub async fn replay_graph_edits(&self, graph_id: i32) -> Result<ReplaySummary> {
        info!("Starting replay of edits for graph {}", graph_id);

        // Get all unapplied edits in sequence order
        let edits = self.get_edits_for_graph(graph_id, true).await?;
        let total_edits = edits.len();

        info!("Found {} unapplied edits to replay", total_edits);

        let mut summary = ReplaySummary {
            total: total_edits,
            applied: 0,
            skipped: 0,
            failed: 0,
            details: Vec::new(),
        };

        // Create applicator
        let applicator = GraphEditApplicator::new(self.db.clone());

        // Apply each edit in sequence
        for edit in edits {
            debug!(
                "Replaying edit #{}: {} {} {}",
                edit.sequence_number, edit.operation, edit.target_type, edit.target_id
            );

            match applicator.apply_edit(&edit).await {
                Ok(ApplyResult::Success { message }) => {
                    summary.applied += 1;
                    summary.details.push(EditResult {
                        sequence_number: edit.sequence_number,
                        target_type: edit.target_type.clone(),
                        target_id: edit.target_id.clone(),
                        operation: edit.operation.clone(),
                        result: "success".to_string(),
                        message,
                    });

                    // Mark as applied
                    if let Err(e) = self.mark_edit_applied(edit.id).await {
                        warn!("Failed to mark edit {} as applied: {}", edit.id, e);
                    }
                }
                Ok(ApplyResult::Skipped { reason }) => {
                    summary.skipped += 1;
                    summary.details.push(EditResult {
                        sequence_number: edit.sequence_number,
                        target_type: edit.target_type.clone(),
                        target_id: edit.target_id.clone(),
                        operation: edit.operation.clone(),
                        result: "skipped".to_string(),
                        message: reason,
                    });
                }
                Ok(ApplyResult::Error { reason }) => {
                    summary.failed += 1;
                    summary.details.push(EditResult {
                        sequence_number: edit.sequence_number,
                        target_type: edit.target_type.clone(),
                        target_id: edit.target_id.clone(),
                        operation: edit.operation.clone(),
                        result: "failed".to_string(),
                        message: reason.clone(),
                    });

                    warn!("Failed to apply edit #{}: {}", edit.sequence_number, reason);
                }
                Err(e) => {
                    summary.failed += 1;
                    summary.details.push(EditResult {
                        sequence_number: edit.sequence_number,
                        target_type: edit.target_type.clone(),
                        target_id: edit.target_id.clone(),
                        operation: edit.operation.clone(),
                        result: "failed".to_string(),
                        message: e.to_string(),
                    });

                    warn!("Failed to apply edit #{}: {}", edit.sequence_number, e);
                }
            }
        }

        // Update last_replay_at
        self.mark_graph_replayed(graph_id).await?;

        info!(
            "Replay complete for graph {}: {} applied, {} skipped, {} failed",
            graph_id, summary.applied, summary.skipped, summary.failed
        );

        Ok(summary)
    }
}

/// Summary of a replay operation
#[derive(Debug, Clone)]
pub struct ReplaySummary {
    pub total: usize,
    pub applied: usize,
    pub skipped: usize,
    pub failed: usize,
    pub details: Vec<EditResult>,
}

/// Result of applying a single edit during replay
#[derive(Debug, Clone)]
pub struct EditResult {
    pub sequence_number: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub result: String, // "success", "skipped", "failed"
    pub message: String,
}
