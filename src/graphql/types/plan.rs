use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait};

use crate::database::entities::{plans, projects};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Project;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Plan {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<plans::Model> for Plan {
    fn from(model: plans::Model) -> Self {
        let dependencies = model.dependencies
            .and_then(|d| serde_json::from_str::<Vec<i32>>(&d).ok());
        
        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            yaml_content: model.yaml_content,
            dependencies,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[ComplexObject]
impl Plan {
    async fn project(&self, ctx: &Context<'_>) -> Result<Option<Project>> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = projects::Entity::find_by_id(self.project_id)
            .one(&context.db)
            .await?;
        
        Ok(project.map(Project::from))
    }
}

#[derive(InputObject)]
pub struct CreatePlanInput {
    pub project_id: i32,
    pub name: String,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
}

#[derive(InputObject)]
pub struct UpdatePlanInput {
    pub name: String,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
}