#![cfg(feature = "console")]

use std::borrow::Cow;

use clap_repl::reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptViMode};
use nu_ansi_term::Color;

use super::context::ConsoleContext;

pub struct ConsolePrompt {
    label: String,
}

impl ConsolePrompt {
    pub fn from(context: &ConsoleContext) -> Self {
        Self {
            label: context.prompt_label(),
        }
    }
}

impl Prompt for ConsolePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{} > ", Color::Cyan.bold().paint(&self.label)))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, mode: PromptEditMode) -> Cow<'_, str> {
        let indicator = match mode {
            PromptEditMode::Default => "»".to_string(),
            PromptEditMode::Emacs => "emacs»".to_string(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Insert => "vi»".to_string(),
                PromptViMode::Normal => "vi:n»".to_string(),
            },
            PromptEditMode::Custom(name) => format!("{name}»"),
        };
        Cow::Owned(format!("{} ", Color::Yellow.paint(indicator)))
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("· ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Borrowed("⌕ ")
    }
}
