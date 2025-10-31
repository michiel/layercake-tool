use async_graphql::*;
use futures_util::Stream;
use std::pin::Pin;

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    ChatEventPayload, NodeExecutionStatusEvent, PlanDagDeltaEvent, PlanDagEdge, PlanDagNode,
};
// REMOVED: CursorPosition import - user presence now handled via WebSocket only

pub struct Subscription;

/// Collaboration event types for real-time updates
#[derive(Clone, Debug, SimpleObject)]
pub struct CollaborationEvent {
    pub event_id: String,
    pub plan_id: String,
    pub user_id: String,
    pub event_type: CollaborationEventType,
    pub timestamp: String,
    pub data: CollaborationEventData,
}

#[derive(Clone, Debug, Enum, Copy, PartialEq, Eq)]
pub enum CollaborationEventType {
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    EdgeCreated,
    EdgeDeleted,
    UserJoined,
    UserLeft,
    CursorMoved,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CollaborationEventData {
    pub node_event: Option<NodeEventData>,
    pub edge_event: Option<EdgeEventData>,
    pub user_event: Option<UserEventData>,
    pub cursor_event: Option<CursorEventData>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct NodeEventData {
    pub node: PlanDagNode,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct EdgeEventData {
    pub edge: PlanDagEdge,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct UserEventData {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CursorEventData {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
    pub position_x: f64,
    pub position_y: f64,
    pub selected_node_id: Option<String>,
}

/// User presence information for collaborative editing
#[derive(Clone, Debug, SimpleObject)]
pub struct UserPresenceEvent {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
    pub plan_id: String,
    pub is_online: bool,
    pub selected_node_id: Option<String>,
    pub last_active: String,
}

/// Plan DAG update events for real-time synchronization
#[derive(Clone, Debug, SimpleObject)]
pub struct PlanDagUpdateEvent {
    pub plan_id: String,
    pub update_type: PlanDagUpdateType,
    pub data: PlanDagUpdateData,
    pub user_id: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Enum, Copy, PartialEq, Eq)]
pub enum PlanDagUpdateType {
    NodeAdded,
    NodeUpdated,
    NodeRemoved,
    EdgeAdded,
    EdgeRemoved,
    MetadataUpdated,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlanDagUpdateData {
    pub node: Option<PlanDagNode>,
    pub edge: Option<PlanDagEdge>,
    pub metadata: Option<String>, // JSON string for metadata
}

#[Subscription]
impl Subscription {
    /// Subscribe to Plan DAG updates for a specific plan
    async fn plan_dag_updated(
        &self,
        ctx: &Context<'_>,
        plan_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = PlanDagUpdateEvent> + Send>>> {
        let _context = ctx.data::<GraphQLContext>()?;

        // Subscribe to collaboration events for this plan
        let mut receiver = COLLABORATION_EVENTS.subscribe(plan_id.clone()).await;

        // Filter events for Plan DAG updates only
        let stream = async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                if event.plan_id == plan_id {
                    let update_event = match event.event_type {
                        CollaborationEventType::NodeCreated => {
                            if let Some(node_data) = &event.data.node_event {
                                Some(PlanDagUpdateEvent {
                                    plan_id: event.plan_id,
                                    update_type: PlanDagUpdateType::NodeAdded,
                                    data: PlanDagUpdateData {
                                        node: Some(node_data.node.clone()),
                                        edge: None,
                                        metadata: None,
                                    },
                                    user_id: event.user_id,
                                    timestamp: event.timestamp,
                                })
                            } else { None }
                        },
                        CollaborationEventType::NodeUpdated => {
                            if let Some(node_data) = &event.data.node_event {
                                Some(PlanDagUpdateEvent {
                                    plan_id: event.plan_id,
                                    update_type: PlanDagUpdateType::NodeUpdated,
                                    data: PlanDagUpdateData {
                                        node: Some(node_data.node.clone()),
                                        edge: None,
                                        metadata: None,
                                    },
                                    user_id: event.user_id,
                                    timestamp: event.timestamp,
                                })
                            } else { None }
                        },
                        CollaborationEventType::NodeDeleted => {
                            if let Some(node_data) = &event.data.node_event {
                                Some(PlanDagUpdateEvent {
                                    plan_id: event.plan_id,
                                    update_type: PlanDagUpdateType::NodeRemoved,
                                    data: PlanDagUpdateData {
                                        node: Some(node_data.node.clone()),
                                        edge: None,
                                        metadata: None,
                                    },
                                    user_id: event.user_id,
                                    timestamp: event.timestamp,
                                })
                            } else { None }
                        },
                        CollaborationEventType::EdgeCreated => {
                            if let Some(edge_data) = &event.data.edge_event {
                                Some(PlanDagUpdateEvent {
                                    plan_id: event.plan_id,
                                    update_type: PlanDagUpdateType::EdgeAdded,
                                    data: PlanDagUpdateData {
                                        node: None,
                                        edge: Some(edge_data.edge.clone()),
                                        metadata: None,
                                    },
                                    user_id: event.user_id,
                                    timestamp: event.timestamp,
                                })
                            } else { None }
                        },
                        CollaborationEventType::EdgeDeleted => {
                            if let Some(edge_data) = &event.data.edge_event {
                                Some(PlanDagUpdateEvent {
                                    plan_id: event.plan_id,
                                    update_type: PlanDagUpdateType::EdgeRemoved,
                                    data: PlanDagUpdateData {
                                        node: None,
                                        edge: Some(edge_data.edge.clone()),
                                        metadata: None,
                                    },
                                    user_id: event.user_id,
                                    timestamp: event.timestamp,
                                })
                            } else { None }
                        },
                        _ => None,
                    };

                    if let Some(update) = update_event {
                        yield update;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Subscribe to all collaboration events for a specific plan
    async fn collaboration_events(
        &self,
        ctx: &Context<'_>,
        plan_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = CollaborationEvent> + Send>>> {
        let _context = ctx.data::<GraphQLContext>()?;

        // Subscribe to collaboration events for this plan
        let mut receiver = COLLABORATION_EVENTS.subscribe(plan_id.clone()).await;

        // Stream all events for this plan with lag detection
        let plan_id_clone = plan_id.clone();
        let stream = async_stream::stream! {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if event.plan_id == plan_id_clone {
                            yield event;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        // Receiver fell behind and missed messages
                        tracing::warn!(
                            "Broadcast receiver lagged for plan {}, skipped {} messages",
                            plan_id_clone,
                            skipped
                        );
                        // Continue receiving after lag
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Broadcast channel closed for plan {}", plan_id_clone);
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    // REMOVED: user_presence_changed subscription - user presence now handled via WebSocket only
    // Real-time presence data is available through the WebSocket collaboration system at /ws/collaboration

    /// Subscribe to Plan DAG delta changes (JSON Patch format) for efficient updates
    async fn plan_dag_delta_changed(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Pin<Box<dyn Stream<Item = PlanDagDeltaEvent> + Send>>> {
        let _context = ctx.data::<GraphQLContext>()?;

        // Subscribe to delta events for this project
        let mut receiver = DELTA_EVENTS.subscribe(project_id).await;

        // Stream delta events for this project
        let stream = async_stream::stream! {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if event.project_id == project_id {
                            yield event;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(
                            "Delta broadcast receiver lagged for project {}, skipped {} messages",
                            project_id,
                            skipped
                        );
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Delta broadcast channel closed for project {}", project_id);
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Subscribe to node execution status changes for efficient real-time updates
    async fn node_execution_status_changed(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<Pin<Box<dyn Stream<Item = NodeExecutionStatusEvent> + Send>>> {
        let _context = ctx.data::<GraphQLContext>()?;

        // Subscribe to execution status events for this project
        let mut receiver = EXECUTION_STATUS_EVENTS.subscribe(project_id).await;

        // Stream execution status events for this project
        let stream = async_stream::stream! {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        if event.project_id == project_id {
                            yield event;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(
                            "Execution status receiver lagged for project {}, skipped {} messages",
                            project_id,
                            skipped
                        );
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Execution status channel closed for project {}", project_id);
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Subscribe to chat events emitted by an active console session.
    async fn chat_events(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "sessionId")] session_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = ChatEventPayload> + Send>>> {
        let context = ctx.data::<GraphQLContext>()?;
        let (history, mut receiver) = context
            .chat_manager
            .subscribe(&session_id)
            .await
            .map_err(|e| StructuredError::service("ChatManager::subscribe", e))?;

        let stream = async_stream::stream! {
            for event in history {
                yield ChatEventPayload::from(event);
            }

            loop {
                use tokio::sync::broadcast::error::RecvError;

                match receiver.recv().await {
                    Ok(event) => yield ChatEventPayload::from(event),
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

// Global storage for plan broadcasters using generic EventBroadcaster
lazy_static::lazy_static! {
    pub static ref COLLABORATION_EVENTS: crate::utils::EventBroadcaster<String, CollaborationEvent> =
        crate::utils::EventBroadcaster::new(1000);

    pub static ref DELTA_EVENTS: crate::utils::EventBroadcaster<i32, PlanDagDeltaEvent> =
        crate::utils::EventBroadcaster::new(1000);

    pub static ref EXECUTION_STATUS_EVENTS: crate::utils::EventBroadcaster<i32, NodeExecutionStatusEvent> =
        crate::utils::EventBroadcaster::new(1000);
}

/// Publish a collaboration event to all subscribers of a plan
pub async fn publish_collaboration_event(event: CollaborationEvent) -> Result<(), String> {
    let plan_id = event.plan_id.clone();

    // Check subscriber count for lag detection
    let receiver_count = COLLABORATION_EVENTS.receiver_count(&plan_id).await;
    if receiver_count > 50 {
        tracing::warn!(
            "High subscriber count ({}) for plan {}, potential lag risk",
            receiver_count,
            plan_id
        );
    }

    let count = COLLABORATION_EVENTS.publish(plan_id.clone(), event).await?;

    if count == 0 {
        tracing::debug!(
            "No active receivers for collaboration event on plan {}",
            plan_id
        );
    }

    Ok(())
}

/// Helper function to create collaboration events
pub fn create_collaboration_event(
    plan_id: String,
    user_id: String,
    event_type: CollaborationEventType,
    data: CollaborationEventData,
) -> CollaborationEvent {
    CollaborationEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        plan_id,
        user_id,
        event_type,
        timestamp: chrono::Utc::now().to_rfc3339(),
        data,
    }
}

/// Create user event data for subscriptions
pub fn create_user_event_data(
    user_id: String,
    user_name: String,
    avatar_color: String,
) -> UserEventData {
    UserEventData {
        user_id,
        user_name,
        avatar_color,
    }
}

/// Publish a delta event to all subscribers of a project
pub async fn publish_delta_event(event: PlanDagDeltaEvent) -> Result<(), String> {
    let project_id = event.project_id;

    let count = DELTA_EVENTS.publish(project_id, event).await?;

    if count == 0 {
        tracing::debug!(
            "No active receivers for delta event on project {}",
            project_id
        );
    }

    Ok(())
}

/// Publish an execution status event to all subscribers of a project
pub async fn publish_execution_status_event(event: NodeExecutionStatusEvent) -> Result<(), String> {
    let project_id = event.project_id;

    // Don't treat no receivers as an error - it's normal for no one to be listening
    let count = EXECUTION_STATUS_EVENTS.publish(project_id, event).await?;

    if count == 0 {
        tracing::debug!(
            "No active receivers for execution status event on project {}",
            project_id
        );
    }

    Ok(())
}
