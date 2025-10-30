#![cfg(feature = "console")]

use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use super::ChatProvider;
use crate::database::entities::chat_credentials;

#[derive(Clone)]
pub struct ProviderConfig {
    pub model: String,
    pub base_url: Option<String>,
}

/// Chat configuration loaded from environment variables and database state.
#[derive(Clone)]
pub struct ChatConfig {
    pub default_provider: ChatProvider,
    pub request_timeout: Duration,
    pub system_prompt: Option<String>,
    pub providers: HashMap<ChatProvider, ProviderConfig>,
}

impl ChatConfig {
    pub async fn load(db: &DatabaseConnection) -> Result<Self> {
        let _ = db; // Database-driven configuration will be wired later.

        let provider = std::env::var("LAYERCAKE_CHAT_PROVIDER")
            .ok()
            .as_deref()
            .and_then(|value| value.parse().ok())
            .unwrap_or_default();

        let timeout_secs = std::env::var("LAYERCAKE_CHAT_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(90);

        let system_prompt = std::env::var("LAYERCAKE_CHAT_SYSTEM_PROMPT").ok();

        let mut providers = HashMap::new();
        providers.insert(
            ChatProvider::OpenAi,
            ProviderConfig {
                model: std::env::var("LAYERCAKE_OPENAI_MODEL")
                    .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
                base_url: std::env::var("OPENAI_BASE_URL").ok(),
            },
        );
        providers.insert(
            ChatProvider::Claude,
            ProviderConfig {
                model: std::env::var("LAYERCAKE_CLAUDE_MODEL")
                    .unwrap_or_else(|_| "claude-3-5-sonnet-20241010".to_string()),
                base_url: None,
            },
        );
        providers.insert(
            ChatProvider::Gemini,
            ProviderConfig {
                model: std::env::var("LAYERCAKE_GEMINI_MODEL")
                    .unwrap_or_else(|_| "gemini-1.5-flash".to_string()),
                base_url: None,
            },
        );
        providers.insert(
            ChatProvider::Ollama,
            ProviderConfig {
                model: std::env::var("LAYERCAKE_OLLAMA_MODEL")
                    .unwrap_or_else(|_| "llama3.1".to_string()),
                base_url: std::env::var("OLLAMA_BASE_URL").ok(),
            },
        );

        Ok(Self {
            default_provider: provider,
            request_timeout: Duration::from_secs(timeout_secs),
            system_prompt,
            providers,
        })
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
}

impl ChatCredentialStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { _db: db }
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
            ChatProvider::OpenAi => std::env::var("OPENAI_API_KEY").ok(),
            ChatProvider::Claude => std::env::var("ANTHROPIC_API_KEY").ok(),
            ChatProvider::Gemini => std::env::var("GOOGLE_API_KEY").ok(),
            ChatProvider::Ollama => std::env::var("OLLAMA_API_KEY").ok(),
        };
        Ok(key.filter(|value| !value.is_empty()))
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

        Ok(env_key
            .and_then(|key| std::env::var(key).ok())
            .filter(|value| !value.is_empty()))
    }
}
