use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use once_cell::sync::Lazy;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tokio::sync::RwLock;

use crate::{
    console::chat::{ChatConfig, ChatProvider},
    database::entities::system_settings,
};

static DESCRIPTORS: Lazy<HashMap<&'static str, &'static SettingDescriptor>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for descriptor in SettingDescriptor::all().iter() {
        map.insert(descriptor.key, descriptor);
    }
    map
});

static DESCRIPTOR_LIST: &[SettingDescriptor] = &[
    SettingDescriptor {
        key: "LAYERCAKE_CHAT_PROVIDER",
        label: "Default Chat Provider",
        category: "Chat",
        description: "Provider used for new chat sessions unless overridden per request.",
        value_type: SettingValueType::Enum,
        allowed_values: &["ollama", "openai", "claude", "gemini"],
        default_value: "ollama",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_CHAT_TIMEOUT_SECS",
        label: "Chat Request Timeout (seconds)",
        category: "Chat",
        description: "Maximum request duration for chat completions.",
        value_type: SettingValueType::Integer,
        allowed_values: &[],
        default_value: "90",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_CHAT_SYSTEM_PROMPT",
        label: "Chat System Prompt",
        category: "Chat",
        description: "Optional prefix prompt applied to every chat session.",
        value_type: SettingValueType::Text,
        allowed_values: &[],
        default_value: "",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_MCP_SERVER_URL",
        label: "MCP Server URL",
        category: "Chat",
        description: "Endpoint used for MCP tool discovery.",
        value_type: SettingValueType::Url,
        allowed_values: &[],
        default_value: "http://localhost:3000/mcp",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_OPENAI_MODEL",
        label: "OpenAI Model",
        category: "OpenAI",
        description: "Model identifier used for OpenAI provider conversations.",
        value_type: SettingValueType::String,
        allowed_values: &[],
        default_value: "gpt-4o-mini",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "OPENAI_API_KEY",
        label: "OpenAI API Key",
        category: "OpenAI",
        description: "Secret used to authenticate with the OpenAI API.",
        value_type: SettingValueType::Secret,
        allowed_values: &[],
        default_value: "",
        is_secret: true,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "OPENAI_BASE_URL",
        label: "OpenAI Base URL",
        category: "OpenAI",
        description: "Optional override for the OpenAI API endpoint.",
        value_type: SettingValueType::Url,
        allowed_values: &[],
        default_value: "",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_CLAUDE_MODEL",
        label: "Claude Model",
        category: "Anthropic",
        description: "Model identifier used for Anthropic Claude requests.",
        value_type: SettingValueType::String,
        allowed_values: &[],
        default_value: "claude-3-5-sonnet-20241022",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "ANTHROPIC_API_KEY",
        label: "Anthropic API Key",
        category: "Anthropic",
        description: "Secret used to authenticate with Anthropic endpoints.",
        value_type: SettingValueType::Secret,
        allowed_values: &[],
        default_value: "",
        is_secret: true,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "ANTHROPIC_BASE_URL",
        label: "Anthropic Base URL",
        category: "Anthropic",
        description: "Optional override for the Anthropic API endpoint.",
        value_type: SettingValueType::Url,
        allowed_values: &[],
        default_value: "",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_GEMINI_MODEL",
        label: "Gemini Model",
        category: "Google Gemini",
        description: "Model identifier used for Google Gemini requests.",
        value_type: SettingValueType::String,
        allowed_values: &[],
        default_value: "gemini-2.0-flash-exp",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "GOOGLE_API_KEY",
        label: "Google API Key",
        category: "Google Gemini",
        description: "Secret used to authenticate with Google AI Studio.",
        value_type: SettingValueType::Secret,
        allowed_values: &[],
        default_value: "",
        is_secret: true,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "GOOGLE_BASE_URL",
        label: "Google Base URL",
        category: "Google Gemini",
        description: "Optional override for the Google AI API endpoint.",
        value_type: SettingValueType::Url,
        allowed_values: &[],
        default_value: "",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_OLLAMA_MODEL",
        label: "Ollama Model",
        category: "Ollama",
        description: "Model identifier used for local Ollama completions.",
        value_type: SettingValueType::String,
        allowed_values: &[],
        default_value: "llama3.2",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "OLLAMA_BASE_URL",
        label: "Ollama Base URL",
        category: "Ollama",
        description: "Endpoint for the Ollama server.",
        value_type: SettingValueType::Url,
        allowed_values: &[],
        default_value: "http://127.0.0.1:11434",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "OLLAMA_API_KEY",
        label: "Ollama API Key",
        category: "Ollama",
        description: "Optional token used when Ollama requires authentication.",
        value_type: SettingValueType::Secret,
        allowed_values: &[],
        default_value: "",
        is_secret: true,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_EMBEDDING_PROVIDER",
        label: "Embedding Provider",
        category: "Data Acquisition",
        description: "Provider used for knowledge base embeddings.",
        value_type: SettingValueType::Enum,
        allowed_values: &["openai", "ollama"],
        default_value: "ollama",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_OPENAI_EMBEDDING_MODEL",
        label: "OpenAI Embedding Model",
        category: "OpenAI",
        description: "Model identifier used when embeddings run against OpenAI.",
        value_type: SettingValueType::String,
        allowed_values: &[],
        default_value: "text-embedding-3-large",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_OLLAMA_EMBEDDING_MODEL",
        label: "Ollama Embedding Model",
        category: "Ollama",
        description: "Model identifier used when embeddings run against a local Ollama server.",
        value_type: SettingValueType::String,
        allowed_values: &[],
        default_value: "nomic-embed-text:v1.5",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_RAG_DEFAULT_TOP_K",
        label: "RAG Default Top K",
        category: "RAG",
        description: "Default number of document chunks to retrieve for chat context.",
        value_type: SettingValueType::Integer,
        allowed_values: &[],
        default_value: "5",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_RAG_DEFAULT_THRESHOLD",
        label: "RAG Default Threshold",
        category: "RAG",
        description: "Default minimum similarity score (0.0-1.0) for including chunks in context.",
        value_type: SettingValueType::Float,
        allowed_values: &[],
        default_value: "0.7",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_RAG_MAX_CONTEXT_TOKENS",
        label: "RAG Max Context Tokens",
        category: "RAG",
        description: "Maximum number of tokens to include from retrieved chunks.",
        value_type: SettingValueType::Integer,
        allowed_values: &[],
        default_value: "4000",
        is_secret: false,
        is_read_only: false,
    },
    SettingDescriptor {
        key: "LAYERCAKE_RAG_ENABLE_CITATIONS",
        label: "RAG Enable Citations",
        category: "RAG",
        description: "Whether to append source citations to chat responses by default.",
        value_type: SettingValueType::Boolean,
        allowed_values: &["true", "false"],
        default_value: "true",
        is_secret: false,
        is_read_only: false,
    },
];

