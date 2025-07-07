use async_graphql::*;
use sea_orm::EntityTrait;

use crate::database::entities::{edges, projects};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, JSON};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Edge {
    pub id: i32,
    pub project_id: i32,
    pub source_node_id: String,
    pub target_node_id: String,
    pub properties: Option<JSON>,
}

impl From<edges::Model> for Edge {
    fn from(model: edges::Model) -> Self {
        let properties = model.properties
            .and_then(|p| serde_json::from_str(&p).ok());
        
        Self {
            id: model.id,
            project_id: model.project_id,
            source_node_id: model.source_node_id,
            target_node_id: model.target_node_id,
            properties,
        }
    }
}

#[ComplexObject]
impl Edge {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(self.project_id)
            .one(&context.db)
            .await?;
        
        Ok(project.map(Project::from))
    }
}

#[derive(InputObject)]
pub struct CreateEdgeInput {
    pub source_node_id: String,
    pub target_node_id: String,
    pub properties: Option<JSON>,
}

#[derive(InputObject)]
pub struct UpdateEdgeInput {
    pub source_node_id: Option<String>,
    pub target_node_id: Option<String>,
    pub properties: Option<JSON>,
}