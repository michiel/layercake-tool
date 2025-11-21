use async_graphql::*;
use base64::Engine;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use super::helpers::{
    generate_edge_id, generate_node_id_from_ids, get_extension_for_format,
    get_mime_type_for_format, parse_export_format, ExecutionActionResult, ExportNodeOutputResult,
    StoredGraphArtefactNodeConfig, StoredTreeArtefactNodeConfig,
};
use crate::database::entities::{
    datasets, graphs, plan_dag_edges, plan_dag_nodes, plans, projects, ExecutionState,
};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan_dag::{
    config::RenderConfig as GraphQLRenderConfig,
    config::LayerSourceStyle as GraphQLLayerSourceStyle,
    config::LayerSourceStyleOverride as GraphQLLayerSourceStyleOverride,
    config::NotePosition as GraphQLNotePosition,
    config::RenderTargetOptions as GraphQLRenderTargetOptions,
    config::Orientation as GraphQLOrientation,
    config::RenderBuiltinStyle as GraphQLRenderBuiltinStyle,
    config::GraphvizCommentStyle as GraphQLGraphvizCommentStyle,
    PlanDag, PlanDagEdge, PlanDagInput, PlanDagMigrationDetail, PlanDagMigrationResult, PlanDagNode,
};
use crate::pipeline::DagExecutor;
use crate::plan::{
    LayerSourceStyle as PlanLayerSourceStyle, LayerSourceStyleOverride as PlanLayerSourceStyleOverride,
    NotePosition as PlanNotePosition, RenderConfig as PlanRenderConfig, RenderConfigBuiltInStyle,
    RenderConfigOrientation, RenderTargetOptions, GraphvizRenderOptions, GraphvizCommentStyle,
};

#[derive(Default)]
pub struct PlanDagMutation;

fn default_artefact_render_config() -> PlanRenderConfig {
    PlanRenderConfig {
        contain_nodes: true,
        orientation: RenderConfigOrientation::TB,
        apply_layers: true,
        built_in_styles: RenderConfigBuiltInStyle::Light,
        target_options: RenderTargetOptions {
            graphviz: Some(GraphvizRenderOptions::default()),
            mermaid: None,
        },
        add_node_comments_as_notes: false,
        note_position: PlanNotePosition::Left,
        use_node_weight: true,
        use_edge_weight: true,
        layer_source_styles: Vec::new(),
    }
}

fn map_note_position(position: GraphQLNotePosition) -> PlanNotePosition {
    match position {
        GraphQLNotePosition::Right => PlanNotePosition::Right,
        GraphQLNotePosition::Top => PlanNotePosition::Top,
        GraphQLNotePosition::Bottom => PlanNotePosition::Bottom,
        GraphQLNotePosition::Left => PlanNotePosition::Left,
    }
}

