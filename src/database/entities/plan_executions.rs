use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plan_executions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub plan_id: i32,
    pub execution_id: String,
    pub status: String,
    pub progress: Option<i32>,
    pub started_at: Option<ChronoDateTimeUtc>,
    pub completed_at: Option<ChronoDateTimeUtc>,
    pub error: Option<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plans::Entity",
        from = "Column::PlanId",
        to = "super::plans::Column::Id"
    )]
    Plans,
}

impl Related<super::plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<ExecutionStatus> for String {
    fn from(status: ExecutionStatus) -> Self {
        match status {
            ExecutionStatus::Queued => "queued".to_string(),
            ExecutionStatus::Running => "running".to_string(),
            ExecutionStatus::Completed => "completed".to_string(),
            ExecutionStatus::Failed => "failed".to_string(),
            ExecutionStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl From<String> for ExecutionStatus {
    fn from(status: String) -> Self {
        match status.as_str() {
            "queued" => ExecutionStatus::Queued,
            "running" => ExecutionStatus::Running,
            "completed" => ExecutionStatus::Completed,
            "failed" => ExecutionStatus::Failed,
            "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Queued,
        }
    }
}

impl Model {
    pub fn get_status(&self) -> ExecutionStatus {
        ExecutionStatus::from(self.status.clone())
    }

    pub fn is_running(&self) -> bool {
        matches!(self.get_status(), ExecutionStatus::Running)
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.get_status(),
            ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled
        )
    }

    pub fn duration_seconds(&self) -> Option<i64> {
        if let (Some(started), Some(completed)) = (&self.started_at, &self.completed_at) {
            Some((completed.timestamp() - started.timestamp()).abs())
        } else {
            None
        }
    }
}