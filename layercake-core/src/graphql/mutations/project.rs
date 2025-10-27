use async_graphql::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::database::entities::projects;
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::project::{CreateProjectInput, Project, UpdateProjectInput};
use crate::graphql::errors::StructuredError;
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

        let project = project
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::insert", e))?;
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
            .map_err(|e| StructuredError::service("SampleProjectService::create_sample_project", e))?;

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
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", id))?;

        let mut project: projects::ActiveModel = project.into();
        project.name = Set(input.name);
        project.description = Set(input.description);

        let project = project
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::update", e))?;
        Ok(Project::from(project))
    }

    /// Delete a project
    async fn delete(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let project = projects::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", id))?;

        projects::Entity::delete_by_id(project.id)
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::delete_by_id", e))?;

        Ok(true)
    }
}