/// Merge a (partial) GraphQL render config with defaults to the concrete plan RenderConfig
fn render_config_from_graphql_input(
    input: &GraphQLRenderConfig,
    defaults: &PlanRenderConfig,
) -> PlanRenderConfig {
    fn map_orientation(value: GraphQLOrientation) -> RenderConfigOrientation {
        match value {
            GraphQLOrientation::Lr => RenderConfigOrientation::LR,
            GraphQLOrientation::Tb => RenderConfigOrientation::TB,
        }
    }

    fn map_builtin_style(value: GraphQLRenderBuiltinStyle) -> RenderConfigBuiltInStyle {
        match value {
            GraphQLRenderBuiltinStyle::None => RenderConfigBuiltInStyle::None,
            GraphQLRenderBuiltinStyle::Light => RenderConfigBuiltInStyle::Light,
            GraphQLRenderBuiltinStyle::Dark => RenderConfigBuiltInStyle::Dark,
        }
    }

    fn map_graphviz_comment_style(value: GraphQLGraphvizCommentStyle) -> GraphvizCommentStyle {
        match value {
            GraphQLGraphvizCommentStyle::Tooltip => GraphvizCommentStyle::Tooltip,
            GraphQLGraphvizCommentStyle::Label => GraphvizCommentStyle::Label,
        }
    }

    fn map_layer_source_style(value: GraphQLLayerSourceStyle) -> PlanLayerSourceStyle {
        match value {
            GraphQLLayerSourceStyle::Default => PlanLayerSourceStyle::Default,
            GraphQLLayerSourceStyle::Light => PlanLayerSourceStyle::Light,
            GraphQLLayerSourceStyle::Dark => PlanLayerSourceStyle::Dark,
        }
    }

    fn map_layer_source_styles(
        input: Option<&Vec<GraphQLLayerSourceStyleOverride>>,
        defaults: &[PlanLayerSourceStyleOverride],
    ) -> Vec<PlanLayerSourceStyleOverride> {
        input
            .map(|items| {
                items
                    .iter()
                    .map(|item| PlanLayerSourceStyleOverride {
                        source_dataset_id: item.source_dataset_id,
                        mode: map_layer_source_style(item.mode),
                    })
                    .collect()
            })
            .unwrap_or_else(|| defaults.to_vec())
    }

    fn map_target_options(
        input: &GraphQLRenderTargetOptions,
        defaults: &RenderTargetOptions,
    ) -> RenderTargetOptions {
        let mut opts = defaults.clone();
        if let Some(gv) = &input.graphviz {
            if opts.graphviz.is_none() {
                opts.graphviz = Some(GraphvizRenderOptions::default());
            }
            if let Some(ref mut graphviz_opts) = opts.graphviz {
                if let Some(layout) = gv.layout {
                    graphviz_opts.layout = match layout {
                        crate::graphql::types::plan_dag::config::GraphvizLayout::Neato => {
                            crate::plan::GraphvizLayout::Neato
                        }
                        crate::graphql::types::plan_dag::config::GraphvizLayout::Fdp => {
                            crate::plan::GraphvizLayout::Fdp
                        }
                        crate::graphql::types::plan_dag::config::GraphvizLayout::Circo => {
                            crate::plan::GraphvizLayout::Circo
                        }
                        _ => crate::plan::GraphvizLayout::Dot,
                    };
                }
                if let Some(overlap) = gv.overlap {
                    graphviz_opts.overlap = overlap;
                }
                if let Some(splines) = gv.splines {
                    graphviz_opts.splines = splines;
                }
                if let Some(nodesep) = gv.nodesep {
                    graphviz_opts.nodesep = nodesep;
                }
                if let Some(ranksep) = gv.ranksep {
                    graphviz_opts.ranksep = ranksep;
                }
                if let Some(style) = gv.comment_style {
                    graphviz_opts.comment_style = map_graphviz_comment_style(style);
                }
            }
        }
        if let Some(mermaid) = &input.mermaid {
            if opts.mermaid.is_none() {
                opts.mermaid = Some(crate::plan::MermaidRenderOptions::default());
            }
            if let Some(ref mut mermaid_opts) = opts.mermaid {
                if let Some(look) = mermaid.look {
                    mermaid_opts.look = match look {
                        crate::graphql::types::plan_dag::config::MermaidLook::HandDrawn => {
                            crate::plan::MermaidLook::HandDrawn
                        }
                        _ => crate::plan::MermaidLook::Default,
                    };
                }
                if let Some(display) = mermaid.display {
                    mermaid_opts.display = match display {
                        crate::graphql::types::plan_dag::config::MermaidDisplay::Compact => {
                            crate::plan::MermaidDisplay::Compact
                        }
                        _ => crate::plan::MermaidDisplay::Full,
                    };
                }
            }
        }
        opts
    }

    PlanRenderConfig {
        contain_nodes: input.contain_nodes.unwrap_or(defaults.contain_nodes),
        orientation: input
            .orientation
            .map(map_orientation)
            .unwrap_or(defaults.orientation),
        apply_layers: input.apply_layers.unwrap_or(defaults.apply_layers),
        built_in_styles: input
            .built_in_styles
            .map(map_builtin_style)
            .unwrap_or(defaults.built_in_styles),
        target_options: input
            .target_options
            .as_ref()
            .map(|options| map_target_options(options, &defaults.target_options))
            .unwrap_or_else(|| defaults.target_options.clone()),
        add_node_comments_as_notes: input
            .add_node_comments_as_notes
            .unwrap_or(defaults.add_node_comments_as_notes),
        note_position: input
            .note_position
            .map(map_note_position)
            .unwrap_or(defaults.note_position),
        use_node_weight: input.use_node_weight.unwrap_or(defaults.use_node_weight),
        use_edge_weight: input.use_edge_weight.unwrap_or(defaults.use_edge_weight),
        layer_source_styles: map_layer_source_styles(
            input.layer_source_styles.as_ref(),
            &defaults.layer_source_styles,
        ),
    }
}

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
                crate::graphql::types::PlanDagNodeType::DataSet => "DataSetNode",
                crate::graphql::types::PlanDagNodeType::Graph => "GraphNode",
                crate::graphql::types::PlanDagNodeType::Transform => "TransformNode",
                crate::graphql::types::PlanDagNodeType::Filter => "FilterNode",
                crate::graphql::types::PlanDagNodeType::Merge => "MergeNode",
                crate::graphql::types::PlanDagNodeType::GraphArtefact => "GraphArtefactNode",
                crate::graphql::types::PlanDagNodeType::TreeArtefact => "TreeArtefactNode",
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

    /// Validate and migrate legacy plan DAG items (e.g., OutputNode -> GraphArtefactNode).
    async fn validate_and_migrate_plan_dag(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
    ) -> Result<PlanDagMigrationResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let outcome = context
            .plan_dag_service
            .validate_and_migrate_legacy_nodes(project_id)
            .await
            .map_err(|e| {
                StructuredError::service("PlanDagService::validate_and_migrate_legacy_nodes", e)
            })?;

        Ok(PlanDagMigrationResult {
            checked_nodes: outcome.checked_nodes as i32,
            updated_nodes: outcome
                .migrated_nodes
                .into_iter()
                .map(|detail| PlanDagMigrationDetail {
                    node_id: detail.node_id,
                    from_type: detail.from_type,
                    to_type: detail.to_type,
                    note: detail.note,
                })
                .collect(),
            warnings: outcome.warnings,
            errors: outcome.errors,
        })
    }

    /// Export a node's output (graph export to various formats)
    async fn export_node_output(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        node_id: String,
        #[graphql(name = "renderConfigOverride")] render_config_override: Option<GraphQLRenderConfig>,
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

        // Get the artefact node
        let artefact_node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&context.db)
            .await
            .map_err(|e| {
                StructuredError::database("plan_dag_nodes::Entity::find (artefact node)", e)
            })?
            .ok_or_else(|| StructuredError::not_found("Artefact node", &node_id))?;

        let node_type = artefact_node.node_type.clone();
        let (render_target, output_path, stored_render_config) = match node_type.as_str() {
            "TreeArtefactNode" | "TreeArtefact" => {
                let stored_config: StoredTreeArtefactNodeConfig =
                    serde_json::from_str(&artefact_node.config_json).map_err(|e| {
                        StructuredError::bad_request(format!(
                            "Failed to parse tree artefact config: {}",
                            e
                        ))
                    })?;
                (
                    stored_config
                        .render_target
                        .unwrap_or_else(|| "PlantUmlMindmap".to_string()),
                    stored_config.output_path,
                    stored_config.render_config.map(|rc| rc.into_render_config()),
                )
            }
            _ => {
                let stored_config: StoredGraphArtefactNodeConfig =
                    serde_json::from_str(&artefact_node.config_json).map_err(|e| {
                        StructuredError::bad_request(format!(
                            "Failed to parse graph artefact config: {}",
                            e
                        ))
                    })?;
                (
                    stored_config
                        .render_target
                        .unwrap_or_else(|| "GML".to_string()),
                    stored_config.output_path,
                    stored_config.render_config.map(|rc| rc.into_render_config()),
                )
            }
        };

        // Get project name for default filename
        let project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

        // Generate filename
        let extension = get_extension_for_format(render_target.as_str());
        let filename = output_path.unwrap_or_else(|| format!("{}.{}", project.name, extension));

        // Merge render config overrides: GraphQL input (if provided) takes priority,
        // then stored config, then defaults.
        let defaults = default_artefact_render_config();
        let gql_render_config = render_config_override
            .as_ref()
            .map(|rc| render_config_from_graphql_input(rc, &defaults));
        let render_config = gql_render_config
            .or(stored_render_config)
            .unwrap_or(defaults);

        // Find the upstream GraphNode connected to this artefact node
        let edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_edges::Entity::find", e))?;

        let upstream_node_id = edges
            .iter()
            .find(|e| e.target_node_id == node_id)
            .map(|e| e.source_node_id.clone())
            .ok_or_else(|| StructuredError::not_found("Upstream graph", "artefact node"))?;

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
                Some(render_config.clone()),
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

    /// Clear execution state for all nodes in a project (keeps edits, config, and datasets)
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

        // Delete all datasets imported for this project
        let datasets_result = datasets::Entity::delete_many()
            .filter(datasets::Column::ProjectId.eq(project_id))
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("datasets::Entity::delete_many", e))?;

        let datasets_deleted = datasets_result.rows_affected;

        Ok(ExecutionActionResult {
        success: true,
        message: format!(
            "Cleared execution state: deleted {} graphs and {} datasets. Configuration and edits preserved.",
            graphs_deleted, datasets_deleted
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

        // Find all datasets in processing or pending state for this project
        let processing_datasets = datasets::Entity::find()
            .filter(datasets::Column::ProjectId.eq(project_id))
            .filter(
                datasets::Column::ExecutionState
                    .eq(ExecutionState::Processing.as_str())
                    .or(datasets::Column::ExecutionState.eq(ExecutionState::Pending.as_str())),
            )
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("datasets::Entity::find", e))?;

        // Update each dataset to not_started state
        for dataset in processing_datasets {
            let mut active: datasets::ActiveModel = dataset.into();
            active.execution_state = Set(ExecutionState::NotStarted.as_str().to_string());
            active.error_message = Set(Some("Execution stopped by user".to_string()));
            active
                .update(&context.db)
                .await
                .map_err(|e| StructuredError::database("datasets::Entity::update", e))?;
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
