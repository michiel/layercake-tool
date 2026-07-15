use async_graphql::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use super::helpers::NodeExecutionResult;
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan_dag::{
    NodePositionInput, PlanDagNode, PlanDagNodeInput, PlanDagNodeUpdateInput, Position,
};
use layercake_core::app_context::{
    PlanDagNodePositionRequest, PlanDagNodeRequest, PlanDagNodeUpdateRequest,
};
use layercake_core::database::entities::{plan_dag_edges, plan_dag_nodes, plans};
use layercake_core::pipeline::DagExecutor;
use serde_json::Value as JsonValue;

#[derive(Default)]
pub struct PlanDagNodesMutation;

#[Object]
impl PlanDagNodesMutation {
    /// Add a single Plan DAG node
    async fn add_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        node: PlanDagNodeInput,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let PlanDagNodeInput {
            node_type,
            position,
            metadata,
            config,
            ..
        } = node;

        let metadata_value = serde_json::to_value(metadata)
            .map_err(|e| StructuredError::bad_request(format!("Invalid node metadata: {}", e)))?;
        let config_value = serde_json::from_str::<JsonValue>(&config).map_err(|e| {
            StructuredError::bad_request(format!("Invalid node configuration JSON: {}", e))
        })?;

        let request = PlanDagNodeRequest {
            node_type: node_type.into(),
            position: position.into(),
            metadata: metadata_value,
            config: config_value,
        };

        let created = context
            .app
            .create_plan_dag_node(&actor, project_id, plan_id, request)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Some(PlanDagNode::from(created)))
    }

    /// Update a Plan DAG node
    async fn update_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        updates: PlanDagNodeUpdateInput,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let PlanDagNodeUpdateInput {
            position,
            metadata,
            config,
        } = updates;

        let metadata_value = if let Some(metadata) = metadata {
            Some(serde_json::to_value(metadata).map_err(|e| {
                StructuredError::bad_request(format!("Invalid node metadata: {}", e))
            })?)
        } else {
            None
        };

        let config_value = if let Some(config) = config {
            let value = serde_json::from_str::<JsonValue>(&config).map_err(|e| {
                StructuredError::bad_request(format!("Invalid node configuration JSON: {}", e))
            })?;
            validate_render_target_extension(&value)?;
            Some(value)
        } else {
            None
        };

        let request = PlanDagNodeUpdateRequest {
            position: position.map(Into::into),
            metadata: metadata_value,
            config: config_value,
        };

        let updated = context
            .app
            .update_plan_dag_node(&actor, project_id, plan_id, node_id, request)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Some(PlanDagNode::from(updated)))
    }

    /// Rename a Plan DAG node to a new (human-readable) id. Rewrites every
    /// reference: connected edges and any computed graph's origin. Fails if the
    /// new id already exists in the plan.
    #[graphql(name = "renamePlanDagNode")]
    async fn rename_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        #[graphql(name = "oldId")] old_id: String,
        #[graphql(name = "newId")] new_id: String,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let renamed = context
            .app
            .rename_plan_dag_node(&actor, project_id, plan_id, old_id, new_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;
        Ok(Some(PlanDagNode::from(renamed)))
    }

    /// Delete a Plan DAG node
    async fn delete_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let deleted = context
            .app
            .delete_plan_dag_node(&actor, project_id, plan_id, node_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Some(PlanDagNode::from(deleted)))
    }

    /// Move a Plan DAG node (update position)
    async fn move_plan_dag_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        position: Position,
    ) -> Result<Option<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let moved = context
            .app
            .move_plan_dag_node(&actor, project_id, plan_id, node_id, position.into())
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(Some(PlanDagNode::from(moved)))
    }

    /// Batch move multiple nodes at once (optimized for layout operations)
    async fn batch_move_plan_dag_nodes(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        node_positions: Vec<NodePositionInput>,
    ) -> Result<Vec<PlanDagNode>> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;

        let requests = node_positions
            .into_iter()
            .map(|node_pos| PlanDagNodePositionRequest {
                node_id: node_pos.node_id,
                position: node_pos.position.into(),
                source_position: node_pos.source_position,
                target_position: node_pos.target_position,
            })
            .collect();

        let moved = context
            .app
            .batch_move_plan_dag_nodes(&actor, project_id, plan_id, requests)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        Ok(moved.into_iter().map(PlanDagNode::from).collect())
    }

    /// Execute a DAG node (builds graph from upstream data sources)
    async fn execute_node(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
    ) -> Result<NodeExecutionResult> {
        let context = ctx.data::<GraphQLContext>()?;

        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .authorize_project_write_access(&actor, project_id)
            .await
            .map_err(crate::graphql::errors::core_error_to_graphql_error)?;

        // Find the plan for this project
        let plan = if let Some(plan_id) = plan_id {
            let plan = plans::Entity::find_by_id(plan_id)
                .one(&context.db)
                .await
                .map_err(|e| StructuredError::database("plans::Entity::find_by_id", e))?
                .ok_or_else(|| StructuredError::not_found("Plan", plan_id))?;
            if plan.project_id != project_id {
                return Err(StructuredError::bad_request(format!(
                    "Plan {} does not belong to project {}",
                    plan_id, project_id
                )));
            }
            plan
        } else {
            plans::Entity::find()
                .filter(plans::Column::ProjectId.eq(project_id))
                .order_by_desc(plans::Column::UpdatedAt)
                .one(&context.db)
                .await
                .map_err(|e| StructuredError::database("plans::Entity::find (ProjectId)", e))?
                .ok_or_else(|| StructuredError::not_found("Plan for project", project_id))?
        };

        // Get all nodes in the plan
        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_nodes::Entity::find", e))?;

        // Get all edges in the plan
        let edges_models = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_edges::Entity::find", e))?;

        // Convert edges to (source, target) tuples
        let edges: Vec<(String, String)> = edges_models
            .iter()
            .map(|e| (e.source_node_id.clone(), e.target_node_id.clone()))
            .collect();

        // Create executor and execute the node with all its upstream dependencies
        let executor = DagExecutor::new(context.db.clone());

        executor
            .execute_with_dependencies(project_id, plan.id, &node_id, &nodes, &edges)
            .await
            .map_err(|e| StructuredError::service("DagExecutor::execute_with_dependencies", e))?;

        executor
            .execute_affected_nodes(project_id, plan.id, &node_id, &nodes, &edges)
            .await
            .map_err(|e| StructuredError::service("DagExecutor::execute_affected_nodes", e))?;

        let warnings = executor.take_warnings();
        Ok(NodeExecutionResult {
            success: true,
            message: format!(
                "Node {} executed successfully; downstream graphs refreshed",
                node_id
            ),
            node_id,
            warnings,
        })
    }
}

