use async_graphql::*;
use sea_orm::EntityTrait;
use chrono::{DateTime, Utc};

use crate::database::entities::{graph_edits, graphs};
use crate::graphql::context::GraphQLContext;
use crate::graphql::types::graph::Graph;
use crate::graphql::types::scalars::JSON;

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct GraphEdit {
    pub id: i32,
    pub graph_id: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub field_name: Option<String>,
    pub old_value: Option<JSON>,
    pub new_value: Option<JSON>,
    pub sequence_number: i32,
    pub applied: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<i32>,
}

impl From<graph_edits::Model> for GraphEdit {
    fn from(model: graph_edits::Model) -> Self {
        Self {
            id: model.id,
            graph_id: model.graph_id,
            target_type: model.target_type,
            target_id: model.target_id,
            operation: model.operation,
            field_name: model.field_name,
            old_value: model.old_value,
            new_value: model.new_value,
            sequence_number: model.sequence_number,
            applied: model.applied,
            created_at: model.created_at,
            created_by: model.created_by,
        }
    }
}

#[ComplexObject]
impl GraphEdit {
    async fn graph(&self, ctx: &Context<'_>) -> Result<Option<Graph>> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph = graphs::Entity::find_by_id(self.graph_id)
            .one(&context.db)
            .await?;

        Ok(graph.map(Graph::from))
    }
}

#[derive(Clone, Debug, InputObject)]
pub struct CreateGraphEditInput {
    pub graph_id: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub field_name: Option<String>,
    pub old_value: Option<JSON>,
    pub new_value: Option<JSON>,
    pub created_by: Option<i32>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ReplaySummary {
    pub total: i32,
    pub applied: i32,
    pub skipped: i32,
    pub failed: i32,
    pub details: Vec<EditResult>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct EditResult {
    pub sequence_number: i32,
    pub target_type: String,
    pub target_id: String,
    pub operation: String,
    pub result: String,
    pub message: String,
}
