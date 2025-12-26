use serde::{Deserialize, Serialize};

/// Note position for sequence edge annotations.
#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NotePosition {
    Source,
    Target,
    Both,
}

/// Reference to an edge in a dataset.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceEdgeRef {
    pub dataset_id: i32,
    pub edge_id: String,
    pub note: Option<String>,
    pub note_position: Option<NotePosition>,
}
