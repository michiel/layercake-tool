use anyhow::{Result, anyhow};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait,
    ActiveModelTrait, Set, QueryOrder
};
use chrono::Utc;

use crate::database::entities::{projects, project_collaborators, users};
use crate::services::{ValidationService, AuthorizationService, ProjectRole};

#[derive(Clone)]
pub struct ProjectService {
    db: DatabaseConnection,
    auth_service: AuthorizationService,
}

impl ProjectService {
    pub fn new(db: DatabaseConnection) -> Self {
        let auth_service = AuthorizationService::new(db.clone());
        Self { db, auth_service }
    }

    /// Create a new project
    pub async fn create_project(&self, user_id: i32, name: &str, description: Option<&str>) -> Result<projects::Model> {
        // Validate input
        let validated_name = ValidationService::validate_project_name(name)
            .map_err(|e| anyhow!("Invalid project name: {}", e))?;

        let validated_description = if let Some(desc) = description {
            ValidationService::validate_project_description(desc)
                .map_err(|e| anyhow!("Invalid project description: {}", e))?
        } else {
            String::new()
        };

        // Create project
        let project = projects::ActiveModel {
            name: Set(validated_name),
            description: Set(Some(validated_description)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let project = project.insert(&self.db).await
            .map_err(|e| anyhow!("Failed to create project: {}", e))?;

        // Add creator as owner
        let collaboration = project_collaborators::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(project.id),
            role: Set("owner".to_string()),
            invitation_status: Set("accepted".to_string()),
            permissions: Set("{}".to_string()),
            is_active: Set(true),
            joined_at: Set(Some(Utc::now())),
            invited_at: Set(Utc::now()),
            ..Default::default()
        };

        collaboration.insert(&self.db).await
            .map_err(|e| anyhow!("Failed to add project owner: {}", e))?;

        Ok(project)
    }

    /// Update project details
    pub async fn update_project(&self, user_id: i32, project_id: i32, name: Option<&str>, description: Option<&str>) -> Result<projects::Model> {
        // Check write access
        self.auth_service.check_project_write_access(user_id, project_id).await?;

        // Find project
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Project not found"))?;

        let mut active_project: projects::ActiveModel = project.into();

        // Update fields if provided
        if let Some(name) = name {
            let validated_name = ValidationService::validate_project_name(name)
                .map_err(|e| anyhow!("Invalid project name: {}", e))?;
            active_project.name = Set(validated_name);
        }

        if let Some(description) = description {
            let validated_description = ValidationService::validate_project_description(description)
                .map_err(|e| anyhow!("Invalid project description: {}", e))?;
            active_project.description = Set(Some(validated_description));
        }

        active_project.updated_at = Set(Utc::now());

        let updated_project = active_project.update(&self.db).await
            .map_err(|e| anyhow!("Failed to update project: {}", e))?;

        Ok(updated_project)
    }

    /// Delete a project (owner only)
    pub async fn delete_project(&self, user_id: i32, project_id: i32) -> Result<()> {
        // Check admin access
        self.auth_service.check_project_admin_access(user_id, project_id).await?;

        // Delete project (cascading deletes should handle collaborators)
        let result = projects::Entity::delete_by_id(project_id)
            .exec(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to delete project: {}", e))?;

        if result.rows_affected == 0 {
            return Err(anyhow!("Project not found"));
        }

        Ok(())
    }

    /// Get project by ID with access check
    pub async fn get_project(&self, user_id: i32, project_id: i32) -> Result<projects::Model> {
        // Check read access
        self.auth_service.check_project_read_access(user_id, project_id).await?;

        // Get project
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Project not found"))?;

        Ok(project)
    }

    /// List projects for a user
    pub async fn list_user_projects(&self, user_id: i32) -> Result<Vec<projects::Model>> {
        // Get all project collaborations for the user
        let collaborations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .filter(project_collaborators::Column::InvitationStatus.eq("accepted"))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        let project_ids: Vec<i32> = collaborations.iter().map(|c| c.project_id).collect();

        // Get projects
        let projects = projects::Entity::find()
            .filter(projects::Column::Id.is_in(project_ids))
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        Ok(projects)
    }

    /// Get project collaborators
    pub async fn get_project_collaborators(&self, user_id: i32, project_id: i32) -> Result<Vec<(project_collaborators::Model, users::Model)>> {
        // Check read access
        self.auth_service.check_project_read_access(user_id, project_id).await?;

        // Get collaborators with user info
        let collaborators = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .find_also_related(users::Entity)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        let result: Vec<(project_collaborators::Model, users::Model)> = collaborators
            .into_iter()
            .filter_map(|(collab, user_opt)| {
                user_opt.map(|user| (collab, user))
            })
            .collect();

        Ok(result)
    }

    // Note: Archive functionality removed - projects entity doesn't have is_archived field

    /// Check if user can access project
    pub async fn can_access_project(&self, user_id: i32, project_id: i32) -> bool {
        self.auth_service.check_project_read_access(user_id, project_id).await.is_ok()
    }

    /// Get user's role in project
    pub async fn get_user_role(&self, user_id: i32, project_id: i32) -> Result<Option<ProjectRole>> {
        self.auth_service.get_user_project_role(user_id, project_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These would be integration tests requiring a test database
    // For now, just testing the service creation

    #[test]
    fn test_service_creation() {
        // This would require a real database connection for proper testing
        // We'll add integration tests when we have a test database setup
    }
}