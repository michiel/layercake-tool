use crate::database::entities::{
    graph_data, graph_data::GraphDataStatus, graph_data_edges, graph_data_nodes,
    graph_edits::{self, Entity as GraphEdits},
};
use crate::services::graph_data_edit_applicator::{ApplyResult, GraphDataEditApplicator};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use serde_json::Value;
use tracing::{debug, info, warn};

pub struct GraphDataService {
    db: DatabaseConnection,
}

impl GraphDataService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, input: GraphDataCreate) -> Result<graph_data::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let active = graph_data::ActiveModel {
            project_id: Set(input.project_id),
            name: Set(input.name),
            source_type: Set(input.source_type),
            dag_node_id: Set(input.dag_node_id),
            file_format: Set(input.file_format),
            origin: Set(input.origin),
            filename: Set(input.filename),
            blob: Set(input.blob),
            file_size: Set(input.file_size),
            processed_at: Set(input.processed_at),
            source_hash: Set(input.source_hash),
            computed_date: Set(input.computed_date),
            last_edit_sequence: Set(input.last_edit_sequence.unwrap_or(0)),
            has_pending_edits: Set(input.has_pending_edits.unwrap_or(false)),
            last_replay_at: Set(input.last_replay_at),
            node_count: Set(0),
            edge_count: Set(0),
            error_message: Set(None),
            metadata: Set(input.metadata),
            annotations: Set(input.annotations),
            status: Set(input.status.unwrap_or(GraphDataStatus::Processing).into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        graph_data::Entity::insert(active).exec_with_returning(&self.db).await
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<graph_data::Model>, sea_orm::DbErr> {
        graph_data::Entity::find_by_id(id).one(&self.db).await
    }

    pub async fn list_by_project_and_source(
        &self,
        project_id: i32,
        source_type: &str,
    ) -> Result<Vec<graph_data::Model>, sea_orm::DbErr> {
        graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::SourceType.eq(source_type))
            .all(&self.db)
            .await
    }

    pub async fn replace_nodes(
        &self,
        graph_data_id: i32,
        nodes: Vec<GraphDataNodeInput>,
    ) -> Result<(), sea_orm::DbErr> {
        let txn = self.db.begin().await?;

        graph_data_nodes::Entity::delete_many()
            .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
            .exec(&txn)
            .await?;

        let now = Utc::now();
        for node in nodes.iter() {
            let active = graph_data_nodes::ActiveModel {
                graph_data_id: Set(graph_data_id),
                external_id: Set(node.external_id.clone()),
                label: Set(node.label.clone()),
                layer: Set(node.layer.clone()),
                weight: Set(node.weight),
                is_partition: Set(node.is_partition.unwrap_or(false)),
                belongs_to: Set(node.belongs_to.clone()),
                comment: Set(node.comment.clone()),
                source_dataset_id: Set(node.source_dataset_id),
                attributes: Set(node.attributes.clone()),
                created_at: Set(node.created_at.unwrap_or(now)),
                ..Default::default()
            };
            graph_data_nodes::Entity::insert(active).exec(&txn).await?;
        }

        graph_data::ActiveModel {
            id: Set(graph_data_id),
            node_count: Set(nodes.len() as i32),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&txn)
        .await?;

        txn.commit().await
    }

    pub async fn replace_edges(
        &self,
        graph_data_id: i32,
        edges: Vec<GraphDataEdgeInput>,
    ) -> Result<(), sea_orm::DbErr> {
        let txn = self.db.begin().await?;

        graph_data_edges::Entity::delete_many()
            .filter(graph_data_edges::Column::GraphDataId.eq(graph_data_id))
            .exec(&txn)
            .await?;

        let now = Utc::now();
        for edge in edges.iter() {
            let active = graph_data_edges::ActiveModel {
                graph_data_id: Set(graph_data_id),
                external_id: Set(edge.external_id.clone()),
                source: Set(edge.source.clone()),
                target: Set(edge.target.clone()),
                label: Set(edge.label.clone()),
                layer: Set(edge.layer.clone()),
                weight: Set(edge.weight),
                comment: Set(edge.comment.clone()),
                source_dataset_id: Set(edge.source_dataset_id),
                attributes: Set(edge.attributes.clone()),
                created_at: Set(edge.created_at.unwrap_or(now)),
                ..Default::default()
            };
            graph_data_edges::Entity::insert(active).exec(&txn).await?;
        }

        graph_data::ActiveModel {
            id: Set(graph_data_id),
            edge_count: Set(edges.len() as i32),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&txn)
        .await?;

        txn.commit().await
    }

    pub async fn load_nodes(
        &self,
        graph_data_id: i32,
    ) -> Result<Vec<graph_data_nodes::Model>, sea_orm::DbErr> {
        graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
            .all(&self.db)
            .await
    }

