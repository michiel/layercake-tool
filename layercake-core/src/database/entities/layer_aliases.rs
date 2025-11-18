use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "layer_aliases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub alias_layer_id: String,
    pub target_layer_id: i32,
    pub created_at: ChronoDateTimeUtc,
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
        belongs_to = "super::project_layers::Entity",
        from = "Column::TargetLayerId",
        to = "super::project_layers::Column::Id"
    )]
    TargetLayer,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::project_layers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TargetLayer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
