use async_graphql::*;
use futures_util::Stream;
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::pin::Pin;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::{PlanDagNode, PlanDagEdge};
use crate::graphql::types::user::CursorPosition;

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
    pub cursor_position: Option<CursorPosition>,
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
pub type SubscriptionBroadcaster = Arc<RwLock<HashMap<String, broadcast::Sender<CollaborationEvent>>>>;

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

        // Stream all events for this plan
        let stream = async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                if event.plan_id == plan_id {
                    yield event;
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Subscribe to user presence for a specific project (matching frontend expectation)
    async fn user_presence_changed(
        &self,
        ctx: &Context<'_>,
        plan_id: String,
    ) -> Result<Pin<Box<dyn Stream<Item = crate::graphql::types::UserPresenceInfo> + Send + 'static>>> {
        use crate::database::entities::{user_presence, users};
        use sea_orm::EntityTrait;

        let context = ctx.data::<GraphQLContext>()?;
        let db = context.db.clone();

        // Parse plan_id to project_id
        let project_id: i32 = plan_id.parse().map_err(|_| async_graphql::Error::new("Invalid plan_id format"))?;

        // Get current active users for this project
        let active_users = user_presence::Entity::find()
            .filter(user_presence::Column::ProjectId.eq(project_id))
            .filter(user_presence::Column::IsOnline.eq(true))
            .all(&db)
            .await?;

        // Create a stream that emits current users and then listens for updates
        let stream = async_stream::stream! {
            // First, emit all currently active users
            for presence in active_users {
                // Get user details
                if let Ok(Some(user)) = users::Entity::find_by_id(presence.user_id)
                    .one(&db)
                    .await
                {
                    let cursor_position = presence.cursor_position
                        .as_ref()
                        .and_then(|pos| serde_json::from_str::<crate::database::entities::user_presence::CursorPosition>(pos).ok())
                        .map(|pos| crate::graphql::types::CursorPosition { x: pos.x, y: pos.y });

                    let user_presence_info = crate::graphql::types::UserPresenceInfo {
                        user_id: presence.user_id.to_string(),
                        user_name: user.display_name,
                        avatar_color: user.avatar_color,
                        cursor_position,
                        selected_node_id: presence.selected_node_id,
                        is_active: presence.is_online && presence.status == "active",
                        last_seen: presence.last_seen.to_rfc3339(),
                    };

                    yield user_presence_info;
                }
            }

            // Then subscribe to future collaboration events for presence changes
            let broadcaster = get_plan_broadcaster(&project_id.to_string()).await;
            let mut receiver = broadcaster.subscribe();

            while let Ok(event) = receiver.recv().await {
                if event.plan_id == project_id.to_string() {
                    match event.event_type {
                        CollaborationEventType::UserJoined => {
                            if let Some(user_data) = &event.data.user_event {
                                let user_presence_info = crate::graphql::types::UserPresenceInfo {
                                    user_id: user_data.user_id.clone(),
                                    user_name: user_data.user_name.clone(),
                                    avatar_color: user_data.avatar_color.clone(),
                                    cursor_position: None,
                                    selected_node_id: None,
                                    is_active: true,
                                    last_seen: event.timestamp,
                                };
                                yield user_presence_info;
                            }
                        },
                        CollaborationEventType::UserLeft => {
                            if let Some(user_data) = &event.data.user_event {
                                let user_presence_info = crate::graphql::types::UserPresenceInfo {
                                    user_id: user_data.user_id.clone(),
                                    user_name: user_data.user_name.clone(),
                                    avatar_color: user_data.avatar_color.clone(),
                                    cursor_position: None,
                                    selected_node_id: None,
                                    is_active: false,
                                    last_seen: event.timestamp,
                                };
                                yield user_presence_info;
                            }
                        },
                        CollaborationEventType::CursorMoved => {
                            if let Some(cursor_data) = &event.data.cursor_event {
                                let user_presence_info = crate::graphql::types::UserPresenceInfo {
                                    user_id: cursor_data.user_id.clone(),
                                    user_name: cursor_data.user_name.clone(),
                                    avatar_color: cursor_data.avatar_color.clone(),
                                    cursor_position: Some(crate::graphql::types::CursorPosition {
                                        x: cursor_data.position_x,
                                        y: cursor_data.position_y,
                                    }),
                                    selected_node_id: cursor_data.selected_node_id.clone(),
                                    is_active: true,
                                    last_seen: event.timestamp,
                                };
                                yield user_presence_info;
                            }
                        },
                        _ => {} // Ignore other event types
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
}

/// Get or create a broadcaster for a specific plan
async fn get_plan_broadcaster(plan_id: &str) -> broadcast::Sender<CollaborationEvent> {
    let broadcasters = PLAN_BROADCASTERS.read().await;

    if let Some(sender) = broadcasters.get(plan_id) {
        sender.clone()
    } else {
        drop(broadcasters);
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
}

/// Publish a collaboration event to all subscribers of a plan
pub async fn publish_collaboration_event(event: CollaborationEvent) -> Result<(), String> {
    let broadcaster = get_plan_broadcaster(&event.plan_id).await;

    if let Err(_) = broadcaster.send(event.clone()) {
        return Err("Failed to broadcast collaboration event".to_string());
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

/// Helper functions to create specific event data
pub fn create_node_event_data(node: PlanDagNode) -> CollaborationEventData {
    CollaborationEventData {
        node_event: Some(NodeEventData { node }),
        edge_event: None,
        user_event: None,
        cursor_event: None,
    }
}

pub fn create_edge_event_data(edge: PlanDagEdge) -> CollaborationEventData {
    CollaborationEventData {
        node_event: None,
        edge_event: Some(EdgeEventData { edge }),
        user_event: None,
        cursor_event: None,
    }
}

pub fn create_user_event_data(user_id: String, user_name: String, avatar_color: String) -> CollaborationEventData {
    CollaborationEventData {
        node_event: None,
        edge_event: None,
        user_event: Some(UserEventData { user_id, user_name, avatar_color }),
        cursor_event: None,
    }
}

pub fn create_cursor_event_data(
    user_id: String,
    user_name: String,
    avatar_color: String,
    position_x: f64,
    position_y: f64,
    selected_node_id: Option<String>,
) -> CollaborationEventData {
    CollaborationEventData {
        node_event: None,
        edge_event: None,
        user_event: None,
        cursor_event: Some(CursorEventData {
            user_id,
            user_name,
            avatar_color,
            position_x,
            position_y,
            selected_node_id,
        }),
    }
}