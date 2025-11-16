use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Set};
use serde::{Deserialize, Serialize};

use super::execution_state::ExecutionState;

/// Graph entity for DAG GraphNode execution tracking
///
/// This entity represents graph nodes in the execution DAG. Each record tracks
/// the execution state, computed results, and metadata for a graph transformation
/// node in the pipeline.
///
/// Related entities:
/// - `plan_dag_nodes`: The DAG node definition this graph implements
/// - `graph_nodes`: Individual nodes in the computed graph output
/// - `graph_edges`: Individual edges in the computed graph output
/// - `layers`: Layer definitions for the computed graph
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graphs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub node_id: String, // Links to plan_dag_nodes.id
    pub name: String,
    pub execution_state: String, // 'not_started', 'pending', 'processing', 'completed', 'error'
    pub computed_date: Option<ChronoDateTimeUtc>,
    pub source_hash: Option<String>, // Hash of upstream data for change detection
    pub node_count: i32,
    pub edge_count: i32,
    #[sea_orm(column_type = "Text")]
    pub error_message: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Option<serde_json::Value>,
    #[sea_orm(column_type = "Text")]
    pub annotations: Option<String>,
    pub last_edit_sequence: i32,
    pub has_pending_edits: bool,
    pub last_replay_at: Option<ChronoDateTimeUtc>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
    #[sea_orm(
        belongs_to = "super::plan_dag_nodes::Entity",
        from = "Column::NodeId",
        to = "super::plan_dag_nodes::Column::Id"
    )]
    PlanDagNodes,
    #[sea_orm(has_many = "super::graph_nodes::Entity")]
    GraphNodes,
    #[sea_orm(has_many = "super::graph_edges::Entity")]
    GraphEdges,
    #[sea_orm(has_many = "super::graph_edits::Entity")]
    GraphEdits,
    #[sea_orm(has_many = "super::graph_layers::Entity")]
    GraphLayers,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::plan_dag_nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlanDagNodes.def()
    }
}

impl Related<super::graph_nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphNodes.def()
    }
}

impl Related<super::graph_edges::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphEdges.def()
    }
}

impl Related<super::graph_edits::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphEdits.def()
    }
}

impl Related<super::graph_layers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphLayers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            project_id: ActiveValue::NotSet,
            node_id: ActiveValue::NotSet,
            name: ActiveValue::NotSet,
            execution_state: Set(ExecutionState::NotStarted.as_str().to_string()),
            computed_date: ActiveValue::NotSet,
            source_hash: ActiveValue::NotSet,
            node_count: Set(0),
            edge_count: Set(0),
            error_message: ActiveValue::NotSet,
            metadata: ActiveValue::NotSet,
            annotations: ActiveValue::NotSet,
            last_edit_sequence: Set(0),
            has_pending_edits: Set(false),
            last_replay_at: ActiveValue::NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        }
    }

    pub fn set_updated_at(mut self) -> Self {
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_state(mut self, state: ExecutionState) -> Self {
        self.execution_state = Set(state.as_str().to_string());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_completed(mut self, source_hash: String, node_count: i32, edge_count: i32) -> Self {
        self.execution_state = Set(ExecutionState::Completed.as_str().to_string());
        self.computed_date = Set(Some(chrono::Utc::now()));
        self.source_hash = Set(Some(source_hash));
        self.node_count = Set(node_count);
        self.edge_count = Set(edge_count);
        self.error_message = Set(None);
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_error(mut self, error_msg: String) -> Self {
        self.execution_state = Set(ExecutionState::Error.as_str().to_string());
        self.error_message = Set(Some(error_msg));
        self.updated_at = Set(chrono::Utc::now());
        self
    }
}

impl Model {
    /// Get the execution state as an enum
    pub fn get_execution_state(&self) -> ExecutionState {
        ExecutionState::from_str(&self.execution_state).unwrap_or(ExecutionState::NotStarted)
    }

    /// Check if the graph is ready for use
    pub fn is_ready(&self) -> bool {
        self.get_execution_state() == ExecutionState::Completed
    }

    /// Check if the graph has an error
    pub fn has_error(&self) -> bool {
        self.get_execution_state() == ExecutionState::Error
    }

    /// Check if the graph is currently processing
    pub fn is_processing(&self) -> bool {
        matches!(
            self.get_execution_state(),
            ExecutionState::Pending | ExecutionState::Processing
        )
    }
}
