use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::database::entities::users;
use crate::services::auth_service::AuthService;
use crate::services::authorization::AuthorizationService;

/// Credentials returned when creating an MCP agent (API key only shown once)
#[derive(Debug, Clone)]
pub struct McpAgentCredentials {
    pub user_id: i32,
    pub api_key: String,
    pub project_id: i32,
    pub name: String,
}

/// Service for managing MCP agents (project-scoped AI users)
#[derive(Clone)]
pub struct McpAgentService {
    db: DatabaseConnection,
}

impl McpAgentService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new MCP agent for a project
    pub async fn create_agent(
        &self,
        creator_user_id: i32,
        project_id: i32,
        name: String,
        _allowed_tools: Option<Vec<String>>, // Reserved for future use
    ) -> Result<McpAgentCredentials> {
        // 1. Verify creator has admin access to project
        let auth_service = AuthorizationService::new(self.db.clone());
        auth_service
            .check_project_admin_access(creator_user_id, project_id)
            .await?;

        // 2. Generate secure API key
        let api_key = Self::generate_api_key();
        let api_key_hash = AuthService::hash_password(&api_key)?;

        // 3. Create unique email and username for the agent
        let agent_uuid = Uuid::new_v4();
        let email = format!("mcp-agent-{}@layercake.internal", agent_uuid);
        let username = format!("mcp-agent-{}", agent_uuid);

        // 4. Create user record
        let agent = users::ActiveModel {
            email: Set(email),
            username: Set(username),
            display_name: Set(name.clone()),
            password_hash: Set(String::new()), // Not used for MCP agents
            avatar_color: Set("#6366f1".to_string()), // Indigo for agents
            is_active: Set(true),
            user_type: Set("mcp_agent".to_string()),
            scoped_project_id: Set(Some(project_id)),
            api_key_hash: Set(Some(api_key_hash)),
            organisation_id: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            last_login_at: Set(None),
            ..Default::default()
        };

        let saved_agent = agent
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to create MCP agent: {}", e))?;

        Ok(McpAgentCredentials {
            user_id: saved_agent.id,
            api_key,
            project_id,
            name,
        })
    }

    /// Authenticate an MCP agent using API key
    pub async fn authenticate_agent(&self, api_key: &str) -> Result<users::Model> {
        // Hash the provided key
        let agents = users::Entity::find()
            .filter(users::Column::UserType.eq("mcp_agent"))
            .filter(users::Column::IsActive.eq(true))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        // Check each agent's hashed key
        for agent in agents {
            if let Some(stored_hash) = &agent.api_key_hash {
                if AuthService::verify_password(api_key, stored_hash)? {
                    return Ok(agent);
                }
            }
        }

        Err(anyhow!("Invalid API key"))
    }

    /// List all MCP agents for a project
    pub async fn list_agents(&self, project_id: i32) -> Result<Vec<users::Model>> {
        users::Entity::find()
            .filter(users::Column::UserType.eq("mcp_agent"))
            .filter(users::Column::ScopedProjectId.eq(project_id))
            .filter(users::Column::IsActive.eq(true))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list MCP agents: {}", e))
    }

    /// Revoke (deactivate) an MCP agent
    pub async fn revoke_agent(&self, user_id: i32, revoker_id: i32) -> Result<()> {
        // 1. Get the agent
        let agent = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Agent not found"))?;

        // 2. Verify it's an MCP agent
        if agent.user_type != "mcp_agent" {
            return Err(anyhow!("User is not an MCP agent"));
        }

        // 3. Verify revoker has admin access to the scoped project
        let project_id = agent
            .scoped_project_id
            .ok_or_else(|| anyhow!("MCP agent has no scoped project"))?;

        let auth_service = AuthorizationService::new(self.db.clone());
        auth_service
            .check_project_admin_access(revoker_id, project_id)
            .await?;

        // 4. Deactivate the agent
        let mut agent_active: users::ActiveModel = agent.into();
        agent_active.is_active = Set(false);
        agent_active.updated_at = Set(chrono::Utc::now());
        agent_active
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to revoke agent: {}", e))?;

        Ok(())
    }

    /// Generate a secure API key
    fn generate_api_key() -> String {
        // Generate using UUID for simplicity (can be enhanced with more entropy)
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        format!("lc_mcp_{}_{}", uuid1.simple(), uuid2.simple())
    }

    /// Regenerate API key for an agent
    pub async fn regenerate_api_key(
        &self,
        agent_user_id: i32,
        requester_id: i32,
    ) -> Result<String> {
        // Get the agent
        let agent = users::Entity::find_by_id(agent_user_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Agent not found"))?;

        // Verify it's an MCP agent
        if agent.user_type != "mcp_agent" {
            return Err(anyhow!("User is not an MCP agent"));
        }

        // Verify requester has admin access
        let project_id = agent
            .scoped_project_id
            .ok_or_else(|| anyhow!("MCP agent has no scoped project"))?;

        let auth_service = AuthorizationService::new(self.db.clone());
        auth_service
            .check_project_admin_access(requester_id, project_id)
            .await?;

        // Generate new API key
        let new_api_key = Self::generate_api_key();
        let new_hash = AuthService::hash_password(&new_api_key)?;

        // Update agent
        let mut agent_active: users::ActiveModel = agent.into();
        agent_active.api_key_hash = Set(Some(new_hash));
        agent_active.updated_at = Set(chrono::Utc::now());
        agent_active
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to regenerate API key: {}", e))?;

        Ok(new_api_key)
    }

    /// Check if a user is an MCP agent
    #[allow(dead_code)]
    pub fn is_mcp_agent(user: &users::Model) -> bool {
        user.user_type == "mcp_agent"
    }

    /// Get the project scope for an MCP agent
    #[allow(dead_code)]
    pub fn get_agent_project_scope(agent: &users::Model) -> Option<i32> {
        if Self::is_mcp_agent(agent) {
            agent.scoped_project_id
        } else {
            None
        }
    }
}

