use super::project::Entity as ProjectEntity;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plan")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    #[sea_orm(column_type = "Text")]
    pub plan_data: String, // JSON string representation of Plan
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Project,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::belongs_to(ProjectEntity)
                .from(Column::ProjectId)
                .to(super::project::Column::Id)
                .into(),
        }
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// Implementation for converting between Plan and Model
impl Model {
    pub fn to_plan(&self) -> Result<crate::plan::Plan, serde_json::Error> {
        serde_json::from_str(&self.plan_data)
    }

    pub fn from_plan(project_id: i32, plan: &crate::plan::Plan) -> Result<Self, serde_json::Error> {
        let plan_data = serde_json::to_string(plan)?;
        Ok(Self {
            id: 0, // Will be set by DB
            project_id,
            plan_data,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
}

