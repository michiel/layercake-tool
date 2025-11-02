# Chat System Enhancement Design

## Executive Summary

This design document outlines a comprehensive enhancement to the Layercake chat implementation, introducing persistent chat history, multi-tenant user authentication with RBAC, project-scoped MCP user identities, and improved UI for session management. The design addresses the need for chat sessions to be project-specific, persistent, and properly scoped with appropriate access controls.

## Current State Analysis

### Existing Chat Implementation

The current chat system (implemented in `layercake-core/src/console/chat/` and exposed via GraphQL) has:

1. **In-memory sessions**: `ChatManager` maintains runtime sessions in a HashMap with no persistence
2. **System-level auth**: Chat sessions run under `SecurityContext::system()` with full privileges
3. **No history persistence**: Conversation history exists only in memory (`VecDeque<ChatEvent>` with 64-item limit)
4. **Single project context**: Each session is tied to a project_id but sessions aren't persisted or selectable
5. **Provider configuration**: Uses `chat_credentials` table for API keys (provider, api_key, base_url)
6. **MCP tool integration**: `McpBridge` exposes tools to LLM via `LayercakeServerState`
7. **Basic UI**: `ProjectChatPage.tsx` creates ephemeral sessions, no history or session selection

### Existing User/Auth System

Current implementation includes:

1. **Users table**: email, username, display_name, password_hash, avatar_color, is_active
2. **User sessions**: session_id, user_id, project_id, expires_at, is_active
3. **Project collaborators**: user-project association with roles (owner, editor, viewer) and permissions
4. **Authorization service**: Role-based checks (owner → editor → viewer hierarchy)
5. **MCP auth**: `McpAuthConfig` supports API keys, OAuth2 (planned), certificates, or anonymous access
6. **Security contexts**: System (full access), authenticated (limited), anonymous (read-only)

### Existing MCP Integration

1. **Tool registry**: `LayercakeToolRegistry` exposes project, plan, graph_data, analysis, and data_source tools
2. **Auth manager**: `LayercakeAuth` validates API keys and checks permissions
3. **Authorization**: `authorize()` method checks resource/action pairs against security context
4. **Security scoping**: Currently uses client_id and session_id but no project-level scoping

## Design Requirements

### Functional Requirements

1. **Persistent chat history**: Store all chat interactions in database with project association
2. **Session management**: Allow users to create, select, resume, and archive chat sessions
3. **Project-scoped access**: Chat sessions and MCP tools must be limited to specific projects
4. **Multi-tenant support**: Support local (Tauri desktop), organisation-based (multi-user), and MCP agent identities
5. **RBAC implementation**: Enforce role-based permissions for chat operations and tool access
6. **System prompt enhancement**: Provide context about Layercake data model and available tools
7. **Output formatting**: Transform raw MCP tool output into user-friendly responses
8. **UI improvements**: Session selection, history display, new session creation

### Non-Functional Requirements

1. **Performance**: Chat history queries must be efficient for projects with many messages
2. **Security**: API keys and passwords secured; federated auth support planned for future
3. **Scalability**: Design must support multiple concurrent chat sessions per user
4. **Maintainability**: Clean separation between auth contexts and business logic
5. **Backward compatibility**: Existing chat functionality must continue to work during migration

## Database Schema Design

### New Tables

#### chat_sessions

Stores persistent chat sessions associated with projects.

```sql
CREATE TABLE chat_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,  -- UUID for external reference
    project_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,  -- Owner of the session
    title TEXT,  -- Optional user-provided title
    provider TEXT NOT NULL,  -- "ollama", "openai", "gemini", "claude"
    model_name TEXT NOT NULL,  -- Model used for this session
    system_prompt TEXT,  -- Custom system prompt if any
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    last_activity_at TIMESTAMP NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_chat_sessions_project ON chat_sessions(project_id);
CREATE INDEX idx_chat_sessions_user ON chat_sessions(user_id);
CREATE INDEX idx_chat_sessions_activity ON chat_sessions(last_activity_at);
```

#### chat_messages

Stores individual messages within chat sessions.

```sql
CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    message_id TEXT NOT NULL UNIQUE,  -- UUID for idempotency
    role TEXT NOT NULL,  -- "user", "assistant", "tool"
    content TEXT NOT NULL,
    tool_name TEXT,  -- For tool invocation messages
    tool_call_id TEXT,  -- Links tool results to calls
    metadata_json TEXT,  -- Additional data (model params, token counts, etc.)
    created_at TIMESTAMP NOT NULL,

    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_chat_messages_session ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_created ON chat_messages(created_at);
```

### Modified Tables

#### users

