use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_data_edges")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub graph_data_id: i32,
    pub external_id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    #[sea_orm(column_type = "Text")]
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    #[sea_orm(column_type = "JsonBinary")]
    pub attributes: Option<serde_json::Value>,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::graph_data::Entity",
        from = "Column::GraphDataId",
        to = "super::graph_data::Column::Id"
    )]
    GraphData,
}

impl Related<super::graph_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
