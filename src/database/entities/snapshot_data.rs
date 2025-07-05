use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "snapshot_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub snapshot_id: i32,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_data: Json,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::graph_snapshots::Entity",
        from = "Column::SnapshotId",
        to = "super::graph_snapshots::Column::Id"
    )]
    GraphSnapshots,
}

impl Related<super::graph_snapshots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphSnapshots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Get the entity type as an enum
    pub fn get_entity_type(&self) -> super::graph_versions::EntityType {
        super::graph_versions::EntityType::from(self.entity_type.clone())
    }

    /// Get a summary of the stored data
    pub fn data_summary(&self) -> String {
        format!(
            "{} {} ({})",
            self.entity_type.to_uppercase(),
            self.entity_id,
            self.entity_data.to_string().len()
        )
    }
}