Extend to support different user types:

```sql
ALTER TABLE users ADD COLUMN user_type TEXT NOT NULL DEFAULT 'human';
  -- Values: 'human' (local/org user), 'mcp_agent' (project-scoped agent)

ALTER TABLE users ADD COLUMN scoped_project_id INTEGER;
  -- NULL for human users, set for MCP agents to limit access

ALTER TABLE users ADD COLUMN api_key_hash TEXT;
  -- For MCP agents, store hashed API key

ALTER TABLE users ADD COLUMN organisation_id INTEGER;
  -- Future: links users to organisations for multi-tenancy

CREATE INDEX idx_users_organisation ON users(organisation_id);
CREATE INDEX idx_users_scoped_project ON users(scoped_project_id);

-- Note: organisation_id FK constraint added when organisations table is created
```

#### user_sessions

Extend to track authentication method:

```sql
ALTER TABLE user_sessions ADD COLUMN auth_method TEXT NOT NULL DEFAULT 'password';
  -- Values: 'password', 'api_key', 'oauth', 'local'

ALTER TABLE user_sessions ADD COLUMN auth_context TEXT;
  -- JSON storing additional auth metadata
```

### Future Tables

#### organisations

For multi-tenant support (to be implemented):

```sql
CREATE TABLE organisations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    settings_json TEXT,  -- Organisation-wide settings
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_organisations_slug ON organisations(slug);
```

#### organisation_members

Links users to organisations with roles:

```sql
CREATE TABLE organisation_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    organisation_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    role TEXT NOT NULL,  -- "admin", "member", "billing"
    permissions TEXT NOT NULL,  -- JSON array of permission strings
    joined_at TIMESTAMP NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    FOREIGN KEY (organisation_id) REFERENCES organisations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(organisation_id, user_id)
);

CREATE INDEX idx_org_members_org ON organisation_members(organisation_id);
CREATE INDEX idx_org_members_user ON organisation_members(user_id);
```

## User Model Design

### User Types Hierarchy

```
User (base type)
├── HumanUser (user_type: 'human')
│   ├── LocalUser (auth_method: 'local')
│   │   └── For Tauri desktop mode
│   │       - No organisation_id
│   │       - Password-based or keyring auth
│   │       - Full local access
│   │
│   └── OrganisationUser (auth_method: 'password' | 'oauth')
│       └── For web/multi-tenant mode
│           - Has organisation_id
│           - Associated via organisation_members
│           - Access controlled by org membership
│
└── McpAgent (user_type: 'mcp_agent')
    └── Project-scoped AI agent
        - Has scoped_project_id (required)
        - Has api_key_hash (required)
        - auth_method: 'api_key'
        - Cannot access other projects
        - Limited to MCP tools within scope
```

### Authentication Methods

| User Type | Auth Method | Credentials | Session Type |
|-----------|-------------|-------------|--------------|
| LocalUser | `local` | Password (optional), keyring | Long-lived local session |
| OrganisationUser | `password` | Email + password hash | Token-based, expires |
| OrganisationUser | `oauth` | OAuth2 token (future) | Token-based, expires |
| McpAgent | `api_key` | API key hash | Per-request or session |

### RBAC Model

#### Levels of Access

1. **System Level**: Internal service operations, full access
2. **Organisation Level**: (Future) Org admins can manage members, billing
3. **Project Level**: Existing role hierarchy (owner → editor → viewer)
4. **MCP Agent Level**: Scoped to single project, limited tool access

#### Permission Structure

```rust
pub enum UserPermission {
    // Project permissions (existing)
    ProjectRead,
    ProjectWrite,
    ProjectAdmin,
    ProjectDelete,

    // Chat permissions (new)
    ChatCreate,
    ChatRead,
    ChatWrite,  // Send messages
    ChatDelete,

    // MCP tool permissions (new)
    ToolExecute,
    ToolReadOutput,

    // Organisation permissions (future)
    OrgManageMembers,
    OrgManageBilling,
    OrgViewAuditLog,
}

pub struct UserContext {
    pub user: User,
    pub auth_method: AuthMethod,
    pub project_role: Option<ProjectRole>,  // When in project context
    pub org_role: Option<OrgRole>,  // When in org context
    pub scoped_project: Option<i32>,  // For MCP agents
}

impl UserContext {
    pub fn has_permission(&self, permission: UserPermission, resource_id: Option<i32>) -> bool {
        // Implementation checks user type, role, and scope
    }

    pub fn can_access_project(&self, project_id: i32) -> bool {
        // For MCP agents: project_id == scoped_project_id
        // For humans: check project_collaborators
    }

    pub fn allowed_mcp_tools(&self) -> Vec<String> {
        // Returns list of MCP tool names this user can invoke
        // Filtered based on user type and permissions
    }
}
```

