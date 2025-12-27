use async_graphql::*;

use layercake_core::app_context::ProjectUpdate;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::project::{CreateProjectInput, Project, UpdateProjectInput};
use layercake_core::services::sample_project_service::SampleProjectService;

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
        let actor = context.actor_for_request(ctx).await;
        let project = context
            .app
            .create_project(&actor, input.name, input.description, input.tags)
            .await
            .map_err(Error::from)?;

        Ok(Project::from(project))
    }

    /// Create a project from a bundled sample definition
    async fn create_sample_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "sampleKey")] sample_key: String,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = SampleProjectService::new(context.db.clone());

        let project = service
            .create_sample_project(&actor, &sample_key)
            .await
            .map_err(Error::from)?;

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
        let actor = context.actor_for_request(ctx).await;
        let update =
            ProjectUpdate::new(Some(input.name), input.description, true, input.tags, None);
        let project = context
            .app
            .update_project(&actor, id, update)
            .await
            .map_err(Error::from)?;

        Ok(Project::from(project))
    }

    /// Delete a project
    async fn delete_project(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .delete_project(&actor, id)
            .await
            .map_err(Error::from)?;

        Ok(true)
    }
}
