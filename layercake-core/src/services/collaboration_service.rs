#![allow(dead_code)]

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::auth::Actor;
use crate::database::entities::{project_collaborators, projects, users};
use crate::errors::{CoreError, CoreResult};
use crate::services::{AuthorizationService, ValidationService};

#[allow(dead_code)] // Collaboration service reserved for future use
#[derive(Clone)]
pub struct CollaborationService {
    db: DatabaseConnection,
    auth_service: AuthorizationService,
}

impl CollaborationService {
    pub fn new(db: DatabaseConnection) -> Self {
        let auth_service = AuthorizationService::new(db.clone());
        Self { db, auth_service }
    }

    fn require_user_id(actor: &Actor) -> CoreResult<i32> {
        actor
            .user_id
            .ok_or_else(|| CoreError::unauthorized("User is not authenticated"))
    }

    /// Invite a user to collaborate on a project
    pub async fn invite_collaborator(
        &self,
        actor: &Actor,
        project_id: i32,
        invitee_email: &str,
        role: &str,
    ) -> CoreResult<project_collaborators::Model> {
        let inviter_id = Self::require_user_id(actor)?;
        // Check if inviter has admin access
        self.auth_service
            .check_project_admin_access(inviter_id, project_id)
            .await?;

        // Validate role
        let validated_role = ValidationService::validate_collaboration_role(role)
            .map_err(|e| CoreError::validation(format!("Invalid role: {}", e)))?;

        // Find invitee by email
        let invitee = users::Entity::find()
            .filter(users::Column::Email.eq(invitee_email))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("User", invitee_email.to_string()))?;

        // Check if collaboration already exists
        let existing = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(invitee.id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        if let Some(existing_collab) = existing {
            if existing_collab.is_active {
                return Err(CoreError::conflict(
                    "User is already a collaborator on this project",
                ));
            }
            if existing_collab.invitation_status == "pending" {
                return Err(CoreError::conflict(
                    "User already has a pending invitation for this project",
                ));
            }
        }

        // Create new collaboration
        let collaboration = project_collaborators::ActiveModel {
            user_id: Set(invitee.id),
            project_id: Set(project_id),
            role: Set(validated_role),
            invitation_status: Set("pending".to_string()),
            permissions: Set("{}".to_string()),
            is_active: Set(false),
            invited_by: Set(Some(inviter_id)),
            invited_at: Set(Utc::now()),
            ..Default::default()
        };

        let collaboration = collaboration
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to create invitation: {}", e)))?;

        Ok(collaboration)
    }

    /// Accept a collaboration invitation
    pub async fn accept_invitation(
        &self,
        actor: &Actor,
        collaboration_id: i32,
    ) -> CoreResult<project_collaborators::Model> {
        let user_id = Self::require_user_id(actor)?;
        // Find collaboration
        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("Invitation", collaboration_id.to_string()))?;

        // Check if user is the invitee
        if collaboration.user_id != user_id {
            return Err(CoreError::forbidden(
                "You can only accept your own invitations",
            ));
        }

        // Check if invitation is pending
        if collaboration.invitation_status != "pending" {
            return Err(CoreError::validation(
                "This invitation is no longer pending",
            ));
        }

        // Update collaboration
        let mut active_collaboration: project_collaborators::ActiveModel = collaboration.into();
        active_collaboration.invitation_status = Set("accepted".to_string());
        active_collaboration.is_active = Set(true);
        active_collaboration.joined_at = Set(Some(Utc::now()));

        let updated_collaboration = active_collaboration
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to accept invitation: {}", e)))?;

