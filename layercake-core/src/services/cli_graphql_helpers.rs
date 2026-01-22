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
}
