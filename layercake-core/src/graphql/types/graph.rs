use async_graphql::*;

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Project;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Graph {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    pub node_id: String,
    pub execution_state: String,
    pub computed_date: Option<chrono::DateTime<chrono::Utc>>,
    pub source_hash: Option<String>,
    pub node_count: i32,
    pub edge_count: i32,
    pub error_message: Option<String>,
    pub metadata: Option<serde_json::Value>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl Graph {
    async fn project(&self, ctx: &Context<'_>) -> Result<Project> {
        let graphql_ctx = ctx.data::<GraphQLContext>()
            .map_err(|_| Error::new("GraphQL context not found"))?;

        use sea_orm::EntityTrait;
        use crate::database::entities::projects;

        let project = projects::Entity::find_by_id(self.project_id)
            .one(&graphql_ctx.db)
            .await
            .map_err(|e| Error::new(format!("Database error: {}", e)))?
            .ok_or_else(|| Error::new("Project not found"))?;

        Ok(Project::from(project))
    }
}

impl From<crate::database::entities::graphs::Model> for Graph {
    fn from(model: crate::database::entities::graphs::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            node_id: model.node_id,
            execution_state: model.execution_state,
            computed_date: model.computed_date,
            source_hash: model.source_hash,
            node_count: model.node_count,
            edge_count: model.edge_count,
            error_message: model.error_message,
            metadata: model.metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(InputObject)]
pub struct CreateGraphInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
}

#[derive(InputObject)]
pub struct UpdateGraphInput {
    pub name: Option<String>,
}
