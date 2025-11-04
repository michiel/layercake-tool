use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "system_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub label: String,
    pub category: String,
    pub description: Option<String>,
    pub is_secret: bool,
    pub is_read_only: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
