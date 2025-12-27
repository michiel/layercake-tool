use async_graphql::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use layercake_core::database::entities::sequences;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{CreateSequenceInput, Sequence, UpdateSequenceInput};

#[derive(Default)]
pub struct SequenceMutation;

#[Object]
impl SequenceMutation {
    /// Create a new sequence
    async fn create_sequence(
        &self,
        ctx: &Context<'_>,
        input: CreateSequenceInput,
    ) -> Result<Sequence> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        if actor.user_id.is_none() {
            return Err(StructuredError::unauthorized("User is not authenticated"));
        }
        let now = Utc::now();

        let enabled_dataset_ids_json =
            serde_json::to_string(&input.enabled_dataset_ids.unwrap_or_default()).map_err(|e| {
                StructuredError::bad_request(format!(
                    "Failed to serialize enabled_dataset_ids: {}",
                    e
                ))
            })?;

        let edge_order_json = serde_json::to_string(&input.edge_order.unwrap_or_default())
            .map_err(|e| {
                StructuredError::bad_request(format!("Failed to serialize edge_order: {}", e))
            })?;

        let sequence = sequences::ActiveModel {
            story_id: Set(input.story_id),
            name: Set(input.name),
            description: Set(input.description),
            enabled_dataset_ids: Set(enabled_dataset_ids_json),
            edge_order: Set(edge_order_json),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let model = sequence
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("sequences::ActiveModel::insert", e))?;

        Ok(Sequence::from(model))
    }

    /// Update an existing sequence
    async fn update_sequence(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateSequenceInput,
    ) -> Result<Sequence> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        if actor.user_id.is_none() {
            return Err(StructuredError::unauthorized("User is not authenticated"));
        }

        let existing = sequences::Entity::find_by_id(id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("sequences::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Sequence", id))?;

        let mut sequence: sequences::ActiveModel = existing.into();
        sequence.updated_at = Set(Utc::now());

        if let Some(name) = input.name {
            sequence.name = Set(name);
        }

        if let Some(description) = input.description {
            sequence.description = Set(Some(description));
        }

        if let Some(enabled_dataset_ids) = input.enabled_dataset_ids {
            let json = serde_json::to_string(&enabled_dataset_ids).map_err(|e| {
                StructuredError::bad_request(format!(
                    "Failed to serialize enabled_dataset_ids: {}",
                    e
                ))
            })?;
            sequence.enabled_dataset_ids = Set(json);
        }

        if let Some(edge_order) = input.edge_order {
            let json = serde_json::to_string(&edge_order).map_err(|e| {
                StructuredError::bad_request(format!("Failed to serialize edge_order: {}", e))
            })?;
            sequence.edge_order = Set(json);
        }

        let model = sequence
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("sequences::ActiveModel::update", e))?;

        Ok(Sequence::from(model))
    }

    /// Delete a sequence
    async fn delete_sequence(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        if actor.user_id.is_none() {
            return Err(StructuredError::unauthorized("User is not authenticated"));
        }

        let result = sequences::Entity::delete_by_id(id)
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("sequences::Entity::delete_by_id", e))?;

        Ok(result.rows_affected > 0)
    }
}
