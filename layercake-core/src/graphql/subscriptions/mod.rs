use async_graphql::*;
use futures_util::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{PlanDagDeltaEvent, PlanDagEdge, PlanDagNode};
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

/// Global subscription broadcaster for managing real-time events
#[allow(dead_code)]
pub type SubscriptionBroadcaster =
    Arc<RwLock<HashMap<String, broadcast::Sender<CollaborationEvent>>>>;

#[Subscription]
impl Subscription {
    /// Subscribe to Plan DAG updates for a specific plan
    async fn plan_dag_updated(
        &self,
        ctx: &Context<'_>,
        plan_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = PlanDagUpdateEvent> + Send>>> {
        let _context = ctx.data::<GraphQLContext>()?;

        // Get or create broadcaster for this plan
        let broadcaster = get_plan_broadcaster(&plan_id).await;
        let mut receiver = broadcaster.subscribe();

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

        // Get or create broadcaster for this plan
        let broadcaster = get_plan_broadcaster(&plan_id).await;
        let mut receiver = broadcaster.subscribe();

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

        // Get or create broadcaster for this project
        let broadcaster = get_delta_broadcaster(project_id).await;
        let mut receiver = broadcaster.subscribe();

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
}

// Global storage for plan broadcasters
lazy_static::lazy_static! {
    static ref PLAN_BROADCASTERS: Arc<RwLock<HashMap<String, broadcast::Sender<CollaborationEvent>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    static ref DELTA_BROADCASTERS: Arc<RwLock<HashMap<i32, broadcast::Sender<PlanDagDeltaEvent>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Get or create a broadcaster for a specific plan
async fn get_plan_broadcaster(plan_id: &str) -> broadcast::Sender<CollaborationEvent> {
    // Fast path: Try read lock first and immediately release
    {
        let broadcasters = PLAN_BROADCASTERS.read().await;
        if let Some(sender) = broadcasters.get(plan_id) {
            return sender.clone();
        }
        // Lock automatically dropped here
    }

    // Slow path: Need to create broadcaster with write lock
    let mut broadcasters = PLAN_BROADCASTERS.write().await;

    // Double-check pattern to avoid race conditions
    if let Some(sender) = broadcasters.get(plan_id) {
        sender.clone()
    } else {
        let (sender, _) = broadcast::channel(1000); // Buffer size of 1000 events
        broadcasters.insert(plan_id.to_string(), sender.clone());
        sender
    }
}

/// Publish a collaboration event to all subscribers of a plan
pub async fn publish_collaboration_event(event: CollaborationEvent) -> Result<(), String> {
    let broadcaster = get_plan_broadcaster(&event.plan_id).await;

    // Check buffer utilisation for lag detection
    let receiver_count = broadcaster.receiver_count();
    if receiver_count > 0 {
        // Note: tokio broadcast channel doesn't expose buffer usage directly
        // We can only detect lag when a send fails or receiver reports lag
        // This is a best-effort warning based on receiver count
        if receiver_count > 50 {
            tracing::warn!(
                "High subscriber count ({}) for plan {}, potential lag risk",
                receiver_count,
                event.plan_id
            );
        }
    }

    match broadcaster.send(event.clone()) {
        Ok(_) => Ok(()),
        Err(_) => {
            tracing::error!(
                "Failed to broadcast collaboration event for plan {} - no active receivers",
                event.plan_id
            );
            Err("Failed to broadcast collaboration event".to_string())
        }
    }
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

/// Get or create a delta broadcaster for a specific project
async fn get_delta_broadcaster(project_id: i32) -> broadcast::Sender<PlanDagDeltaEvent> {
    // Fast path: Try read lock first
    {
        let broadcasters = DELTA_BROADCASTERS.read().await;
        if let Some(sender) = broadcasters.get(&project_id) {
            return sender.clone();
        }
    }

    // Slow path: Create broadcaster with write lock
    let mut broadcasters = DELTA_BROADCASTERS.write().await;

    // Double-check pattern
    if let Some(sender) = broadcasters.get(&project_id) {
        sender.clone()
    } else {
        let (sender, _) = broadcast::channel(1000);
        broadcasters.insert(project_id, sender.clone());
        sender
    }
}

/// Publish a delta event to all subscribers of a project
pub async fn publish_delta_event(event: PlanDagDeltaEvent) -> Result<(), String> {
    let broadcaster = get_delta_broadcaster(event.project_id).await;

    match broadcaster.send(event.clone()) {
        Ok(_) => Ok(()),
        Err(_) => {
            tracing::error!(
                "Failed to broadcast delta event for project {} - no active receivers",
                event.project_id
            );
            Err("Failed to broadcast delta event".to_string())
        }
    }
}
