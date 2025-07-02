use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub node_id: String,
    pub label: String,
    pub layer_id: Option<String>,
    pub properties: Option<String>, // JSON
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
        belongs_to = "super::layers::Entity",
        from = "Column::LayerId",
        to = "super::layers::Column::LayerId"
    )]
    Layers,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::layers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Layers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}