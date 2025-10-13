//! Layercake resource registry for MCP
//! Implements layercake:// URI scheme for accessing project data, plans, and graph exports

use crate::services::GraphService;
use axum_mcp::prelude::*;
use axum_mcp::protocol::ToolContent;
use axum_mcp::server::resource::{
    Resource, ResourceContent, ResourceRegistry, ResourceSubscription, ResourceTemplate,
    UriSchemeConfig,
};
use sea_orm::DatabaseConnection;
use serde_json::json;
use std::collections::HashMap;

/// Layercake resource registry implementing layercake:// URI scheme
#[derive(Clone)]
pub struct LayercakeResourceRegistry {
    scheme_config: UriSchemeConfig,
    db: DatabaseConnection,
}

impl LayercakeResourceRegistry {
    pub fn new(db: DatabaseConnection) -> Self {
        let scheme_config = UriSchemeConfig::new(
            "layercake",
            "Layercake graph visualization and transformation resources",
        )
        .with_types(vec![
            "project".to_string(),
            "plan".to_string(),
            "graph".to_string(),
            "export".to_string(),
            "analysis".to_string(),
        ]);

        Self { scheme_config, db }
    }

    /// Get project resource
    async fn get_project_resource(&self, project_id: i32) -> McpResult<Resource> {
        let _graph_service = GraphService::new(self.db.clone());

        // Get project details (assuming we have a method for this)
        // For now, create a basic project resource structure
        let project_data = json!({
            "project_id": project_id,
            "name": format!("Project {}", project_id),
            "description": "Layercake graph project",
            "created_at": "2025-07-03T00:00:00Z",
            "status": "active"
        });

        Ok(Resource {
            uri: format!("layercake://projects/{}", project_id),
            name: format!("Project {}", project_id),
            description: Some("Layercake project configuration and metadata".to_string()),
            mime_type: Some("application/json".to_string()),
            content: ResourceContent::Text {
                text: serde_json::to_string_pretty(&project_data).unwrap(),
            },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("project_id".to_string(), json!(project_id));
                meta.insert("resource_type".to_string(), json!("project"));
                meta
            },
        })
    }

    /// Get plan resource  
    async fn get_plan_resource(&self, plan_id: i32) -> McpResult<Resource> {
        // For now, create a basic plan resource structure
        let plan_data = json!({
            "plan_id": plan_id,
            "name": format!("Plan {}", plan_id),
            "yaml_content": "# Transformation plan YAML would go here",
            "status": "draft",
            "created_at": "2025-07-03T00:00:00Z"
        });

        Ok(Resource {
            uri: format!("layercake://plans/{}", plan_id),
            name: format!("Plan {}", plan_id),
            description: Some("Transformation plan configuration".to_string()),
            mime_type: Some("application/yaml".to_string()),
            content: ResourceContent::Text {
                text: serde_json::to_string_pretty(&plan_data).unwrap(),
            },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("plan_id".to_string(), json!(plan_id));
                meta.insert("resource_type".to_string(), json!("plan"));
                meta
            },
        })
    }

    /// Get graph export resource
    async fn get_graph_resource(&self, project_id: i32, format: &str) -> McpResult<Resource> {
        // TODO: Fix this function after data model refactoring
        Err(McpError::Internal {
            message: "get_graph_resource is not implemented yet".to_string(),
        })
    }

    /// Get analysis resource
    async fn get_analysis_resource(
        &self,
        project_id: i32,
        analysis_type: &str,
    ) -> McpResult<Resource> {
        match analysis_type {
            "connectivity" => {
                // Use existing analysis tool functionality
                let arguments = Some(json!({ "project_id": project_id }));
                let result =
                    crate::mcp::tools::analysis::analyze_connectivity(arguments, &self.db).await?;

                // Extract the analysis content from the tool result
                let content = if let Some(ToolContent::Text { text }) = result.content.first() {
                    text.clone()
                } else {
                    json!({"error": "No analysis data available"}).to_string()
                };

                Ok(Resource {
                    uri: format!("layercake://analysis/{}/connectivity", project_id),
                    name: format!("Connectivity Analysis - Project {}", project_id),
                    description: Some("Graph connectivity and structural analysis".to_string()),
                    mime_type: Some("application/json".to_string()),
                    content: ResourceContent::Text { text: content },
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("project_id".to_string(), json!(project_id));
                        meta.insert("analysis_type".to_string(), json!("connectivity"));
                        meta.insert("resource_type".to_string(), json!("analysis"));
                        meta
                    },
                })
            }
            _ => Err(McpError::Validation {
                message: format!("Unsupported analysis type: {}", analysis_type),
            }),
        }
    }
}

