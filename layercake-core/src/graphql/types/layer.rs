use async_graphql::*;
use sea_orm::EntityTrait;

use crate::database::entities::{graph_layers, graphs, project_layers};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::graph::Graph;
use crate::graphql::types::scalars::JSON;

// Input type for bulk layer updates
#[derive(InputObject, Clone, Debug)]
pub struct LayerUpdateInput {
    pub id: i32,
    pub name: Option<String>,
    pub properties: Option<JSON>,
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectLayerInput {
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
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
            comment: model.comment,
            properties,
            dataset_id: model.dataset_id,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectLayer {
    pub id: i32,
    pub project_id: i32,
    pub layer_id: String,
    pub name: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    pub source_dataset_id: Option<i32>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
        let graph = graphs::Entity::find_by_id(self.graph_id)
            .one(&context.db)
            .await?;

        Ok(graph.map(Graph::from))
    }
}
