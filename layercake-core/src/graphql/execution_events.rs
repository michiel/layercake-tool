/// Helper module for publishing execution status change events
///
/// This module provides convenience functions to publish execution status
/// updates via GraphQL subscriptions when datasets or graphs change state.
use crate::database::entities::{datasets, graphs};
use crate::graphql::subscriptions::publish_execution_status_event;
use crate::graphql::types::*;
use sea_orm::DatabaseConnection;

/// Publish dataset execution status change event
///
/// Finds the Plan DAG node associated with this dataset and broadcasts
/// the execution status change to all subscribed clients.
///
/// This is fire-and-forget - failures to publish are logged but don't
/// affect dataset processing.
pub async fn publish_dataset_status(
    _db: &DatabaseConnection,
    project_id: i32,
    node_id: &str,
    dataset: &datasets::Model,
) {
    let event = NodeExecutionStatusEvent {
        project_id,
        node_id: node_id.to_string(),
        node_type: PlanDagNodeType::DataSet,
        dataset_execution: Some(DataSetExecutionMetadata {
            data_set_id: dataset.id,
            filename: dataset.file_path.clone(),
            status: if dataset.execution_state == "completed" {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            processed_at: dataset.import_date.map(|dt| dt.to_rfc3339()),
            execution_state: dataset.execution_state.clone(),
            error_message: dataset.error_message.clone(),
        }),
        graph_execution: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Fire and forget - spawn async task so it doesn't block processing
    tokio::spawn(async move {
        if let Err(e) = publish_execution_status_event(event).await {
            tracing::debug!("Failed to publish dataset status: {}", e);
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
        dataset_execution: None,
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
