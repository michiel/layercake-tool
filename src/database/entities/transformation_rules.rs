use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transformation_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub pipeline_id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_data: String, // JSON string
    pub enabled: bool,
    pub order_index: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transformation_pipelines::Entity",
        from = "Column::PipelineId",
        to = "super::transformation_pipelines::Column::Id"
    )]
    TransformationPipelines,
}

impl Related<super::transformation_pipelines::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransformationPipelines.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}