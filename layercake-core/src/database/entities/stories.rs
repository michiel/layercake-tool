use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", default_value = "[]")]
    pub tags: String, // JSON array of strings
    #[sea_orm(column_type = "Text", default_value = "[]")]
    pub enabled_dataset_ids: String, // JSON array of dataset IDs
    #[sea_orm(column_type = "Text", default_value = "{}")]
    pub layer_config: String, // JSON layer configuration
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
    #[sea_orm(has_many = "super::sequences::Entity")]
    Sequences,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::sequences::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sequences.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
