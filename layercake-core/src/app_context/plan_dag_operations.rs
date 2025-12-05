use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::Value;
use uuid::Uuid;

use super::{AppContext, PlanDagNodeRequest, PlanDagNodeUpdateRequest, PlanDagSnapshot};
use super::{PlanDagEdgeRequest, PlanDagEdgeUpdateRequest, PlanDagNodePositionRequest};
use crate::database::entities::{data_sets, graphs, projects};
use crate::graphql::types::plan_dag::{
    DataSetExecutionMetadata, GraphExecutionMetadata, PlanDagEdge, PlanDagMetadata, PlanDagNode,
    PlanDagNodeType, Position,
};
use crate::services::plan_dag_service::PlanDagNodePositionUpdate;

fn node_type_prefix(node_type: &PlanDagNodeType) -> &'static str {
    match node_type {
        PlanDagNodeType::DataSet => "dataset",
        PlanDagNodeType::Graph => "graph",
        PlanDagNodeType::Transform => "transform",
        PlanDagNodeType::Filter => "filter",
        PlanDagNodeType::Merge => "merge",
        PlanDagNodeType::GraphArtefact => "graphartefact",
        PlanDagNodeType::TreeArtefact => "treeartefact",
        PlanDagNodeType::Story => "story",
        PlanDagNodeType::SequenceArtefact => "sequenceartefact",
    }
}

fn node_type_storage_name(node_type: &PlanDagNodeType) -> &'static str {
    match node_type {
        PlanDagNodeType::DataSet => "DataSetNode",
        PlanDagNodeType::Graph => "GraphNode",
        PlanDagNodeType::Transform => "TransformNode",
        PlanDagNodeType::Filter => "FilterNode",
        PlanDagNodeType::Merge => "MergeNode",
        PlanDagNodeType::GraphArtefact => "GraphArtefactNode",
        PlanDagNodeType::TreeArtefact => "TreeArtefactNode",
        PlanDagNodeType::Story => "StoryNode",
        PlanDagNodeType::SequenceArtefact => "SequenceArtefactNode",
    }
}

fn generate_node_id(
    node_type: &PlanDagNodeType,
    _existing_nodes: &[PlanDagNode],
) -> Result<String> {
    // Generate a globally unique ID using UUID to prevent collisions across projects/plans
    // Format: <node_type_prefix>_<uuid>
    let prefix = node_type_prefix(node_type);
    let uuid = Uuid::new_v4().simple().to_string();

    // Use first 12 characters of UUID for readability
    let short_uuid = uuid.chars().take(12).collect::<String>();

    Ok(format!("{}_{}", prefix, short_uuid))
}

fn generate_edge_id(_source: &str, _target: &str) -> String {
    // Generate a globally unique ID using UUID to prevent collisions
    // Format: edge_<uuid>
    let uuid = Uuid::new_v4().simple().to_string();

    // Use first 12 characters of UUID for readability while maintaining uniqueness
    let short_uuid = uuid.chars().take(12).collect::<String>();

    format!("edge_{}", short_uuid)
}

