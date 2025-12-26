use async_graphql::{Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};

use layercake_core::services::system_settings_service::{SettingValueType, SystemSettingView};

#[derive(SimpleObject, Clone, Debug)]
#[graphql(name = "SystemSetting")]
pub struct SystemSetting {
    pub key: String,
    pub label: String,
    pub category: String,
    pub description: Option<String>,
    pub value: Option<String>,
    #[graphql(name = "valueType")]
    pub value_type: SystemSettingValueType,
    #[graphql(name = "allowedValues")]
    pub allowed_values: Vec<String>,
    #[graphql(name = "isSecret")]
    pub is_secret: bool,
    #[graphql(name = "isReadOnly")]
    pub is_read_only: bool,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum SystemSettingValueType {
    String,
    Text,
    Url,
    Integer,
    Float,
    Boolean,
    Enum,
    Secret,
}

impl From<SettingValueType> for SystemSettingValueType {
    fn from(value: SettingValueType) -> Self {
        match value {
            SettingValueType::String => Self::String,
            SettingValueType::Text => Self::Text,
            SettingValueType::Url => Self::Url,
            SettingValueType::Integer => Self::Integer,
            SettingValueType::Float => Self::Float,
            SettingValueType::Boolean => Self::Boolean,
            SettingValueType::Enum => Self::Enum,
            SettingValueType::Secret => Self::Secret,
        }
    }
}

impl From<SystemSettingView> for SystemSetting {
    fn from(view: SystemSettingView) -> Self {
        Self {
            key: view.key,
            label: view.label,
            category: view.category,
            description: view.description,
            value: view.value,
            value_type: view.value_type.into(),
            allowed_values: view.allowed_values,
            is_secret: view.is_secret,
            is_read_only: view.is_read_only,
            updated_at: view.updated_at,
        }
    }
}

#[derive(InputObject)]
#[graphql(name = "SystemSettingUpdateInput")]
pub struct SystemSettingUpdateInput {
    pub key: String,
    pub value: String,
}
