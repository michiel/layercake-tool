use std::pin::Pin;
use std::sync::Arc;

use async_graphql::{
    futures_util::Stream, Context, Error, InputObject, Json, Object, Result, Schema, SimpleObject,
    Subscription, ID,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::service::{
    ProjectionCreateInput, ProjectionExportBundle, ProjectionGraphEvent, ProjectionGraphView,
    ProjectionService, ProjectionStateEvent, ProjectionUpdateInput,
};

pub type ProjectionsSchema = Schema<ProjectionQuery, ProjectionMutation, ProjectionSubscription>;

#[derive(Clone)]
pub struct ProjectionSchemaContext {
    pub projections: Arc<ProjectionService>,
}

impl ProjectionSchemaContext {
    pub fn new(projections: Arc<ProjectionService>) -> Self {
        Self { projections }
    }
}

#[derive(Default)]
pub struct ProjectionQuery;

#[Object]
impl ProjectionQuery {
    async fn projections(&self, ctx: &Context<'_>, project_id: ID) -> Result<Vec<Projection>> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let project_id: i32 = project_id
            .parse()
            .map_err(|_| Error::new("invalid project id"))?;
        let models = service.projections.list_by_project(project_id).await?;
        Ok(models.into_iter().map(Projection::from).collect())
    }

    async fn projection(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Projection>> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let model = service.projections.get(id).await?;
        Ok(model.map(Projection::from))
    }

    async fn projection_graph(&self, ctx: &Context<'_>, id: ID) -> Result<ProjectionGraph> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let graph = service.projections.load_graph(id).await?;
        Ok(ProjectionGraph::from(graph))
    }

    async fn projection_state(&self, ctx: &Context<'_>, id: ID) -> Result<ProjectionState> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let model = service
            .projections
            .get(id)
            .await?
            .ok_or_else(|| Error::new("projection not found"))?;

        let state = service
            .projections
            .get_state(id)
            .await
            .or(model.settings_json);

        Ok(ProjectionState {
            projection_id: ID::from(id.to_string()),
            projection_type: model.projection_type,
            state_json: state.map(Json),
        })
    }
}

pub struct ProjectionMutation;

#[Object]
impl ProjectionMutation {
    async fn create_projection(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectionInput,
    ) -> Result<Projection> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let created = service
            .projections
            .create(ProjectionCreateInput {
                project_id: input.project_id,
                graph_id: input.graph_id,
                name: input.name,
                projection_type: input
                    .projection_type
                    .unwrap_or_else(|| "force3d".to_string()),
                settings_json: input.settings_json.map(|Json(v)| v),
            })
            .await?;

        Ok(Projection::from(created))
    }

    async fn update_projection(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateProjectionInput,
    ) -> Result<Projection> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;

        let updated = service
            .projections
            .update(
                id,
                ProjectionUpdateInput {
                    name: input.name,
                    projection_type: input.projection_type,
                    settings_json: input.settings_json.map(|v| Some(v.0)),
                },
            )
            .await?;

        Ok(Projection::from(updated))
    }

    async fn delete_projection(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        service.projections.delete(id).await?;
        Ok(true)
    }

    async fn save_projection_state(
        &self,
        ctx: &Context<'_>,
        id: ID,
        state: Json<serde_json::Value>,
    ) -> Result<bool> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        service.projections.save_state(id, state.0).await?;
        Ok(true)
    }

    async fn refresh_projection_graph(&self, ctx: &Context<'_>, id: ID) -> Result<ProjectionGraph> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let graph = service.projections.load_graph(id).await?;
        Ok(ProjectionGraph::from(graph))
    }

    async fn export_projection(&self, ctx: &Context<'_>, id: ID) -> Result<ProjectionExport> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let bundle = service.projections.export_bundle(id).await?;
        Ok(ProjectionExport::from(bundle))
    }
}

pub struct ProjectionSubscription;