## MCP User Identity Design

### Project-Scoped MCP Agents

MCP agents are special users that:
- Are created per-project (1:1 relationship)
- Authenticate via API key only
- Have limited tool access (no project/user management tools)
- Cannot escalate privileges or access other projects
- Exist to provide secure, scoped access for AI assistants

### Creating MCP Agents

```rust
pub struct CreateMcpAgentRequest {
    pub project_id: i32,
    pub name: String,  // e.g., "Project Assistant for Attack Tree"
    pub allowed_tools: Vec<String>,  // Optional whitelist
}

pub async fn create_mcp_agent(
    db: &DatabaseConnection,
    creator_user_id: i32,
    req: CreateMcpAgentRequest,
) -> Result<McpAgentCredentials> {
    // 1. Verify creator has ProjectAdmin role
    // 2. Generate API key
    // 3. Create user with user_type='mcp_agent', scoped_project_id=project_id
    // 4. Hash and store API key
    // 5. Create user_session with auth_method='api_key'
    // 6. Return API key (only time it's visible)
}
```

### Tool Filtering

Certain tools should NEVER be exposed to MCP agents:

```rust
const MCP_AGENT_BLACKLIST: &[&str] = &[
    "create_project",
    "delete_project",
    "list_projects",  // Already scoped, no need to list
    "register_user",
    "login_user",
    "change_password",
    "create_mcp_agent",
];

impl LayercakeToolRegistry {
    async fn list_tools(&self, context: &SecurityContext) -> McpResult<Vec<Tool>> {
        let all_tools = self.get_all_tools();

        if context.is_mcp_agent() {
            // Filter out blacklisted tools
            all_tools.into_iter()
                .filter(|tool| !MCP_AGENT_BLACKLIST.contains(&tool.name.as_str()))
                .collect()
        } else {
            all_tools
        }
    }
}
```

### Tool Execution with Scoping

```rust
impl LayercakeToolRegistry {
    async fn execute_tool(
        &self,
        tool_name: &str,
        context: &SecurityContext,
        arguments: Option<Value>,
    ) -> McpResult<ToolsCallResult> {
        // 1. Check if user has permission to execute this tool
        if !context.can_execute_tool(tool_name) {
            return Err(McpError::PermissionDenied { ... });
        }

        // 2. If MCP agent, inject scoped project_id into arguments
        let scoped_args = if let Some(project_id) = context.scoped_project_id() {
            self.inject_project_scope(arguments, project_id)?
        } else {
            arguments
        };

        // 3. Execute tool with scoped arguments
        self.dispatch_tool(tool_name, scoped_args).await
    }
}
```

## System Prompt Design

### Enhanced System Prompt Structure

```rust
fn compose_enhanced_system_prompt(
    config: &ChatConfig,
    project_id: i32,
    project: &Project,
    available_tools: &[Tool],
    user_context: &UserContext,
) -> String {
    let mut prompt = String::new();

    // 1. Identity and role
    prompt.push_str(&format!(
        "You are the Layercake assistant for project '{}' (ID: {}).\n\n",
        project.name, project_id
    ));

    // 2. Explain Layercake data model
    prompt.push_str(&load_data_model_explanation());

    // 3. Available tools with descriptions
    prompt.push_str("\n## Available Tools\n\n");
    prompt.push_str("You have access to the following MCP tools. ");
    prompt.push_str("Always use tools to fetch fresh data before answering questions.\n\n");

    for tool in available_tools {
        prompt.push_str(&format!(
            "- **{}**: {}\n",
            tool.name,
            tool.description.as_deref().unwrap_or("No description")
        ));
    }

    // 4. Output formatting guidelines
    prompt.push_str("\n");
    prompt.push_str(&load_output_formatting_guidelines());

    // 5. Custom prompt from config
    if let Some(custom) = &config.system_prompt {
        prompt.push_str("\n## Additional Instructions\n\n");
        prompt.push_str(custom);
    }

    prompt
}

// Load data model explanation from resources
fn load_data_model_explanation() -> String {
    std::fs::read_to_string("resources/chat-data-model.md")
        .unwrap_or_else(|_| "Data model documentation not available.".to_string())
}

// Load output formatting guidelines from resources
fn load_output_formatting_guidelines() -> String {
    std::fs::read_to_string("resources/chat-output-formatting.md")
        .unwrap_or_else(|_| "Output formatting guidelines not available.".to_string())
}
```

## Output Formatting Strategy

### Tool Output Transformer

