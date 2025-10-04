use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// DataSource row storage (normalized CSV data)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "datasource_rows")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub datasource_id: i32,
    pub row_number: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: serde_json::Value, // Row data as JSON
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::datasources::Entity",
        from = "Column::DatasourceId",
        to = "super::datasources::Column::Id"
    )]
    Datasources,
}

impl Related<super::datasources::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Datasources.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
