use async_graphql::*;

use crate::app_context::ProjectUpdate;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::project::{CreateProjectInput, Project, UpdateProjectInput};
use crate::services::sample_project_service::SampleProjectService;

#[derive(Default)]
pub struct ProjectMutation;

#[Object]
impl ProjectMutation {
    /// Create a new project
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let project = context
            .app
            .create_project(input.name, input.description, input.tags)
            .await
            .map_err(|e| StructuredError::service("AppContext::create_project", e))?;

        Ok(Project::from(project))
    }

    /// Create a project from a bundled sample definition
    async fn create_sample_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "sampleKey")] sample_key: String,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = SampleProjectService::new(context.db.clone());

        let project = service
            .create_sample_project(&sample_key)
            .await
            .map_err(|e| {
                StructuredError::service("SampleProjectService::create_sample_project", e)
            })?;

        Ok(Project::from(project))
    }

    /// Update an existing project
    async fn update_project(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let update =
            ProjectUpdate::new(Some(input.name), input.description, true, input.tags, None);
        let project = context
            .app
            .update_project(id, update)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_project", e))?;

        Ok(Project::from(project))
    }

    /// Delete a project
    async fn delete_project(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        context
            .app
            .delete_project(id)
            .await
            .map_err(|e| StructuredError::service("AppContext::delete_project", e))?;

        Ok(true)
    }
}
