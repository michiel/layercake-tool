use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::plans::Entity")]
    Plans,
    #[sea_orm(has_many = "super::nodes::Entity")]
    Nodes,
    #[sea_orm(has_many = "super::edges::Entity")]
    Edges,
    #[sea_orm(has_many = "super::layers::Entity")]
    Layers,
}

impl Related<super::plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl Related<super::edges::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Edges.def()
    }
}

impl Related<super::layers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Layers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            name: ActiveValue::NotSet,
            description: ActiveValue::NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        }
    }

    pub fn set_updated_at(mut self) -> Self {
        self.updated_at = Set(chrono::Utc::now());
        self
    }
}