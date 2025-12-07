use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "code_analysis_profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub project_id: i32,
    pub file_path: String,
    pub dataset_id: Option<i32>,
    pub last_run: Option<ChronoDateTimeUtc>,
    pub report: Option<String>,
    pub no_infra: Option<bool>,
    pub options: Option<String>,
    pub analysis_type: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
