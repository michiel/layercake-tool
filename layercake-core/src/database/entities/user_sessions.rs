use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveValue};
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub session_id: String,
    pub user_id: i32,
    pub user_name: String,
    pub project_id: i32,
    pub layercake_graph_id: Option<i32>,
    pub cursor_position: Option<String>, // JSON: {x: f64, y: f64}
    pub selected_node_id: Option<String>,
    pub last_activity: ChronoDateTimeUtc,
    pub is_active: bool,
    pub created_at: ChronoDateTimeUtc,
    pub expires_at: ChronoDateTimeUtc,
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

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn new(user_id: i32, user_name: String, project_id: i32) -> Self {
        let now = chrono::Utc::now();
        let session_id = format!("sess_{}_{}", user_id, now.timestamp_millis());

        Self {
            id: ActiveValue::NotSet,
            session_id: Set(session_id),
            user_id: Set(user_id),
            user_name: Set(user_name),
            project_id: Set(project_id),
            layercake_graph_id: ActiveValue::NotSet,
            cursor_position: ActiveValue::NotSet,
            selected_node_id: ActiveValue::NotSet,
            last_activity: Set(now),
            is_active: Set(true),
            created_at: Set(now),
            expires_at: Set(now + chrono::Duration::hours(24)), // 24 hour session
        }
    }

    pub fn set_activity(mut self) -> Self {
        self.last_activity = Set(chrono::Utc::now());
        self
    }

    pub fn set_cursor_position(mut self, x: f64, y: f64) -> Self {
        let cursor_json = format!(r#"{{"x": {}, "y": {}}}"#, x, y);
        self.cursor_position = Set(Some(cursor_json));
        self.last_activity = Set(chrono::Utc::now());
        self
    }

    pub fn set_selected_node(mut self, node_id: Option<String>) -> Self {
        self.selected_node_id = Set(node_id);
        self.last_activity = Set(chrono::Utc::now());
        self
    }

    pub fn set_current_graph(mut self, graph_id: Option<i32>) -> Self {
        self.layercake_graph_id = Set(graph_id);
        self.last_activity = Set(chrono::Utc::now());
        self
    }

    pub fn deactivate(mut self) -> Self {
        self.is_active = Set(false);
        self.last_activity = Set(chrono::Utc::now());
        self
    }

    pub fn extend_session(mut self, hours: i64) -> Self {
        let new_expiry = chrono::Utc::now() + chrono::Duration::hours(hours);
        self.expires_at = Set(new_expiry);
        self
    }
}

impl Model {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn cursor_position_parsed(&self) -> Option<(f64, f64)> {
        self.cursor_position.as_ref().and_then(|pos| {
            serde_json::from_str::<serde_json::Value>(pos)
                .ok()
                .and_then(|json| {
                    Some((
                        json.get("x")?.as_f64()?,
                        json.get("y")?.as_f64()?,
                    ))
                })
        })
    }

    pub fn time_since_activity(&self) -> chrono::Duration {
        chrono::Utc::now() - self.last_activity
    }

    pub fn is_recently_active(&self, minutes: i64) -> bool {
        self.time_since_activity() < chrono::Duration::minutes(minutes)
    }
}