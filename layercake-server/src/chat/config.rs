
use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use super::ChatProvider;
use layercake_core::database::entities::chat_credentials;
use layercake_core::services::system_settings_service::SystemSettingsService;

#[derive(Clone)]
pub struct ProviderConfig {
    pub model: String,
    pub base_url: Option<String>,
}

/// Chat configuration loaded from environment variables and database state.
#[derive(Clone)]
pub struct ChatConfig {
    pub default_provider: ChatProvider,
    #[allow(dead_code)]
    pub request_timeout: Duration,
    pub system_prompt: Option<String>,
    pub providers: HashMap<ChatProvider, ProviderConfig>,
    #[allow(dead_code)]
    pub mcp_server_url: String,
}

impl ChatConfig {
    #[allow(dead_code)]
    pub async fn load(db: &DatabaseConnection) -> Result<Self> {
        let _ = db;
        Ok(Self::from_env())
    }

    #[allow(dead_code)]
    pub fn from_env() -> Self {
        let mut values = HashMap::new();
        for (key, default) in Self::tracked_keys() {
            let value = std::env::var(key).unwrap_or_else(|_| default.to_string());
            values.insert(key.to_string(), value);
        }
        Self::from_map(&values)
    }

    pub fn from_map(values: &HashMap<String, String>) -> Self {
        fn prioritized_value(values: &HashMap<String, String>, key: &str) -> Option<String> {
            values
                .get(key)
                .cloned()
                .filter(|value| !value.is_empty())
                .or_else(|| std::env::var(key).ok().filter(|value| !value.is_empty()))
        }

        fn read(values: &HashMap<String, String>, key: &str, default: &str) -> String {
            prioritized_value(values, key).unwrap_or_else(|| default.to_string())
        }

        let provider = prioritized_value(values, "LAYERCAKE_CHAT_PROVIDER")
            .and_then(|value| value.parse().ok())
            .unwrap_or_default();
        let timeout_secs = prioritized_value(values, "LAYERCAKE_CHAT_TIMEOUT_SECS")
            .and_then(|value| value.parse().ok())
            .unwrap_or(90);
        let system_prompt = prioritized_value(values, "LAYERCAKE_CHAT_SYSTEM_PROMPT");

        let mut providers = HashMap::new();
        providers.insert(
            ChatProvider::OpenAi,
            ProviderConfig {
                model: read(values, "LAYERCAKE_OPENAI_MODEL", "gpt-4o-mini"),
                base_url: prioritized_value(values, "OPENAI_BASE_URL"),
            },
        );
        providers.insert(
            ChatProvider::Claude,
            ProviderConfig {
                model: read(
                    values,
                    "LAYERCAKE_CLAUDE_MODEL",
                    "claude-3-5-sonnet-20241010",
                ),
                base_url: None,
            },
        );
        providers.insert(
            ChatProvider::Gemini,
            ProviderConfig {
                model: read(values, "LAYERCAKE_GEMINI_MODEL", "gemini-2.0-flash-exp"),
                base_url: None,
            },
        );
        providers.insert(
            ChatProvider::Ollama,
            ProviderConfig {
                model: read(values, "LAYERCAKE_OLLAMA_MODEL", "llama3.2"),
                base_url: Some(read(values, "OLLAMA_BASE_URL", "http://127.0.0.1:11434")),
            },
        );

        let mcp_server_url = read(
            values,
            "LAYERCAKE_MCP_SERVER_URL",
            "http://localhost:3000/mcp",
        );

        Self {
            default_provider: provider,
            request_timeout: Duration::from_secs(timeout_secs),
            system_prompt,
            providers,
            mcp_server_url,
        }
    }

    #[allow(dead_code)]
    fn tracked_keys() -> Vec<(&'static str, &'static str)> {
        vec![
            ("LAYERCAKE_CHAT_PROVIDER", "ollama"),
            ("LAYERCAKE_CHAT_TIMEOUT_SECS", "90"),
            ("LAYERCAKE_CHAT_SYSTEM_PROMPT", ""),
            ("LAYERCAKE_MCP_SERVER_URL", "http://localhost:3000/mcp"),
            ("LAYERCAKE_OPENAI_MODEL", "gpt-4o-mini"),
            ("OPENAI_API_KEY", ""),
            ("OPENAI_BASE_URL", ""),
            ("LAYERCAKE_CLAUDE_MODEL", "claude-3-5-sonnet-20241010"),
            ("ANTHROPIC_API_KEY", ""),
            ("LAYERCAKE_GEMINI_MODEL", "gemini-2.0-flash-exp"),
            ("GOOGLE_API_KEY", ""),
            ("LAYERCAKE_OLLAMA_MODEL", "llama3.2"),
            ("OLLAMA_BASE_URL", "http://127.0.0.1:11434"),
            ("OLLAMA_API_KEY", ""),
        ]
    }

    pub fn provider(&self, provider: ChatProvider) -> &ProviderConfig {
        self.providers
            .get(&provider)
            .expect("provider configuration missing")
    }
}

/// Thin wrapper to access persisted provider credentials.
#[derive(Clone)]
pub struct ChatCredentialStore {
    pub(crate) _db: DatabaseConnection,
    settings: Option<Arc<SystemSettingsService>>,
}

impl ChatCredentialStore {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            _db: db,
            settings: None,
        }
    }

    pub fn with_settings(db: DatabaseConnection, settings: Arc<SystemSettingsService>) -> Self {
        Self {
            _db: db,
            settings: Some(settings),
        }
    }

    async fn setting_value(&self, key: &str) -> Option<String> {
        if let Some(service) = &self.settings {
            if let Some(value) = service.raw_value(key).await {
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }

        std::env::var(key).ok().filter(|value| !value.is_empty())
    }

    async fn find_credentials(
        &self,
        provider: ChatProvider,
    ) -> Result<Option<chat_credentials::Model>> {
        chat_credentials::Entity::find()
            .filter(chat_credentials::Column::Provider.eq(provider.to_string()))
            .one(&self._db)
            .await
            .map_err(|err| anyhow::anyhow!("failed to load chat credentials: {}", err))
    }

    pub async fn api_key(&self, provider: ChatProvider) -> Result<Option<String>> {
        if let Some(record) = self.find_credentials(provider).await? {
            if let Some(key) = record.api_key.filter(|v| !v.is_empty()) {
                return Ok(Some(key));
            }
        }

        let key = match provider {
            ChatProvider::OpenAi => self.setting_value("OPENAI_API_KEY").await,
            ChatProvider::Claude => self.setting_value("ANTHROPIC_API_KEY").await,
            ChatProvider::Gemini => self.setting_value("GOOGLE_API_KEY").await,
            ChatProvider::Ollama => self.setting_value("OLLAMA_API_KEY").await,
        };
        Ok(key)
    }

    pub async fn base_url_override(&self, provider: ChatProvider) -> Result<Option<String>> {
        if let Some(record) = self.find_credentials(provider).await? {
            if let Some(url) = record.base_url.filter(|v| !v.is_empty()) {
                return Ok(Some(url));
            }
        }

        let env_key = match provider {
            ChatProvider::Ollama => Some("OLLAMA_BASE_URL"),
            ChatProvider::OpenAi => Some("OPENAI_BASE_URL"),
            ChatProvider::Claude => None,
            ChatProvider::Gemini => None,
        };

        if let Some(key) = env_key {
            return Ok(self.setting_value(key).await);
        }

        Ok(None)
    }
}
