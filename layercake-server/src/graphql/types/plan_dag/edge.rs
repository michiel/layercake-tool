use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use layercake_core::database::entities::plan_dag_edges;

use super::metadata::{DataType, EdgeMetadata};

// Plan DAG Edge Structure
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    // Removed source_handle and target_handle for floating edges
    pub metadata: EdgeMetadata,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl From<plan_dag_edges::Model> for PlanDagEdge {
    fn from(model: plan_dag_edges::Model) -> Self {
        let metadata: EdgeMetadata =
            serde_json::from_str(&model.metadata_json).unwrap_or(EdgeMetadata {
                label: None,
                data_type: DataType::GraphData,
            });

        Self {
            id: model.id,
            source: model.source_node_id,
            target: model.target_node_id,
            // Removed source_handle and target_handle for floating edges
            metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<layercake_core::plan_dag::PlanDagEdge> for PlanDagEdge {
    fn from(edge: layercake_core::plan_dag::PlanDagEdge) -> Self {
        Self {
            id: edge.id,
            source: edge.source,
            target: edge.target,
            metadata: edge.metadata.into(),
            created_at: edge.created_at,
            updated_at: edge.updated_at,
        }
    }
}
