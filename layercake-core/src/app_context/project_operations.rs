use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};

use super::{AppContext, ProjectSummary, ProjectUpdate};
use crate::database::entities::projects;
use crate::errors::{CoreError, CoreResult};
use crate::auth::Actor;

impl AppContext {
    // ----- Project helpers -------------------------------------------------
    pub async fn list_projects(&self) -> Result<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        Ok(projects.into_iter().map(ProjectSummary::from).collect())
    }

    pub async fn list_projects_filtered(
        &self,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list projects: {}", e))?;

        // If tags filter is provided, filter projects by tags
        let filtered_projects = if let Some(filter_tags) = tags {
            if filter_tags.is_empty() {
                projects
            } else {
                projects
                    .into_iter()
                    .filter(|project| {
                        let project_tags: Vec<String> =
                            serde_json::from_str(&project.tags).unwrap_or_default();
                        // Check if any filter tag matches any project tag
                        filter_tags
                            .iter()
                            .any(|filter_tag| project_tags.contains(filter_tag))
                    })
                    .collect()
            }
        } else {
            projects
        };

        Ok(filtered_projects
            .into_iter()
            .map(ProjectSummary::from)
            .collect())
    }

    pub async fn get_project(&self, id: i32) -> Result<Option<ProjectSummary>> {
        let project = projects::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project {}: {}", id, e))?;

        Ok(project.map(ProjectSummary::from))
    }

    pub async fn create_project(
        &self,
        _actor: &Actor,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> CoreResult<ProjectSummary> {
        let now = Utc::now();
        let tags_json =
            serde_json::to_string(&tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
        let project = projects::ActiveModel {
            name: Set(name),
            description: Set(description),
            tags: Set(tags_json),
            import_export_path: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let project = project
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to create project: {}", e)))?;

        // Create default "Main plan" for new project if no plans exist
        let _ = self.plan_service.ensure_default_plan(project.id).await;

        Ok(ProjectSummary::from(project))
    }

    pub async fn update_project(
        &self,
        _actor: &Actor,
        id: i32,
        update: ProjectUpdate,
    ) -> CoreResult<ProjectSummary> {
        let project = projects::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load project {}: {}", id, e)))?
            .ok_or_else(|| CoreError::not_found("Project", id.to_string()))?;

        let mut active: projects::ActiveModel = project.into();
        if let Some(name) = update.name {
            active.name = Set(name);
        }
        if update.description_is_set {
            active.description = Set(update.description);
        }
        if let Some(tags) = update.tags {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            active.tags = Set(tags_json);
        }
        if let Some(path) = update.import_export_path {
            active.import_export_path = Set(path);
        }
        active.updated_at = Set(Utc::now());

        let project = active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update project {}: {}", id, e)))?;

        Ok(ProjectSummary::from(project))
    }

    pub async fn delete_project(&self, _actor: &Actor, id: i32) -> CoreResult<()> {
        let result = projects::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete project {}: {}", id, e)))?;

        if result.rows_affected == 0 {
            return Err(CoreError::not_found("Project", id.to_string()));
        }

        Ok(())
    }
}
