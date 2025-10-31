#![cfg(feature = "console")]

use std::{fmt, str::FromStr};

use anyhow::anyhow;
use clap::ValueEnum;
use llm::builder::LLMBackend;

/// Supported chat providers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, ValueEnum)]
pub enum ChatProvider {
    Ollama,
    OpenAi,
    Gemini,
    Claude,
}

impl Default for ChatProvider {
    fn default() -> Self {
        ChatProvider::Ollama
    }
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

    pub fn backend(&self) -> LLMBackend {
        match self {
            ChatProvider::Ollama => LLMBackend::Ollama,
            ChatProvider::OpenAi => LLMBackend::OpenAI,
            ChatProvider::Gemini => LLMBackend::Google,
            ChatProvider::Claude => LLMBackend::Anthropic,
        }
    }
}