impl AppContext {
    // ----- Plan DAG helpers -------------------------------------------------
    pub async fn load_plan_dag(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> Result<Option<PlanDagSnapshot>> {
        let project = match projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", project_id, e))?
        {
            Some(project) => project,
            None => return Ok(None),
        };

        let plan = match plan_id {
            Some(plan_id) => Some(self.resolve_plan_model(project_id, Some(plan_id)).await?),
            None => self
                .plan_service
                .get_default_plan(project_id)
                .await
                .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?,
        };

        if let Some(plan) = plan {
            let mut nodes = self
                .plan_dag_service
                .get_nodes(project_id, Some(plan.id))
                .await
                .map_err(|e| anyhow!("Failed to load Plan DAG nodes: {}", e))?;
            let edges = self
                .plan_dag_service
                .get_edges(project_id, Some(plan.id))
                .await
                .map_err(|e| anyhow!("Failed to load Plan DAG edges: {}", e))?;

            for idx in 0..nodes.len() {
                let node_type = nodes[idx].node_type;
                let node_id = nodes[idx].id.clone();

                match node_type {
                    PlanDagNodeType::DataSet => {
                        if let Ok(config) = serde_json::from_str::<Value>(&nodes[idx].config) {
                            if let Some(data_set_id) = config
                                .get("dataSetId")
                                .and_then(|v| v.as_i64())
                                .map(|v| v as i32)
                            {
                                if let Some(data_set) = data_sets::Entity::find_by_id(data_set_id)
                                    .one(&self.db)
                                    .await
                                    .map_err(|e| {
                                        anyhow!("Failed to load data set {}: {}", data_set_id, e)
                                    })?
                                {
                                    let execution_state = match data_set.status.as_str() {
                                        "active" => "completed",
                                        "processing" => "processing",
                                        "error" => "error",
                                        _ => "not_started",
                                    }
                                    .to_string();

                                    nodes[idx].dataset_execution = Some(DataSetExecutionMetadata {
                                        data_set_id: data_set.id,
                                        filename: data_set.filename.clone(),
                                        status: data_set.status.clone(),
                                        processed_at: data_set.processed_at.map(|d| d.to_rfc3339()),
                                        execution_state,
                                        error_message: data_set.error_message.clone(),
                                    });
                                }
                            }
                        }
                    }
                    PlanDagNodeType::Graph => {
                        if let Some(graph) = graphs::Entity::find()
                            .filter(graphs::Column::ProjectId.eq(project_id))
                            .filter(graphs::Column::NodeId.eq(node_id.clone()))
                            .one(&self.db)
                            .await
                            .map_err(|e| {
                                anyhow!(
                                    "Failed to load graph execution for node {}: {}",
                                    node_id,
                                    e
                                )
                            })?
                        {
                            nodes[idx].graph_execution = Some(GraphExecutionMetadata {
                                graph_id: graph.id,
                                node_count: graph.node_count,
                                edge_count: graph.edge_count,
                                execution_state: graph.execution_state.clone(),
                                computed_date: graph.computed_date.map(|d| d.to_rfc3339()),
                                error_message: graph.error_message.clone(),
                                annotations: graph.annotations.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            let metadata = PlanDagMetadata {
                version: plan.version.to_string(),
                name: Some(plan.name.clone()),
                description: None,
                created: Some(plan.created_at.to_rfc3339()),
                last_modified: Some(plan.updated_at.to_rfc3339()),
                author: None,
            };

            Ok(Some(PlanDagSnapshot {
                version: metadata.version.clone(),
                nodes,
                edges,
                metadata,
            }))
        } else {
            let metadata = PlanDagMetadata {
                version: "1.0".to_string(),
                name: Some(format!("{} Plan DAG", project.name)),
                description: project.description.clone(),
                created: Some(project.created_at.to_rfc3339()),
                last_modified: Some(project.updated_at.to_rfc3339()),
                author: None,
            };

            Ok(Some(PlanDagSnapshot {
                version: metadata.version.clone(),
                nodes: Vec::new(),
                edges: Vec::new(),
                metadata,
            }))
        }
    }

    // ----- Plan DAG mutations ----------------------------------------------

    pub async fn create_plan_dag_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        request: PlanDagNodeRequest,
    ) -> Result<PlanDagNode> {
        let existing_nodes = self
            .plan_dag_service
            .get_nodes(project_id, plan_id)
            .await
            .unwrap_or_default();

        let node_id = generate_node_id(&request.node_type, &existing_nodes)?;
        let node_type = node_type_storage_name(&request.node_type).to_string();
        let metadata_json = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Invalid node metadata: {}", e))?;
        let config_json = serde_json::to_string(&request.config)
            .map_err(|e| anyhow!("Invalid node config: {}", e))?;

        self.plan_dag_service
            .create_node(
                project_id,
                plan_id,
                node_id,
                node_type,
                request.position,
                metadata_json,
                config_json,
            )
            .await
    }

    pub async fn update_plan_dag_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        updates: PlanDagNodeUpdateRequest,
    ) -> Result<PlanDagNode> {
        let metadata_json = if let Some(metadata) = updates.metadata {
            Some(
                serde_json::to_string(&metadata)
                    .map_err(|e| anyhow!("Invalid node metadata: {}", e))?,
            )
        } else {
            None
        };

        let config_json = if let Some(config) = updates.config {
            Some(
                serde_json::to_string(&config)
                    .map_err(|e| anyhow!("Invalid node config: {}", e))?,
            )
        } else {
            None
        };

        self.plan_dag_service
            .update_node(
                project_id,
                plan_id,
                node_id,
                updates.position,
                metadata_json,
                config_json,
            )
            .await
    }

    pub async fn delete_plan_dag_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
    ) -> Result<PlanDagNode> {
        self.plan_dag_service
            .delete_node(project_id, plan_id, node_id)
            .await
    }

    pub async fn move_plan_dag_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        position: Position,
    ) -> Result<PlanDagNode> {
        self.plan_dag_service
            .move_node(project_id, plan_id, node_id, position)
            .await
    }

    pub async fn batch_move_plan_dag_nodes(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        positions: Vec<PlanDagNodePositionRequest>,
    ) -> Result<Vec<PlanDagNode>> {
        let updates = positions
            .into_iter()
            .map(|p| PlanDagNodePositionUpdate {
                node_id: p.node_id,
                position: p.position,
                source_position: p.source_position,
                target_position: p.target_position,
            })
            .collect();

        self.plan_dag_service
            .batch_move_nodes(project_id, plan_id, updates)
            .await
    }

    pub async fn create_plan_dag_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        request: PlanDagEdgeRequest,
    ) -> Result<PlanDagEdge> {
        let edge_id = generate_edge_id(&request.source, &request.target);
        let metadata_json = serde_json::to_string(&request.metadata)
            .map_err(|e| anyhow!("Invalid edge metadata: {}", e))?;

        self.plan_dag_service
            .create_edge(
                project_id,
                plan_id,
                edge_id,
                request.source,
                request.target,
                metadata_json,
            )
            .await
    }

    pub async fn update_plan_dag_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
        updates: PlanDagEdgeUpdateRequest,
    ) -> Result<PlanDagEdge> {
        let metadata_json = if let Some(metadata) = updates.metadata {
            Some(
                serde_json::to_string(&metadata)
                    .map_err(|e| anyhow!("Invalid edge metadata: {}", e))?,
            )
        } else {
            None
        };

        self.plan_dag_service
            .update_edge(project_id, plan_id, edge_id, metadata_json)
            .await
    }

    pub async fn delete_plan_dag_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
    ) -> Result<PlanDagEdge> {
        self.plan_dag_service
            .delete_edge(project_id, plan_id, edge_id)
            .await
    }
}
