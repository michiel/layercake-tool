use async_graphql::*;
use sea_orm::EntityTrait;

use crate::database::entities::{layers, projects};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{Project, JSON};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Layer {
    pub id: i32,
    pub project_id: i32,
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
            project_id: model.project_id,
            layer_id: model.layer_id,
            name: model.name,
            color: model.color,
            properties,
        }
    }
}

#[ComplexObject]
impl Layer {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(self.project_id)
            .one(&context.db)
            .await?;
        
        Ok(project.map(Project::from))
    }
}

#[derive(InputObject)]
pub struct CreateLayerInput {
    pub layer_id: String,
    pub name: String,
    pub color: Option<String>,
    pub properties: Option<JSON>,
}

#[derive(InputObject)]
pub struct UpdateLayerInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub properties: Option<JSON>,
}