use anyhow::Result;
use rig::providers::openai;
use rig::{client::CompletionClient, completion::Prompt};

/// Parameters for dataset generation jobs fed into rig.
#[derive(Debug, Clone)]
pub struct DatasetGenerationRequest {
    pub project_id: i32,
    pub prompt: String,
    pub tag_names: Vec<String>,
}

pub struct DatasetGenerator {
    openai: openai::Client,
}

impl DatasetGenerator {
    pub fn new(openai: openai::Client) -> Self {
        Self { openai }
    }

    pub async fn run(&self, request: DatasetGenerationRequest) -> Result<String> {
        let agent = self
            .openai
            .agent("gpt-4o-mini")
            .preamble("You are a data acquisition specialist that creates YAML datasets usable by Layercake's pipeline.")
            .build();

        let prompt_text = if request.tag_names.is_empty() {
            request.prompt.clone()
        } else {
            format!(
                "{}\n\nTags to emphasize: {}",
                request.prompt,
                request.tag_names.join(", ")
            )
        };

        let completion = agent.prompt(prompt_text).await?;

        Ok(completion)
    }
}
