//! Layercake MCP server implementation using axum-mcp

use axum_mcp::prelude::*;
use axum_mcp::server::{PromptRegistry, ResourceRegistry};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;

use crate::app_context::AppContext;
use crate::mcp::security::{build_user_security_context, scoped_project_from_context};

/// Layercake-specific server state implementing axum-mcp traits
#[derive(Clone)]
pub struct LayercakeServerState {
    #[allow(dead_code)] // Available for MCP tools that need direct database access
    pub db: DatabaseConnection,
    #[allow(dead_code)]
    pub app: Arc<AppContext>,
    pub tools: LayercakeToolRegistry,
    pub resources: super::resources::LayercakeResourceRegistry,
    pub prompts: super::prompts::LayercakePromptRegistry,
    pub auth: LayercakeAuth,
}

/// Authentication manager with configurable security levels
#[derive(Clone)]
pub struct LayercakeAuth {
    pub allow_anonymous: bool,
    pub require_api_key: bool,
    pub valid_api_keys: std::collections::HashSet<String>,
    pub db: DatabaseConnection,
}

impl LayercakeAuth {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            allow_anonymous: std::env::var("LAYERCAKE_ALLOW_ANONYMOUS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            require_api_key: std::env::var("LAYERCAKE_REQUIRE_API_KEY")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            valid_api_keys: std::env::var("LAYERCAKE_API_KEYS")
                .unwrap_or_default()
                .split(',')
                .filter(|key| !key.is_empty())
                .map(|key| key.trim().to_string())
                .collect(),
            db,
        }
    }

    fn validate_api_key(&self, key: &str) -> bool {
        if !self.require_api_key {
            return true;
        }
        self.valid_api_keys.contains(key)
    }
}

#[async_trait]
impl McpAuth for LayercakeAuth {
    async fn authenticate(&self, client_info: &ClientContext) -> McpResult<SecurityContext> {
        // Try to authenticate as MCP agent first
        if let Some(auth_header) = client_info.metadata.get("authorization") {
            if let Some(api_key) = auth_header.strip_prefix("Bearer ") {
                // Check if this is a Layercake MCP agent API key
                if api_key.starts_with("lc_mcp_") {
                    use crate::services::mcp_agent_service::McpAgentService;
                    let service = McpAgentService::new(self.db.clone());

                    match service.authenticate_agent(api_key).await {
                        Ok(agent) => {
                            return Ok(build_user_security_context(
                                client_info.clone(),
                                agent.id,
                                &agent.user_type,
                                agent.scoped_project_id,
                            ));
                        }
                        Err(_) => {
                            return Err(McpError::Authentication {
                                message: "Invalid MCP agent API key".to_string(),
                            });
                        }
                    }
                }

                // Check if this is a legacy API key
                if self.validate_api_key(api_key) {
                    return Ok(SecurityContext::authenticated(
                        client_info.clone(),
                        vec!["api_key".to_string(), "authenticated".to_string()],
                    ));
                }
            }
        }

        // Check for API key in query parameters
        if let Some(api_key) = client_info.metadata.get("api_key") {
            if self.validate_api_key(api_key) {
                return Ok(SecurityContext::authenticated(
                    client_info.clone(),
                    vec!["api_key".to_string(), "authenticated".to_string()],
                ));
            }
        }

        if self.require_api_key {
            return Err(McpError::Authentication {
                message: "Valid API key required".to_string(),
            });
        }

        if self.allow_anonymous {
            Ok(SecurityContext::anonymous())
        } else {
            Err(McpError::Authentication {
                message: "Authentication required".to_string(),
            })
        }
    }

    async fn authorize(&self, context: &SecurityContext, resource: &str, action: &str) -> bool {
        if context.is_system() {
            // System context has full access
            true
        } else if context.is_authenticated() {
            // Authenticated users can access most resources but not destructive system operations
            !matches!((resource, action), ("projects", "delete") | ("system", _))
        } else if context.is_anonymous() {
            // Anonymous users have read-only access to non-sensitive resources
            matches!(
                (resource, action),
                ("projects", "read") | ("graph_data", "read") | ("analysis", "read")
            )
        } else {
            // Default deny
            false
        }
    }
}

