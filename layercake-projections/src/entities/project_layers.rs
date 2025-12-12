use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Project-level layer palette rows, shared with graph rendering
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project_layers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub layer_id: String,
    pub name: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    pub alias: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub enabled: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
