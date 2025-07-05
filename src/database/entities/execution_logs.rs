use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "execution_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub execution_id: String,
    pub level: String,
    pub message: String,
    pub details: Option<String>,
    pub timestamp: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Success,
    Error,
    Debug,
}

impl From<LogLevel> for String {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Info => "info".to_string(),
            LogLevel::Warning => "warning".to_string(),
            LogLevel::Success => "success".to_string(),
            LogLevel::Error => "error".to_string(),
            LogLevel::Debug => "debug".to_string(),
        }
    }
}

impl From<String> for LogLevel {
    fn from(level: String) -> Self {
        match level.as_str() {
            "info" => LogLevel::Info,
            "warning" => LogLevel::Warning,
            "success" => LogLevel::Success,
            "error" => LogLevel::Error,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }
}

impl Model {
    pub fn get_level(&self) -> LogLevel {
        LogLevel::from(self.level.clone())
    }

    pub fn is_error(&self) -> bool {
        matches!(self.get_level(), LogLevel::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.get_level(), LogLevel::Warning)
    }
}