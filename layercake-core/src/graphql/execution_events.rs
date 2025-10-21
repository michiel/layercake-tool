/// Helper module for publishing execution status change events
///
/// This module provides convenience functions to publish execution status
/// updates via GraphQL subscriptions when datasources or graphs change state.
use crate::database::entities::{datasources, graphs};
use crate::graphql::subscriptions::publish_execution_status_event;
use crate::graphql::types::*;
use sea_orm::DatabaseConnection;

/// Publish datasource execution status change event
///
/// Finds the Plan DAG node associated with this datasource and broadcasts
/// the execution status change to all subscribed clients.
///
/// This is fire-and-forget - failures to publish are logged but don't
/// affect datasource processing.
pub async fn publish_datasource_status(
    _db: &DatabaseConnection,
    project_id: i32,
    node_id: &str,
    datasource: &datasources::Model,
) {
    let event = NodeExecutionStatusEvent {
        project_id,
        node_id: node_id.to_string(),
        node_type: PlanDagNodeType::DataSource,
        datasource_execution: Some(DataSourceExecutionMetadata {
            data_source_id: datasource.id,
            filename: datasource.file_path.clone(),
            status: if datasource.execution_state == "completed" {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            processed_at: datasource.import_date.map(|dt| dt.to_rfc3339()),
            execution_state: datasource.execution_state.clone(),
            error_message: datasource.error_message.clone(),
        }),
        graph_execution: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Fire and forget - spawn async task so it doesn't block processing
    tokio::spawn(async move {
        if let Err(e) = publish_execution_status_event(event).await {
            tracing::debug!("Failed to publish datasource status: {}", e);
        }
    });
}

/// Publish graph execution status change event
///
/// Finds the Plan DAG node associated with this graph and broadcasts
/// the execution status change to all subscribed clients.
///
/// This is fire-and-forget - failures to publish are logged but don't
/// affect graph computation.
pub async fn publish_graph_status(
    _db: &DatabaseConnection,
    project_id: i32,
    node_id: &str,
    graph: &graphs::Model,
) {
    let event = NodeExecutionStatusEvent {
        project_id,
        node_id: node_id.to_string(),
        node_type: PlanDagNodeType::Graph,
        datasource_execution: None,
        graph_execution: Some(GraphExecutionMetadata {
            graph_id: graph.id,
            node_count: graph.node_count,
            edge_count: graph.edge_count,
            execution_state: graph.execution_state.clone(),
            computed_date: graph.computed_date.map(|dt| dt.to_rfc3339()),
            error_message: graph.error_message.clone(),
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Fire and forget - spawn async task so it doesn't block computation
    tokio::spawn(async move {
        if let Err(e) = publish_execution_status_event(event).await {
            tracing::debug!("Failed to publish graph status: {}", e);
        }
    });
}
