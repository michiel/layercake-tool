use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vector_index_state")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: i32,
    pub status: String,
    pub last_indexed_at: Option<ChronoDateTimeUtc>,
    pub last_error: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub config: Option<serde_json::Value>,
    pub updated_at: ChronoDateTimeUtc,
    pub embedding_provider: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
