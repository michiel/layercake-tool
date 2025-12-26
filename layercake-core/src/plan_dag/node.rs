use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::plan_dag_nodes;

use super::metadata::{DataSetExecutionMetadata, GraphExecutionMetadata, NodeMetadata};
use super::position::Position;
use super::PlanDagNodeType;

// Plan DAG Node Structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagNode {
    pub id: String,
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub source_position: Option<String>,
    pub target_position: Option<String>,
    pub metadata: NodeMetadata,
    pub config: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dataset_execution: Option<DataSetExecutionMetadata>,
    pub graph_execution: Option<GraphExecutionMetadata>,
}

// Conversions from database entities
impl From<plan_dag_nodes::Model> for PlanDagNode {
    fn from(model: plan_dag_nodes::Model) -> Self {
        let node_type = match model.node_type.as_str() {
            "DataSetNode" => PlanDagNodeType::DataSet,
            "GraphNode" => PlanDagNodeType::Graph,
            "TransformNode" => PlanDagNodeType::Transform,
            "FilterNode" => PlanDagNodeType::Filter,
            "MergeNode" => PlanDagNodeType::Merge,
            "GraphArtefactNode" | "OutputNode" | "Output" => PlanDagNodeType::GraphArtefact,
            "TreeArtefactNode" => PlanDagNodeType::TreeArtefact,
            "ProjectionNode" => PlanDagNodeType::Projection,
            "StoryNode" => PlanDagNodeType::Story,
            "SequenceArtefactNode" => PlanDagNodeType::SequenceArtefact,
            _ => PlanDagNodeType::DataSet,
        };

        let metadata: NodeMetadata =
            serde_json::from_str(&model.metadata_json).unwrap_or_else(|_| NodeMetadata {
                label: "Unnamed Node".to_string(),
                description: None,
            });

        Self {
            id: model.id,
            node_type,
            position: Position {
                x: model.position_x,
                y: model.position_y,
            },
            source_position: model.source_position,
            target_position: model.target_position,
            metadata,
            config: model.config_json,
            created_at: model.created_at,
            updated_at: model.updated_at,
            dataset_execution: None,
            graph_execution: None,
        }
    }
}
