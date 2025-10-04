use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Graph node storage
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub graph_id: i32,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub is_partition: bool,
    #[sea_orm(column_type = "JsonBinary")]
    pub attrs: Option<serde_json::Value>,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::graphs::Entity",
        from = "Column::GraphId",
        to = "super::graphs::Column::Id"
    )]
    Graphs,
}

impl Related<super::graphs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Graphs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
