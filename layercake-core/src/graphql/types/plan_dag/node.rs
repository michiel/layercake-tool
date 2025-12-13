use async_graphql::*;
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
    pub config: String, // JSON string for now, will be parsed as NodeConfig
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Optional execution metadata populated by query resolver
    pub dataset_execution: Option<DataSetExecutionMetadata>,
    pub graph_execution: Option<GraphExecutionMetadata>,
}

#[Object]
impl PlanDagNode {
    async fn id(&self) -> &str {
        &self.id
    }

    #[graphql(name = "nodeType")]
    async fn node_type(&self) -> PlanDagNodeType {
        self.node_type
    }

    async fn position(&self) -> &Position {
        &self.position
    }

    #[graphql(name = "sourcePosition")]
    async fn source_position(&self) -> Option<&String> {
        self.source_position.as_ref()
    }

    #[graphql(name = "targetPosition")]
    async fn target_position(&self) -> Option<&String> {
        self.target_position.as_ref()
    }

    async fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }

    async fn config(&self) -> &str {
        &self.config
    }

    #[graphql(name = "createdAt")]
    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[graphql(name = "updatedAt")]
    async fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(name = "datasetExecution")]
    async fn dataset_execution(&self) -> Option<&DataSetExecutionMetadata> {
        self.dataset_execution.as_ref()
    }

    #[graphql(name = "graphExecution")]
    async fn graph_execution(&self) -> Option<&GraphExecutionMetadata> {
        self.graph_execution.as_ref()
    }

    /// Parse the config JSON into a specific node configuration type
    async fn parsed_config(&self) -> Result<String> {
        // For now, return the raw JSON. In the future, this could parse to specific types
        Ok(self.config.clone())
    }
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
            _ => PlanDagNodeType::DataSet, // Default fallback
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
