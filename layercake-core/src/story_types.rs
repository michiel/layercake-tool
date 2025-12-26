use serde::{Deserialize, Serialize};

/// Layer source configuration for a story.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryLayerConfig {
    pub source_dataset_id: Option<i32>,
    pub mode: String,
}
