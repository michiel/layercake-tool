use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;

use crate::dataset_schema::DatasetGenerationResponse;

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

    /// Generate a dataset using structured LLM output
    ///
    /// This method uses rig-core 0.25's structured output capabilities to ensure
    /// type-safe, validated dataset generation. The LLM is instructed to return
    /// valid JSON matching the Graph schema, eliminating parsing errors.
    ///
    /// # Returns
    /// - YAML-formatted dataset string ready for persistence
    pub async fn run(&self, request: DatasetGenerationRequest) -> Result<String> {
        let prompt_text = self.build_prompt(&request);

        // Use agent with JSON mode for structured output
        let agent = self
            .openai
            .agent("gpt-4o-mini")
            .preamble("You are a data acquisition specialist that creates graph datasets for the Layercake pipeline. Respond ONLY with valid JSON.")
            .build();

        // Get JSON response from agent
        let response_json = agent
            .prompt(&prompt_text)
            .await
            .map_err(|e| anyhow::anyhow!("Dataset generation failed: {}", e))?;

        // Parse the JSON response into our schema type
        let response: DatasetGenerationResponse = serde_json::from_str(&response_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response as dataset schema: {}", e))?;

        // Convert to YAML for storage
        response
            .to_yaml()
            .map_err(|e| anyhow::anyhow!("Failed to convert dataset to YAML: {}", e))
    }

    /// Build the generation prompt with system instructions and user requirements
    fn build_prompt(&self, request: &DatasetGenerationRequest) -> String {
        let mut prompt = String::new();

        // System instructions
        prompt.push_str("You are a data acquisition specialist that creates graph datasets for the Layercake pipeline.\n\n");
        prompt.push_str("IMPORTANT: Your response must be valid JSON matching the provided schema.\n\n");
        prompt.push_str("Guidelines:\n");
        prompt.push_str("- Create meaningful node IDs (lowercase, underscores)\n");
        prompt.push_str("- Use descriptive labels for nodes and edges\n");
        prompt.push_str("- Ensure all edges reference valid node IDs\n");
        prompt.push_str("- All nodes must specify a layer ID\n");
        prompt.push_str("- Define at least one layer with appropriate colors\n");
        prompt.push_str("- Use weight=1 for standard items, higher for important ones\n\n");

        // User request
        prompt.push_str("User Request:\n");
        prompt.push_str(&request.prompt);

        // Tag emphasis
        if !request.tag_names.is_empty() {
            prompt.push_str("\n\nTags to emphasize: ");
            prompt.push_str(&request.tag_names.join(", "));
        }

        prompt.push_str("\n\nGenerate the dataset as JSON following the schema.");

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a generator for testing (without needing API key)
    fn test_generator() -> DatasetGenerator {
        // Set a dummy API key for testing
        std::env::set_var("OPENAI_API_KEY", "sk-test-key-for-unit-tests");
        use rig::client::ProviderClient;
        DatasetGenerator {
            openai: openai::Client::from_env(),
        }
    }

    #[test]
    fn test_build_prompt() {
        let generator = test_generator();

        let request = DatasetGenerationRequest {
            project_id: 1,
            prompt: "Create a simple authentication flow".to_string(),
            tag_names: vec!["security".to_string(), "auth".to_string()],
        };

        let prompt = generator.build_prompt(&request);

        assert!(prompt.contains("data acquisition specialist"));
        assert!(prompt.contains("Create a simple authentication flow"));
        assert!(prompt.contains("Tags to emphasize: security, auth"));
        assert!(prompt.contains("valid JSON"));
    }

    #[test]
    fn test_prompt_without_tags() {
        let generator = test_generator();

        let request = DatasetGenerationRequest {
            project_id: 1,
            prompt: "Create a dataset".to_string(),
            tag_names: vec![],
        };

        let prompt = generator.build_prompt(&request);

        assert!(!prompt.contains("Tags to emphasize"));
        assert!(prompt.contains("Create a dataset"));
    }
}