#[Subscription]
impl ProjectionSubscription {
    async fn projection_state_updated(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Pin<Box<dyn Stream<Item = ProjectionState> + Send>>> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let id_filter = id;
        let stream = BroadcastStream::new(service.projections.subscribe_state()).filter_map(
            move |msg| match msg {
                Ok(ProjectionStateEvent {
                    projection_id,
                    projection_type,
                    state,
                }) if projection_id == id_filter => Some(ProjectionState {
                    projection_id: ID::from(projection_id.to_string()),
                    projection_type,
                    state_json: Some(Json(state)),
                }),
                _ => None,
            },
        );

        Ok(Box::pin(stream))
    }

    async fn projection_graph_updated(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Pin<Box<dyn Stream<Item = ProjectionGraph> + Send>>> {
        let service = ctx.data::<ProjectionSchemaContext>()?;
        let id: i32 = id
            .parse()
            .map_err(|_| Error::new("invalid projection id"))?;
        let id_filter = id;
        let stream = BroadcastStream::new(service.projections.subscribe_graph()).filter_map(
            move |msg| match msg {
                Ok(ProjectionGraphEvent {
                    projection_id,
                    graph,
                }) if projection_id == id_filter => Some(ProjectionGraph::from(graph)),
                _ => None,
            },
        );

        Ok(Box::pin(stream))
    }
}

#[derive(SimpleObject, Clone)]
pub struct Projection {
    pub id: ID,
    pub project_id: ID,
    pub graph_id: ID,
    pub name: String,
    pub projection_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub settings_json: Option<Json<serde_json::Value>>,
}

impl From<layercake_core::database::entities::projections::Model> for Projection {
    fn from(model: layercake_core::database::entities::projections::Model) -> Self {
        Self {
            id: ID::from(model.id.to_string()),
            project_id: ID::from(model.project_id.to_string()),
            graph_id: ID::from(model.graph_id.to_string()),
            name: model.name,
            projection_type: model.projection_type,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
            settings_json: model.settings_json.map(Json),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ProjectionGraph {
    pub nodes: Vec<ProjectionGraphNode>,
    pub edges: Vec<ProjectionGraphEdge>,
}

impl From<ProjectionGraphView> for ProjectionGraph {
    fn from(view: ProjectionGraphView) -> Self {
        Self {
            nodes: view
                .nodes
                .into_iter()
                .map(ProjectionGraphNode::from)
                .collect(),
            edges: view
                .edges
                .into_iter()
                .map(ProjectionGraphEdge::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ProjectionGraphNode {
    pub id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub attributes: Option<Json<serde_json::Value>>,
}

impl From<crate::service::ProjectionGraphNode> for ProjectionGraphNode {
    fn from(node: crate::service::ProjectionGraphNode) -> Self {
        Self {
            id: node.id,
            label: node.label,
            layer: node.layer,
            weight: node.weight,
            attributes: node.attributes.map(Json),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ProjectionGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub attributes: Option<Json<serde_json::Value>>,
}

impl From<crate::service::ProjectionGraphEdge> for ProjectionGraphEdge {
    fn from(edge: crate::service::ProjectionGraphEdge) -> Self {
        Self {
            id: edge.id,
            source: edge.source,
            target: edge.target,
            label: edge.label,
            layer: edge.layer,
            weight: edge.weight,
            attributes: edge.attributes.map(Json),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ProjectionState {
    pub projection_id: ID,
    pub projection_type: String,
    pub state_json: Option<Json<serde_json::Value>>,
}

#[derive(SimpleObject, Clone)]
pub struct ProjectionExport {
    pub filename: String,
    pub content_base64: String,
}

impl ProjectionExport {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
            "contentBase64": self.content_base64,
        })
    }
}

impl From<ProjectionExportBundle> for ProjectionExport {
    fn from(bundle: ProjectionExportBundle) -> Self {
        Self {
            filename: bundle.filename,
            content_base64: STANDARD.encode(bundle.bytes),
        }
    }
}

#[derive(InputObject)]
pub struct CreateProjectionInput {
    pub project_id: i32,
    pub graph_id: i32,
    pub name: String,
    pub projection_type: Option<String>,
    pub settings_json: Option<Json<serde_json::Value>>,
}

#[derive(InputObject)]
pub struct UpdateProjectionInput {
    pub name: Option<String>,
    pub projection_type: Option<String>,
    pub settings_json: Option<Json<serde_json::Value>>,
}
