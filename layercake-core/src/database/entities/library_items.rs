use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};

/// Generic library items shared between datasets, sample projects, and templates.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "library_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// dataset, project, or project_template
    pub item_type: String,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", default_value = "[]")]
    pub tags: String,
    #[sea_orm(column_type = "Text", default_value = "{}")]
    pub metadata: String,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub content_blob: Vec<u8>,
    pub content_size: Option<i64>,
    pub content_type: Option<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new() -> Self {
        Self {
            id: ActiveValue::NotSet,
            item_type: ActiveValue::NotSet,
            name: ActiveValue::NotSet,
            description: ActiveValue::NotSet,
            tags: ActiveValue::NotSet,
            metadata: ActiveValue::NotSet,
            content_blob: ActiveValue::NotSet,
            content_size: ActiveValue::NotSet,
            content_type: ActiveValue::NotSet,
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        }
    }
}
