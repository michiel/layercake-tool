use super::{graph::Entity as GraphEntity, plan::Entity as PlanEntity};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Plan,
    Graph,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Plan => Entity::has_one(PlanEntity)
                .from(Column::Id)
                .to(super::plan::Column::ProjectId)
                .into(),
            Self::Graph => Entity::has_one(GraphEntity)
                .from(Column::Id)
                .to(super::graph::Column::ProjectId)
                .into(),
        }
    }
}

impl Related<super::plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plan.def()
    }
}

impl Related<super::graph::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Graph.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

