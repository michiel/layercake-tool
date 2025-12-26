use async_graphql::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use layercake_core::database::entities::{project_collaborators, users};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    InviteCollaboratorInput, ProjectCollaborator, UpdateCollaboratorRoleInput,
};

#[derive(Default)]
pub struct CollaborationMutation;

#[Object]
impl CollaborationMutation {
    /// Invite a user to collaborate on a project
    async fn invite_collaborator(
        &self,
        ctx: &Context<'_>,
        input: InviteCollaboratorInput,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        // Find user by email
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(&input.email))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("users::Entity::find (invite email)", e))?
            .ok_or_else(|| StructuredError::not_found("User", &input.email))?;

        // Check if user is already a collaborator
        let existing = project_collaborators::Entity::find()
            .filter(project_collaborators::Column::ProjectId.eq(input.project_id))
            .filter(project_collaborators::Column::UserId.eq(user.id))
            .filter(project_collaborators::Column::IsActive.eq(true))
            .one(&context.db)
            .await
            .map_err(|e| {
                StructuredError::database("project_collaborators::Entity::find (existing)", e)
            })?;

        if existing.is_some() {
            return Err(StructuredError::conflict(
                "ProjectCollaborator",
                "User is already a collaborator on this project",
            ));
        }

        // Parse role
        let role =
            layercake_core::database::entities::project_collaborators::ProjectRole::from_str(&input.role)
                .map_err(|_| StructuredError::validation("role", "Invalid role"))?;

        // Create collaboration
        // Note: In a real app, you'd get invited_by from the authentication context
        let collaboration = project_collaborators::ActiveModel::new(
            input.project_id,
            user.id,
            role,
            Some(1), // TODO: Get from auth context
        );

        let collaboration = collaboration
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::insert", e))?;

        Ok(ProjectCollaborator::from(collaboration))
    }

    /// Accept collaboration invitation
    async fn accept_collaboration(
        &self,
        ctx: &Context<'_>,
        collaboration_id: i32,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Collaboration", collaboration_id))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.accept_invitation();
        let updated = collaboration_active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::update", e))?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Decline collaboration invitation
    async fn decline_collaboration(
        &self,
        ctx: &Context<'_>,
        collaboration_id: i32,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Collaboration", collaboration_id))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.decline_invitation();
        let updated = collaboration_active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::update", e))?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Update collaborator role
    async fn update_collaborator_role(
        &self,
        ctx: &Context<'_>,
        input: UpdateCollaboratorRoleInput,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(input.collaborator_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Collaboration", input.collaborator_id))?;

        // Parse new role
        let role =
            layercake_core::database::entities::project_collaborators::ProjectRole::from_str(&input.role)
                .map_err(|_| StructuredError::validation("role", "Invalid role"))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.update_role(role);
        let updated = collaboration_active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::update", e))?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Remove collaborator from project
    async fn remove_collaborator(&self, ctx: &Context<'_>, collaboration_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Collaboration", collaboration_id))?;

        let mut collaboration_active: project_collaborators::ActiveModel = collaboration.into();
        collaboration_active = collaboration_active.deactivate();
        collaboration_active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::update", e))?;

        Ok(true)
    }

    /// Join a project for collaboration
    async fn join_project_collaboration(&self, ctx: &Context<'_>, project_id: i32) -> Result<bool> {
        let _context = ctx.data::<GraphQLContext>()?;

        // TODO: Extract from authenticated user context when authentication is implemented
        let (user_id, user_name, avatar_color) = {
            (
                "demo_user".to_string(),
                "Demo User".to_string(),
                "#3B82F6".to_string(),
            )
        };

        let plan_id = format!("project_{}", project_id);

        // Create user joined event data
        let user_data = crate::graphql::subscriptions::create_user_event_data(
            user_id.clone(),
            user_name,
            avatar_color,
        );

        // Create collaboration event
        let event_data = crate::graphql::subscriptions::CollaborationEventData {
            node_event: None,
            edge_event: None,
            user_event: Some(user_data),
            cursor_event: None,
        };
        let event = crate::graphql::subscriptions::create_collaboration_event(
            plan_id,
            user_id,
            crate::graphql::subscriptions::CollaborationEventType::UserJoined,
            event_data,
        );

        // Broadcast the event
        match crate::graphql::subscriptions::publish_collaboration_event(event).await {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Leave a project collaboration
    async fn leave_project_collaboration(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<bool> {
        let _context = ctx.data::<GraphQLContext>()?;

        // TODO: Extract from authenticated user context when authentication is implemented
        let (user_id, user_name, avatar_color) = {
            (
                "demo_user".to_string(),
                "Demo User".to_string(),
                "#3B82F6".to_string(),
            )
        };

        let plan_id = format!("project_{}", project_id);

        // Create user left event data
        let user_data = crate::graphql::subscriptions::create_user_event_data(
            user_id.clone(),
            user_name,
            avatar_color,
        );

        // Create collaboration event
        let event_data = crate::graphql::subscriptions::CollaborationEventData {
            node_event: None,
            edge_event: None,
            user_event: Some(user_data),
            cursor_event: None,
        };
        let event = crate::graphql::subscriptions::create_collaboration_event(
            plan_id,
            user_id,
            crate::graphql::subscriptions::CollaborationEventType::UserLeft,
            event_data,
        );

        // Broadcast the event
        match crate::graphql::subscriptions::publish_collaboration_event(event).await {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
