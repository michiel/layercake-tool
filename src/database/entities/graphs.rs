use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "graphs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub plan_id: i32,
    pub plan_node_id: String, // The plan node that produced this graph
    pub name: String,
    pub description: Option<String>,
    pub graph_data: String, // JSON string containing nodes, edges, layers
    pub metadata: Option<String>, // JSON metadata (statistics, provenance, etc.)
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
        belongs_to = "super::plan_nodes::Entity",
        from = "Column::PlanNodeId",
        to = "super::plan_nodes::Column::Id"
    )]
    PlanNodes,
}

impl Related<super::plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plans.def()
    }
}

impl Related<super::plan_nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlanNodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse graph data as JSON
    pub fn get_graph_data_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.graph_data)
    }

    /// Set graph data from JSON
    pub fn set_graph_data_json(&mut self, json_value: &serde_json::Value) -> Result<(), serde_json::Error> {
        self.graph_data = serde_json::to_string_pretty(json_value)?;
        Ok(())
    }

    /// Parse metadata as JSON
    pub fn get_metadata_json(&self) -> Result<Option<serde_json::Value>, serde_json::Error> {
        match &self.metadata {
            Some(metadata) => Ok(Some(serde_json::from_str(metadata)?)),
            None => Ok(None),
        }
    }

    /// Set metadata from JSON
    pub fn set_metadata_json(&mut self, json_value: Option<&serde_json::Value>) -> Result<(), serde_json::Error> {
        self.metadata = match json_value {
            Some(value) => Some(serde_json::to_string_pretty(value)?),
            None => None,
        };
        Ok(())
    }

    /// Get basic statistics about the graph
    pub fn get_statistics(&self) -> Result<GraphStatistics, serde_json::Error> {
        let graph_data = self.get_graph_data_json()?;
        
        let nodes_count = graph_data.get("nodes")
            .and_then(|n| n.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
            
        let edges_count = graph_data.get("edges")
            .and_then(|e| e.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
            
        let layers_count = graph_data.get("layers")
            .and_then(|l| l.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(GraphStatistics {
            nodes_count,
            edges_count,
            layers_count,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub nodes_count: usize,
    pub edges_count: usize,
    pub layers_count: usize,
}