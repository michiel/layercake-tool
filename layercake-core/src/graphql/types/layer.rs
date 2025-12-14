use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::database::entities::{graph_data, graph_layers, layer_aliases, project_layers};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::graph::Graph;
use crate::graphql::types::scalars::JSON;

// Input type for bulk layer updates
#[derive(InputObject, Clone, Debug)]
pub struct LayerUpdateInput {
    pub id: i32,
    pub name: Option<String>,
    pub properties: Option<JSON>,
    pub alias: Option<String>,
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectLayerInput {
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub alias: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Layer {
    pub id: i32,
    pub graph_id: i32,
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub alias: Option<String>,
    pub comment: Option<String>,
    pub properties: Option<JSON>,
    pub dataset_id: Option<i32>,
}

impl From<graph_layers::Model> for Layer {
    fn from(model: graph_layers::Model) -> Self {
        let properties = model.properties.and_then(|p| serde_json::from_str(&p).ok());

        Self {
            id: model.id,
            graph_id: model.graph_id,
            layer_id: model.layer_id,
            name: model.name,
            background_color: model.background_color,
            text_color: model.text_color,
            border_color: model.border_color,
            alias: model.alias,
            comment: model.comment,
            properties,
            dataset_id: model.dataset_id,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct ProjectLayer {
    pub id: i32,
    pub project_id: i32,
    pub layer_id: String,
    pub name: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    pub alias: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct LayerAlias {
    pub id: i32,
    pub project_id: i32,
    pub alias_layer_id: String,
    pub target_layer_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<layer_aliases::Model> for LayerAlias {
    fn from(model: layer_aliases::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            alias_layer_id: model.alias_layer_id,
            target_layer_id: model.target_layer_id,
            created_at: model.created_at,
        }
    }
}

impl From<project_layers::Model> for ProjectLayer {
    fn from(model: project_layers::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            layer_id: model.layer_id,
            name: model.name,
            background_color: model.background_color,
            text_color: model.text_color,
            border_color: model.border_color,
            alias: model.alias,
            source_dataset_id: model.source_dataset_id,
            enabled: model.enabled,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[ComplexObject]
impl Layer {
    async fn graph(&self, ctx: &Context<'_>) -> Result<Option<Graph>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph = graph_data::Entity::find_by_id(self.graph_id)
            .one(&context.db)
            .await?;

        Ok(graph.map(Graph::from))
    }
}

#[ComplexObject]
impl ProjectLayer {
    async fn aliases(&self, ctx: &Context<'_>) -> Result<Vec<LayerAlias>> {
        let context = ctx.data::<GraphQLContext>()?;
        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::TargetLayerId.eq(self.id))
            .all(&context.db)
            .await?;

        Ok(aliases.into_iter().map(LayerAlias::from).collect())
    }
}

#[ComplexObject]
impl LayerAlias {
    async fn target_layer(&self, ctx: &Context<'_>) -> Result<Option<ProjectLayer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let layer = project_layers::Entity::find_by_id(self.target_layer_id)
            .one(&context.db)
            .await?;

        Ok(layer.map(ProjectLayer::from))
    }
}
