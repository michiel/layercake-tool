use async_graphql::*;
use sea_orm::EntityTrait;

use layercake_core::database::entities::{project_collaborators, users};
use layercake_core::services::collaboration_service::CollaborationService;
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
        let actor = context.actor_for_request(ctx).await;
        let service = CollaborationService::new(context.db.clone());

        let collaboration = service
            .invite_collaborator(&actor, input.project_id, &input.email, &input.role)
            .await
            .map_err(Error::from)?;

        Ok(ProjectCollaborator::from(collaboration))
    }

    /// Accept collaboration invitation
    async fn accept_collaboration(
        &self,
        ctx: &Context<'_>,
        collaboration_id: i32,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = CollaborationService::new(context.db.clone());

        let updated = service
            .accept_invitation(&actor, collaboration_id)
            .await
            .map_err(Error::from)?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Decline collaboration invitation
    async fn decline_collaboration(
        &self,
        ctx: &Context<'_>,
        collaboration_id: i32,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = CollaborationService::new(context.db.clone());

        let updated = service
            .decline_invitation(&actor, collaboration_id)
            .await
            .map_err(Error::from)?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Update collaborator role
    async fn update_collaborator_role(
        &self,
        ctx: &Context<'_>,
        input: UpdateCollaboratorRoleInput,
    ) -> Result<ProjectCollaborator> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = CollaborationService::new(context.db.clone());

        let collaboration = project_collaborators::Entity::find_by_id(input.collaborator_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Collaboration", input.collaborator_id))?;

        let updated = service
            .update_collaborator_role(
                &actor,
                collaboration.project_id,
                collaboration.user_id,
                &input.role,
            )
            .await
            .map_err(Error::from)?;

        Ok(ProjectCollaborator::from(updated))
    }

    /// Remove collaborator from project
    async fn remove_collaborator(&self, ctx: &Context<'_>, collaboration_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let service = CollaborationService::new(context.db.clone());

        let collaboration = project_collaborators::Entity::find_by_id(collaboration_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("project_collaborators::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Collaboration", collaboration_id))?;

        service
            .remove_collaborator(&actor, collaboration.project_id, collaboration.user_id)
            .await
            .map_err(Error::from)?;

        Ok(true)
    }

    /// Join a project for collaboration
    async fn join_project_collaboration(&self, ctx: &Context<'_>, project_id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let user_id = actor
            .user_id
            .ok_or_else(|| StructuredError::unauthorized("User is not authenticated"))?;
        let (user_name, avatar_color) = match users::Entity::find_by_id(user_id)
            .one(&context.db)
            .await
        {
            Ok(Some(user)) => (user.display_name, user.avatar_color),
            _ => (format!("User {}", user_id), "#3B82F6".to_string()),
        };
        let user_id = format!("user_{}", user_id);

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
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let user_id = actor
            .user_id
            .ok_or_else(|| StructuredError::unauthorized("User is not authenticated"))?;
        let (user_name, avatar_color) = match users::Entity::find_by_id(user_id)
            .one(&context.db)
            .await
        {
            Ok(Some(user)) => (user.display_name, user.avatar_color),
            _ => (format!("User {}", user_id), "#3B82F6".to_string()),
        };
        let user_id = format!("user_{}", user_id);

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