```rust
pub struct ToolOutputFormatter {
    config: FormatterConfig,
}

impl ToolOutputFormatter {
    /// Transform raw MCP tool output into user-friendly text
    pub fn format_tool_result(
        &self,
        tool_name: &str,
        result: &ToolsCallResult,
    ) -> String {
        if result.is_error {
            return self.format_error(tool_name, result);
        }

        match tool_name {
            "list_graphs" => self.format_graph_list(result),
            "get_graph_data" => self.format_graph_data(result),
            "analyse_graph" => self.format_analysis(result),
            "list_data_sources" => self.format_data_sources(result),
            _ => self.format_generic(result),
        }
    }

    fn format_graph_list(&self, result: &ToolsCallResult) -> String {
        // Parse JSON, create readable table
        let json = self.extract_json(result);
        let graphs: Vec<Graph> = serde_json::from_value(json).unwrap();

        let mut output = format!("Found {} graphs:\n\n", graphs.len());
        for g in graphs {
            output.push_str(&format!(
                "- {} (ID: {}): {} nodes, {} edges\n",
                g.name, g.id, g.node_count, g.edge_count
            ));
        }
        output
    }

    // Similar methods for other tool types...
}
```

### Integration with Chat Session

```rust
impl ChatSession {
    async fn handle_tool_calls(&mut self, tool_calls: Vec<ToolCall>) -> Result<()> {
        for call in tool_calls {
            // Execute tool
            let result = self.bridge.execute_tool(&call.function.name, &self.security, args).await?;

            // Format output for user
            let formatted = self.formatter.format_tool_result(&call.function.name, &result);

            // Store formatted output for UI
            self.observer(ChatEvent::ToolInvocation {
                name: call.function.name.clone(),
                summary: formatted,
            });

            // Send raw result to LLM for processing
            let llm_payload = self.bridge.serialize_tool_result(&result);
            self.messages.push(ChatMessage::tool_result(llm_payload));
        }

        Ok(())
    }
}
```

## UI Design Changes

### Session Management UI

```typescript
// New component: ChatSessionList.tsx
interface ChatSession {
  id: string
  sessionId: string
  title: string | null
  projectId: number
  provider: string
  model: string
  isArchived: boolean
  lastActivityAt: string
  messageCount: number
}

const ChatSessionList = ({ projectId }: { projectId: number }) => {
  const { data, loading } = useQuery(LIST_CHAT_SESSIONS, {
    variables: { projectId }
  })

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>Chat Sessions</Title>
        <Button onClick={createNewSession}>New Chat</Button>
      </Group>

      <ScrollArea>
        {data?.chatSessions.map(session => (
          <Card key={session.id} onClick={() => selectSession(session.sessionId)}>
            <Text weight={500}>{session.title || 'Untitled Chat'}</Text>
            <Group>
              <Badge>{session.provider}</Badge>
              <Text size="sm" c="dimmed">{session.model}</Text>
            </Group>
            <Text size="xs" c="dimmed">
              {formatRelativeTime(session.lastActivityAt)} • {session.messageCount} messages
            </Text>
          </Card>
        ))}
      </ScrollArea>
    </Stack>
  )
}
```

### Updated ProjectChatPage

```typescript
export const ProjectChatPage = () => {
  const { projectId } = useParams()
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const [showSessionList, setShowSessionList] = useState(true)

  return (
    <Group h="100%" align="stretch" gap={0}>
      {/* Left sidebar: Session list */}
      {showSessionList && (
        <Paper w={300} withBorder p="md">
          <ChatSessionList
            projectId={projectId}
            selectedSessionId={selectedSessionId}
            onSelectSession={setSelectedSessionId}
          />
        </Paper>
      )}

      {/* Main area: Chat interface */}
      <Box style={{ flex: 1 }}>
        {selectedSessionId ? (
          <ChatInterface
            sessionId={selectedSessionId}
            onBack={() => setSelectedSessionId(null)}
          />
        ) : (
          <EmptyState onCreateNew={() => createSession()} />
        )}
      </Box>
    </Group>
  )
}
```

### Chat History Display

```typescript
const ChatInterface = ({ sessionId }: { sessionId: string }) => {
  // Load existing messages from database
  const { data: history } = useQuery(GET_CHAT_HISTORY, {
    variables: { sessionId }
  })

  // Subscribe to new messages
  const { data: newMessages } = useSubscription(CHAT_MESSAGES, {
    variables: { sessionId }
  })

  const allMessages = useMemo(() => {
    return [...(history?.chatMessages || []), ...(newMessages || [])]
  }, [history, newMessages])

  return (
    <Stack h="100%">
      {/* Session header with title, model, archive button */}
      <ChatSessionHeader sessionId={sessionId} />

      {/* Scrollable message history */}
      <ScrollArea style={{ flex: 1 }}>
        {allMessages.map(msg => (
          <ChatMessage key={msg.id} message={msg} />
        ))}
      </ScrollArea>

      {/* Input area */}
      <ChatInput onSend={sendMessage} />
    </Stack>
  )
}
```

