use async_graphql::*;
use base64::Engine;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::database::entities::{datasources, graphs, plan_dag_edges, plan_dag_nodes, plans, projects, ExecutionState};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan_dag::{PlanDag, PlanDagEdge, PlanDagInput, PlanDagNode};
use crate::pipeline::DagExecutor;
use super::helpers::{generate_edge_id, generate_node_id_from_ids, parse_export_format, get_extension_for_format, get_mime_type_for_format, ExportNodeOutputResult, ExecutionActionResult, StoredOutputNodeConfig};

#[derive(Default)]
pub struct PlanDagMutation;

#[Object]
impl PlanDagMutation {
async fn update_plan_dag(
    &self,
    ctx: &Context<'_>,
    project_id: i32,
    plan_dag: PlanDagInput,
) -> Result<Option<PlanDag>> {
    let context = ctx.data::<GraphQLContext>()?;

    // Verify project exists
    let _project = projects::Entity::find_by_id(project_id)
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
        .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

    // Clear existing Plan DAG nodes and edges for this project
    plan_dag_nodes::Entity::delete_many()
        .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
        .exec(&context.db)
        .await?;

    plan_dag_edges::Entity::delete_many()
        .filter(plan_dag_edges::Column::PlanId.eq(project_id))
        .exec(&context.db)
        .await?;

    // Collect existing node IDs for ID generation
    let mut existing_node_ids: Vec<String> = Vec::new();

    // Insert new Plan DAG nodes
    for node in &plan_dag.nodes {
        // Generate ID if not provided
        let node_id = node.id.clone().unwrap_or_else(|| {
            let id_refs: Vec<&str> = existing_node_ids.iter().map(|s| s.as_str()).collect();
            generate_node_id_from_ids(&node.node_type, &id_refs)
        });
        existing_node_ids.push(node_id.clone());

        let node_type_str = match node.node_type {
            crate::graphql::types::PlanDagNodeType::DataSource => "DataSourceNode",
            crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
            crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
            crate::graphql::types::PlanDagNodeType::Filter => "FilterNode",
            crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
            crate::graphql::types::PlanDagNodeType::Output => "OutputNode",
        };

        let metadata_json = serde_json::to_string(&node.metadata)?;

        let dag_node = plan_dag_nodes::ActiveModel {
            id: Set(node_id),
            plan_id: Set(project_id), // Use project_id directly
            node_type: Set(node_type_str.to_string()),
            position_x: Set(node.position.x),
            position_y: Set(node.position.y),
            source_position: Set(None),
            target_position: Set(None),
            metadata_json: Set(metadata_json),
            config_json: Set(node.config.clone()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        dag_node.insert(&context.db).await?;
    }

    // Insert new Plan DAG edges
    for edge in &plan_dag.edges {
        // Generate ID if not provided
        let edge_id = edge
            .id
            .clone()
            .unwrap_or_else(|| generate_edge_id(&edge.source, &edge.target));

        let metadata_json = serde_json::to_string(&edge.metadata)?;

        let dag_edge = plan_dag_edges::ActiveModel {
            id: Set(edge_id),
            plan_id: Set(project_id), // Use project_id directly
            source_node_id: Set(edge.source.clone()),
            target_node_id: Set(edge.target.clone()),
            // Removed source_handle and target_handle for floating edges
            metadata_json: Set(metadata_json),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        dag_edge.insert(&context.db).await?;
    }

    // Return the updated Plan DAG
    let dag_nodes = plan_dag_nodes::Entity::find()
        .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
        .all(&context.db)
        .await?;

    let dag_edges = plan_dag_edges::Entity::find()
        .filter(plan_dag_edges::Column::PlanId.eq(project_id))
        .all(&context.db)
        .await?;

    let nodes: Vec<PlanDagNode> = dag_nodes.into_iter().map(PlanDagNode::from).collect();
    let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();

    Ok(Some(PlanDag {
        version: plan_dag.version,
        nodes,
        edges,
        metadata: plan_dag.metadata,
    }))
}

/// Export a node's output (graph export to various formats)
async fn export_node_output(
    &self,
    ctx: &Context<'_>,
    project_id: i32,
    node_id: String,
    preview_rows: Option<i32>,
) -> Result<ExportNodeOutputResult> {
    let context = ctx.data::<GraphQLContext>()?;

    // Find the plan for this project
    let plan = plans::Entity::find()
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("plans::Entity::find (ProjectId)", e))?
        .ok_or_else(|| StructuredError::not_found("Plan for project", project_id))?;

    // Get the output node
    let output_node = plan_dag_nodes::Entity::find()
        .filter(
            plan_dag_nodes::Column::PlanId
                .eq(plan.id)
                .and(plan_dag_nodes::Column::Id.eq(&node_id)),
        )
        .one(&context.db)
        .await
        .map_err(|e| {
            StructuredError::database("plan_dag_nodes::Entity::find (output node)", e)
        })?
        .ok_or_else(|| StructuredError::not_found("Output node", &node_id))?;

    // Parse node config to get renderTarget/outputPath/renderConfig overrides
    let stored_config: StoredOutputNodeConfig = serde_json::from_str(&output_node.config_json)
        .map_err(|e| {
            StructuredError::bad_request(format!("Failed to parse node config: {}", e))
        })?;

    let render_config_override = stored_config
        .render_config
        .map(|rc| rc.into_render_config());
    let render_target = stored_config
        .render_target
        .unwrap_or_else(|| "GML".to_string());
    let output_path = stored_config.output_path;

    // Get project name for default filename
    let project = projects::Entity::find_by_id(project_id)
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
        .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

    // Generate filename
    let extension = get_extension_for_format(render_target.as_str());
    let filename = output_path.unwrap_or_else(|| format!("{}.{}", project.name, extension));

    // Find the upstream GraphNode connected to this OutputNode
    let edges = plan_dag_edges::Entity::find()
        .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
        .all(&context.db)
        .await
        .map_err(|e| StructuredError::database("plan_dag_edges::Entity::find", e))?;

    let upstream_node_id = edges
        .iter()
        .find(|e| e.target_node_id == node_id)
        .map(|e| e.source_node_id.clone())
        .ok_or_else(|| StructuredError::not_found("Upstream graph", "output node"))?;

    // Get all nodes and edges for DAG execution
    let all_nodes = plan_dag_nodes::Entity::find()
        .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
        .all(&context.db)
        .await
        .map_err(|e| StructuredError::database("plan_dag_nodes::Entity::find", e))?;

    let edge_tuples: Vec<(String, String)> = edges
        .iter()
        .map(|e| (e.source_node_id.clone(), e.target_node_id.clone()))
        .collect();

    // Execute the upstream GraphNode and its dependencies to ensure graph is built
    let executor = DagExecutor::new(context.db.clone());
    executor
        .execute_with_dependencies(
            project_id,
            plan.id,
            &upstream_node_id,
            &all_nodes,
            &edge_tuples,
        )
        .await
        .map_err(|e| StructuredError::service("DagExecutor::execute_with_dependencies", e))?;

    // Get the built graph from the graphs table using the GraphNode's ID
    use crate::database::entities::graphs;
    let graph_model = graphs::Entity::find()
        .filter(graphs::Column::ProjectId.eq(project_id))
        .filter(graphs::Column::NodeId.eq(&upstream_node_id))
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("graphs::Entity::find (node export)", e))?
        .ok_or_else(|| {
            StructuredError::not_found("Graph for node", upstream_node_id.clone())
        })?;

    let export_format = parse_export_format(render_target.as_str())?;

    let preview_limit = preview_rows.and_then(|value| {
        if value > 0 {
            Some(value as usize)
        } else {
            None
        }
    });

    let content = context
        .app
        .preview_graph_export(
            graph_model.id,
            export_format,
            render_config_override,
            preview_limit,
        )
        .await
        .map_err(|e| StructuredError::service("AppContext::preview_graph_export", e))?;

    // Encode as base64
    use base64::Engine;
    let encoded_content = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());

    // Get MIME type
    let mime_type = get_mime_type_for_format(render_target.as_str());

    Ok(ExportNodeOutputResult {
        success: true,
        message: format!("Successfully exported {} as {}", filename, render_target),
        content: encoded_content,
        filename,
        mime_type,
    })
}

/// Clear execution state for all nodes in a project (keeps edits, config, and datasources)
async fn clear_project_execution(
    &self,
    ctx: &Context<'_>,
    project_id: i32,
) -> Result<ExecutionActionResult> {
    let context = ctx.data::<GraphQLContext>()?;

    // Verify project exists
    let _project = projects::Entity::find_by_id(project_id)
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
        .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

    // Delete all graphs generated by this project's plan nodes
    // This will cascade delete graph_nodes, graph_edges, and graph_layers
    let delete_result = graphs::Entity::delete_many()
        .filter(graphs::Column::ProjectId.eq(project_id))
        .exec(&context.db)
        .await
        .map_err(|e| StructuredError::database("graphs::Entity::delete_many", e))?;

    let graphs_deleted = delete_result.rows_affected;

    // Delete all datasources imported for this project
    let datasources_result = datasources::Entity::delete_many()
        .filter(datasources::Column::ProjectId.eq(project_id))
        .exec(&context.db)
        .await
        .map_err(|e| StructuredError::database("datasources::Entity::delete_many", e))?;

    let datasources_deleted = datasources_result.rows_affected;

    Ok(ExecutionActionResult {
        success: true,
        message: format!(
            "Cleared execution state: deleted {} graphs and {} datasources. Configuration and edits preserved.",
            graphs_deleted, datasources_deleted
        ),
    })
}

/// Stop plan execution in progress
async fn stop_plan_execution(
    &self,
    ctx: &Context<'_>,
    project_id: i32,
) -> Result<ExecutionActionResult> {
    let context = ctx.data::<GraphQLContext>()?;

    // Verify project exists
    let _project = projects::Entity::find_by_id(project_id)
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
        .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

    // Find all graphs in processing or pending state for this project
    let processing_graphs = graphs::Entity::find()
        .filter(graphs::Column::ProjectId.eq(project_id))
        .filter(
            graphs::Column::ExecutionState
                .eq(ExecutionState::Processing.as_str())
                .or(graphs::Column::ExecutionState.eq(ExecutionState::Pending.as_str())),
        )
        .all(&context.db)
        .await
        .map_err(|e| StructuredError::database("graphs::Entity::find (stop execution)", e))?;

    let mut stopped_count = 0;

    // Update each graph to not_started state
    for graph in processing_graphs {
        let mut active: graphs::ActiveModel = graph.into();
        active = active.set_state(ExecutionState::NotStarted);
        active.error_message = Set(Some("Execution stopped by user".to_string()));
        active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("graphs::Entity::update", e))?;
        stopped_count += 1;
    }

    // Find all datasources in processing or pending state for this project
    let processing_datasources = datasources::Entity::find()
        .filter(datasources::Column::ProjectId.eq(project_id))
        .filter(
            datasources::Column::ExecutionState
                .eq(ExecutionState::Processing.as_str())
                .or(datasources::Column::ExecutionState.eq(ExecutionState::Pending.as_str())),
        )
        .all(&context.db)
        .await
        .map_err(|e| StructuredError::database("datasources::Entity::find", e))?;

    // Update each datasource to not_started state
    for datasource in processing_datasources {
        let mut active: datasources::ActiveModel = datasource.into();
        active.execution_state = Set(ExecutionState::NotStarted.as_str().to_string());
        active.error_message = Set(Some("Execution stopped by user".to_string()));
        active
            .update(&context.db)
            .await
            .map_err(|e| StructuredError::database("datasources::Entity::update", e))?;
        stopped_count += 1;
    }

    let message = if stopped_count > 0 {
        format!("Stopped {} running executions", stopped_count)
    } else {
        "No executions were in progress".to_string()
    };

    Ok(ExecutionActionResult {
        success: true,
        message,
    })
}
}