impl McpServerState for LayercakeServerState {
    type ToolRegistry = LayercakeToolRegistry;
    type AuthManager = LayercakeAuth;

    fn tool_registry(&self) -> &Self::ToolRegistry {
        &self.tools
    }

    fn auth_manager(&self) -> &Self::AuthManager {
        &self.auth
    }

    fn resource_registry(&self) -> Option<&dyn ResourceRegistry> {
        Some(&self.resources)
    }

    fn prompt_registry(&self) -> Option<&dyn PromptRegistry> {
        Some(&self.prompts)
    }

    fn server_info(&self) -> axum_mcp::protocol::ServerInfo {
        axum_mcp::protocol::ServerInfo {
            name: "Layercake MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "description".to_string(),
                    json!("Graph visualization and transformation MCP server"),
                );
                meta.insert(
                    "capabilities".to_string(),
                    json!(["projects", "plans", "graph_data", "import", "export"]),
                );
                meta
            },
        }
    }

    fn server_capabilities(&self) -> axum_mcp::protocol::ServerCapabilities {
        axum_mcp::protocol::ServerCapabilities {
            experimental: HashMap::new(),
            logging: None,
            prompts: Some(axum_mcp::protocol::messages::PromptsCapability {
                list_changed: false,
            }),
            resources: Some(axum_mcp::protocol::messages::ResourcesCapability {
                subscribe: true,
                list_changed: false,
            }),
            tools: Some(axum_mcp::protocol::ToolsCapability { list_changed: true }),
            batch: None,
        }
    }
}

/// Tools that MCP agents should not have access to
const MCP_AGENT_BLACKLIST: &[&str] = &["create_project", "delete_project", "list_projects"];

/// Custom tool registry for Layercake tools
#[derive(Clone)]
pub struct LayercakeToolRegistry {
    pub app: Arc<AppContext>,
}

