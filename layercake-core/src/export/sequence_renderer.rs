use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceRenderContext {
    pub config: SequenceRenderConfigResolved,
    pub story: SequenceStorySummary,
    pub stories: Vec<SequenceStoryWithSequences>,
    pub sequences: Vec<SequenceRender>,
    pub participants: Vec<SequenceParticipant>,
    pub participant_groups: Vec<SequenceParticipantGroup>,
    pub graph_data: SequenceGraphData,
    pub layers: Vec<SequenceLayer>,
    pub first_participant_alias: Option<String>,
    pub last_participant_alias: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceStorySummary {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceStoryWithSequences {
    pub story: SequenceStorySummary,
    pub sequences: Vec<SequenceRender>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceRenderConfigResolved {
    pub contain_nodes: String,
    pub built_in_styles: String,
    pub mermaid_theme: String,
    pub plantuml_theme: String,
    pub show_notes: bool,
    pub render_all_sequences: bool,
    pub enabled_sequence_ids: Vec<i32>,
    pub use_story_layers: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceRender {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<SequenceStep>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceStep {
    pub id: String,
    pub dataset_id: i32,
    pub dataset_name: String,
    pub source: SequenceParticipantRef,
    pub target: SequenceParticipantRef,
    pub label: String,
    pub note: Option<String>,
    pub note_position: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceParticipantRef {
    pub alias: String,
    pub label: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceParticipant {
    pub alias: String,
    pub id: String,
    pub label: String,
    pub dataset_id: i32,
    pub dataset_name: String,
    pub layer: Option<String>,
    pub layer_color: Option<String>,
    pub partition_id: Option<String>,
    pub partition_label: Option<String>,
    pub partition_layer_color: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceParticipantGroup {
    pub id: String,
    pub label: String,
    pub color: Option<String>,
    pub participants: Vec<SequenceParticipant>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceGraphData {
    pub nodes: Vec<SequenceNode>,
    pub edges: Vec<SequenceEdge>,
    pub layers: Vec<SequenceLayer>,
    pub datasets: Vec<SequenceDataset>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceDataset {
    pub id: i32,
    pub name: String,
    pub nodes: Vec<SequenceNode>,
    pub edges: Vec<SequenceEdge>,
    pub layers: Vec<SequenceLayer>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceNode {
    pub id: String,
    pub label: String,
    pub layer: Option<String>,
    pub dataset_id: i32,
    pub dataset_name: String,
    pub belongs_to: Option<String>,
    pub partition_label: Option<String>,
    pub is_partition: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub dataset_id: i32,
    pub dataset_name: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceLayer {
    pub id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub dataset_id: Option<i32>,
    pub dataset_name: Option<String>,
}

pub fn render_sequence_template(
    context: &SequenceRenderContext,
    template: &str,
) -> Result<String, Box<dyn Error>> {
    let handlebars = crate::common::get_handlebars();
    let payload = json!({ "sequence": context });
    let rendered = handlebars.render_template(template, &payload)?;
    Ok(rendered)
}
