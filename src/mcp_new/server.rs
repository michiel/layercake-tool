//! Layercake MCP server implementation using axum-mcp

use axum_mcp::prelude::*;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

/// Layercake-specific server state implementing axum-mcp traits
#[derive(Clone)]
pub struct LayercakeServerState {
    pub db: DatabaseConnection,
    pub tools: LayercakeToolRegistry,
    pub auth: LayercakeAuth,
}

/// Simple authentication that allows all operations for now
#[derive(Clone)]
pub struct LayercakeAuth;

#[async_trait]
impl McpAuth for LayercakeAuth {
    async fn authenticate(&self, _client_info: &ClientContext) -> McpResult<SecurityContext> {
        // For now, all clients get full access
        // TODO: Implement proper authentication if needed
        Ok(SecurityContext::system())
    }

    async fn authorize(&self, _context: &SecurityContext, _resource: &str, _action: &str) -> bool {
        // Allow all operations for now
        // TODO: Implement proper authorization if needed
        true
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

    fn server_info(&self) -> axum_mcp::protocol::ServerInfo {
        axum_mcp::protocol::ServerInfo {
            name: "Layercake MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("description".to_string(), json!("Graph visualization and transformation MCP server"));
                meta.insert("capabilities".to_string(), json!(["projects", "plans", "graph_data", "import", "export"]));
                meta
            },
        }
    }

    fn server_capabilities(&self) -> axum_mcp::protocol::ServerCapabilities {
        axum_mcp::protocol::ServerCapabilities {
            experimental: HashMap::new(),
            logging: None,
            prompts: None,
            resources: None,
            tools: Some(axum_mcp::protocol::ToolsCapability {
                list_changed: true,
            }),
            batch: None,
        }
    }
}

/// Custom tool registry for Layercake tools
#[derive(Clone)]
pub struct LayercakeToolRegistry {
    pub db: DatabaseConnection,
}

impl LayercakeToolRegistry {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ToolRegistry for LayercakeToolRegistry {
    async fn list_tools(&self, _context: &SecurityContext) -> McpResult<Vec<Tool>> {
        let mut tools = Vec::new();
        
        // Project management tools
        tools.extend(super::tools::projects::get_project_tools());
        
        // Plan management tools  
        tools.extend(super::tools::plans::get_plan_tools());
        
        // Graph data tools
        tools.extend(super::tools::graph_data::get_graph_data_tools());
        
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
                        }
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
            
            _ => None,
        };
        
        Ok(tool)
    }

    async fn execute_tool(&self, name: &str, context: ToolExecutionContext) -> McpResult<ToolsCallResult> {
        match name {
            // Project tools
            "list_projects" => super::tools::projects::list_projects(&self.db).await,
            "create_project" => super::tools::projects::create_project(context.arguments, &self.db).await,
            "get_project" => super::tools::projects::get_project(context.arguments, &self.db).await,
            "delete_project" => super::tools::projects::delete_project(context.arguments, &self.db).await,
            
            // Plan tools
            "create_plan" => super::tools::plans::create_plan(context.arguments, &self.db).await,
            "execute_plan" => super::tools::plans::execute_plan(context.arguments, &self.db).await,
            "get_plan_status" => super::tools::plans::get_plan_status(context.arguments, &self.db).await,
            
            // Graph data tools
            "import_csv" => super::tools::graph_data::import_csv(context.arguments, &self.db).await,
            "export_graph" => super::tools::graph_data::export_graph(context.arguments, &self.db).await,
            "get_graph_data" => super::tools::graph_data::get_graph_data(context.arguments, &self.db).await,
            
            _ => Err(McpError::ToolNotFound {
                name: name.to_string(),
            }),
        }
    }

    async fn can_access_tool(&self, name: &str, _context: &SecurityContext) -> bool {
        // Allow access to all our tools for now
        matches!(name, 
            "list_projects" | "create_project" | "get_project" | "delete_project" |
            "create_plan" | "execute_plan" | "get_plan_status" |
            "import_csv" | "export_graph" | "get_graph_data"
        )
    }
}