## GraphQL Schema Changes

### New Types

```graphql
enum UserType {
  HUMAN
  MCP_AGENT
}

enum AuthMethod {
  LOCAL
  PASSWORD
  API_KEY
  OAUTH
}

type User {
  id: Int!
  email: String!
  username: String!
  displayName: String!
  userType: UserType!
  authMethod: AuthMethod!
  scopedProjectId: Int
  organisationId: Int
  isActive: Boolean!
  createdAt: DateTime!
}

type ChatSession {
  id: Int!
  sessionId: String!
  project: Project!
  user: User!
  title: String
  provider: ChatProviderOption!
  modelName: String!
  isArchived: Boolean!
  createdAt: DateTime!
  updatedAt: DateTime!
  lastActivityAt: DateTime!
  messageCount: Int!
  messages(limit: Int, offset: Int): [ChatMessage!]!
}

type ChatMessage {
  id: Int!
  messageId: String!
  role: ChatMessageRole!
  content: String!
  toolName: String
  toolCallId: String
  metadata: JSON
  createdAt: DateTime!
}

enum ChatMessageRole {
  USER
  ASSISTANT
  TOOL
}

input CreateChatSessionInput {
  projectId: Int!
  provider: ChatProviderOption!
  title: String
}

input SendChatMessageInput {
  sessionId: String!
  content: String!
}

type CreateMcpAgentInput {
  projectId: Int!
  name: String!
  allowedTools: [String!]
}

type McpAgentCredentials {
  userId: Int!
  apiKey: String!
  projectId: Int!
  name: String!
}
```

### New Queries

```graphql
extend type Query {
  """List chat sessions for a project"""
  chatSessions(
    projectId: Int!
    includeArchived: Boolean = false
    limit: Int = 50
    offset: Int = 0
  ): [ChatSession!]!

  """Get a specific chat session"""
  chatSession(sessionId: String!): ChatSession

  """Get chat history for a session"""
  chatHistory(
    sessionId: String!
    limit: Int = 100
    offset: Int = 0
  ): [ChatMessage!]!
}
```

### New Mutations

```graphql
extend type Mutation {
  """Create a new chat session"""
  createChatSession(input: CreateChatSessionInput!): ChatSession!

  """Update chat session metadata"""
  updateChatSession(
    sessionId: String!
    title: String
    isArchived: Boolean
  ): ChatSession!

  """Delete a chat session and all messages"""
  deleteChatSession(sessionId: String!): Boolean!

  """Send a message in a chat session"""
  sendChatMessage(input: SendChatMessageInput!): ChatMessage!

  """Create a project-scoped MCP agent"""
  createMcpAgent(input: CreateMcpAgentInput!): McpAgentCredentials!

  """Revoke MCP agent access"""
  revokeMcpAgent(userId: Int!): Boolean!
}
```

### Updated Subscriptions

```graphql
extend type Subscription {
  """Subscribe to new messages in a chat session"""
  chatMessages(sessionId: String!): ChatMessage!

  """Subscribe to session updates (title, archive status)"""
  chatSessionUpdated(sessionId: String!): ChatSession!
}
```

## Service Layer Design

### ChatHistoryService

```rust
pub struct ChatHistoryService {
    db: DatabaseConnection,
}

impl ChatHistoryService {
    pub async fn create_session(
        &self,
        project_id: i32,
        user_id: i32,
        provider: ChatProvider,
        model_name: String,
    ) -> Result<ChatSessionModel> {
        // 1. Verify user has access to project
        // 2. Create chat_sessions record
        // 3. Return session with UUID
    }

    pub async fn list_sessions(
        &self,
        project_id: i32,
        user_id: Option<i32>,  // Filter by user
        include_archived: bool,
    ) -> Result<Vec<ChatSessionModel>> {
        // Query chat_sessions with filters
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<ChatSessionModel>> {
        // Get session by UUID
    }

    pub async fn store_message(
        &self,
        session_id: &str,
        role: MessageRole,
        content: String,
        tool_name: Option<String>,
        tool_call_id: Option<String>,
    ) -> Result<ChatMessageModel> {
        // 1. Insert into chat_messages
        // 2. Update chat_sessions.last_activity_at
        // 3. Return message
    }

    pub async fn get_history(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ChatMessageModel>> {
        // Query chat_messages for session
    }

    pub async fn update_session_title(
        &self,
        session_id: &str,
        title: String,
    ) -> Result<()> {
        // Update chat_sessions.title
    }

    pub async fn archive_session(&self, session_id: &str) -> Result<()> {
        // Set chat_sessions.is_archived = true
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        // CASCADE delete will remove messages
    }
}
```

