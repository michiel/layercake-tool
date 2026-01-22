use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::graph_edge::GraphEdge;
use crate::graphql::types::graph_node::GraphNode;
use crate::graphql::types::Project;
use layercake_core::database::entities::{graph_data, graph_data_edges, graph_data_nodes};

#[derive(SimpleObject, Serialize, Deserialize, Clone)]
pub struct GraphDataAnnotationGql {
    pub title: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub body: String,
}

/// Unified type representing both datasets and computed graphs
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct GraphData {
    pub id: i32,
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,

    /// Type discriminator: "dataset" or "computed"
    #[graphql(name = "sourceType")]
    pub source_type: String,

    /// DAG node ID for computed graphs
    #[graphql(name = "dagNodeId")]
    pub dag_node_id: Option<String>,

    /// File metadata (for datasets)
    #[graphql(name = "fileFormat")]
    pub file_format: Option<String>,
    pub origin: Option<String>,
    pub filename: Option<String>,
    #[graphql(name = "fileSize")]
    pub file_size: Option<i64>,

    /// Processing metadata
    pub status: String,
    #[graphql(name = "errorMessage")]
    pub error_message: Option<String>,
    #[graphql(name = "processedAt")]
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[graphql(name = "computedDate")]
    pub computed_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Content hash for change detection
    #[graphql(name = "sourceHash")]
    pub source_hash: Option<String>,

    /// Graph statistics
    #[graphql(name = "nodeCount")]
    pub node_count: i32,
    #[graphql(name = "edgeCount")]
    pub edge_count: i32,

    /// Edit tracking
    #[graphql(name = "lastEditSequence")]
    pub last_edit_sequence: i32,
    #[graphql(name = "hasPendingEdits")]
    pub has_pending_edits: bool,
    #[graphql(name = "lastReplayAt")]
    pub last_replay_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Metadata and annotations
    pub metadata: Option<serde_json::Value>,
    pub annotations: Vec<GraphDataAnnotationGql>,

    /// Timestamps
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[ComplexObject]
impl GraphData {
    /// Get the project this graph belongs to
    async fn project(&self, ctx: &Context<'_>) -> Result<Project> {
        let graphql_ctx = ctx
            .data::<GraphQLContext>()
            .map_err(|_| StructuredError::internal("GraphQL context not found"))?;

        use layercake_core::database::entities::projects;
        use sea_orm::EntityTrait;

        let project = projects::Entity::find_by_id(self.project_id)
            .one(&graphql_ctx.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", self.project_id))?;

        Ok(Project::from(project))
    }

    /// Best-effort lookup of legacy graphs.id for this graph_data (by dag_node_id and project_id)
    #[graphql(name = "legacyGraphId")]
    async fn legacy_graph_id_resolved(&self, _ctx: &Context<'_>) -> Result<Option<i32>> {
        // Legacy graphs table has been removed; keep the field for schema
        // compatibility but always return None.
        Ok(None)
    }

    /// Lazy-load nodes from graph_data_nodes table
    async fn nodes(&self, ctx: &Context<'_>) -> Result<Vec<GraphNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let nodes = graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(self.id))
            .all(&context.db)
            .await?;

        // Convert graph_data_nodes to GraphNode format
        Ok(nodes
            .into_iter()
            .map(|node| GraphNode {
                id: node.external_id,
                graph_id: self.id, // Use parent graph_data ID
                label: node.label,
                layer: node.layer,
                weight: node.weight,
                is_partition: node.is_partition,
                belongs_to: node.belongs_to,
                comment: node.comment,
                attrs: node.attributes.clone(),
                attributes: node.attributes,
                dataset_id: node.source_dataset_id,
                created_at: node.created_at,
            })
            .collect())
    }

    /// Lazy-load edges from graph_data_edges table
    async fn edges(&self, ctx: &Context<'_>) -> Result<Vec<GraphEdge>> {
        let context = ctx.data::<GraphQLContext>()?;
        let edges = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(self.id))
            .all(&context.db)
            .await?;

        // Convert graph_data_edges to GraphEdge format
        Ok(edges
            .into_iter()
            .map(|edge| GraphEdge {
                id: edge.external_id,
                graph_id: self.id, // Use parent graph_data ID
                source: edge.source,
                target: edge.target,
                label: edge.label,
                layer: edge.layer,
                weight: edge.weight,
                comment: edge.comment,
                attrs: edge.attributes.clone(),
                attributes: edge.attributes,
                dataset_id: edge.source_dataset_id,
                created_at: edge.created_at,
            })
            .collect())
    }

    /// Check if this is a dataset (vs computed graph)
    #[graphql(name = "isDataset")]
    async fn is_dataset(&self) -> bool {
        self.source_type == "dataset"
    }

    /// Check if this is a computed graph
    #[graphql(name = "isComputed")]
    async fn is_computed(&self) -> bool {
        self.source_type == "computed"
    }

    /// Check if the graph is ready to use
    #[graphql(name = "isReady")]
    async fn is_ready(&self) -> bool {
        self.status == graph_data::GraphDataStatus::Active.as_str()
    }

    /// Check if there was an error
    #[graphql(name = "hasError")]
    async fn has_error(&self) -> bool {
        self.status == graph_data::GraphDataStatus::Error.as_str()
    }

    /// Formatted file size (for datasets)
    #[graphql(name = "fileSizeFormatted")]
    async fn file_size_formatted(&self) -> Option<String> {
        self.file_size.map(|size| {
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        })
    }
}

impl From<graph_data::Model> for GraphData {
    fn from(model: graph_data::Model) -> Self {
        let annotations: Vec<GraphDataAnnotationGql> = model
            .annotations
            .as_ref()
            .and_then(|val| serde_json::from_value(val.clone()).ok())
            .unwrap_or_default();

        Self {
            id: model.id,
            project_id: model.project_id,
            name: model.name,
            source_type: model.source_type,
            dag_node_id: model.dag_node_id,
            file_format: model.file_format,
            origin: model.origin,
            filename: model.filename,
            file_size: model.file_size,
            status: model.status,
            error_message: model.error_message,
            processed_at: model.processed_at,
            computed_date: model.computed_date,
            source_hash: model.source_hash,
            node_count: model.node_count,
            edge_count: model.edge_count,
            last_edit_sequence: model.last_edit_sequence,
            has_pending_edits: model.has_pending_edits,
            last_replay_at: model.last_replay_at,
            metadata: model.metadata,
            annotations,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Input for creating a new dataset
#[derive(InputObject)]
pub struct CreateGraphDataInput {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    pub name: String,
    #[graphql(name = "fileContent")]
    pub file_content: String, // Base64 encoded
    pub filename: String,
    #[graphql(name = "fileFormat")]
    pub file_format: Option<String>,
}

/// Input for updating graph data
#[derive(InputObject)]
pub struct UpdateGraphDataInput {
    pub name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Result for bulk operations
#[derive(SimpleObject)]
pub struct GraphDataBulkResult {
    pub success: bool,
    #[graphql(name = "affectedCount")]
    pub affected_count: i32,
    pub message: String,
}
