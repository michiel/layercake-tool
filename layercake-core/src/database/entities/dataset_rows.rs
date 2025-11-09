use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// DataSet row storage (normalized CSV data)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dataset_rows")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub dataset_node_id: i32,
    pub row_number: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: serde_json::Value, // Row data as JSON
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::datasets::Entity",
        from = "Column::DatasetNodeId",
        to = "super::datasets::Column::Id"
    )]
    DatasetNodes,
}

impl Related<super::datasets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DatasetNodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