### McpAgentService

```rust
pub struct McpAgentService {
    db: DatabaseConnection,
    auth_service: AuthService,
}

impl McpAgentService {
    pub async fn create_agent(
        &self,
        creator_user_id: i32,
        project_id: i32,
        name: String,
        allowed_tools: Option<Vec<String>>,
    ) -> Result<McpAgentCredentials> {
        // 1. Check creator has ProjectAdmin role
        let auth = AuthorizationService::new(self.db.clone());
        auth.check_project_admin_access(creator_user_id, project_id).await?;

        // 2. Generate API key
        let api_key = generate_secure_api_key();
        let api_key_hash = hash_api_key(&api_key);

        // 3. Create user record
        let agent_user = users::ActiveModel {
            email: Set(format!("mcp-agent-{}@layercake.internal", Uuid::new_v4())),
            username: Set(format!("mcp-agent-{}", Uuid::new_v4())),
            display_name: Set(name.clone()),
            password_hash: Set(String::new()),  // Not used
            user_type: Set("mcp_agent".to_string()),
            scoped_project_id: Set(Some(project_id)),
            api_key_hash: Set(Some(api_key_hash)),
            is_active: Set(true),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let agent = agent_user.insert(&self.db).await?;

        // 4. Return credentials (only time API key is visible)
        Ok(McpAgentCredentials {
            user_id: agent.id,
            api_key,
            project_id,
            name,
        })
    }

    pub async fn authenticate_agent(&self, api_key: &str) -> Result<UserContext> {
        // 1. Hash provided key
        let key_hash = hash_api_key(api_key);

        // 2. Find user with matching api_key_hash
        let agent = users::Entity::find()
            .filter(users::Column::ApiKeyHash.eq(key_hash))
            .filter(users::Column::UserType.eq("mcp_agent"))
            .filter(users::Column::IsActive.eq(true))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Invalid API key"))?;

        // 3. Create user context with project scope
        Ok(UserContext {
            user: agent.clone(),
            auth_method: AuthMethod::ApiKey,
            project_role: Some(ProjectRole::Editor),  // Fixed role for agents
            org_role: None,
            scoped_project: agent.scoped_project_id,
        })
    }

    pub async fn list_agents(&self, project_id: i32) -> Result<Vec<User>> {
        // List all MCP agents for a project
        users::Entity::find()
            .filter(users::Column::UserType.eq("mcp_agent"))
            .filter(users::Column::ScopedProjectId.eq(project_id))
            .filter(users::Column::IsActive.eq(true))
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn revoke_agent(&self, user_id: i32, revoker_id: i32) -> Result<()> {
        // 1. Get agent
        let agent = users::Entity::find_by_id(user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Agent not found"))?;

        // 2. Verify agent is MCP agent
        if agent.user_type != "mcp_agent" {
            return Err(anyhow!("Not an MCP agent"));
        }

        // 3. Verify revoker has admin access to project
        let auth = AuthorizationService::new(self.db.clone());
        auth.check_project_admin_access(
            revoker_id,
            agent.scoped_project_id.expect("MCP agent must have scoped project")
        ).await?;

        // 4. Deactivate agent
        let mut agent_active: users::ActiveModel = agent.into();
        agent_active.is_active = Set(false);
        agent_active.update(&self.db).await?;

        Ok(())
    }
}
```

## Migration Strategy

### Phase 1: Database Schema

- [ ] Create migration for `chat_sessions` table
- [ ] Create migration for `chat_messages` table
- [ ] Add `user_type` column to `users` table
- [ ] Add `scoped_project_id` column to `users` table
- [ ] Add `api_key_hash` column to `users` table
- [ ] Add `organisation_id` column to `users` table (nullable, for future)
- [ ] Add `auth_method` column to `user_sessions` table
- [ ] Add `auth_context` column to `user_sessions` table
- [ ] Create entity models for `chat_sessions`
- [ ] Create entity models for `chat_messages`
- [ ] Update entity models for `users` with new fields
- [ ] Update entity models for `user_sessions` with new fields
- [ ] Test migrations on development database
- [ ] Verify build passes: `cargo build`
- [ ] Verify tests pass: `cargo test`

### Phase 2: Service Layer

