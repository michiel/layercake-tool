use async_graphql::*;

use crate::graphql::context::{GraphQLContext, RequestSession};
use crate::graphql::errors::StructuredError;
use crate::services::authorization::AuthorizationService;

#[derive(Default)]
pub struct McpMutation;

#[Object]
impl McpMutation {
    /// Create a new MCP agent for a project
    async fn create_mcp_agent(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        name: String,
    ) -> Result<crate::graphql::types::McpAgentCredentials> {
        use crate::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let session = ctx
            .data_opt::<RequestSession>()
            .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

        let auth_service = AuthorizationService::new(context.db.clone());
        let user = auth_service
            .get_user_from_session(session.as_str())
            .await
            .map_err(|_| StructuredError::unauthorized("Invalid session"))?;

        let service = McpAgentService::new(context.db.clone());

        let credentials = service
            .create_agent(user.id, project_id, name, None)
            .await
            .map_err(|e| StructuredError::service("McpAgentService::create_agent", e))?;

        Ok(crate::graphql::types::McpAgentCredentials::from(
            credentials,
        ))
    }

    /// Revoke (deactivate) an MCP agent
    async fn revoke_mcp_agent(&self, ctx: &Context<'_>, user_id: i32) -> Result<bool> {
        use crate::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let session = ctx
            .data_opt::<RequestSession>()
            .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

        let auth_service = AuthorizationService::new(context.db.clone());
        let user = auth_service
            .get_user_from_session(session.as_str())
            .await
            .map_err(|_| StructuredError::unauthorized("Invalid session"))?;

        let service = McpAgentService::new(context.db.clone());

        service
            .revoke_agent(user_id, user.id)
            .await
            .map_err(|e| StructuredError::service("McpAgentService::revoke_agent", e))?;

        Ok(true)
    }

    /// Regenerate API key for an MCP agent
    async fn regenerate_mcp_agent_key(&self, ctx: &Context<'_>, user_id: i32) -> Result<String> {
        use crate::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let session = ctx
            .data_opt::<RequestSession>()
            .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

        let auth_service = AuthorizationService::new(context.db.clone());
        let user = auth_service
            .get_user_from_session(session.as_str())
            .await
            .map_err(|_| StructuredError::unauthorized("Invalid session"))?;

        let service = McpAgentService::new(context.db.clone());

        let new_key = service
            .regenerate_api_key(user_id, user.id)
            .await
            .map_err(|e| StructuredError::service("McpAgentService::regenerate_api_key", e))?;

        Ok(new_key)
    }
}
