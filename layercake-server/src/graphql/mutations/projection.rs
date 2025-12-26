use std::collections::{HashMap, HashSet};

use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use layercake_core::database::entities::{graph_data_edges, projections, sequences, stories};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;

#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionStorySelectionInput {
    /// Story to verify
    pub story_id: i32,
    /// Sequence IDs to include (defaults to all sequences for the story)
    pub enabled_sequence_ids: Option<Vec<i32>>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionStoryMissingEdge {
    pub dataset_id: i32,
    pub edge_id: String,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionStorySequenceReport {
    pub story_id: i32,
    pub sequence_id: i32,
    pub missing_edges: Vec<ProjectionStoryMissingEdge>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ProjectionStoryVerificationResult {
    pub success: bool,
    /// Only sequences with missing edges are returned
    pub sequences: Vec<ProjectionStorySequenceReport>,
}

#[derive(Deserialize)]
struct SequenceEdgeRef {
    #[serde(rename = "datasetId")]
    dataset_id: Option<i32>,
    #[serde(rename = "edgeId")]
    edge_id: String,
}

fn parse_edge_order(raw: &str) -> Vec<SequenceEdgeRef> {
    serde_json::from_str(raw).unwrap_or_default()
}

#[derive(Default)]
pub struct ProjectionMutation;

#[Object]
impl ProjectionMutation {
    /// Verify that selected stories/sequences reference edges present in the projection's graph.
    async fn verify_projection_story_match(
        &self,
        ctx: &Context<'_>,
        projection_id: i32,
        stories: Vec<ProjectionStorySelectionInput>,
    ) -> Result<ProjectionStoryVerificationResult> {
        let context = ctx.data::<GraphQLContext>()?;

        let projection = projections::Entity::find_by_id(projection_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projections::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Projection", projection_id))?;

        if stories.is_empty() {
            return Ok(ProjectionStoryVerificationResult {
                success: true,
                sequences: Vec::new(),
            });
        }

        let story_ids: Vec<i32> = stories.iter().map(|s| s.story_id).collect();

        let story_models = stories::Entity::find()
            .filter(stories::Column::Id.is_in(story_ids.clone()))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("stories::Entity::find", e))?;

        if story_models.len() != story_ids.len() {
            return Err(StructuredError::not_found(
                "Story",
                stories
                    .iter()
                    .find(|s| !story_models.iter().any(|m| m.id == s.story_id))
                    .map(|s| s.story_id)
                    .unwrap_or_default(),
            )
            .into());
        }

        if story_models
            .iter()
            .any(|s| s.project_id != projection.project_id)
        {
            return Err(StructuredError::validation(
                "storyIds",
                "All stories must belong to the projection's project",
            )
            .into());
        }

        let sequence_models = sequences::Entity::find()
            .filter(sequences::Column::StoryId.is_in(story_ids.clone()))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("sequences::Entity::find", e))?;

        let mut sequences_by_story: HashMap<i32, Vec<sequences::Model>> = HashMap::new();
        for seq in sequence_models {
            sequences_by_story
                .entry(seq.story_id)
                .or_default()
                .push(seq);
        }

        let graph_edges = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(projection.graph_id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data_edges::Entity::find", e))?;

        let mut edge_set: HashSet<(Option<i32>, String)> = HashSet::new();
        for edge in graph_edges {
            edge_set.insert((edge.source_dataset_id, edge.external_id.clone()));
        }

        let mut missing_reports: Vec<ProjectionStorySequenceReport> = Vec::new();

        for selection in stories {
            let Some(seq_list) = sequences_by_story.get(&selection.story_id) else {
                continue;
            };
            let enabled_ids: Option<HashSet<i32>> = selection
                .enabled_sequence_ids
                .map(|ids| ids.into_iter().collect());

            for seq in seq_list {
                if let Some(enabled) = &enabled_ids {
                    if !enabled.contains(&seq.id) {
                        continue;
                    }
                }

                let mut missing_edges: Vec<ProjectionStoryMissingEdge> = Vec::new();
                for edge_ref in parse_edge_order(&seq.edge_order) {
                    let key = (edge_ref.dataset_id, edge_ref.edge_id.clone());
                    if !edge_set.contains(&key) {
                        missing_edges.push(ProjectionStoryMissingEdge {
                            dataset_id: edge_ref.dataset_id.unwrap_or_default(),
                            edge_id: edge_ref.edge_id,
                        });
                    }
                }

                if !missing_edges.is_empty() {
                    missing_reports.push(ProjectionStorySequenceReport {
                        story_id: selection.story_id,
                        sequence_id: seq.id,
                        missing_edges,
                    });
                }
            }
        }

        let success = missing_reports.is_empty();

        Ok(ProjectionStoryVerificationResult {
            success,
            sequences: missing_reports,
        })
    }
}
