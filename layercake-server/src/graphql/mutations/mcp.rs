use async_graphql::*;

use crate::graphql::context::{GraphQLContext, RequestSession};
use crate::graphql::errors::StructuredError;
use layercake_core::services::authorization::AuthorizationService;

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
        use layercake_core::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let session = ctx
            .data_opt::<RequestSession>()
            .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

        let auth_service = AuthorizationService::new(context.db.clone());
        let user = auth_service
            .get_user_from_session(session.as_str())
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        let service = McpAgentService::new(context.db.clone());

        let credentials = service
            .create_agent(user.id, project_id, name, None)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(crate::graphql::types::McpAgentCredentials::from(
            credentials,
        ))
    }

    /// Revoke (deactivate) an MCP agent
    async fn revoke_mcp_agent(&self, ctx: &Context<'_>, user_id: i32) -> Result<bool> {
        use layercake_core::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let session = ctx
            .data_opt::<RequestSession>()
            .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

        let auth_service = AuthorizationService::new(context.db.clone());
        let user = auth_service
            .get_user_from_session(session.as_str())
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        let service = McpAgentService::new(context.db.clone());

        service
            .revoke_agent(user_id, user.id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(true)
    }

    /// Regenerate API key for an MCP agent
    async fn regenerate_mcp_agent_key(&self, ctx: &Context<'_>, user_id: i32) -> Result<String> {
        use layercake_core::services::mcp_agent_service::McpAgentService;
        let context = ctx.data::<GraphQLContext>()?;
        let session = ctx
            .data_opt::<RequestSession>()
            .ok_or_else(|| StructuredError::unauthorized("Active session required"))?;

        let auth_service = AuthorizationService::new(context.db.clone());
        let user = auth_service
            .get_user_from_session(session.as_str())
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        let service = McpAgentService::new(context.db.clone());

        let new_key = service
            .regenerate_api_key(user_id, user.id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(new_key)
    }
}
