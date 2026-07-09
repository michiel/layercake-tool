use crate::database::entities::graph_data;
use crate::database::entities::graph_edits::{self, Entity as GraphEdits};
use crate::errors::{CoreError, CoreResult};
use crate::services::GraphDataService;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};

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
    /// * `applied` - Whether the edit has already been applied (true for manual edits, false for edits awaiting replay)
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
        applied: bool,
    ) -> CoreResult<graph_edits::Model> {
        // Insert the edit, allocating a sequence number as max+1. A unique index
        // on (graph_id, sequence_number) turns a concurrent-allocation collision
        // into an insert error, which we retry with a freshly computed number
        // instead of silently writing a duplicate sequence (ambiguous replay).
        // Under contention many writers can read the same max before any
        // commits, so allow generous retries; SQLite serialises writes, so a
        // fresh max read after each conflict converges quickly.
        const MAX_ATTEMPTS: usize = 50;
        let mut last_err: Option<sea_orm::DbErr> = None;

        for _ in 0..MAX_ATTEMPTS {
            let next_sequence = self.get_next_sequence_number(graph_id).await?;

            let edit = graph_edits::ActiveModel {
                id: ActiveValue::NotSet,
                graph_id: Set(graph_id),
                target_type: Set(target_type.clone()),
                target_id: Set(target_id.clone()),
                operation: Set(operation.clone()),
                field_name: Set(field_name.clone()),
                old_value: Set(old_value.clone()),
                new_value: Set(new_value.clone()),
                sequence_number: Set(next_sequence),
                applied: Set(applied),
                created_at: Set(chrono::Utc::now()),
                created_by: Set(created_by),
            };

            match edit.insert(&self.db).await {
                Ok(inserted) => {
                    // Update the graph's last_edit_sequence and has_pending_edits
                    self.update_graph_edit_metadata(graph_id, next_sequence, applied)
                        .await?;
                    return Ok(inserted);
                }
                Err(e) if is_unique_violation(&e) => {
                    // Another writer took this sequence number; recompute and retry.
                    last_err = Some(e);
                    continue;
                }
                Err(e) => {
                    return Err(CoreError::internal(format!(
                        "Failed to insert graph edit: {}",
                        e
                    )));
                }
            }
        }

        Err(CoreError::internal(format!(
            "Failed to insert graph edit after {} attempts due to sequence contention: {}",
            MAX_ATTEMPTS,
            last_err
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )))
    }

    /// Get the next sequence number for a graph
    async fn get_next_sequence_number(&self, graph_id: i32) -> CoreResult<i32> {
        let last_edit = GraphEdits::find()
            .filter(graph_edits::Column::GraphId.eq(graph_id))
            .order_by_desc(graph_edits::Column::SequenceNumber)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph edits: {}", e)))?;

        Ok(last_edit.map(|e| e.sequence_number + 1).unwrap_or(1))
    }

    /// Update graph metadata after creating an edit
    async fn update_graph_edit_metadata(
        &self,
        graph_id: i32,
        sequence_number: i32,
        applied: bool,
    ) -> CoreResult<()> {
        if graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph data: {}", e)))?
            .is_some()
        {
            let graph_data_service = GraphDataService::new(self.db.clone());
            graph_data_service
                .update_edit_metadata(graph_id, sequence_number, applied)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to update graph data metadata: {}", e))
                })?;
        } else {
            use crate::database::entities::graphs::{self, Entity as Graphs};

            let graph = Graphs::find_by_id(graph_id)
                .one(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to load graph: {}", e)))?
                .ok_or_else(|| CoreError::not_found("Graph", graph_id.to_string()))?;

            let mut active_model: graphs::ActiveModel = graph.into();
            active_model.last_edit_sequence = Set(sequence_number);
            // Only mark as pending if the edit is not yet applied
            if !applied {
                active_model.has_pending_edits = Set(true);
            }
            active_model.updated_at = Set(chrono::Utc::now());

            active_model
                .update(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to update graph: {}", e)))?;
        }
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
    ) -> CoreResult<Vec<graph_edits::Model>> {
        let mut query = GraphEdits::find().filter(graph_edits::Column::GraphId.eq(graph_id));

        if unapplied_only {
            query = query.filter(graph_edits::Column::Applied.eq(false));
        }

        let edits = query
            .order_by_asc(graph_edits::Column::SequenceNumber)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph edits: {}", e)))?;

        Ok(edits)
    }

    /// Mark an edit as applied
    pub async fn mark_edit_applied(&self, edit_id: i32) -> CoreResult<()> {
        let edit = GraphEdits::find_by_id(edit_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph edit: {}", e)))?
            .ok_or_else(|| CoreError::not_found("GraphEdit", edit_id.to_string()))?;

        let mut active_model: graph_edits::ActiveModel = edit.into();
        active_model.applied = Set(true);
        active_model
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update graph edit: {}", e)))?;

        Ok(())
    }

    /// Mark multiple edits as applied
    #[allow(dead_code)]
    pub async fn mark_edits_applied(&self, edit_ids: Vec<i32>) -> CoreResult<()> {
        for edit_id in edit_ids {
            self.mark_edit_applied(edit_id).await?;
        }
        Ok(())
    }

    /// Clear all edits for a graph
    pub async fn clear_graph_edits(&self, graph_id: i32) -> CoreResult<u64> {
        use crate::database::entities::graphs::{self, Entity as Graphs};

        // Delete all edits
        let result = GraphEdits::delete_many()
            .filter(graph_edits::Column::GraphId.eq(graph_id))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete graph edits: {}", e)))?;

        // Reset graph metadata
        if graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph data: {}", e)))?
            .is_some()
        {
            let graph_data_service = GraphDataService::new(self.db.clone());
            graph_data_service
                .reset_edit_metadata(graph_id)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to reset graph data metadata: {}", e))
                })?;
            graph_data_service
                .mark_replayed(graph_id)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to mark graph data replayed: {}", e))
                })?;
        } else {
            let graph = Graphs::find_by_id(graph_id)
                .one(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to load graph: {}", e)))?
                .ok_or_else(|| CoreError::not_found("Graph", graph_id.to_string()))?;

            let mut active_model: graphs::ActiveModel = graph.into();
            active_model.last_edit_sequence = Set(0);
            active_model.has_pending_edits = Set(false);
            active_model.last_replay_at = Set(None);
            active_model.updated_at = Set(chrono::Utc::now());

            active_model
                .update(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to update graph: {}", e)))?;
        }

        Ok(result.rows_affected)
    }

    /// Update last_replay_at timestamp for a graph
    pub async fn mark_graph_replayed(&self, graph_id: i32) -> CoreResult<()> {
        if graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph data: {}", e)))?
            .is_some()
        {
            let graph_data_service = GraphDataService::new(self.db.clone());
            graph_data_service
                .mark_replayed(graph_id)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to mark graph data replayed: {}", e))
                })?;
        } else {
            use crate::database::entities::graphs::{self, Entity as Graphs};

            let graph = Graphs::find_by_id(graph_id)
                .one(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to load graph: {}", e)))?
                .ok_or_else(|| CoreError::not_found("Graph", graph_id.to_string()))?;

            let mut active_model: graphs::ActiveModel = graph.into();
            active_model.last_replay_at = Set(Some(chrono::Utc::now()));
            active_model.updated_at = Set(chrono::Utc::now());

            active_model
                .update(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to update graph: {}", e)))?;
        }
        Ok(())
    }

    /// Get edit count for a graph
    pub async fn get_edit_count(&self, graph_id: i32, unapplied_only: bool) -> CoreResult<u64> {
        let mut query = GraphEdits::find().filter(graph_edits::Column::GraphId.eq(graph_id));

        if unapplied_only {
            query = query.filter(graph_edits::Column::Applied.eq(false));
        }

        let count = query
            .count(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to count graph edits: {}", e)))?;
        Ok(count)
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

/// Detect a unique-constraint violation across backends. SeaORM surfaces the
/// underlying driver error; for SQLite the message contains "UNIQUE constraint
/// failed", for Postgres "duplicate key value violates unique constraint".
fn is_unique_violation(err: &sea_orm::DbErr) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("unique constraint failed") || msg.contains("duplicate key value")
}
