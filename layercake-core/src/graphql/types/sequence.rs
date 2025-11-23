use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::database::entities::{sequences, stories};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::Story;

/// Note position for sequence edge annotations
#[derive(Clone, Debug, Serialize, Deserialize, Enum, Copy, PartialEq, Eq)]
#[graphql(name = "SequenceNotePosition", rename_items = "PascalCase")]
pub enum NotePosition {
    Source,
    Target,
    Both,
}

/// Reference to an edge in a dataset
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, InputObject)]
#[graphql(input_name = "SequenceEdgeRefInput")]
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
