use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Story;
use layercake_core::database::entities::{sequences, stories};

/// Note position for sequence edge annotations
///
/// serde uses camelCase (lowercase variants) to match the persisted / pipeline
/// shape; GraphQL uses PascalCase for the enum items. The two renamings are
/// independent (serde ≠ GraphQL wire), and both must stay aligned with
/// `layercake_core::sequence_types::NotePosition`.
#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
#[graphql(name = "SequenceNotePosition", rename_items = "PascalCase")]
#[serde(rename_all = "camelCase")]
pub enum NotePosition {
    Source,
    Target,
    Both,
}

/// Reference to an edge in a dataset (GraphQL boundary type).
///
/// This is the API-facing shape only. The persisted (DB) representation is
/// owned by the pipeline type `layercake_core::sequence_types::SequenceEdgeRef`
/// — the mutation converts to it before serializing, so there is exactly one
/// on-disk shape. Do NOT `serde_json::to_string` this type into the database.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "SequenceEdgeRefInput")]
#[serde(rename_all = "camelCase")]
pub struct SequenceEdgeRef {
    #[graphql(name = "datasetId")]
    pub dataset_id: i32,
    #[graphql(name = "edgeId")]
    pub edge_id: String,
    /// Optional note text for this edge in the sequence
    pub note: Option<String>,
    /// Where to display the note: Source, Target, or Both
    #[graphql(name = "notePosition")]
    pub note_position: Option<NotePosition>,
}

impl From<NotePosition> for layercake_core::sequence_types::NotePosition {
    fn from(value: NotePosition) -> Self {
        match value {
            NotePosition::Source => layercake_core::sequence_types::NotePosition::Source,
            NotePosition::Target => layercake_core::sequence_types::NotePosition::Target,
            NotePosition::Both => layercake_core::sequence_types::NotePosition::Both,
        }
    }
}

impl From<SequenceEdgeRef> for layercake_core::sequence_types::SequenceEdgeRef {
    fn from(value: SequenceEdgeRef) -> Self {
        layercake_core::sequence_types::SequenceEdgeRef {
            dataset_id: value.dataset_id,
            edge_id: value.edge_id,
            note: value.note,
            note_position: value.note_position.map(Into::into),
        }
    }
}

/// Serialize a list of API edge refs to the canonical persisted JSON shape
/// (owned by the pipeline `SequenceEdgeRef`).
pub fn edge_refs_to_persisted_json(
    refs: Vec<SequenceEdgeRef>,
) -> Result<String, serde_json::Error> {
    let pipeline: Vec<layercake_core::sequence_types::SequenceEdgeRef> =
        refs.into_iter().map(Into::into).collect();
    serde_json::to_string(&pipeline)
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Sequence {
    pub id: i32,
    #[graphql(name = "storyId")]
    pub story_id: i32,
    pub name: String,
    pub description: Option<String>,
    #[graphql(name = "enabledDatasetIds")]
    pub enabled_dataset_ids: Vec<i32>,
    #[graphql(name = "edgeOrder")]
    pub edge_order: Vec<SequenceEdgeRef>,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl From<sequences::Model> for Sequence {
    fn from(model: sequences::Model) -> Self {
        let enabled_dataset_ids: Vec<i32> =
            serde_json::from_str(&model.enabled_dataset_ids).unwrap_or_default();
        let edge_order: Vec<SequenceEdgeRef> =
            serde_json::from_str(&model.edge_order).unwrap_or_default();

        Self {
            id: model.id,
            story_id: model.story_id,
            name: model.name,
            description: model.description,
            enabled_dataset_ids,
            edge_order,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[ComplexObject]
impl Sequence {
    async fn story(&self, ctx: &Context<'_>) -> Result<Option<Story>> {
        let context = ctx.data::<GraphQLContext>()?;
        let story = stories::Entity::find_by_id(self.story_id)
            .one(&context.db)
            .await?;

        Ok(story.map(Story::from))
    }

    #[graphql(name = "edgeCount")]
    async fn edge_count(&self) -> i32 {
        self.edge_order.len() as i32
    }
}

#[derive(InputObject)]
pub struct CreateSequenceInput {
    #[graphql(name = "storyId")]
    pub story_id: i32,
    pub name: String,
    pub description: Option<String>,
    #[graphql(name = "enabledDatasetIds")]
    pub enabled_dataset_ids: Option<Vec<i32>>,
    #[graphql(name = "edgeOrder")]
    pub edge_order: Option<Vec<SequenceEdgeRef>>,
}

#[derive(InputObject)]
pub struct UpdateSequenceInput {
    pub name: Option<String>,
    pub description: Option<String>,
    #[graphql(name = "enabledDatasetIds")]
    pub enabled_dataset_ids: Option<Vec<i32>>,
    #[graphql(name = "edgeOrder")]
    pub edge_order: Option<Vec<SequenceEdgeRef>>,
}

#[cfg(test)]
mod b1_tests {
    use super::*;

    #[test]
    fn persisted_json_parses_into_pipeline_type() {
        let refs = vec![SequenceEdgeRef {
            dataset_id: 5,
            edge_id: "e1".into(),
            note: Some("hi".into()),
            note_position: Some(NotePosition::Target),
        }];
        let json = edge_refs_to_persisted_json(refs).unwrap();
        // Canonical shape: camelCase keys + lowercase note position.
        assert!(json.contains("\"datasetId\":5"), "{json}");
        assert!(json.contains("\"edgeId\":\"e1\""), "{json}");
        assert!(json.contains("\"notePosition\":\"target\""), "{json}");
        // And it must round-trip into the pipeline consumer without loss.
        let parsed: Vec<layercake_core::sequence_types::SequenceEdgeRef> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].edge_id, "e1");
        assert!(matches!(
            parsed[0].note_position,
            Some(layercake_core::sequence_types::NotePosition::Target)
        ));
    }

    #[test]
    fn api_type_reads_back_what_pipeline_wrote() {
        // The GraphQL read path (From<Model>) parses edge_order with the API
        // type; it must accept the canonical camelCase shape.
        let json = r#"[{"datasetId":5,"edgeId":"e1","note":null,"notePosition":"source"}]"#;
        let parsed: Vec<SequenceEdgeRef> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed[0].dataset_id, 5);
        assert!(matches!(parsed[0].note_position, Some(NotePosition::Source)));
    }
}
