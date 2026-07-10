use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_layers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub graph_id: i32,
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub alias: Option<String>,
    pub comment: Option<String>,
    pub dataset_id: Option<i32>,
    pub properties: Option<String>, // JSON
}

// NOTE: The `graph_layers` table was dropped in
// m20251215_000001_drop_legacy_graph_tables. This entity is retained only until
// the per-graph layer-editing surface is migrated to `project_layers`
// (tracked in plans/20260710-phase0-graph-data-cutover.md, WS3 deferred item).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