#[async_trait]
impl ResourceRegistry for LayercakeResourceRegistry {
    fn uri_scheme(&self) -> &UriSchemeConfig {
        &self.scheme_config
    }

    async fn list_resource_templates(
        &self,
        _context: &SecurityContext,
    ) -> McpResult<Vec<ResourceTemplate>> {
        Ok(vec![
            ResourceTemplate {
                uri_template: "layercake://projects/{project_id}".to_string(),
                name: "Project Configuration".to_string(),
                description: Some("Layercake project configuration and metadata".to_string()),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("projects"));
                    meta
                },
            },
            ResourceTemplate {
                uri_template: "layercake://plans/{plan_id}".to_string(),
                name: "Transformation Plan".to_string(),
                description: Some("YAML transformation plan configuration".to_string()),
                mime_type: Some("application/yaml".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("plans"));
                    meta
                },
            },
            ResourceTemplate {
                uri_template: "layercake://graphs/{project_id}/{format}".to_string(),
                name: "Graph Export".to_string(),
                description: Some(
                    "Graph data in various export formats (json, dot, mermaid)".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("exports"));
                    meta.insert(
                        "supported_formats".to_string(),
                        json!(["json", "dot", "mermaid"]),
                    );
                    meta
                },
            },
            ResourceTemplate {
                uri_template: "layercake://analysis/{project_id}/{analysis_type}".to_string(),
                name: "Graph Analysis".to_string(),
                description: Some("Graph analysis results (connectivity, paths, etc.)".to_string()),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("analysis"));
                    meta.insert("supported_types".to_string(), json!(["connectivity"]));
                    meta
                },
            },
        ])
    }

    async fn get_resource(&self, uri: &str, _context: &SecurityContext) -> McpResult<Resource> {
        if !self.can_handle_uri(uri) {
            return Err(McpError::ResourceNotFound {
                uri: uri.to_string(),
            });
        }

        // Parse URI manually since we don't have the framework's URI parser
        let uri_without_scheme =
            uri.strip_prefix("layercake://")
                .ok_or_else(|| McpError::Validation {
                    message: "Invalid layercake URI".to_string(),
                })?;

        let segments: Vec<&str> = uri_without_scheme.split('/').collect();

        match segments.as_slice() {
            ["projects", project_id] => {
                let id = project_id
                    .parse::<i32>()
                    .map_err(|_| McpError::Validation {
                        message: "Invalid project ID".to_string(),
                    })?;
                self.get_project_resource(id).await
            }
            ["plans", plan_id] => {
                let id = plan_id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid plan ID".to_string(),
                })?;
                self.get_plan_resource(id).await
            }
            ["graphs", project_id, format] => {
                let id = project_id
                    .parse::<i32>()
                    .map_err(|_| McpError::Validation {
                        message: "Invalid project ID".to_string(),
                    })?;
                self.get_graph_resource(id, format).await
            }
            ["analysis", project_id, analysis_type] => {
                let id = project_id
                    .parse::<i32>()
                    .map_err(|_| McpError::Validation {
                        message: "Invalid project ID".to_string(),
                    })?;
                self.get_analysis_resource(id, analysis_type).await
            }
            _ => Err(McpError::ResourceNotFound {
                uri: uri.to_string(),
            }),
        }
    }

    async fn resource_exists(&self, uri: &str, context: &SecurityContext) -> McpResult<bool> {
        match self.get_resource(uri, context).await {
            Ok(_) => Ok(true),
            Err(McpError::ResourceNotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn subscribe_to_resource(
        &self,
        uri: &str,
        _context: &SecurityContext,
    ) -> McpResult<ResourceSubscription> {
        // For now, return a basic subscription
        Ok(ResourceSubscription {
            subscription_id: format!(
                "layercake-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            ),
            uri: uri.to_string(),
        })
    }

    async fn unsubscribe_from_resource(
        &self,
        _subscription_id: &str,
        _context: &SecurityContext,
    ) -> McpResult<()> {
        // For now, just return success
        Ok(())
    }
}