impl LayercakeToolRegistry {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    /// Inject project_id into arguments for MCP agents to enforce scope
    fn inject_project_scope(
        arguments: &mut Option<serde_json::Value>,
        scoped_project_id: i32,
    ) -> McpResult<()> {
        if let Some(args) = arguments {
            if let Some(obj) = args.as_object_mut() {
                // If project_id is already present, verify it matches the scoped project
                if let Some(existing_project_id) = obj.get("project_id") {
                    if let Some(id) = existing_project_id.as_i64() {
                        if id as i32 != scoped_project_id {
                            return Err(McpError::Authorization {
                                message: format!(
                                    "MCP agent scoped to project {} cannot access project {}",
                                    scoped_project_id, id
                                ),
                            });
                        }
                    }
                } else {
                    // Inject the scoped project_id
                    obj.insert(
                        "project_id".to_string(),
                        serde_json::Value::Number(scoped_project_id.into()),
                    );
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ToolRegistry for LayercakeToolRegistry {
    async fn list_tools(&self, context: &SecurityContext) -> McpResult<Vec<Tool>> {
        let mut tools = Vec::new();

        // Project management tools
        tools.extend(super::tools::projects::get_project_tools());

        // Plan management tools
        tools.extend(super::tools::plans::get_plan_tools());

        // Data source tools
        tools.extend(super::tools::data_sets::get_data_set_tools());

        // Plan DAG tools
        tools.extend(super::tools::plan_dag::get_plan_dag_tools());

        // Graph data tools
        tools.extend(super::tools::graph_data::get_graph_data_tools());

        // Graph editing tools
        tools.extend(super::tools::graph_edit::get_graph_edit_tools());

        // Analysis tools
        tools.extend(super::tools::analysis::get_analysis_tools());

        if scoped_project_from_context(context).is_some() {
            tools.retain(|tool| !MCP_AGENT_BLACKLIST.contains(&tool.name.as_str()));
        }

        Ok(tools)
    }

    async fn get_tool(&self, name: &str, _context: &SecurityContext) -> McpResult<Option<McpTool>> {
        // Create tool definitions for each available tool
        let tool = match name {
            // Project tools
            "list_projects" => Some(McpTool::new(
                "list_projects",
                "List all available graph projects",
                json!({"type": "object", "properties": {}, "additionalProperties": false}),
                "projects"
            ).public()),

            "create_project" => Some(McpTool::new(
                "create_project", 
                "Create a new graph project",
                json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Name of the project"},
                        "description": {"type": "string", "description": "Optional description of the project"}
                    },
                    "required": ["name"],
                    "additionalProperties": false
                }),
                "projects"
            ).public()),

            "update_project" => Some(McpTool::new(
                "update_project",
                "Update an existing graph project",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project to update"},
                        "name": {"type": "string", "description": "Updated project name"},
                        "description": {"type": "string", "description": "Updated project description"}
                    },
                    "required": ["project_id"],
                    "additionalProperties": false
                }),
                "projects"
            ).public()),

            "get_project" => Some(McpTool::new(
                "get_project",
                "Get details of a specific project", 
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project to retrieve"}
                    },
                    "required": ["project_id"],
                    "additionalProperties": false
                }),
                "projects"
            ).public()),

            "delete_project" => Some(McpTool::new(
                "delete_project",
                "Delete a project and all its data",
                json!({
                    "type": "object", 
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project to delete"}
                    },
                    "required": ["project_id"],
                    "additionalProperties": false
                }),
                "projects"
            ).public()),

            // Plan tools
            "create_plan" => Some(McpTool::new(
                "create_plan",
                "Create a new transformation plan",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project"},
                        "name": {"type": "string", "description": "Name of the plan"},
                        "yaml_content": {"type": "string", "description": "YAML configuration for the plan"},
                        "dependencies": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "List of plan IDs this plan depends on"
                        }
                    },
                    "required": ["project_id", "name", "yaml_content"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "execute_plan" => Some(McpTool::new(
                "execute_plan",
                "Execute a transformation plan",
                json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {"type": "integer", "description": "ID of the plan to execute"}
                    },
                    "required": ["plan_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "get_plan_status" => Some(McpTool::new(
                "get_plan_status",
                "Get the execution status of a plan",
                json!({
                    "type": "object",
                    "properties": {
                        "plan_id": {"type": "integer", "description": "ID of the plan to check"}
                    },
                    "required": ["plan_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),
            "get_plan_dag" => Some(McpTool::new(
                "get_plan_dag",
                "Retrieve the plan DAG definition for a project",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project to inspect"}
                    },
                    "required": ["project_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "add_plan_dag_node" => Some(McpTool::new(
                "add_plan_dag_node",
                "Create a new Plan DAG node",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "Project identifier"},
                        "node_type": {"type": "string", "description": "Node type (e.g. DataSetNode)"},
                        "position": {
                            "type": "object",
                            "properties": {"x": {"type": "number"}, "y": {"type": "number"}},
                            "required": ["x", "y"],
                            "additionalProperties": false
                        },
                        "metadata": {"type": "object"},
                        "config": {"type": "object"}
                    },
                    "required": ["project_id", "node_type", "position"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "update_plan_dag_node" => Some(McpTool::new(
                "update_plan_dag_node",
                "Update an existing Plan DAG node",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "Project identifier"},
                        "node_id": {"type": "string", "description": "Node identifier"},
                        "position": {
                            "type": "object",
                            "properties": {"x": {"type": "number"}, "y": {"type": "number"}}
                        },
                        "metadata": {"type": "object"},
                        "config": {"type": "object"}
                    },
                    "required": ["project_id", "node_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "delete_plan_dag_node" => Some(McpTool::new(
                "delete_plan_dag_node",
                "Delete a Plan DAG node and its edges",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "Project identifier"},
                        "node_id": {"type": "string", "description": "Node identifier"}
                    },
                    "required": ["project_id", "node_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "move_plan_dag_node" => Some(McpTool::new(
                "move_plan_dag_node",
                "Move a Plan DAG node to a new position",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "Project identifier"},
                        "node_id": {"type": "string"},
                        "position": {
                            "type": "object",
                            "properties": {"x": {"type": "number"}, "y": {"type": "number"}},
                            "required": ["x", "y"],
                            "additionalProperties": false
                        }
                    },
                    "required": ["project_id", "node_id", "position"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "batch_move_plan_dag_nodes" => Some(McpTool::new(
                "batch_move_plan_dag_nodes",
                "Move multiple Plan DAG nodes",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer"},
                        "nodes": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "node_id": {"type": "string"},
                                    "position": {
                                        "type": "object",
                                        "properties": {"x": {"type": "number"}, "y": {"type": "number"}},
                                        "required": ["x", "y"],
                                        "additionalProperties": false
                                    },
                                    "source_position": {"type": "string"},
                                    "target_position": {"type": "string"}
                                },
                                "required": ["node_id", "position"],
                                "additionalProperties": false
                            }
                        }
                    },
                    "required": ["project_id", "nodes"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "add_plan_dag_edge" => Some(McpTool::new(
                "add_plan_dag_edge",
                "Create a Plan DAG edge between nodes",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer"},
                        "source": {"type": "string"},
                        "target": {"type": "string"},
                        "metadata": {"type": "object"}
                    },
                    "required": ["project_id", "source", "target"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "update_plan_dag_edge" => Some(McpTool::new(
                "update_plan_dag_edge",
                "Update Plan DAG edge metadata",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer"},
                        "edge_id": {"type": "string"},
                        "metadata": {"type": "object"}
                    },
                    "required": ["project_id", "edge_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "delete_plan_dag_edge" => Some(McpTool::new(
                "delete_plan_dag_edge",
                "Delete a Plan DAG edge",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer"},
                        "edge_id": {"type": "string"}
                    },
                    "required": ["project_id", "edge_id"],
                    "additionalProperties": false
                }),
                "plans"
            ).public()),

            "update_graph_node" => Some(McpTool::new(
                "update_graph_node",
                "Update a graph node's metadata",
                json!({
                    "type": "object",
                    "properties": {
                        "graph_id": {"type": "integer", "description": "Graph identifier"},
                        "node_id": {"type": "string", "description": "Graph node identifier"},
                        "label": {"type": "string"},
                        "layer": {"type": "string"},
                        "attrs": {"type": "object"},
                        "belongs_to": {"type": "string"}
                    },
                    "required": ["graph_id", "node_id"],
                    "additionalProperties": false
                }),
                "graphs"
            ).public()),

            "bulk_update_graph_data" => Some(McpTool::new(
                "bulk_update_graph_data",
                "Bulk update graph nodes and layers",
                json!({
                    "type": "object",
                    "properties": {
                        "graph_id": {"type": "integer", "description": "Graph identifier"},
                        "nodes": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "node_id": {"type": "string"},
                                    "label": {"type": "string"},
                                    "layer": {"type": "string"},
                                    "attrs": {"type": "object"}
                                },
                                "required": ["node_id"],
                                "additionalProperties": false
                            }
                        },
                        "layers": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {"type": "integer"},
                                    "name": {"type": "string"},
                                    "properties": {"type": "object"}
                                },
                                "required": ["id"],
                                "additionalProperties": false
                            }
                        }
                    },
                    "required": ["graph_id"],
                    "additionalProperties": false
                }),
                "graphs"
            ).public()),

            "replay_graph_edits" => Some(McpTool::new(
                "replay_graph_edits",
                "Replay pending graph edits for a graph",
                json!({
                    "type": "object",
                    "properties": {
                        "graph_id": {"type": "integer", "description": "Graph identifier"}
                    },
                    "required": ["graph_id"],
                    "additionalProperties": false
                }),
                "graphs"
            ).public()),

            // Graph data tools
            "import_csv" => Some(McpTool::new(
                "import_csv",
                "Import graph data from CSV content",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project to import data into"},
                        "nodes_csv": {"type": "string", "description": "CSV content for nodes (optional)"},
                        "edges_csv": {"type": "string", "description": "CSV content for edges (optional)"},
                        "layers_csv": {"type": "string", "description": "CSV content for layers (optional)"}
                    },
                    "required": ["project_id"],
                    "additionalProperties": false
                }),
                "graph_data"
            ).public()),

            "export_graph" => Some(McpTool::new(
                "export_graph",
                "Export graph data in various formats", 
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project to export"},
                        "format": {
                            "type": "string",
                            "enum": ["json", "csv", "dot", "gml", "plantuml", "mermaid"],
                            "description": "Export format"
                        },
                        "preview_limit": {"type": "integer", "description": "Optional maximum number of rows to include in CSV previews"}
                    },
                    "required": ["project_id", "format"],
                    "additionalProperties": false
                }),
                "graph_data"
            ).public()),

            "get_graph_data" => Some(McpTool::new(
                "get_graph_data",
                "Retrieve graph structure (nodes, edges, layers)",
                json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "integer", "description": "ID of the project"},
                        "include_nodes": {"type": "boolean", "description": "Include nodes in response (default: true)"},
                        "include_edges": {"type": "boolean", "description": "Include edges in response (default: true)"},
                        "include_layers": {"type": "boolean", "description": "Include layers in response (default: true)"}
                    },
                    "required": ["project_id"],
                    "additionalProperties": false
                }),
                "graph_data"
            ).public()),

            // Analysis tools
            "analyze_connectivity" => Some(McpTool::new(
                "analyze_connectivity",
                "Analyze graph connectivity and structure",
                json!({
                    "type": "object",
                    "properties": {
                        "graph_id": {"type": "integer", "description": "Graph identifier to analyze"}
                    },
                    "required": ["graph_id"],
                    "additionalProperties": false
                }),
                "analysis"
            ).public()),

            "find_paths" => Some(McpTool::new(
                "find_paths",
                "Find paths between nodes in the graph",
                json!({
                    "type": "object",
                    "properties": {
                        "graph_id": {"type": "integer", "description": "Graph identifier"},
                        "source_node": {"type": "string", "description": "ID of the source node"},
                        "target_node": {"type": "string", "description": "ID of the target node"},
                        "max_paths": {"type": "integer", "description": "Maximum number of paths to find (default: 10)"}
                    },
                    "required": ["graph_id", "source_node", "target_node"],
                    "additionalProperties": false
                }),
                "analysis"
            ).public()),

            _ => None,
        };

        Ok(tool)
    }

    async fn execute_tool(
        &self,
        name: &str,
        mut context: ToolExecutionContext,
    ) -> McpResult<ToolsCallResult> {
        if let Some(scoped_project_id) = scoped_project_from_context(&context.security) {
            if MCP_AGENT_BLACKLIST.contains(&name) {
                return Err(McpError::Authorization {
                    message: format!("Project-scoped chat cannot access tool: {}", name),
                });
            }

            Self::inject_project_scope(&mut context.arguments, scoped_project_id)?;
        }

        match name {
            "list_projects" => super::tools::projects::list_projects(&self.app).await,
            "create_project" => {
                super::tools::projects::create_project(context.arguments, &self.app).await
            }
            "update_project" => {
                super::tools::projects::update_project(context.arguments, &self.app).await
            }
            "get_project" => {
                super::tools::projects::get_project(context.arguments, &self.app).await
            }
            "delete_project" => {
                super::tools::projects::delete_project(context.arguments, &self.app).await
            }

            // Plan tools

            // Data source tools
            "list_data_sets" => {
                super::tools::data_sets::list_data_sets(context.arguments, &self.app).await
            }
            "get_data_set" => {
                super::tools::data_sets::get_data_set(context.arguments, &self.app).await
            }
            "create_data_set_from_file" => {
                super::tools::data_sets::create_data_set_from_file(context.arguments, &self.app)
                    .await
            }
            "create_empty_data_set" => {
                super::tools::data_sets::create_empty_data_set(context.arguments, &self.app).await
            }
            "update_data_set" => {
                super::tools::data_sets::update_data_set(context.arguments, &self.app).await
            }
            "delete_data_set" => {
                super::tools::data_sets::delete_data_set(context.arguments, &self.app).await
            }
            "reprocess_data_set" => {
                super::tools::data_sets::reprocess_data_set(context.arguments, &self.app).await
            }
            "export_data_sets" => {
                super::tools::data_sets::export_data_sets(context.arguments, &self.app).await
            }
            "import_data_sets" => {
                super::tools::data_sets::import_data_sets(context.arguments, &self.app).await
            }

            "create_plan" => super::tools::plans::create_plan(context.arguments, &self.app).await,
            "update_plan" => super::tools::plans::update_plan(context.arguments, &self.app).await,
            "get_plan" => super::tools::plans::get_plan(context.arguments, &self.app).await,
            "delete_plan" => super::tools::plans::delete_plan(context.arguments, &self.app).await,
            "execute_plan" => super::tools::plans::execute_plan(context.arguments, &self.app).await,
            "get_plan_status" => {
                super::tools::plans::get_plan_status(context.arguments, &self.app).await
            }
            "get_plan_dag" => super::tools::plans::get_plan_dag(context.arguments, &self.app).await,
            "add_plan_dag_node" => {
                super::tools::plan_dag::add_plan_dag_node(context.arguments, &self.app).await
            }
            "update_plan_dag_node" => {
                super::tools::plan_dag::update_plan_dag_node(context.arguments, &self.app).await
            }
            "delete_plan_dag_node" => {
                super::tools::plan_dag::delete_plan_dag_node(context.arguments, &self.app).await
            }
            "move_plan_dag_node" => {
                super::tools::plan_dag::move_plan_dag_node(context.arguments, &self.app).await
            }
            "batch_move_plan_dag_nodes" => {
                super::tools::plan_dag::batch_move_plan_dag_nodes(context.arguments, &self.app)
                    .await
            }
            "add_plan_dag_edge" => {
                super::tools::plan_dag::add_plan_dag_edge(context.arguments, &self.app).await
            }
            "update_plan_dag_edge" => {
                super::tools::plan_dag::update_plan_dag_edge(context.arguments, &self.app).await
            }
            "delete_plan_dag_edge" => {
                super::tools::plan_dag::delete_plan_dag_edge(context.arguments, &self.app).await
            }

            // Graph editing tools
            "update_graph_node" => {
                super::tools::graph_edit::update_graph_node(context.arguments, &self.app).await
            }
            "bulk_update_graph_data" => {
                super::tools::graph_edit::bulk_update_graph_data(context.arguments, &self.app).await
            }
            "replay_graph_edits" => {
                super::tools::graph_edit::replay_graph_edits(context.arguments, &self.app).await
            }

            // Graph data tools
            "import_csv" => {
                super::tools::graph_data::import_csv(context.arguments, self.app.db()).await
            }
            "export_graph" => {
                super::tools::graph_data::export_graph(context.arguments, &self.app).await
            }
            "get_graph_data" => {
                super::tools::graph_data::get_graph_data(context.arguments, self.app.db()).await
            }

            // Analysis tools
            "analyze_connectivity" => {
                super::tools::analysis::analyze_connectivity(context.arguments, &self.app).await
            }
            "find_paths" => super::tools::analysis::find_paths(context.arguments, &self.app).await,

            _ => Err(McpError::ToolNotFound {
                name: name.to_string(),
            }),
        }
    }

    async fn can_access_tool(&self, name: &str, _context: &SecurityContext) -> bool {
        // Allow access to all our tools for now
        matches!(
            name,
            "list_projects"
                | "create_project"
                | "update_project"
                | "get_project"
                | "delete_project"
                | "create_plan"
                | "execute_plan"
                | "get_plan_status"
                | "list_data_sets"
                | "get_data_set"
                | "create_data_set_from_file"
                | "create_empty_data_set"
                | "update_data_set"
                | "delete_data_set"
                | "reprocess_data_set"
                | "export_data_sets"
                | "import_data_sets"
                | "get_plan_dag"
                | "add_plan_dag_node"
                | "update_plan_dag_node"
                | "delete_plan_dag_node"
                | "move_plan_dag_node"
                | "batch_move_plan_dag_nodes"
                | "add_plan_dag_edge"
                | "update_plan_dag_edge"
                | "delete_plan_dag_edge"
                | "update_graph_node"
                | "bulk_update_graph_data"
                | "replay_graph_edits"
                | "import_csv"
                | "export_graph"
                | "get_graph_data"
                | "analyze_connectivity"
                | "find_paths"
        )
    }
}
