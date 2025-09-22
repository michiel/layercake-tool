use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[sea_orm(table_name = "user_presence")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub project_id: i32,
    pub session_id: String,
    pub layercake_graph_id: Option<i32>,
    pub cursor_position: Option<String>, // JSON: {x: f64, y: f64}
    pub selected_node_id: Option<String>,
    pub viewport_position: Option<String>, // JSON: {x: f64, y: f64, zoom: f64}
    pub current_tool: Option<String>, // "select", "pan", "node_creation", etc.
    pub is_online: bool,
    pub last_seen: ChronoDateTimeUtc,
    pub last_heartbeat: ChronoDateTimeUtc,
    pub status: String, // "active", "idle", "away", "offline"
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id"
    )]
    Projects,
    #[sea_orm(
        belongs_to = "super::user_sessions::Entity",
        from = "Column::SessionId",
        to = "super::user_sessions::Column::SessionId"
    )]
    UserSessions,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::projects::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::user_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Idle,
    Away,
    Offline,
}

impl ToString for UserStatus {
    fn to_string(&self) -> String {
        match self {
            UserStatus::Active => "active".to_string(),
            UserStatus::Idle => "idle".to_string(),
            UserStatus::Away => "away".to_string(),
            UserStatus::Offline => "offline".to_string(),
        }
    }
}

impl UserStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(UserStatus::Active),
            "idle" => Ok(UserStatus::Idle),
            "away" => Ok(UserStatus::Away),
            "offline" => Ok(UserStatus::Offline),
            _ => Err(format!("Invalid user status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportPosition {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl ActiveModel {
    pub fn new(user_id: i32, project_id: i32, session_id: String) -> Self {
        let now = chrono::Utc::now();

        Self {
            id: ActiveValue::NotSet,
            user_id: Set(user_id),
            project_id: Set(project_id),
            session_id: Set(session_id),
            layercake_graph_id: ActiveValue::NotSet,
            cursor_position: ActiveValue::NotSet,
            selected_node_id: ActiveValue::NotSet,
            viewport_position: ActiveValue::NotSet,
            current_tool: Set(Some("select".to_string())),
            is_online: Set(true),
            last_seen: Set(now),
            last_heartbeat: Set(now),
            status: Set(UserStatus::Active.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }

    pub fn update_cursor_position(mut self, x: f64, y: f64) -> Self {
        let cursor = CursorPosition { x, y };
        let cursor_json = serde_json::to_string(&cursor)
            .unwrap_or_else(|_| "{}".to_string());

        self.cursor_position = Set(Some(cursor_json));
        self.last_seen = Set(chrono::Utc::now());
        self.last_heartbeat = Set(chrono::Utc::now());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn update_viewport(mut self, x: f64, y: f64, zoom: f64) -> Self {
        let viewport = ViewportPosition { x, y, zoom };
        let viewport_json = serde_json::to_string(&viewport)
            .unwrap_or_else(|_| "{}".to_string());

        self.viewport_position = Set(Some(viewport_json));
        self.last_seen = Set(chrono::Utc::now());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn select_node(mut self, node_id: Option<String>) -> Self {
        self.selected_node_id = Set(node_id);
        self.last_seen = Set(chrono::Utc::now());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_current_graph(mut self, graph_id: Option<i32>) -> Self {
        self.layercake_graph_id = Set(graph_id);
        self.last_seen = Set(chrono::Utc::now());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_tool(mut self, tool: String) -> Self {
        self.current_tool = Set(Some(tool));
        self.last_seen = Set(chrono::Utc::now());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn set_status(mut self, status: UserStatus) -> Self {
        self.status = Set(status.to_string());
        self.last_seen = Set(chrono::Utc::now());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn heartbeat(mut self) -> Self {
        let now = chrono::Utc::now();
        self.last_heartbeat = Set(now);
        self.updated_at = Set(now);

        // Auto-update status based on activity - use current last_seen if set, otherwise now
        let last_seen_time = match &self.last_seen {
            ActiveValue::Set(time) => *time,
            _ => now,
        };

        let time_since_seen = now - last_seen_time;
        let new_status = if time_since_seen > chrono::Duration::minutes(10) {
            UserStatus::Idle
        } else {
            UserStatus::Active
        };

        self.status = Set(new_status.to_string());
        self
    }

    pub fn go_offline(mut self) -> Self {
        self.is_online = Set(false);
        self.status = Set(UserStatus::Offline.to_string());
        self.updated_at = Set(chrono::Utc::now());
        self
    }

    pub fn go_online(mut self) -> Self {
        let now = chrono::Utc::now();
        self.is_online = Set(true);
        self.status = Set(UserStatus::Active.to_string());
        self.last_seen = Set(now);
        self.last_heartbeat = Set(now);
        self.updated_at = Set(now);
        self
    }
}

impl Model {
    pub fn get_status(&self) -> Result<UserStatus, String> {
        UserStatus::from_str(&self.status)
    }

    pub fn get_cursor_position(&self) -> Option<CursorPosition> {
        self.cursor_position
            .as_ref()
            .and_then(|pos| serde_json::from_str(pos).ok())
    }

    pub fn get_viewport_position(&self) -> Option<ViewportPosition> {
        self.viewport_position
            .as_ref()
            .and_then(|pos| serde_json::from_str(pos).ok())
    }

    pub fn is_active(&self) -> bool {
        self.is_online && matches!(self.get_status(), Ok(UserStatus::Active))
    }

    pub fn is_idle(&self) -> bool {
        self.is_online && matches!(self.get_status(), Ok(UserStatus::Idle))
    }

    pub fn is_away(&self) -> bool {
        self.is_online && matches!(self.get_status(), Ok(UserStatus::Away))
    }

    pub fn minutes_since_last_seen(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.last_seen).num_minutes()
    }

    pub fn minutes_since_last_heartbeat(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.last_heartbeat).num_minutes()
    }

    pub fn should_auto_logout(&self) -> bool {
        // Auto-logout after 30 minutes of no heartbeat
        self.minutes_since_last_heartbeat() > 30
    }

    pub fn should_mark_idle(&self) -> bool {
        // Mark idle after 5 minutes of no activity
        self.minutes_since_last_seen() > 5 && self.is_active()
    }

    pub fn should_mark_away(&self) -> bool {
        // Mark away after 15 minutes of no activity
        self.minutes_since_last_seen() > 15 && (self.is_active() || self.is_idle())
    }

    pub fn has_cursor_moved_recently(&self, minutes: i64) -> bool {
        self.minutes_since_last_seen() <= minutes
    }

    pub fn is_in_same_view(&self, other_position: &ViewportPosition, tolerance: f64) -> bool {
        if let Some(my_viewport) = self.get_viewport_position() {
            let dx = (my_viewport.x - other_position.x).abs();
            let dy = (my_viewport.y - other_position.y).abs();
            let zoom_diff = (my_viewport.zoom - other_position.zoom).abs();

            dx < tolerance && dy < tolerance && zoom_diff < 0.5
        } else {
            false
        }
    }

    pub fn cursor_distance_from(&self, other_position: &CursorPosition) -> Option<f64> {
        self.get_cursor_position().map(|my_cursor| {
            let dx = my_cursor.x - other_position.x;
            let dy = my_cursor.y - other_position.y;
            (dx * dx + dy * dy).sqrt()
        })
    }
}