/// Canonical metadata describing a configurable runtime setting.
#[derive(Clone, Debug)]
pub struct SettingDescriptor {
    pub key: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub value_type: SettingValueType,
    pub allowed_values: &'static [&'static str],
    pub default_value: &'static str,
    pub is_secret: bool,
    pub is_read_only: bool,
}

impl SettingDescriptor {
    fn all() -> &'static [Self] {
        DESCRIPTOR_LIST
    }

    fn initial_value(&self) -> String {
        std::env::var(self.key)
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.default_value.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingValueType {
    String,
    Text,
    Url,
    Integer,
    Float,
    Boolean,
    Enum,
    Secret,
}

impl SettingValueType {
    fn as_str(&self) -> &'static str {
        match self {
            SettingValueType::String => "string",
            SettingValueType::Text => "text",
            SettingValueType::Url => "url",
            SettingValueType::Integer => "integer",
            SettingValueType::Float => "float",
            SettingValueType::Boolean => "boolean",
            SettingValueType::Enum => "enum",
            SettingValueType::Secret => "secret",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SystemSettingView {
    pub key: String,
    pub label: String,
    pub category: String,
    pub description: Option<String>,
    pub value: Option<String>,
    pub raw_value: String,
    pub value_type: SettingValueType,
    pub allowed_values: Vec<String>,
    pub is_secret: bool,
    pub is_read_only: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct SystemSettingsService {
    db: DatabaseConnection,
    cache: Arc<RwLock<HashMap<String, system_settings::Model>>>,
    chat_config: Arc<RwLock<Arc<ChatConfig>>>,
}

impl SystemSettingsService {
    pub async fn new(db: DatabaseConnection) -> Result<Self> {
        let service = Self {
            db: db.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            chat_config: Arc::new(RwLock::new(Arc::new(ChatConfig::from_map(&HashMap::new())))),
        };
        service.sync_defaults().await?;
        Ok(service)
    }

    fn descriptor(&self, key: &str) -> Option<&'static SettingDescriptor> {
        DESCRIPTORS.get(key).copied()
    }

    async fn sync_defaults(&self) -> Result<()> {
        for descriptor in SettingDescriptor::all().iter() {
            self.ensure_row(descriptor).await?;
        }
        self.refresh_cache().await?;
        Ok(())
    }

    async fn ensure_row(&self, descriptor: &SettingDescriptor) -> Result<()> {
        let existing = system_settings::Entity::find()
            .filter(system_settings::Column::Key.eq(descriptor.key))
            .one(&self.db)
            .await?;

        let now = chrono::Utc::now();

        if let Some(record) = existing {
            let needs_update = record.label != descriptor.label
                || record.category != descriptor.category
                || record.description.as_deref().unwrap_or_default() != descriptor.description
                || record.value_type != descriptor.value_type.as_str()
                || record.is_secret != descriptor.is_secret
                || record.is_read_only != descriptor.is_read_only;

            if needs_update {
                let mut active: system_settings::ActiveModel = record.into();
                active.label = Set(descriptor.label.to_string());
                active.category = Set(descriptor.category.to_string());
                active.description = Set(Some(descriptor.description.to_string()));
                active.value_type = Set(descriptor.value_type.as_str().to_string());
                active.is_secret = Set(descriptor.is_secret);
                active.is_read_only = Set(descriptor.is_read_only);
                active.updated_at = Set(now);
                active.update(&self.db).await?;
            }
            return Ok(());
        }

        let active = system_settings::ActiveModel {
            key: Set(descriptor.key.to_string()),
            value: Set(descriptor.initial_value()),
            value_type: Set(descriptor.value_type.as_str().to_string()),
            label: Set(descriptor.label.to_string()),
            category: Set(descriptor.category.to_string()),
            description: Set(Some(descriptor.description.to_string())),
            is_secret: Set(descriptor.is_secret),
            is_read_only: Set(descriptor.is_read_only),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active.insert(&self.db).await?;
        Ok(())
    }

    async fn refresh_cache(&self) -> Result<()> {
        let records = system_settings::Entity::find().all(&self.db).await?;
        let mut entries = HashMap::new();
        for record in records {
            entries.insert(record.key.clone(), record);
        }
        *self.cache.write().await = entries.clone();
        self.recompute_chat_config(&entries).await?;
        Ok(())
    }

    async fn recompute_chat_config(
        &self,
        entries: &HashMap<String, system_settings::Model>,
    ) -> Result<()> {
        let mut values = HashMap::new();
        for (key, record) in entries {
            values.insert(key.clone(), record.value.clone());
        }
        let new_config = Arc::new(ChatConfig::from_map(&values));
        *self.chat_config.write().await = new_config;
        Ok(())
    }

    fn mask_value(descriptor: &SettingDescriptor, value: &str) -> Option<String> {
        if descriptor.is_secret {
            None
        } else {
            Some(value.to_string())
        }
    }

    fn view_from_record(record: &system_settings::Model) -> Result<SystemSettingView> {
        let descriptor = DESCRIPTORS
            .get(record.key.as_str())
            .copied()
            .ok_or_else(|| anyhow!("missing descriptor for {}", record.key))?;

        Ok(SystemSettingView {
            key: record.key.clone(),
            label: descriptor.label.to_string(),
            category: descriptor.category.to_string(),
            description: Some(descriptor.description.to_string()),
            value: Self::mask_value(descriptor, &record.value),
            raw_value: record.value.clone(),
            value_type: descriptor.value_type,
            allowed_values: descriptor
                .allowed_values
                .iter()
                .map(|v| v.to_string())
                .collect(),
            is_secret: descriptor.is_secret,
            is_read_only: descriptor.is_read_only,
            updated_at: record.updated_at,
        })
    }

    pub async fn list_settings(&self) -> Result<Vec<SystemSettingView>> {
        let cache = self.cache.read().await;
        let mut views = Vec::new();
        for record in cache.values() {
            views.push(Self::view_from_record(record)?);
        }
        views.sort_by(|a, b| a.category.cmp(&b.category).then(a.label.cmp(&b.label)));
        Ok(views)
    }

    pub async fn get_setting(&self, key: &str) -> Result<SystemSettingView> {
        let cache = self.cache.read().await;
        let record = cache
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Unknown setting {}", key))?;
        Self::view_from_record(&record)
    }

    pub async fn update_setting(&self, key: &str, value: String) -> Result<SystemSettingView> {
        let descriptor = self
            .descriptor(key)
            .ok_or_else(|| anyhow!("Unknown setting {}", key))?;
        if descriptor.is_read_only {
            return Err(anyhow!("Setting {} is read-only", key));
        }

        self.validate_value(descriptor, &value)?;

        let now = Utc::now();
        let existing = system_settings::Entity::find()
            .filter(system_settings::Column::Key.eq(key))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Setting {} not found", key))?;

        let mut active: system_settings::ActiveModel = existing.into();
        active.value = Set(value);
        active.updated_at = Set(now);
        active.update(&self.db).await?;

        self.refresh_cache().await?;
        self.get_setting(key).await
    }

    fn validate_value(&self, descriptor: &SettingDescriptor, value: &str) -> Result<()> {
        match descriptor.value_type {
            SettingValueType::Integer => {
                value
                    .parse::<i64>()
                    .with_context(|| format!("{} expects an integer", descriptor.label))?;
            }
            SettingValueType::Enum => {
                let found = descriptor
                    .allowed_values
                    .iter()
                    .any(|allowed| allowed == &value);
                if !found {
                    return Err(anyhow!(
                        "{} must be one of {:?}",
                        descriptor.label,
                        descriptor.allowed_values
                    ));
                }
            }
            SettingValueType::Url => {
                if !value.is_empty() {
                    url::Url::parse(value).context("invalid URL")?;
                }
            }
            _ => {}
        }

        if descriptor.key == "LAYERCAKE_CHAT_PROVIDER" && !value.is_empty() {
            value
                .parse::<ChatProvider>()
                .context("invalid chat provider value")?;
        }

        Ok(())
    }

    pub async fn chat_config(&self) -> Arc<ChatConfig> {
        self.chat_config.read().await.clone()
    }

    pub async fn raw_value(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        cache.get(key).map(|record| record.value.clone())
    }
}
