use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Document types for different editing contexts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Canvas,
    Spreadsheet,
    #[serde(rename = "3d")]
    ThreeD,
    Timeline,
    CodeEditor,
}

/// Position data specific to each document type
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CursorPosition {
    Canvas {
        x: f64,
        y: f64,
        zoom: Option<f64>,
    },
    Spreadsheet {
        row: i32,
        column: i32,
        sheet: Option<String>,
    },
    #[serde(rename = "3d")]
    ThreeD {
        x: f64,
        y: f64,
        z: f64,
        rotation: Option<(f64, f64, f64)>,
        scale: Option<f64>,
        viewport: Option<String>,
    },
    Timeline {
        timestamp: i64,
        track: Option<i32>,
    },
    CodeEditor {
        line: i32,
        column: i32,
        file: Option<String>,
    },
}

/// Outbound messages (Client → Server)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    JoinSession { data: JoinSessionData },
    CursorUpdate { data: CursorUpdateData },
    SwitchDocument { data: DocumentSwitchData },
    LeaveSession { data: LeaveSessionData },
    Ping,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinSessionData {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
    pub document_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorUpdateData {
    pub document_id: String,
    pub document_type: DocumentType,
    pub position: CursorPosition,
    pub selected_node_id: Option<String>,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSwitchData {
    pub document_id: String,
    pub document_type: DocumentType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveSessionData {
    pub document_id: Option<String>,
}

/// Inbound messages (Server → Client)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    UserPresence { data: UserPresenceData },
    BulkPresence { data: Vec<UserPresenceData> },
    DocumentActivity { data: DocumentActivityData },
    Error { message: String },
    Pong,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPresenceData {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
    pub is_online: bool,
    pub last_active: String,
    pub documents: std::collections::HashMap<String, DocumentPresence>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPresence {
    pub document_type: DocumentType,
    pub position: Option<CursorPosition>,
    pub selected_node_id: Option<String>,
    pub last_active_in_document: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentActivityData {
    pub document_id: String,
    pub active_users: Vec<DocumentUser>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentUser {
    pub user_id: String,
    pub user_name: String,
    pub position: Option<CursorPosition>,
    pub selected_node_id: Option<String>,
}

/// Internal data structures for memory storage
#[derive(Clone, Debug)]
pub struct UserPresence {
    pub user_id: String,
    pub user_name: String,
    pub avatar_color: String,
    pub is_online: bool,
    pub last_active: Instant,
}

#[derive(Clone, Debug)]
pub struct DocumentUserState {
    pub position: Option<CursorPosition>,
    pub selected_node_id: Option<String>,
    pub last_active_in_document: Instant,
}

#[derive(Clone, Debug)]
pub struct DocumentSession {
    pub document_type: DocumentType,
    pub active_users: dashmap::DashMap<String, DocumentUserState>,
}

#[derive(Clone, Debug)]
pub struct ProjectSession {
    pub users: dashmap::DashMap<String, UserPresence>,
    pub documents: dashmap::DashMap<String, DocumentSession>,
    pub connections: dashmap::DashMap<String, tokio::sync::mpsc::Sender<ServerMessage>>,
}

/// Global collaboration state (memory-only)
#[derive(Clone, Debug)]
pub struct CollaborationState {
    pub projects: dashmap::DashMap<i32, ProjectSession>,
}

impl CollaborationState {
    pub fn new() -> Self {
        Self {
            projects: dashmap::DashMap::new(),
        }
    }

    pub fn get_or_create_project(
        &self,
        project_id: i32,
    ) -> dashmap::mapref::one::Ref<i32, ProjectSession> {
        // Use or_insert_with to create the entry if it doesn't exist
        self.projects
            .entry(project_id)
            .or_insert_with(|| ProjectSession {
                users: dashmap::DashMap::new(),
                documents: dashmap::DashMap::new(),
                connections: dashmap::DashMap::new(),
            });

        // Since we just ensured the entry exists, this should never panic,
        // but we can handle it safely
        self.projects
            .get(&project_id)
            .expect("Project entry should exist after or_insert_with")
    }
}

impl Default for CollaborationState {
    fn default() -> Self {
        Self::new()
    }
}
