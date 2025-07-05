use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_snapshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub is_automatic: bool,
    pub created_at: ChronoDateTimeUtc,
    pub created_by: Option<String>,
    pub node_count: i32,
    pub edge_count: i32,
    pub layer_count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Get a formatted version string
    pub fn version_string(&self) -> String {
        format!("v{}", self.version)
    }

    /// Check if this is an automatic snapshot
    pub fn is_auto_snapshot(&self) -> bool {
        self.is_automatic
    }

    /// Get total entity count
    pub fn total_entities(&self) -> i32 {
        self.node_count + self.edge_count + self.layer_count
    }

    /// Get a summary description
    pub fn summary(&self) -> String {
        match &self.description {
            Some(desc) => desc.clone(),
            None => format!(
                "Snapshot with {} nodes, {} edges, {} layers",
                self.node_count, self.edge_count, self.layer_count
            ),
        }
    }
}
