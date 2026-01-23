use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    app_context::{
        AppContext, DataSetSummary, PlanDagEdgeRequest, PlanDagEdgeUpdateRequest,
        PlanDagNodeRequest, PlanDagNodeUpdateRequest, PlanDagSnapshot, PlanSummary,
    },
    auth::Actor,
    errors::{CoreError, CoreResult},
    plan::{ExportFileType, RenderConfig},
    plan_dag::{PlanDagEdge, PlanDagMetadata, PlanDagNode, PlanDagNodeType, Position},
};

/// Shared context injected into CLI helpers that mirror the GraphQL bindings.
#[derive(Clone)]
pub struct CliContext {
    pub app: Arc<AppContext>,
    pub actor: Actor,
    pub session_id: Option<String>,
}

impl CliContext {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            app,
            actor: Actor::system(),
            session_id: None,
        }
    }

    pub fn with_actor(mut self, actor: Actor) -> Self {
        self.actor = actor;
        self
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub async fn resolve_plan_id(&self, project_id: i32, plan_id: Option<i32>) -> CoreResult<i32> {
        let plan = self.app.resolve_plan_model(project_id, plan_id).await?;
        Ok(plan.id)
    }
}

/// Helpers to produce deterministic canonical identifiers for key artifacts.
pub fn canonical_dataset_id(project_id: i32, data_set_id: i32) -> String {
    format!("dataset:{project_id}:{data_set_id}")
}

pub fn canonical_plan_id(project_id: i32, plan_id: i32) -> String {
    format!("plan:{project_id}:{plan_id}")
}

pub fn canonical_plannode_id(project_id: i32, plan_id: i32, node_id: &str) -> String {
    format!("plannode:{project_id}:{plan_id}:{node_id}")
}

