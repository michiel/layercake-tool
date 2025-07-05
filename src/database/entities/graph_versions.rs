use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graph_versions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub snapshot_id: Option<i32>,
    pub change_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub old_data: Option<Json>,
    pub new_data: Option<Json>,
    pub changed_at: ChronoDateTimeUtc,
    pub changed_by: Option<String>,
    pub change_description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
    #[sea_orm(
        belongs_to = "super::graph_snapshots::Entity",
        from = "Column::SnapshotId",
        to = "super::graph_snapshots::Column::Id"
    )]
    GraphSnapshots,
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::graph_snapshots::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GraphSnapshots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Update,
    Delete,
    Restore,
}

impl From<ChangeType> for String {
    fn from(change_type: ChangeType) -> Self {
        match change_type {
            ChangeType::Create => "create".to_string(),
            ChangeType::Update => "update".to_string(),
            ChangeType::Delete => "delete".to_string(),
            ChangeType::Restore => "restore".to_string(),
        }
    }
}

impl From<String> for ChangeType {
    fn from(change_type: String) -> Self {
        match change_type.as_str() {
            "create" => ChangeType::Create,
            "update" => ChangeType::Update,
            "delete" => ChangeType::Delete,
            "restore" => ChangeType::Restore,
            _ => ChangeType::Update,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Node,
    Edge,
    Layer,
}

impl From<EntityType> for String {
    fn from(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Node => "node".to_string(),
            EntityType::Edge => "edge".to_string(),
            EntityType::Layer => "layer".to_string(),
        }
    }
}

impl From<String> for EntityType {
    fn from(entity_type: String) -> Self {
        match entity_type.as_str() {
            "node" => EntityType::Node,
            "edge" => EntityType::Edge,
            "layer" => EntityType::Layer,
            _ => EntityType::Node,
        }
    }
}

impl Model {
    /// Get the change type as an enum
    pub fn get_change_type(&self) -> ChangeType {
        ChangeType::from(self.change_type.clone())
    }

    /// Get the entity type as an enum
    pub fn get_entity_type(&self) -> EntityType {
        EntityType::from(self.entity_type.clone())
    }

    /// Get a human-readable description of the change
    pub fn get_change_summary(&self) -> String {
        match &self.change_description {
            Some(desc) => desc.clone(),
            None => {
                let action = match self.get_change_type() {
                    ChangeType::Create => "created",
                    ChangeType::Update => "updated",
                    ChangeType::Delete => "deleted",
                    ChangeType::Restore => "restored",
                };
                format!(
                    "{} {} {}",
                    action.to_uppercase(),
                    self.entity_type,
                    self.entity_id
                )
            }
        }
    }

    /// Check if this change has data differences
    pub fn has_data_changes(&self) -> bool {
        self.old_data.is_some() || self.new_data.is_some()
    }
}