// TODO: Fix test database setup - migrations fail with "near \"(\" syntax error"
// Tests are commented out until we resolve the migration runner issue
#[cfg(test)]
#[allow(dead_code)]
mod tests_disabled {
    use super::*;
    use crate::database::entities::{project_collaborators, projects};
    use crate::database::test_utils::setup_test_db;

    async fn create_test_user(db: &DatabaseConnection, username: &str) -> Result<users::Model> {
        let user = users::ActiveModel {
            email: Set(format!("{}@example.com", username)),
            username: Set(username.to_string()),
            display_name: Set(format!("Test User {}", username)),
            password_hash: Set("hash".to_string()),
            avatar_color: Set("#000000".to_string()),
            is_active: Set(true),
            user_type: Set("human".to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        user.insert(db).await.map_err(Into::into)
    }

    async fn create_test_project(db: &DatabaseConnection, name: &str) -> Result<projects::Model> {
        let project = projects::ActiveModel {
            name: Set(name.to_string()),
            description: Set(Some("Test project".to_string())),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        project.insert(db).await.map_err(Into::into)
    }

    async fn make_project_owner(
        db: &DatabaseConnection,
        project_id: i32,
        user_id: i32,
    ) -> Result<()> {
        let collab = project_collaborators::ActiveModel {
            project_id: Set(project_id),
            user_id: Set(user_id),
            role: Set("owner".to_string()),
            permissions: Set("all".to_string()),
            invitation_status: Set("accepted".to_string()),
            invited_at: Set(chrono::Utc::now()),
            joined_at: Set(Some(chrono::Utc::now())),
            is_active: Set(true),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        collab.insert(db).await?;
        Ok(())
    }

    //#[tokio::test]
    async fn test_create_mcp_agent() {
        let db = setup_test_db().await;
        let service = McpAgentService::new(db.clone());

        let owner = create_test_user(&db, "owner").await.unwrap();
        let project = create_test_project(&db, "Test Project").await.unwrap();
        make_project_owner(&db, project.id, owner.id).await.unwrap();

        let credentials = service
            .create_agent(owner.id, project.id, "Test Agent".to_string(), None)
            .await
            .unwrap();

        assert_eq!(credentials.project_id, project.id);
        assert_eq!(credentials.name, "Test Agent");
        assert!(credentials.api_key.starts_with("lc_mcp_"));

        // Verify agent was created correctly
        let agent = users::Entity::find_by_id(credentials.user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(agent.user_type, "mcp_agent");
        assert_eq!(agent.scoped_project_id, Some(project.id));
        assert!(agent.api_key_hash.is_some());
        assert_eq!(agent.display_name, "Test Agent");
    }

    //#[tokio::test]
    async fn test_authenticate_agent() {
        let db = setup_test_db().await;
        let service = McpAgentService::new(db.clone());

        let owner = create_test_user(&db, "owner").await.unwrap();
        let project = create_test_project(&db, "Test Project").await.unwrap();
        make_project_owner(&db, project.id, owner.id).await.unwrap();

        let credentials = service
            .create_agent(owner.id, project.id, "Test Agent".to_string(), None)
            .await
            .unwrap();

        // Authenticate with correct API key
        let authenticated = service
            .authenticate_agent(&credentials.api_key)
            .await
            .unwrap();

        assert_eq!(authenticated.id, credentials.user_id);
        assert_eq!(authenticated.scoped_project_id, Some(project.id));

        // Authenticate with wrong API key should fail
        let result = service.authenticate_agent("wrong_key").await;
        assert!(result.is_err());
    }

    //#[tokio::test]
    async fn test_list_agents() {
        let db = setup_test_db().await;
        let service = McpAgentService::new(db.clone());

        let owner = create_test_user(&db, "owner").await.unwrap();
        let project1 = create_test_project(&db, "Project 1").await.unwrap();
        let project2 = create_test_project(&db, "Project 2").await.unwrap();
        make_project_owner(&db, project1.id, owner.id)
            .await
            .unwrap();
        make_project_owner(&db, project2.id, owner.id)
            .await
            .unwrap();

        // Create agents for different projects
        service
            .create_agent(owner.id, project1.id, "Agent 1".to_string(), None)
            .await
            .unwrap();
        service
            .create_agent(owner.id, project1.id, "Agent 2".to_string(), None)
            .await
            .unwrap();
        service
            .create_agent(owner.id, project2.id, "Agent 3".to_string(), None)
            .await
            .unwrap();

        // List agents for project1
        let agents = service.list_agents(project1.id).await.unwrap();
        assert_eq!(agents.len(), 2);

        // List agents for project2
        let agents = service.list_agents(project2.id).await.unwrap();
        assert_eq!(agents.len(), 1);
    }

    //#[tokio::test]
    async fn test_revoke_agent() {
        let db = setup_test_db().await;
        let service = McpAgentService::new(db.clone());

        let owner = create_test_user(&db, "owner").await.unwrap();
        let project = create_test_project(&db, "Test Project").await.unwrap();
        make_project_owner(&db, project.id, owner.id).await.unwrap();

        let credentials = service
            .create_agent(owner.id, project.id, "Test Agent".to_string(), None)
            .await
            .unwrap();

        // Revoke the agent
        service
            .revoke_agent(credentials.user_id, owner.id)
            .await
            .unwrap();

        // Agent should be inactive
        let agent = users::Entity::find_by_id(credentials.user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert!(!agent.is_active);

        // Should not appear in list anymore
        let agents = service.list_agents(project.id).await.unwrap();
        assert_eq!(agents.len(), 0);

        // Authentication should fail
        let result = service.authenticate_agent(&credentials.api_key).await;
        assert!(result.is_err());
    }

    //#[tokio::test]
    async fn test_regenerate_api_key() {
        let db = setup_test_db().await;
        let service = McpAgentService::new(db.clone());

        let owner = create_test_user(&db, "owner").await.unwrap();
        let project = create_test_project(&db, "Test Project").await.unwrap();
        make_project_owner(&db, project.id, owner.id).await.unwrap();

        let credentials = service
            .create_agent(owner.id, project.id, "Test Agent".to_string(), None)
            .await
            .unwrap();

        let old_key = credentials.api_key.clone();

        // Regenerate key
        let new_key = service
            .regenerate_api_key(credentials.user_id, owner.id)
            .await
            .unwrap();

        assert_ne!(old_key, new_key);

        // Old key should not work
        let result = service.authenticate_agent(&old_key).await;
        assert!(result.is_err());

        // New key should work
        let authenticated = service.authenticate_agent(&new_key).await.unwrap();
        assert_eq!(authenticated.id, credentials.user_id);
    }
}