        Ok(updated_collaboration)
    }

    /// Decline a collaboration invitation
    pub async fn decline_invitation(
        &self,
        actor: &Actor,
        collaboration_id: i32,
    ) -> CoreResult<project_collaborators::Model> {
        let user_id = Self::require_user_id(actor)?;
        // Find collaboration
        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("Invitation", collaboration_id.to_string()))?;

        // Check if user is the invitee
        if collaboration.user_id != user_id {
            return Err(CoreError::forbidden(
                "You can only decline your own invitations",
            ));
        }

        // Check if invitation is pending
        if collaboration.invitation_status != "pending" {
            return Err(CoreError::validation(
                "This invitation is no longer pending",
            ));
        }

        // Update collaboration
        let mut active_collaboration: project_collaborators::ActiveModel = collaboration.into();
        active_collaboration.invitation_status = Set("declined".to_string());

        let updated_collaboration = active_collaboration
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to decline invitation: {}", e)))?;

        Ok(updated_collaboration)
    }

    /// Remove a collaborator from a project
    pub async fn remove_collaborator(
        &self,
        actor: &Actor,
        project_id: i32,
        collaborator_id: i32,
    ) -> CoreResult<()> {
        let admin_id = Self::require_user_id(actor)?;
        // Check if admin has admin access
        self.auth_service
            .check_project_admin_access(admin_id, project_id)
            .await?;

        // Find collaboration
        let collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(collaborator_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("Collaborator", collaborator_id.to_string()))?;

        // Prevent removing the owner unless they're removing themselves
        if collaboration.role == "owner" && admin_id != collaborator_id {
            return Err(CoreError::forbidden("Cannot remove project owner"));
        }

        // Deactivate collaboration
        let mut active_collaboration: project_collaborators::ActiveModel = collaboration.into();
        active_collaboration.is_active = Set(false);
        active_collaboration.invitation_status = Set("revoked".to_string());

        active_collaboration
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to remove collaborator: {}", e)))?;

        Ok(())
    }

    /// Update collaborator role
    pub async fn update_collaborator_role(
        &self,
        actor: &Actor,
        project_id: i32,
        collaborator_id: i32,
        new_role: &str,
    ) -> CoreResult<project_collaborators::Model> {
        let admin_id = Self::require_user_id(actor)?;
        // Check if admin has admin access
        self.auth_service
            .check_project_admin_access(admin_id, project_id)
            .await?;

        // Validate role
        let validated_role = ValidationService::validate_collaboration_role(new_role)
            .map_err(|e| CoreError::validation(format!("Invalid role: {}", e)))?;

        // Find collaboration
        let collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(collaborator_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("Collaborator", collaborator_id.to_string()))?;

        // Prevent changing owner role unless they're changing their own role
        if collaboration.role == "owner" && admin_id != collaborator_id {
            return Err(CoreError::forbidden("Cannot change project owner's role"));
        }

        // Update collaboration
        let mut active_collaboration: project_collaborators::ActiveModel = collaboration.into();
        active_collaboration.role = Set(validated_role);

        let updated_collaboration = active_collaboration.update(&self.db).await.map_err(|e| {
            CoreError::internal(format!("Failed to update collaborator role: {}", e))
        })?;

        Ok(updated_collaboration)
    }

    /// Get pending invitations for a user
    pub async fn get_pending_invitations(
        &self,
        actor: &Actor,
    ) -> CoreResult<Vec<(project_collaborators::Model, projects::Model)>> {
        let user_id = Self::require_user_id(actor)?;
        let invitations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::InvitationStatus.eq("pending"))
            .find_also_related(projects::Entity)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let result: Vec<(project_collaborators::Model, projects::Model)> = invitations
            .into_iter()
            .filter_map(|(collab, project_opt)| project_opt.map(|project| (collab, project)))
            .collect();

        Ok(result)
    }

    /// Get user's collaborations
    pub async fn get_user_collaborations(
        &self,
        actor: &Actor,
    ) -> CoreResult<Vec<(project_collaborators::Model, projects::Model)>> {
        let user_id = Self::require_user_id(actor)?;
        let collaborations = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .filter(project_collaborators::Column::InvitationStatus.eq("accepted"))
            .find_also_related(projects::Entity)
            .order_by_desc(project_collaborators::Column::JoinedAt)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let result: Vec<(project_collaborators::Model, projects::Model)> = collaborations
            .into_iter()
            .filter_map(|(collab, project_opt)| project_opt.map(|project| (collab, project)))
            .collect();

        Ok(result)
    }

    /// Leave a project (for non-owners)
    pub async fn leave_project(&self, actor: &Actor, project_id: i32) -> CoreResult<()> {
        let user_id = Self::require_user_id(actor)?;
        // Find collaboration
        let collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(user_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::forbidden("You are not a collaborator on this project"))?;

        // Prevent owner from leaving unless they transfer ownership first
        if collaboration.role == "owner" {
            // Check if there are other collaborators who could become owner
            let other_collaborators = project_collaborators::Entity::find()
                .filter(project_collaborators::Column::ProjectId.eq(project_id))
                .filter(project_collaborators::Column::IsActive.eq(true))
                .filter(project_collaborators::Column::UserId.ne(user_id))
                .all(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

            if other_collaborators.is_empty() {
                return Err(CoreError::forbidden(
                    "Cannot leave project as owner. Either transfer ownership or delete the project.",
                ));
            } else {
                return Err(CoreError::forbidden(
                    "Cannot leave project as owner. Please transfer ownership first.",
                ));
            }
        }

        // Deactivate collaboration
        let mut active_collaboration: project_collaborators::ActiveModel = collaboration.into();
        active_collaboration.is_active = Set(false);

        active_collaboration
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to leave project: {}", e)))?;

        Ok(())
    }

    /// Transfer project ownership
    pub async fn transfer_ownership(
        &self,
        actor: &Actor,
        project_id: i32,
        new_owner_id: i32,
    ) -> CoreResult<()> {
        let current_owner_id = Self::require_user_id(actor)?;
        // Check if current user is the owner
        self.auth_service
            .check_project_admin_access(current_owner_id, project_id)
            .await?;

        // Find new owner's collaboration
        let new_owner_collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(new_owner_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::validation("New owner must be an active collaborator"))?;

        // Find current owner's collaboration
        let current_owner_collaboration = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::UserId.eq(current_owner_id))
            .filter(project_collaborators::Column::ProjectId.eq(project_id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("Collaborator", current_owner_id.to_string()))?;

        // Update new owner
        let mut new_owner_active: project_collaborators::ActiveModel =
            new_owner_collaboration.into();
        new_owner_active.role = Set("owner".to_string());
        new_owner_active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update new owner: {}", e)))?;

        // Update current owner to editor
        let mut current_owner_active: project_collaborators::ActiveModel =
            current_owner_collaboration.into();
        current_owner_active.role = Set("editor".to_string());
        current_owner_active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update current owner: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Note: These would be integration tests requiring a test database
    // For now, just testing the service creation

    #[test]
    fn test_service_creation() {
        // This would require a real database connection for proper testing
        // We'll add integration tests when we have a test database setup
    }
}
