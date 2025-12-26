use serde::{Deserialize, Serialize};

// Story Node Configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryNodeConfig {
    pub story_id: Option<i32>,
}

// Sequence Artefact Node Configuration
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum SequenceArtefactRenderTarget {
    MermaidSequence,
    PlantUmlSequence,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum SequenceContainNodes {
    One,
    All,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceRenderConfig {
    pub contain_nodes: Option<SequenceContainNodes>,
    pub built_in_styles: Option<RenderBuiltinStyle>,
    pub show_notes: Option<bool>,
    pub render_all_sequences: Option<bool>,
    pub enabled_sequence_ids: Option<Vec<i32>>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum RenderBuiltinStyle {
    None,
    Light,
    Dark,
}
