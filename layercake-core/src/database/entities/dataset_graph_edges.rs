use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dataset_graph_edges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub dataset_id: i32,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub weight: i32,
    pub comment: Option<String>,
    pub dataset: Option<i32>,
    #[sea_orm(column_type = "JsonBinary")]
    pub attributes: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
