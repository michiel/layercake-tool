use async_graphql::*;
use sea_orm::EntityTrait;

use crate::database::entities::{graphs, graph_layers};
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
    pub datasource_id: Option<i32>,
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
            datasource_id: model.datasource_id,
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
