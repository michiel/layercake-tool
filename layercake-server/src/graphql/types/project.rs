use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Plan;
use async_graphql::*;
use chrono::{DateTime, Utc};
use layercake_core::app_context::ProjectSummary;
use layercake_core::database::entities::projects;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[graphql(name = "importExportPath")]
    pub import_export_path: Option<String>,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl From<projects::Model> for Project {
    fn from(model: projects::Model) -> Self {
        let tags = serde_json::from_str::<Vec<String>>(&model.tags).unwrap_or_default();
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            tags,
            import_export_path: model.import_export_path,
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
            tags: summary.tags,
            import_export_path: summary.import_export_path,
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
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(plan.map(Plan::from))
    }
}

#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(InputObject)]
pub struct UpdateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}
