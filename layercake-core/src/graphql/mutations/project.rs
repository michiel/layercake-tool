use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::database::entities::projects;
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::project::{CreateProjectInput, Project, UpdateProjectInput};
use crate::services::sample_project_service::SampleProjectService;

pub struct ProjectMutations;

#[Object]
impl ProjectMutations {
    /// Create a new project
    async fn create(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;

        let mut project = projects::ActiveModel::new();
        project.name = Set(input.name);
        project.description = Set(input.description);

        let project = project.insert(&context.db).await?;
        Ok(Project::from(project))
    }

    /// Create a project from a bundled sample definition
    async fn create_from_sample(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "sampleKey")] sample_key: String,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;
        let service = SampleProjectService::new(context.db.clone());

        let project = service
            .create_sample_project(&sample_key)
            .await
            .map_err(|e| Error::new(format!("Failed to create sample project: {}", e)))?;

        Ok(Project::from(project))
    }

    /// Update an existing project
    async fn update(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateProjectInput,
    ) -> Result<Project> {
        let context = ctx.data::<GraphQLContext>()?;

        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        let mut project: projects::ActiveModel = project.into();
        project.name = Set(input.name);
        project.description = Set(input.description);

        let project = project.update(&context.db).await?;
        Ok(Project::from(project))
    }

    /// Delete a project
    async fn delete(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await?
            .ok_or_else(|| Error::new("Project not found"))?;

        projects::Entity::delete_by_id(project.id)
            .exec(&context.db)
            .await?;

        Ok(true)
    }
}