/// If a node config is a sequence-artefact config (`renderTarget` +
/// `outputPath`), reject a `renderTarget` that doesn't match the output file
/// extension. This catches the silent drift where a UI edit flips
/// MermaidSequence→PlantUmlSequence while `outputPath` stays `.mmd` (so a
/// `.mmd` file ends up containing PlantUML).
fn validate_render_target_extension(config: &JsonValue) -> Result<()> {
    let (Some(target), Some(path)) = (
        config.get("renderTarget").and_then(|v| v.as_str()),
        config.get("outputPath").and_then(|v| v.as_str()),
    ) else {
        return Ok(());
    };

    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (allowed, kind): (&[&str], &str) = match target {
        "MermaidSequence" => (&["mmd", "md"], "MermaidSequence"),
        "PlantUmlSequence" => (&["puml", "txt"], "PlantUmlSequence"),
        _ => return Ok(()), // unknown/other target → don't constrain
    };

    if !allowed.contains(&ext.as_str()) {
        return Err(StructuredError::validation(
            "outputPath",
            format!(
                "renderTarget {} expects an output extension of {} but outputPath is '{}'. \
                 Update the extension or the renderTarget so they match.",
                kind,
                allowed
                    .iter()
                    .map(|e| format!(".{}", e))
                    .collect::<Vec<_>>()
                    .join(" or "),
                path
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod render_target_validation_tests {
    use super::validate_render_target_extension;
    use serde_json::json;

    #[test]
    fn rejects_mismatched_extension() {
        // MermaidSequence writing to .puml → reject.
        let bad = json!({"renderTarget":"MermaidSequence","outputPath":"scenario.puml"});
        assert!(validate_render_target_extension(&bad).is_err());
        // PlantUmlSequence writing to .mmd → reject (the reviewer's exact drift).
        let bad2 = json!({"renderTarget":"PlantUmlSequence","outputPath":"scenario.mmd"});
        assert!(validate_render_target_extension(&bad2).is_err());
    }

    #[test]
    fn accepts_matching_extension_and_ignores_non_sequence_configs() {
        assert!(validate_render_target_extension(
            &json!({"renderTarget":"MermaidSequence","outputPath":"s.mmd"})
        )
        .is_ok());
        assert!(validate_render_target_extension(
            &json!({"renderTarget":"PlantUmlSequence","outputPath":"s.puml"})
        )
        .is_ok());
        // A config without renderTarget/outputPath (e.g. a DataSetNode) is untouched.
        assert!(validate_render_target_extension(&json!({"dataSetId":5})).is_ok());
    }
}