- [ ] Create `ChatHistoryService` struct
- [ ] Implement `create_session()` method
- [ ] Implement `list_sessions()` method
- [ ] Implement `get_session()` method
- [ ] Implement `store_message()` method
- [ ] Implement `get_history()` method
- [ ] Implement `update_session_title()` method
- [ ] Implement `archive_session()` method
- [ ] Implement `delete_session()` method
- [ ] Create `McpAgentService` struct
- [ ] Implement `create_agent()` method with API key generation
- [ ] Implement `authenticate_agent()` method
- [ ] Implement `list_agents()` method
- [ ] Implement `revoke_agent()` method
- [ ] Update `AuthorizationService` to handle user types
- [ ] Add `UserContext` struct with permission checking
- [ ] Write unit tests for `ChatHistoryService`
- [ ] Write unit tests for `McpAgentService`
- [ ] Write unit tests for updated `AuthorizationService`
- [ ] Verify build passes: `cargo build`
- [ ] Verify tests pass: `cargo test`

### Phase 3: MCP Integration

- [ ] Update `LayercakeAuth` to support MCP agent API key auth
- [ ] Add `is_mcp_agent()` method to `SecurityContext`
- [ ] Add `scoped_project_id()` method to `SecurityContext`
- [ ] Create tool blacklist constant for MCP agents
- [ ] Update `LayercakeToolRegistry::list_tools()` to filter by user type
- [ ] Implement `inject_project_scope()` helper method
- [ ] Update `LayercakeToolRegistry::execute_tool()` with scope checking
- [ ] Add integration test for MCP agent authentication
- [ ] Add integration test for MCP agent tool filtering
- [ ] Add integration test for project scope enforcement
- [ ] Verify build passes: `cargo build`
- [ ] Verify tests pass: `cargo test`

### Phase 4: Chat Manager Updates

- [ ] Update `ChatManager::start_session()` to persist to database
- [ ] Add `resume_session()` method to `ChatManager`
- [ ] Update `ChatSession` to accept existing history on creation
- [ ] Modify `ChatSession::send_message_with_observer()` to persist messages
- [ ] Update `compose_system_prompt()` to load from markdown files
- [ ] Create `ToolOutputFormatter` struct
- [ ] Implement `format_tool_result()` method
- [ ] Implement tool-specific formatters (graph_list, analysis, etc.)
- [ ] Update `handle_tool_calls()` to use formatter
- [ ] Write unit tests for `ToolOutputFormatter`
- [ ] Verify build passes: `cargo build`
- [ ] Verify tests pass: `cargo test`

### Phase 5: GraphQL Schema

- [ ] Add `UserType` enum to GraphQL schema
- [ ] Add `AuthMethod` enum to GraphQL schema
- [ ] Add `ChatSession` type to GraphQL schema
- [ ] Add `ChatMessage` type to GraphQL schema
- [ ] Add `ChatMessageRole` enum to GraphQL schema
- [ ] Add `CreateChatSessionInput` input type
- [ ] Add `SendChatMessageInput` input type
- [ ] Add `CreateMcpAgentInput` input type
- [ ] Add `McpAgentCredentials` type to GraphQL schema
- [ ] Implement `chatSessions` query resolver
- [ ] Implement `chatSession` query resolver
- [ ] Implement `chatHistory` query resolver
- [ ] Implement `createChatSession` mutation resolver
- [ ] Implement `updateChatSession` mutation resolver
- [ ] Implement `deleteChatSession` mutation resolver
- [ ] Implement `sendChatMessage` mutation resolver
- [ ] Implement `createMcpAgent` mutation resolver
- [ ] Implement `revokeMcpAgent` mutation resolver
- [ ] Update `chatMessages` subscription to use database history
- [ ] Add `chatSessionUpdated` subscription resolver
- [ ] Write integration tests for GraphQL operations
- [ ] Verify build passes: `cargo build --features graphql`
- [ ] Verify tests pass: `cargo test --features graphql`

### Phase 6: UI Updates

- [ ] Create `ChatSessionList.tsx` component
- [ ] Create `ChatSessionHeader.tsx` component
- [ ] Create `ChatMessage.tsx` component with role-based styling
- [ ] Update `ProjectChatPage.tsx` with session list sidebar
- [ ] Add `LIST_CHAT_SESSIONS` GraphQL query
- [ ] Add `GET_CHAT_HISTORY` GraphQL query
- [ ] Add `CREATE_CHAT_SESSION` GraphQL mutation
- [ ] Add `UPDATE_CHAT_SESSION` GraphQL mutation
- [ ] Add `DELETE_CHAT_SESSION` GraphQL mutation
- [ ] Update `useChatSession` hook to load history
- [ ] Add session selection state management
- [ ] Add session archive/delete controls
- [ ] Add session title editing
- [ ] Add empty state for new sessions
- [ ] Create `McpAgentManagement.tsx` component (admin)
- [ ] Add MCP agent creation UI
- [ ] Add MCP agent list/revoke UI
- [ ] Test UI flows end-to-end
- [ ] Verify frontend build passes: `npm run build`

