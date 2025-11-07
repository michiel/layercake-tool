#![cfg(feature = "console")]

use std::{fmt, str::FromStr};

use anyhow::anyhow;
use clap::ValueEnum;

/// Supported chat providers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, ValueEnum)]
#[derive(Default)]
pub enum ChatProvider {
    #[default]
    Ollama,
    OpenAi,
    Gemini,
    Claude,
}


impl fmt::Display for ChatProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ChatProvider::Ollama => "ollama",
                ChatProvider::OpenAi => "openai",
                ChatProvider::Gemini => "gemini",
                ChatProvider::Claude => "claude",
            }
        )
    }
}

impl FromStr for ChatProvider {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(ChatProvider::Ollama),
            "openai" | "open_ai" | "open-ai" => Ok(ChatProvider::OpenAi),
            "gemini" => Ok(ChatProvider::Gemini),
            "claude" | "anthropic" => Ok(ChatProvider::Claude),
            _ => Err(anyhow!("unknown provider '{s}'")),
        }
    }
}

impl ChatProvider {
    pub fn display_name(&self) -> &'static str {
        match self {
            ChatProvider::Ollama => "Ollama",
            ChatProvider::OpenAi => "OpenAI",
            ChatProvider::Gemini => "Google Gemini",
            ChatProvider::Claude => "Anthropic Claude",
        }
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, ChatProvider::Ollama)
    }

    /// Get the default model for this provider
    pub fn default_model(&self) -> &'static str {
        match self {
            ChatProvider::Ollama => "llama3.2",
            ChatProvider::OpenAi => "gpt-4o-mini",
            ChatProvider::Gemini => "gemini-1.5-flash",
            ChatProvider::Claude => "claude-3-5-sonnet-20241022",
        }
    }

    /// Get the environment variable name for the API key
    pub fn api_key_env_var(&self) -> Option<&'static str> {
        match self {
            ChatProvider::Ollama => None,
            ChatProvider::OpenAi => Some("OPENAI_API_KEY"),
            ChatProvider::Gemini => Some("GOOGLE_API_KEY"),
            ChatProvider::Claude => Some("ANTHROPIC_API_KEY"),
        }
    }
}
