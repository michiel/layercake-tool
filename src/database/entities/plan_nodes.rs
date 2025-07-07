use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[sea_orm(table_name = "plan_nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub plan_id: i32,
    pub node_type: String, // "import", "transformation", "export"
    pub name: String,
    pub description: Option<String>,
    pub configuration: String, // JSON configuration for the node
    pub graph_id: Option<String>, // Reference to the graph this node produces/consumes
    pub position_x: Option<f64>, // For visual layout in plan editor
    pub position_y: Option<f64>, // For visual layout in plan editor
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::plans::Entity",
        from = "Column::PlanId",
        to = "super::plans::Column::Id"
    )]
    Plans,
    #[sea_orm(
        belongs_to = "super::graphs::Entity",
        from = "Column::GraphId",
        to = "super::graphs::Column::Id"
    )]
    Graphs,
}

impl Related<super::plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl Related<super::graphs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Graphs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse node configuration as JSON
    pub fn get_configuration_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.configuration)
    }

    /// Set node configuration from JSON
    pub fn set_configuration_json(&mut self, json_value: &serde_json::Value) -> Result<(), serde_json::Error> {
        self.configuration = serde_json::to_string_pretty(json_value)?;
        Ok(())
    }

    /// Check if this node type produces a graph
    pub fn produces_graph(&self) -> bool {
        matches!(self.node_type.as_str(), "import" | "transformation")
    }

    /// Check if this node type consumes a graph
    pub fn consumes_graph(&self) -> bool {
        matches!(self.node_type.as_str(), "transformation" | "export")
    }

    /// Get the node type as an enum
    pub fn get_node_type(&self) -> PlanNodeType {
        match self.node_type.as_str() {
            "import" => PlanNodeType::Import,
            "transformation" => PlanNodeType::Transformation,
            "export" => PlanNodeType::Export,
            _ => PlanNodeType::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlanNodeType {
    Import,
    Transformation,
    Export,
    Unknown,
}

impl std::fmt::Display for PlanNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanNodeType::Import => write!(f, "import"),
            PlanNodeType::Transformation => write!(f, "transformation"),
            PlanNodeType::Export => write!(f, "export"),
            PlanNodeType::Unknown => write!(f, "unknown"),
        }
    }
}