### Phase 7: Testing & Documentation

- [ ] Write integration test for chat session persistence
- [ ] Write integration test for message storage
- [ ] Write integration test for session resumption
- [ ] Write integration test for MCP agent lifecycle
- [ ] Write integration test for project scope enforcement
- [ ] Load test with multiple concurrent sessions
- [ ] Document GraphQL API endpoints
- [ ] Document MCP agent creation process
- [ ] Create migration guide for existing deployments
- [ ] Update README with chat features
- [ ] Performance optimisation if needed
- [ ] Final verification: `cargo build && cargo test`
- [ ] Final verification: `npm run build`

## Security Considerations

### API Key Management

1. **Generation**: Use cryptographically secure random key generation (32+ bytes)
2. **Storage**: Never store plaintext API keys; use Argon2 or bcrypt hashing
3. **Transmission**: API keys only visible once at creation; show warning to user
4. **Rotation**: Provide mechanism to regenerate keys (revoke old, create new)
5. **Audit**: Log all API key usage and failed authentication attempts

### Access Control

1. **Project scoping**: MCP agents cannot list or access other projects
2. **Tool filtering**: Agents cannot execute user/project management tools
3. **Rate limiting**: Implement per-agent rate limits to prevent abuse
4. **Session expiry**: MCP agent sessions should have reasonable timeout
5. **Audit trail**: Log all tool executions with user context

### Data Privacy

1. **Chat isolation**: Users can only see their own chat sessions
2. **Project isolation**: Chat sessions scoped to projects user has access to
3. **Message encryption**: Consider encrypting chat_messages.content at rest (future)
4. **Data retention**: Implement chat history retention policies (future)
5. **GDPR compliance**: Provide data export and deletion mechanisms

## Open Questions

1. **Organisation hierarchy**: How should organisations relate to projects? Ownership model?
2. **OAuth2 providers**: Which OAuth2 providers to support initially? (Google, GitHub, Microsoft?)
3. **Chat session limits**: Should there be limits on number of sessions per project?
4. **Message retention**: How long to keep archived chat history? Auto-deletion policy?
5. **Tool output size**: Should large tool outputs be truncated or paginated in chat?
6. **Concurrent sessions**: Should users be limited to one active session per project?
7. **Agent sharing**: Should MCP agents be shareable across multiple projects? (Proposal: No)
8. **Billing integration**: How to track MCP API usage for billing purposes? (Future)

## Future Enhancements

### Short-term (3-6 months)

1. **Chat export**: Export chat sessions to Markdown/PDF
2. **Search**: Full-text search across chat history
3. **Sharing**: Share chat sessions with project collaborators
4. **Templates**: Save and reuse system prompts
5. **Analytics**: Track tool usage, popular queries, session duration

### Medium-term (6-12 months)

1. **Multi-modal**: Support image uploads and analysis in chat
2. **Voice**: Voice input/output for chat sessions
3. **Collaboration**: Real-time collaborative chat sessions
4. **Suggestions**: AI-suggested queries based on project state
5. **Automation**: Scheduled chat-based reports or alerts

### Long-term (12+ months)

1. **Marketplace**: Public/private agent marketplace
2. **Federation**: Cross-organisation agent sharing
3. **Blockchain**: Immutable audit trail for compliance
4. **Custom models**: Fine-tuned models per organisation
5. **Multi-agent**: Orchestrate multiple agents in single session

## Conclusion

This design provides a comprehensive enhancement to the Layercake chat system, addressing persistent history, multi-tenant authentication, project-scoped MCP agents, and improved user experience. The phased migration approach allows for incremental implementation while maintaining backward compatibility.

Key benefits:

- **Persistent chat history**: Users can resume conversations and maintain context
- **Secure multi-tenancy**: Support for local users, organisation users, and MCP agents
- **Project-scoped access**: MCP agents limited to specific projects with filtered tool access
- **Enhanced prompts**: Better context and formatting for more helpful responses
- **Improved UI**: Session management and history browsing
- **RBAC foundation**: Extensible permission model for future features

The implementation can proceed in parallel across backend (database, services, MCP) and frontend (UI, GraphQL) tracks, with integration points clearly defined.
