use super::execution_state::ExecutionState;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Set};
use serde::{Deserialize, Serialize};

/// Datasource entity for DAG DataSourceNode execution tracking
///
/// This entity represents datasource nodes in the execution DAG, NOT the uploaded
/// data files themselves (those are in the `data_sources` table). Each record tracks
/// the execution state and metadata for a datasource node reference in the pipeline.
///
/// Related entities:
/// - `data_sources`: The actual uploaded file data this datasource references
/// - `plan_dag_nodes`: The DAG node definition this datasource implements
/// - `graphs`: Graph nodes that may consume this datasource's output
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "datasources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub node_id: String, // Links to plan_dag_nodes.id
    pub name: String,
    pub file_path: String,
    pub file_type: String, // 'nodes' or 'edges'
    pub import_date: Option<ChronoDateTimeUtc>,
    pub row_count: Option<i32>,
    #[sea_orm(column_type = "JsonBinary")]
    pub column_info: Option<serde_json::Value>, // Schema: [{name, type, nullable}, ...]
    pub execution_state: String, // 'not_started', 'pending', 'processing', 'completed', 'error'
    #[sea_orm(column_type = "Text")]
    pub error_message: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Option<serde_json::Value>,
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
    #[sea_orm(has_many = "super::datasource_rows::Entity")]
    DatasourceRows,
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

impl Related<super::datasource_rows::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DatasourceRows.def()
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
            file_path: ActiveValue::NotSet,
            file_type: ActiveValue::NotSet,
            import_date: ActiveValue::NotSet,
            row_count: ActiveValue::NotSet,
            column_info: ActiveValue::NotSet,
            execution_state: Set(ExecutionState::NotStarted.as_str().to_string()),
            error_message: ActiveValue::NotSet,
            metadata: ActiveValue::NotSet,
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

    pub fn set_completed(mut self, row_count: i32, column_info: serde_json::Value) -> Self {
        self.execution_state = Set(ExecutionState::Completed.as_str().to_string());
        self.import_date = Set(Some(chrono::Utc::now()));
        self.row_count = Set(Some(row_count));
        self.column_info = Set(Some(column_info));
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

    /// Check if the datasource is ready for use
    pub fn is_ready(&self) -> bool {
        self.get_execution_state() == ExecutionState::Completed && self.row_count.is_some()
    }

    /// Check if the datasource has an error
    pub fn has_error(&self) -> bool {
        self.get_execution_state() == ExecutionState::Error
    }

    /// Check if the datasource is currently processing
    pub fn is_processing(&self) -> bool {
        matches!(
            self.get_execution_state(),
            ExecutionState::Pending | ExecutionState::Processing
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_status_checks() {
        let mut model = Model {
            id: 1,
            project_id: 1,
            node_id: "test_node".to_string(),
            name: "Test".to_string(),
            file_path: "/test/path.csv".to_string(),
            file_type: "nodes".to_string(),
            import_date: Some(chrono::Utc::now()),
            row_count: Some(100),
            column_info: None,
            execution_state: ExecutionState::Completed.as_str().to_string(),
            error_message: None,
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(model.is_ready());
        assert!(!model.has_error());
        assert!(!model.is_processing());

        model.execution_state = ExecutionState::Error.as_str().to_string();
        assert!(!model.is_ready());
        assert!(model.has_error());
        assert!(!model.is_processing());

        model.execution_state = ExecutionState::Processing.as_str().to_string();
        assert!(!model.is_ready());
        assert!(!model.has_error());
        assert!(model.is_processing());
    }
}
