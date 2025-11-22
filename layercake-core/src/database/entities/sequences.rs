use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sequences")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub story_id: i32,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", default_value = "[]")]
    pub enabled_dataset_ids: String, // JSON array of dataset IDs enabled for this sequence
    #[sea_orm(column_type = "Text", default_value = "[]")]
    pub edge_order: String, // JSON array of { datasetId, edgeId } references
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stories::Entity",
        from = "Column::StoryId",
        to = "super::stories::Column::Id"
    )]
    Stories,
}

impl Related<super::stories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