    pub async fn load_edges(
        &self,
        graph_data_id: i32,
    ) -> Result<Vec<graph_data_edges::Model>, sea_orm::DbErr> {
        graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(graph_data_id))
            .all(&self.db)
            .await
    }

    pub async fn get_by_dag_node(
        &self,
        dag_node_id: &str,
    ) -> Result<Option<graph_data::Model>, sea_orm::DbErr> {
        graph_data::Entity::find()
            .filter(graph_data::Column::DagNodeId.eq(dag_node_id))
            .one(&self.db)
            .await
    }

    pub async fn mark_status(
        &self,
        graph_data_id: i32,
        status: GraphDataStatus,
        source_hash: Option<String>,
    ) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.status = Set(status.into());
        if let Some(hash) = source_hash {
            model.source_hash = Set(Some(hash));
        }
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    pub async fn mark_complete(
        &self,
        graph_data_id: i32,
        source_hash: String,
    ) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.status = Set(GraphDataStatus::Active.into());
        model.source_hash = Set(Some(source_hash));
        model.computed_date = Set(Some(Utc::now()));
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    pub async fn load_full(
        &self,
        graph_data_id: i32,
    ) -> Result<(graph_data::Model, Vec<graph_data_nodes::Model>, Vec<graph_data_edges::Model>), sea_orm::DbErr>
    {
        let graph = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?;

        if let Some(graph) = graph {
            let nodes = self.load_nodes(graph.id).await?;
            let edges = self.load_edges(graph.id).await?;
            Ok((graph, nodes, edges))
        } else {
            Err(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))
        }
    }

    /// Convenience method for listing datasets in a project
    pub async fn list_datasets(&self, project_id: i32) -> Result<Vec<graph_data::Model>, sea_orm::DbErr> {
        self.list_by_project_and_source(project_id, "dataset").await
    }

    /// Convenience method for listing computed graphs in a project
    pub async fn list_computed(&self, project_id: i32) -> Result<Vec<graph_data::Model>, sea_orm::DbErr> {
        self.list_by_project_and_source(project_id, "computed").await
    }

    /// Mark a graph_data as processing (transitional status)
    pub async fn mark_processing(&self, graph_data_id: i32) -> Result<(), sea_orm::DbErr> {
        self.mark_status(graph_data_id, GraphDataStatus::Processing, None).await
    }

    /// Mark a graph_data as error with an error message
    pub async fn mark_error(&self, graph_data_id: i32, error: String) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.status = Set(GraphDataStatus::Error.into());
        model.error_message = Set(Some(error));
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    /// Create a computed graph_data entry (convenience wrapper)
    pub async fn create_computed(
        &self,
        project_id: i32,
        dag_node_id: String,
        name: String,
    ) -> Result<graph_data::Model, sea_orm::DbErr> {
        self.create(GraphDataCreate {
            project_id,
            name,
            source_type: "computed".to_string(),
            dag_node_id: Some(dag_node_id),
            file_format: None,
            origin: None,
            filename: None,
            blob: None,
            file_size: None,
            processed_at: None,
            source_hash: None,
            computed_date: None,
            last_edit_sequence: Some(0),
            has_pending_edits: Some(false),
            last_replay_at: None,
            metadata: None,
            annotations: None,
            status: Some(GraphDataStatus::Processing),
        })
        .await
    }

    /// Create a dataset graph_data entry from JSON (convenience wrapper)
    pub async fn create_from_json(
        &self,
        project_id: i32,
        name: String,
        metadata: Option<Value>,
    ) -> Result<graph_data::Model, sea_orm::DbErr> {
        self.create(GraphDataCreate {
            project_id,
            name,
            source_type: "dataset".to_string(),
            dag_node_id: None,
            file_format: Some("json".to_string()),
            origin: Some("api".to_string()),
            filename: None,
            blob: None,
            file_size: None,
            processed_at: Some(Utc::now()),
            source_hash: None,
            computed_date: None,
            last_edit_sequence: None,
            has_pending_edits: None,
            last_replay_at: None,
            metadata,
            annotations: None,
            status: Some(GraphDataStatus::Active),
        })
        .await
    }

    /// Update graph_data metadata after creating an edit
    ///
    /// Sets last_edit_sequence and optionally marks has_pending_edits
    pub async fn update_edit_metadata(
        &self,
        graph_data_id: i32,
        sequence_number: i32,
        applied: bool,
    ) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.last_edit_sequence = Set(sequence_number);
        // Only mark as pending if the edit is not yet applied
        if !applied {
            model.has_pending_edits = Set(true);
        }
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    /// Set the pending edits state for a graph_data
    pub async fn set_pending_state(
        &self,
        graph_data_id: i32,
        has_pending: bool,
    ) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.has_pending_edits = Set(has_pending);
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    /// Update last_replay_at timestamp for a graph_data
    pub async fn mark_replayed(&self, graph_data_id: i32) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.last_replay_at = Set(Some(Utc::now()));
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    /// Reset edit metadata for a graph_data (used when clearing edits)
    pub async fn reset_edit_metadata(&self, graph_data_id: i32) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.last_edit_sequence = Set(0);
        model.has_pending_edits = Set(false);
        model.last_replay_at = Set(None);
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    /// Get all edits for a graph_data in sequence order
    ///
    /// # Arguments
    /// * `graph_data_id` - ID of the graph_data
    /// * `unapplied_only` - If true, only return edits where applied=false
    pub async fn get_edits(
        &self,
        graph_data_id: i32,
        unapplied_only: bool,
    ) -> Result<Vec<graph_edits::Model>, sea_orm::DbErr> {
        let mut query = GraphEdits::find().filter(graph_edits::Column::GraphId.eq(graph_data_id));

        if unapplied_only {
            query = query.filter(graph_edits::Column::Applied.eq(false));
        }

        query
            .order_by_asc(graph_edits::Column::SequenceNumber)
            .all(&self.db)
            .await
    }

    /// Mark an edit as applied
    pub async fn mark_edit_applied(&self, edit_id: i32) -> Result<(), sea_orm::DbErr> {
        let edit = GraphEdits::find_by_id(edit_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "Edit {}",
                edit_id
            )))?;

        let mut active_model: graph_edits::ActiveModel = edit.into();
        active_model.applied = Set(true);
        active_model.update(&self.db).await.map(|_| ())
    }

    /// Get edit count for a graph_data
    pub async fn get_edit_count(
        &self,
        graph_data_id: i32,
        unapplied_only: bool,
    ) -> Result<u64, sea_orm::DbErr> {
        use sea_orm::PaginatorTrait;

        let mut query = GraphEdits::find().filter(graph_edits::Column::GraphId.eq(graph_data_id));

        if unapplied_only {
            query = query.filter(graph_edits::Column::Applied.eq(false));
        }

        query.count(&self.db).await
    }

    /// Replay all unapplied edits for a graph_data
    ///
    /// Returns a summary of the replay operation
    pub async fn replay_edits(&self, graph_data_id: i32) -> Result<ReplaySummary, sea_orm::DbErr> {
        info!("Starting replay of edits for graph_data {}", graph_data_id);

        // Get all unapplied edits in sequence order
        let edits = self.get_edits(graph_data_id, true).await?;
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
        let applicator = GraphDataEditApplicator::new(self.db.clone());

        // Apply each edit in sequence
        for edit in edits {
            debug!(
                "Replaying edit #{}: {} {} {}",
                edit.sequence_number, edit.operation, edit.target_type, edit.target_id
            );

            match applicator.apply_edit(&edit).await {
                Ok(ApplyResult::Success { message }) => {
                    summary.applied += 1;
                    summary.details.push(GraphDataEditResult {
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
                    summary.details.push(GraphDataEditResult {
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
                    summary.details.push(GraphDataEditResult {
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
                    summary.details.push(GraphDataEditResult {
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
        self.mark_replayed(graph_data_id).await?;

        if self.get_edit_count(graph_data_id, true).await? == 0 {
            self.set_pending_state(graph_data_id, false).await?;
        }

        info!(
            "Replay complete for graph_data {}: {} applied, {} skipped, {} failed",
            graph_data_id, summary.applied, summary.skipped, summary.failed
        );

        Ok(summary)
    }

    /// Clear all edits for a graph_data
    pub async fn clear_edits(&self, graph_data_id: i32) -> Result<u64, sea_orm::DbErr> {
        // Delete all edits
        let result = GraphEdits::delete_many()
            .filter(graph_edits::Column::GraphId.eq(graph_data_id))
            .exec(&self.db)
            .await?;

        // Reset metadata
        self.reset_edit_metadata(graph_data_id).await?;

        Ok(result.rows_affected)
    }
}

/// Summary of a replay operation
#[derive(Debug, Clone)]
pub struct ReplaySummary {
    pub total: usize,
    pub applied: usize,
    pub skipped: usize,
    pub failed: usize,
    pub details: Vec<GraphDataEditResult>,
}

/// Result of applying a single edit during replay (graph_data variant)
#[derive(Debug, Clone)]
pub struct GraphDataEditResult {
    pub sequence_number: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub result: String, // "success", "skipped", "failed"
    pub message: String,
}

pub struct GraphDataCreate {
    pub project_id: i32,
    pub name: String,
    pub source_type: String,
    pub dag_node_id: Option<String>,
    pub file_format: Option<String>,
    pub origin: Option<String>,
    pub filename: Option<String>,
    pub blob: Option<Vec<u8>>,
    pub file_size: Option<i64>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source_hash: Option<String>,
    pub computed_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_edit_sequence: Option<i32>,
    pub has_pending_edits: Option<bool>,
    pub last_replay_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<Value>,
    pub annotations: Option<Value>,
    pub status: Option<GraphDataStatus>,
}

pub struct GraphDataNodeInput {
    pub external_id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub is_partition: Option<bool>,
    pub belongs_to: Option<String>,
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub attributes: Option<Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct GraphDataEdgeInput {
    pub external_id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub attributes: Option<Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
