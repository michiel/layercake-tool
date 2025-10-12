use async_graphql::*;
use sea_orm::EntityTrait;

use crate::database::entities::{layers, graphs};
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
    pub color: Option<String>,
    pub properties: Option<JSON>,
}

impl From<layers::Model> for Layer {
    fn from(model: layers::Model) -> Self {
        let properties = model.properties
            .and_then(|p| serde_json::from_str(&p).ok());
        
        Self {
            id: model.id,
            graph_id: model.graph_id,
            layer_id: model.layer_id,
            name: model.name,
            color: model.color,
            properties,
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