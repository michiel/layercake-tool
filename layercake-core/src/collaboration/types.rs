use tokio::sync::{mpsc, oneshot};
use crate::server::websocket::types::{ServerMessage, CursorPosition};

/// Commands sent to the CollaborationCoordinator
pub enum CoordinatorCommand {
    JoinProject {
        project_id: i32,
        user_id: String,
        user_name: String,
        avatar_color: Option<String>,
        sender: mpsc::Sender<ServerMessage>,
        response: oneshot::Sender<Result<(), String>>,
    },
    LeaveProject {
        project_id: i32,
        user_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    UpdateCursor {
        project_id: i32,
        user_id: String,
        document_id: String,
        position: CursorPosition,
        selected_node_id: Option<String>,
    },
    SwitchDocument {
        project_id: i32,
        user_id: String,
        document_id: String,
        document_type: crate::server::websocket::types::DocumentType,
    },
    #[allow(dead_code)]
    GetProjectHealth {
        project_id: i32,
        response: oneshot::Sender<ProjectHealthReport>,
    },
    #[allow(dead_code)]
    Shutdown {
        response: oneshot::Sender<()>,
    },
}

/// Commands sent to a ProjectActor
pub enum ProjectCommand {
    Join {
        user_id: String,
        user_name: String,
        avatar_color: Option<String>,
        sender: mpsc::Sender<ServerMessage>,
        response: oneshot::Sender<Result<(), String>>,
    },
    Leave {
        user_id: String,
        response: oneshot::Sender<Result<(), String>>,
    },
    UpdateCursor {
        user_id: String,
        document_id: String,
        position: CursorPosition,
        selected_node_id: Option<String>,
    },
    SwitchDocument {
        user_id: String,
        document_id: String,
        document_type: crate::server::websocket::types::DocumentType,
    },
    IsEmpty {
        response: oneshot::Sender<bool>,
    },
    HealthReport {
        response: oneshot::Sender<ProjectHealthReport>,
    },
    Shutdown {
        response: oneshot::Sender<()>,
    },
}

/// Health report for a project
#[derive(Debug, Clone)]
pub struct ProjectHealthReport {
    pub project_id: i32,
    pub active_users: usize,
    pub active_connections: usize,
    pub active_documents: usize,
}

impl ProjectHealthReport {
    pub fn not_found() -> Self {
        Self {
            project_id: 0,
            active_users: 0,
            active_connections: 0,
            active_documents: 0,
        }
    }
}