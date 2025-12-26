use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::plan_dag_edges;

use super::metadata::{DataType, EdgeMetadata};

// Plan DAG Edge Structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub metadata: EdgeMetadata,
    pub created_at: DateTime<Utc>,
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
            metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
