use async_graphql::*;
use chrono::{DateTime, Utc};

use crate::app_context::ProjectSummary;
use crate::database::entities::projects;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::Plan;

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

impl From<ProjectSummary> for Project {
    fn from(summary: ProjectSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            description: summary.description,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
        }
    }
}

#[ComplexObject]
impl Project {
    async fn plan(&self, ctx: &Context<'_>) -> Result<Option<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        let plan = context
            .app
            .get_plan_for_project(self.id)
            .await
            .map_err(|e| StructuredError::service("AppContext::get_plan_for_project", e))?;

        Ok(plan.map(Plan::from))
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