pub fn canonical_edge_id(project_id: i32, plan_id: i32, edge_id: &str) -> String {
    format!("edge:{project_id}:{plan_id}:{edge_id}")
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliDataSet {
    pub canonical_id: String,
    pub summary: DataSetSummary,
}

impl CliDataSet {
    pub fn from_summary(summary: DataSetSummary) -> Self {
        let canonical_id = canonical_dataset_id(summary.project_id, summary.id);
        Self {
            canonical_id,
            summary,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlan {
    pub canonical_id: String,
    pub summary: PlanSummary,
}

impl CliPlan {
    pub fn from_summary(summary: PlanSummary) -> Self {
        let canonical_id = canonical_plan_id(summary.project_id, summary.id);
        Self {
            canonical_id,
            summary,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanDagNode {
    pub canonical_id: String,
    pub node: PlanDagNode,
}

impl CliPlanDagNode {
    pub fn new(project_id: i32, plan_id: i32, node: PlanDagNode) -> Self {
        let canonical_id = canonical_plannode_id(project_id, plan_id, &node.id);
        Self { canonical_id, node }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanDagEdge {
    pub canonical_id: String,
    pub edge: PlanDagEdge,
}

impl CliPlanDagEdge {
    pub fn new(project_id: i32, plan_id: i32, edge: PlanDagEdge) -> Self {
        let canonical_id = canonical_edge_id(project_id, plan_id, &edge.id);
        Self { canonical_id, edge }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanDagSnapshot {
    pub canonical_plan_id: String,
    pub project_id: i32,
    pub plan_id: i32,
    pub version: String,
    pub metadata: PlanDagMetadata,
    pub nodes: Vec<CliPlanDagNode>,
    pub edges: Vec<CliPlanDagEdge>,
}

impl CliPlanDagSnapshot {
    pub fn from_snapshot(project_id: i32, plan_id: i32, snapshot: PlanDagSnapshot) -> Self {
        let nodes = snapshot
            .nodes
            .into_iter()
            .map(|node| CliPlanDagNode::new(project_id, plan_id, node))
            .collect();
        let edges = snapshot
            .edges
            .into_iter()
            .map(|edge| CliPlanDagEdge::new(project_id, plan_id, edge))
            .collect();

        Self {
            canonical_plan_id: canonical_plan_id(project_id, plan_id),
            project_id,
            plan_id,
            version: snapshot.version,
            metadata: snapshot.metadata,
            nodes,
            edges,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanNodeInput {
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub metadata: Value,
    pub config: Value,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanNodeUpdateInput {
    pub position: Option<Position>,
    pub metadata: Option<Value>,
    pub config: Option<Value>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanEdgeInput {
    pub source: String,
    pub target: String,
    pub metadata: Value,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliPlanEdgeUpdateInput {
    pub metadata: Option<Value>,
}

impl CliContext {
    pub async fn list_datasets(&self, project_id: i32) -> CoreResult<Vec<CliDataSet>> {
        let summaries = self.app.list_data_sets(project_id).await?;
        Ok(summaries
            .into_iter()
            .map(CliDataSet::from_summary)
            .collect())
    }

    pub async fn get_dataset(&self, data_set_id: i32) -> CoreResult<Option<CliDataSet>> {
        let summary = self.app.get_data_set(data_set_id).await?;
        Ok(summary.map(CliDataSet::from_summary))
    }

    pub async fn list_plans(&self, project_id: Option<i32>) -> CoreResult<Vec<CliPlan>> {
        let plans = self.app.list_plans(project_id).await?;
        Ok(plans.into_iter().map(CliPlan::from_summary).collect())
    }

    pub async fn get_plan(&self, plan_id: i32) -> CoreResult<Option<CliPlan>> {
        let plan = self.app.get_plan(plan_id).await?;
        Ok(plan.map(CliPlan::from_summary))
    }

    pub async fn load_plan_dag(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<CliPlanDagSnapshot> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;
        let snapshot = self
            .app
            .load_plan_dag(project_id, Some(resolved_plan_id))
            .await?
            .ok_or_else(|| CoreError::not_found("PlanDag", resolved_plan_id.to_string()))?;

        Ok(CliPlanDagSnapshot::from_snapshot(
            project_id,
            resolved_plan_id,
            snapshot,
        ))
    }

    pub async fn create_plan_node(
        &self,
        project_id: i32,
        plan_id: i32,
        input: CliPlanNodeInput,
    ) -> CoreResult<CliPlanDagNode> {
        let request = PlanDagNodeRequest {
            node_type: input.node_type,
            position: input.position,
            metadata: input.metadata,
            config: input.config,
        };

        let node = self
            .app
            .create_plan_dag_node(&self.actor, project_id, Some(plan_id), request)
            .await?;

        Ok(CliPlanDagNode::new(project_id, plan_id, node))
    }

    pub async fn update_plan_node(
        &self,
        project_id: i32,
        plan_id: i32,
        node_id: String,
        input: CliPlanNodeUpdateInput,
    ) -> CoreResult<CliPlanDagNode> {
        let request = PlanDagNodeUpdateRequest {
            position: input.position,
            metadata: input.metadata,
            config: input.config,
        };

        let node = self
            .app
            .update_plan_dag_node(&self.actor, project_id, Some(plan_id), node_id, request)
            .await?;

        Ok(CliPlanDagNode::new(project_id, plan_id, node))
    }

    pub async fn delete_plan_node(
        &self,
        project_id: i32,
        plan_id: i32,
        node_id: String,
    ) -> CoreResult<CliPlanDagNode> {
        let node = self
            .app
            .delete_plan_dag_node(&self.actor, project_id, Some(plan_id), node_id)
            .await?;

        Ok(CliPlanDagNode::new(project_id, plan_id, node))
    }

    pub async fn move_plan_node(
        &self,
        project_id: i32,
        plan_id: i32,
        node_id: String,
        position: Position,
    ) -> CoreResult<CliPlanDagNode> {
        let node = self
            .app
            .move_plan_dag_node(&self.actor, project_id, Some(plan_id), node_id, position)
            .await?;

        Ok(CliPlanDagNode::new(project_id, plan_id, node))
    }

    pub async fn create_plan_edge(
        &self,
        project_id: i32,
        plan_id: i32,
        input: CliPlanEdgeInput,
    ) -> CoreResult<CliPlanDagEdge> {
        let request = PlanDagEdgeRequest {
            source: input.source,
            target: input.target,
            metadata: input.metadata,
        };

        let edge = self
            .app
            .create_plan_dag_edge(&self.actor, project_id, Some(plan_id), request)
            .await?;

        Ok(CliPlanDagEdge::new(project_id, plan_id, edge))
    }

    pub async fn update_plan_edge(
        &self,
        project_id: i32,
        plan_id: i32,
        edge_id: String,
        input: CliPlanEdgeUpdateInput,
    ) -> CoreResult<CliPlanDagEdge> {
        let request = PlanDagEdgeUpdateRequest {
            metadata: input.metadata,
        };

        let edge = self
            .app
            .update_plan_dag_edge(&self.actor, project_id, Some(plan_id), edge_id, request)
            .await?;

        Ok(CliPlanDagEdge::new(project_id, plan_id, edge))
    }

    pub async fn delete_plan_edge(
        &self,
        project_id: i32,
        plan_id: i32,
        edge_id: String,
    ) -> CoreResult<CliPlanDagEdge> {
        let edge = self
            .app
            .delete_plan_dag_edge(&self.actor, project_id, Some(plan_id), edge_id)
            .await?;

        Ok(CliPlanDagEdge::new(project_id, plan_id, edge))
    }

    pub async fn preview_graph_export(
        &self,
        graph_id: i32,
        format: ExportFileType,
        render_config: Option<RenderConfig>,
        max_rows: Option<usize>,
    ) -> CoreResult<String> {
        self.app
            .preview_graph_export(&self.actor, graph_id, format, render_config, max_rows)
            .await
    }

    /// Phase 1.1: List nodes with optional filtering
    pub async fn list_nodes_filtered(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_type: Option<String>,
        label_pattern: Option<String>,
        bounds: Option<(f64, f64, f64, f64)>,
    ) -> CoreResult<Vec<CliPlanDagNode>> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;

        let nodes = self
            .app
            .plan_dag_service()
            .get_nodes_filtered(project_id, Some(resolved_plan_id), node_type, label_pattern, bounds)
            .await?;

        Ok(nodes
            .into_iter()
            .map(|n| CliPlanDagNode::new(project_id, resolved_plan_id, n))
            .collect())
    }

    /// Phase 1.2: Get a single node by ID with metadata enrichment
    pub async fn get_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
    ) -> CoreResult<Option<CliPlanDagNode>> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;

        let node = self
            .app
            .plan_dag_service()
            .get_node_by_id(project_id, Some(resolved_plan_id), &node_id)
            .await?;

        if let Some(node) = node {
            // Enrich with execution metadata by loading as a single-node DAG
            // This reuses the existing enrichment logic from load_plan_dag
            let enriched = self.enrich_single_node(node).await?;
            Ok(Some(CliPlanDagNode::new(
                project_id,
                resolved_plan_id,
                enriched,
            )))
        } else {
            Ok(None)
        }
    }

    /// Helper to enrich a single node with execution metadata
    async fn enrich_single_node(&self, mut node: PlanDagNode) -> CoreResult<PlanDagNode> {
        use crate::plan_dag::{DataSetExecutionMetadata, GraphExecutionMetadata, PlanDagNodeType};
        use crate::services::GraphDataService;

        match node.node_type {
            PlanDagNodeType::DataSet => {
                if let Ok(config) = serde_json::from_str::<Value>(&node.config) {
                    if let Some(data_set_id) = config
                        .get("dataSetId")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32)
                    {
                        use crate::database::entities::data_sets;
                        use sea_orm::EntityTrait;

                        if let Some(data_set) = data_sets::Entity::find_by_id(data_set_id)
                            .one(self.app.db())
                            .await
                            .map_err(|e| {
                                CoreError::internal(format!(
                                    "Failed to load data set {}: {}",
                                    data_set_id, e
                                ))
                            })?
                        {
                            let execution_state = match data_set.status.as_str() {
                                "active" => "completed",
                                "processing" => "processing",
                                "error" => "error",
                                _ => "not_started",
                            }
                            .to_string();

                            node.dataset_execution = Some(DataSetExecutionMetadata {
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
                let gd_service = GraphDataService::new(self.app.db().clone());
                if let Ok(Some(gd)) = gd_service.get_by_dag_node(&node.id).await {
                    let execution_state = match gd.status.as_str() {
                        "active" => "completed".to_string(),
                        "processing" => "processing".to_string(),
                        "pending" => "pending".to_string(),
                        "error" => "error".to_string(),
                        other => other.to_string(),
                    };
                    node.graph_execution = Some(GraphExecutionMetadata {
                        graph_id: gd.id,
                        graph_data_id: Some(gd.id),
                        node_count: gd.node_count,
                        edge_count: gd.edge_count,
                        execution_state,
                        computed_date: gd.computed_date.map(|d| d.to_rfc3339()),
                        error_message: gd.error_message.clone(),
                        annotations: gd
                            .annotations
                            .as_ref()
                            .and_then(|v| v.as_str().map(|s| s.to_string())),
                    });
                }
            }
            _ => {}
        }

        Ok(node)
    }

    /// Phase 1.3: Traverse graph from a starting node
    pub async fn traverse_from_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        start_node: String,
        direction: String,
        max_depth: usize,
    ) -> CoreResult<(Vec<CliPlanDagNode>, Vec<CliPlanDagEdge>)> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;

        let (nodes, edges) = self
            .app
            .plan_dag_service()
            .traverse_from_node(
                project_id,
                Some(resolved_plan_id),
                &start_node,
                &direction,
                max_depth,
            )
            .await?;

        let cli_nodes: Vec<CliPlanDagNode> = nodes
            .into_iter()
            .map(|n| CliPlanDagNode::new(project_id, resolved_plan_id, n))
            .collect();

        let cli_edges: Vec<CliPlanDagEdge> = edges
            .into_iter()
            .map(|e| CliPlanDagEdge::new(project_id, resolved_plan_id, e))
            .collect();

        Ok((cli_nodes, cli_edges))
    }

    /// Phase 1.3: Find path between two nodes
    pub async fn find_path(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        start_node: String,
        end_node: String,
    ) -> CoreResult<Option<Vec<String>>> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;

        self.app
            .plan_dag_service()
            .find_path(project_id, Some(resolved_plan_id), &start_node, &end_node)
            .await
    }

    /// Phase 2.2: Search nodes
    pub async fn search_nodes(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        query: String,
        fields: Vec<String>,
    ) -> CoreResult<Vec<CliPlanDagNode>> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;

        let nodes = self
            .app
            .plan_dag_service()
            .search_nodes(project_id, Some(resolved_plan_id), &query, fields)
            .await?;

        Ok(nodes
            .into_iter()
            .map(|n| CliPlanDagNode::new(project_id, resolved_plan_id, n))
            .collect())
    }

    /// Phase 2.2: Find nodes by edge filter
    pub async fn find_nodes_by_edge_filter(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        filter: String,
    ) -> CoreResult<Vec<CliPlanDagNode>> {
        let resolved_plan_id = self.resolve_plan_id(project_id, plan_id).await?;

        let nodes = self
            .app
            .plan_dag_service()
            .find_nodes_by_edge_filter(project_id, Some(resolved_plan_id), &filter)
            .await?;

        Ok(nodes
            .into_iter()
            .map(|n| CliPlanDagNode::new(project_id, resolved_plan_id, n))
            .collect())
    }
}
