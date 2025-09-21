use async_graphql::*;
use sea_orm::EntityTrait;

use crate::database::entities::{nodes, projects};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, JSON};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Node {
    pub id: i32,
    pub project_id: i32,
    pub node_id: String,
    pub label: String,
    pub layer_id: Option<String>,
    pub properties: Option<JSON>,
}

impl From<nodes::Model> for Node {
    fn from(model: nodes::Model) -> Self {
        let properties = model.properties
            .and_then(|p| serde_json::from_str(&p).ok());
        
        Self {
            id: model.id,
            project_id: model.project_id,
            node_id: model.node_id,
            label: model.label,
            layer_id: model.layer_id,
            properties,
        }
    }
}

#[ComplexObject]
impl Node {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(self.project_id)
            .one(&context.db)
            .await?;
        
        Ok(project.map(Project::from))
    }
}

#[derive(InputObject)]
pub struct CreateNodeInput {
    pub node_id: String,
    pub label: String,
    pub layer_id: Option<String>,
    pub properties: Option<JSON>,
}