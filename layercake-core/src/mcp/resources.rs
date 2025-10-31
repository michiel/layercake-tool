//! Layercake resource registry for MCP backed by the shared application context.

use crate::app_context::AppContext;
use crate::database::entities::graphs;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use axum_mcp::prelude::*;
use axum_mcp::protocol::ToolContent;
use axum_mcp::server::resource::{
    Resource, ResourceContent, ResourceRegistry, ResourceSubscription, ResourceTemplate,
    UriSchemeConfig,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct LayercakeResourceRegistry {
    scheme_config: UriSchemeConfig,
    app: Arc<AppContext>,
}

impl LayercakeResourceRegistry {
    pub fn new(app: Arc<AppContext>) -> Self {
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

        Self { scheme_config, app }
    }

    async fn get_project_resource(&self, project_id: i32) -> McpResult<Resource> {
        let project = self
            .app
            .get_project(project_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to load project: {}", e),
            })?
            .ok_or_else(|| McpError::ResourceNotFound {
                uri: format!("layercake://projects/{}", project_id),
            })?;

        let content = serde_json::to_string_pretty(&project).map_err(|e| McpError::Internal {
            message: format!("Failed to serialize project: {}", e),
        })?;

        Ok(Resource {
            uri: format!("layercake://projects/{}", project_id),
            name: format!("Project {}", project_id),
            description: Some("Layercake project configuration and metadata".to_string()),
            mime_type: Some("application/json".to_string()),
            content: ResourceContent::Text { text: content },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("project_id".to_string(), json!(project_id));
                meta.insert("resource_type".to_string(), json!("project"));
                meta
            },
        })
    }

    async fn get_analysis_resource(
        &self,
        project_id: i32,
        analysis_type: &str,
    ) -> McpResult<Resource> {
        match analysis_type {
            "connectivity" => {
                let graph = graphs::Entity::find()
                    .filter(graphs::Column::ProjectId.eq(project_id))
                    .order_by_asc(graphs::Column::Id)
                    .one(self.app.db())
                    .await
                    .map_err(|e| McpError::Internal {
                        message: format!("Failed to load graphs for project {}: {}", project_id, e),
                    })?
                    .ok_or_else(|| McpError::ResourceNotFound {
                        uri: format!("layercake://analysis/{}/connectivity", project_id),
                    })?;

                let arguments = Some(json!({ "graph_id": graph.id }));
                let result =
                    crate::mcp::tools::analysis::analyze_connectivity(arguments, &self.app).await?;

                let content = if let Some(ToolContent::Text { text }) = result.content.first() {
                    text.clone()
                } else {
                    json!({"error": "No analysis data available"}).to_string()
                };

                Ok(Resource {
                    uri: format!("layercake://analysis/{}/connectivity", graph.id),
                    name: format!("Connectivity Analysis - Graph {}", graph.id),
                    description: Some("Graph connectivity analysis".to_string()),
                    mime_type: Some("application/json".to_string()),
                    content: ResourceContent::Text { text: content },
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("project_id".to_string(), json!(project_id));
                        meta.insert("graph_id".to_string(), json!(graph.id));
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
        Ok(vec![ResourceTemplate {
            uri_template: "layercake://projects/{project_id}".to_string(),
            name: "Project Configuration".to_string(),
            description: Some("Layercake project configuration and metadata".to_string()),
            mime_type: Some("application/json".to_string()),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), json!("projects"));
                meta
            },
        }])
    }

    async fn get_resource(&self, uri: &str, _context: &SecurityContext) -> McpResult<Resource> {
        if !self.scheme_config.matches_uri(uri) {
            return Err(McpError::ResourceNotFound {
                uri: uri.to_string(),
            });
        }

        let path = uri.trim_start_matches("layercake://");
        let segments: Vec<&str> = path.split('/').collect();

        match segments.as_slice() {
            ["projects", id] => {
                let project_id = id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid project ID".to_string(),
                })?;
                self.get_project_resource(project_id).await
            }
            ["analysis", id, analysis_type] => {
                let project_id = id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid project ID".to_string(),
                })?;
                self.get_analysis_resource(project_id, analysis_type).await
            }
            _ => Err(McpError::ResourceNotFound {
                uri: uri.to_string(),
            }),
        }
    }

    async fn resource_exists(
        &self,
        uri: &str,
        context: &SecurityContext,
    ) -> McpResult<bool> {
        match self.get_resource(uri, context).await {
            Ok(_) => Ok(true),
            Err(McpError::ResourceNotFound { .. }) => Ok(false),
            Err(err) => Err(err),
        }
    }

    async fn subscribe_to_resource(
        &self,
        _uri: &str,
        _context: &SecurityContext,
    ) -> McpResult<ResourceSubscription> {
        Err(McpError::Configuration {
            message: "Resource subscriptions are not supported".to_string(),
        })
    }

    async fn unsubscribe_from_resource(
        &self,
        _subscription_id: &str,
        _context: &SecurityContext,
    ) -> McpResult<()> {
        Err(McpError::Configuration {
            message: "Resource subscriptions are not supported".to_string(),
        })
    }
}
