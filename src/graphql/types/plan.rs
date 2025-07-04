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
    pub plan_content: String,
    pub plan_schema_version: String,
    pub plan_format: String,
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
            plan_content: model.plan_content,
            plan_schema_version: model.plan_schema_version,
            plan_format: model.plan_format,
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

    /// Get plan content as parsed JSON
    async fn plan_json(&self) -> Result<serde_json::Value> {
        let model = plans::Model {
            id: self.id,
            project_id: self.project_id,
            name: self.name.clone(),
            plan_content: self.plan_content.clone(),
            plan_schema_version: self.plan_schema_version.clone(),
            plan_format: self.plan_format.clone(),
            dependencies: None,
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        model.get_plan_json()
            .map_err(|e| Error::new(format!("Failed to parse plan JSON: {}", e)))
    }

    /// Validate plan schema
    async fn is_valid(&self) -> Result<bool> {
        let model = plans::Model {
            id: self.id,
            project_id: self.project_id,
            name: self.name.clone(),
            plan_content: self.plan_content.clone(),
            plan_schema_version: self.plan_schema_version.clone(),
            plan_format: self.plan_format.clone(),
            dependencies: None,
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        Ok(model.validate_plan_schema().is_ok())
    }

    /// Get validation errors if any
    async fn validation_errors(&self) -> Result<Option<String>> {
        let model = plans::Model {
            id: self.id,
            project_id: self.project_id,
            name: self.name.clone(),
            plan_content: self.plan_content.clone(),
            plan_schema_version: self.plan_schema_version.clone(),
            plan_format: self.plan_format.clone(),
            dependencies: None,
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        };

        match model.validate_plan_schema() {
            Ok(_) => Ok(None),
            Err(error) => Ok(Some(error)),
        }
    }
}

#[derive(InputObject)]
pub struct CreatePlanInput {
    pub project_id: i32,
    pub name: String,
    pub plan_content: String,
    pub plan_format: Option<String>, // Defaults to "json"
    pub dependencies: Option<Vec<i32>>,
}

#[derive(InputObject)]
pub struct UpdatePlanInput {
    pub name: String,
    pub plan_content: String,
    pub plan_format: Option<String>,
    pub dependencies: Option<Vec<i32>>,
}

#[derive(InputObject)]
pub struct PlanJsonPatch {
    pub path: String,
    pub operation: String, // "add", "remove", "replace", "move", "copy", "test"
    pub value: Option<serde_json::Value>,
    pub from: Option<String>, // For move/copy operations
}

#[derive(InputObject)]
pub struct UpdatePlanJsonInput {
    pub patches: Vec<PlanJsonPatch>,
}