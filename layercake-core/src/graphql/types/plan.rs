use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;

use crate::app_context::PlanSummary;
use crate::database::entities::{plans, projects};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Project;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Plan {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PlanSummary> for Plan {
    fn from(summary: PlanSummary) -> Self {
        Self {
            id: summary.id,
            project_id: summary.project_id,
            name: summary.name,
            description: summary.description,
            tags: summary.tags,
            yaml_content: summary.yaml_content,
            dependencies: summary.dependencies,
            status: summary.status,
            version: summary.version,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
        }
    }
}

impl From<plans::Model> for Plan {
    fn from(model: plans::Model) -> Self {
        PlanSummary::from(model).into()
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
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
}

#[derive(InputObject)]
pub struct UpdatePlanInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub yaml_content: Option<String>,
    pub dependencies: Option<Vec<i32>>,
}
