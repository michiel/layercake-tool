use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::database::entities::{projects, plans, nodes, edges, layers};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Plan, Node, Edge, Layer};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl From<projects::Model> for Project {
    fn from(model: projects::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[ComplexObject]
impl Project {
    async fn plans(&self, ctx: &Context<'_>) -> Result<Vec<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plans = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(self.id))
            .all(&context.db)
            .await?;
        
        Ok(plans.into_iter().map(Plan::from).collect())
    }

    async fn nodes(&self, ctx: &Context<'_>) -> Result<Vec<Node>> {
        let context = ctx.data::<GraphQLContext>()?;
        let nodes = nodes::Entity::find()
            .filter(nodes::Column::ProjectId.eq(self.id))
            .all(&context.db)
            .await?;
        
        Ok(nodes.into_iter().map(Node::from).collect())
    }

    async fn edges(&self, ctx: &Context<'_>) -> Result<Vec<Edge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let edges = edges::Entity::find()
            .filter(edges::Column::ProjectId.eq(self.id))
            .all(&context.db)
            .await?;
        
        Ok(edges.into_iter().map(Edge::from).collect())
    }

    async fn layers(&self, ctx: &Context<'_>) -> Result<Vec<Layer>> {
        let context = ctx.data::<GraphQLContext>()?;
        let layers = layers::Entity::find()
            .filter(layers::Column::ProjectId.eq(self.id))
            .all(&context.db)
            .await?;
        
        Ok(layers.into_iter().map(Layer::from).collect())
    }
}

#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateProjectInput {
    pub name: String,
    pub description: Option<String>,
}