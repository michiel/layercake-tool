//! Layercake resource registry for MCP backed by the shared application context.

use crate::app_context::AppContext;
use crate::database::entities::graph_data;
use axum_mcp::prelude::*;
use axum_mcp::protocol::ToolContent;
use axum_mcp::server::resource::{
    Resource, ResourceContent, ResourceRegistry, ResourceSubscription, ResourceTemplate,
    UriSchemeConfig,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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
            "dataset".to_string(),
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
    async fn get_data_set_resource(&self, data_set_id: i32) -> McpResult<Resource> {
        let data_set = self
            .app
            .get_data_set(data_set_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to load data source: {}", e),
            })?
            .ok_or_else(|| McpError::ResourceNotFound {
                uri: format!("layercake://datasets/{}", data_set_id),
            })?;

        let content = serde_json::to_string_pretty(&data_set).map_err(|e| McpError::Internal {
            message: format!("Failed to serialize data source: {}", e),
        })?;

        Ok(Resource {
            uri: format!("layercake://datasets/{}", data_set_id),
            name: format!("Data Source {}", data_set_id),
            description: Some("Layercake data source metadata".to_string()),
            mime_type: Some("application/json".to_string()),
            content: ResourceContent::Text { text: content },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("data_set_id".to_string(), json!(data_set_id));
                meta.insert("project_id".to_string(), json!(data_set.project_id));
                meta.insert("resource_type".to_string(), json!("dataset"));
                meta
            },
        })
    }

    async fn get_project_data_sets_resource(&self, project_id: i32) -> McpResult<Resource> {
        let data_sets =
            self.app
                .list_data_sets(project_id)
                .await
                .map_err(|e| McpError::Internal {
                    message: format!("Failed to list data sources: {}", e),
                })?;

        let content = serde_json::to_string_pretty(&json!({
            "projectId": project_id,
            "count": data_sets.len(),
            "dataSets": data_sets,
        }))
        .map_err(|e| McpError::Internal {
            message: format!("Failed to serialize data source list: {}", e),
        })?;

        Ok(Resource {
            uri: format!("layercake://projects/{}/datasets", project_id),
            name: format!("Project {} Data Sources", project_id),
            description: Some("List of data sources for the project".to_string()),
            mime_type: Some("application/json".to_string()),
            content: ResourceContent::Text { text: content },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("project_id".to_string(), json!(project_id));
                meta.insert("resource_type".to_string(), json!("dataset"));
                meta
            },
        })
    }

    async fn get_plan_resource(&self, plan_id: i32) -> McpResult<Resource> {
        let plan = self
            .app
            .get_plan(plan_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to load plan: {}", e),
            })?
            .ok_or_else(|| McpError::ResourceNotFound {
                uri: format!("layercake://plans/{}", plan_id),
            })?;

        let content = serde_json::to_string_pretty(&plan).map_err(|e| McpError::Internal {
            message: format!("Failed to serialize plan: {}", e),
        })?;

        Ok(Resource {
            uri: format!("layercake://plans/{}", plan_id),
            name: format!("Plan {}", plan.name),
            description: Some("Layercake plan summary and configuration".to_string()),
            mime_type: Some("application/json".to_string()),
            content: ResourceContent::Text { text: content },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("plan_id".to_string(), json!(plan.id));
                meta.insert("project_id".to_string(), json!(plan.project_id));
                meta.insert("resource_type".to_string(), json!("plan"));
                meta
            },
        })
    }

    async fn get_project_plan_resource(&self, project_id: i32) -> McpResult<Resource> {
        let plan = self
            .app
            .get_plan_for_project(project_id)
            .await
            .map_err(|e| McpError::Internal {
                message: format!("Failed to load plan for project {}: {}", project_id, e),
            })?
            .ok_or_else(|| McpError::ResourceNotFound {
                uri: format!("layercake://projects/{}/plan", project_id),
            })?;

        let content = serde_json::to_string_pretty(&plan).map_err(|e| McpError::Internal {
            message: format!("Failed to serialize plan: {}", e),
        })?;

        Ok(Resource {
            uri: format!("layercake://projects/{}/plan", project_id),
            name: format!("Project {} Plan", project_id),
            description: Some("Layercake plan summary associated with the project".to_string()),
            mime_type: Some("application/json".to_string()),
            content: ResourceContent::Text { text: content },
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("plan_id".to_string(), json!(plan.id));
                meta.insert("project_id".to_string(), json!(plan.project_id));
                meta.insert("resource_type".to_string(), json!("plan"));
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
                let graph = graph_data::Entity::find()
                    .filter(graph_data::Column::ProjectId.eq(project_id))
                    .order_by_asc(graph_data::Column::Id)
                    .one(self.app.db())
                    .await
                    .map_err(|e| McpError::Internal {
                        message: format!(
                            "Failed to load graph_data for project {}: {}",
                            project_id, e
                        ),
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
                name: "Plan Summary".to_string(),
                description: Some("Layercake plan summary including YAML content".to_string()),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("plans"));
                    meta
                },
            },
            ResourceTemplate {
                uri_template: "layercake://projects/{project_id}/plan".to_string(),
                name: "Project Plan Summary".to_string(),
                description: Some("Plan summary associated with a specific project".to_string()),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("plans"));
                    meta.insert("relationship".to_string(), json!("project"));
                    meta
                },
            },
            ResourceTemplate {
                uri_template: "layercake://datasets/{data_set_id}".to_string(),
                name: "Data Source Summary".to_string(),
                description: Some("Layercake data source metadata".to_string()),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("datasets"));
                    meta
                },
            },
            ResourceTemplate {
                uri_template: "layercake://projects/{project_id}/datasets".to_string(),
                name: "Project Data Sources".to_string(),
                description: Some("List of data sources for a specific project".to_string()),
                mime_type: Some("application/json".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("category".to_string(), json!("datasets"));
                    meta.insert("relationship".to_string(), json!("project"));
                    meta
                },
            },
        ])
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
            ["projects", id, "plan"] => {
                let project_id = id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid project ID".to_string(),
                })?;
                self.get_project_plan_resource(project_id).await
            }
            ["plans", id] => {
                let plan_id = id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid plan ID".to_string(),
                })?;
                self.get_plan_resource(plan_id).await
            }
            ["datasets", id] => {
                let data_set_id = id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid data source ID".to_string(),
                })?;
                self.get_data_set_resource(data_set_id).await
            }
            ["projects", id, "datasets"] => {
                let project_id = id.parse::<i32>().map_err(|_| McpError::Validation {
                    message: "Invalid project ID".to_string(),
                })?;
                self.get_project_data_sets_resource(project_id).await
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

    async fn resource_exists(&self, uri: &str, context: &SecurityContext) -> McpResult<bool> {
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
