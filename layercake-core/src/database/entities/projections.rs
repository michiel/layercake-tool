use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Projection metadata linking a project and a graph to a visualization type
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub graph_id: i32,
    pub name: String,
    pub projection_type: String, // 'force3d' (default), 'layer3d'
    #[sea_orm(column_type = "JsonBinary")]
    pub settings_json: Option<serde_json::Value>,
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
        belongs_to = "super::graph_data::Entity",
        from = "Column::GraphId",
        to = "super::graph_data::Column::Id"
    )]
    GraphData,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::graph_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
