use anyhow::anyhow;
use async_graphql::*;
use base64::Engine;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use super::helpers::{
    generate_edge_id, generate_node_id_from_ids, get_extension_for_format,
    get_mime_type_for_format, parse_export_format, ExecutionActionResult, ExportNodeOutputResult,
    StoredGraphArtefactNodeConfig, StoredSequenceArtefactNodeConfig, StoredSequenceRenderConfig,
    StoredTreeArtefactNodeConfig,
};
use layercake_core::database::entities::graph_data;
use layercake_core::database::entities::{
    datasets, graph_data as graph_data_model, graph_data_edges, graph_data_nodes, plan_dag_edges,
    plan_dag_nodes, plans, projects, ExecutionState,
};
use layercake_core::export::{
    sequence_renderer::SequenceRenderConfigResolved, to_mermaid_sequence, to_plantuml_sequence,
};
use layercake_core::graph::{Edge, Graph, Layer, Node};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::plan_dag::{
    config::GraphvizCommentStyle as GraphQLGraphvizCommentStyle,
    config::LayerSourceStyle as GraphQLLayerSourceStyle,
    config::LayerSourceStyleOverride as GraphQLLayerSourceStyleOverride,
    config::NotePosition as GraphQLNotePosition, config::Orientation as GraphQLOrientation,
    config::RenderBuiltinStyle as GraphQLRenderBuiltinStyle,
    config::RenderConfig as GraphQLRenderConfig,
    config::RenderTargetOptions as GraphQLRenderTargetOptions,
    config::SequenceArtefactRenderTarget, config::StoryNodeConfig, PlanDag, PlanDagEdge,
    PlanDagInput, PlanDagMigrationDetail, PlanDagMigrationResult, PlanDagNode,
};
use layercake_core::pipeline::DagExecutor;
use layercake_core::plan::{
    ExportFileType, GraphvizCommentStyle, GraphvizRenderOptions,
    LayerSourceStyle as PlanLayerSourceStyle,
    LayerSourceStyleOverride as PlanLayerSourceStyleOverride, NotePosition as PlanNotePosition,
    RenderConfig as PlanRenderConfig, RenderConfigBuiltInStyle, RenderConfigOrientation,
    RenderTargetOptions,
};
use layercake_core::sequence_context::{apply_render_config, SequenceStoryContext};
use layercake_core::services::{GraphDataService, GraphService};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

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
                            layercake_core::plan::GraphvizLayout::Neato
                        }
                        crate::graphql::types::plan_dag::config::GraphvizLayout::Fdp => {
                            layercake_core::plan::GraphvizLayout::Fdp
                        }
                        crate::graphql::types::plan_dag::config::GraphvizLayout::Circo => {
                            layercake_core::plan::GraphvizLayout::Circo
                        }
                        _ => layercake_core::plan::GraphvizLayout::Dot,
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
                opts.mermaid = Some(layercake_core::plan::MermaidRenderOptions::default());
            }
            if let Some(ref mut mermaid_opts) = opts.mermaid {
                if let Some(look) = mermaid.look {
                    mermaid_opts.look = match look {
                        crate::graphql::types::plan_dag::config::MermaidLook::HandDrawn => {
                            layercake_core::plan::MermaidLook::HandDrawn
                        }
                        _ => layercake_core::plan::MermaidLook::Default,
                    };
                }
                if let Some(display) = mermaid.display {
                    mermaid_opts.display = match display {
                        crate::graphql::types::plan_dag::config::MermaidDisplay::Compact => {
                            layercake_core::plan::MermaidDisplay::Compact
                        }
                        _ => layercake_core::plan::MermaidDisplay::Full,
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

fn map_sequence_render_target(value: &str) -> Result<SequenceArtefactRenderTarget> {
    match value {
        "PlantUmlSequence" => Ok(SequenceArtefactRenderTarget::PlantUmlSequence),
        "MermaidSequence" => Ok(SequenceArtefactRenderTarget::MermaidSequence),
        other => Err(StructuredError::bad_request(format!(
            "Unsupported sequence render target: {}",
            other
        ))),
    }
}

fn resolve_sequence_render_config(
    stored: Option<StoredSequenceRenderConfig>,
    use_story_layers: bool,
) -> SequenceRenderConfigResolved {
    let mut config = SequenceRenderConfigResolved::default();
    let stored = stored.unwrap_or_default();

    config.contain_nodes = stored
        .contain_nodes
        .unwrap_or_else(|| "one".to_string())
        .to_lowercase();
    if config.contain_nodes != "one" && config.contain_nodes != "all" {
        config.contain_nodes = "one".to_string();
    }

    config.built_in_styles = stored
        .built_in_styles
        .unwrap_or_else(|| "light".to_string())
        .to_lowercase();
    config.show_notes = stored.show_notes.unwrap_or(true);
    config.render_all_sequences = stored.render_all_sequences.unwrap_or(true);
    config.enabled_sequence_ids = stored.enabled_sequence_ids.unwrap_or_default();
    config.use_story_layers = use_story_layers;
    config.mermaid_theme = map_mermaid_theme(&config.built_in_styles);
    config.plantuml_theme = map_plantuml_theme(&config.built_in_styles);
    config
}

fn apply_preview_limit(
    content: String,
    format: &ExportFileType,
    max_rows: Option<usize>,
) -> String {
    match (format, max_rows) {
        (
            ExportFileType::CSVNodes | ExportFileType::CSVEdges | ExportFileType::CSVMatrix,
            Some(limit),
        ) => {
            let mut limited_lines = Vec::new();

            for (index, line) in content.lines().enumerate() {
                if index == 0 || index <= limit {
                    limited_lines.push(line.to_string());
                } else {
                    break;
                }
            }

            limited_lines.join("\n")
        }
        _ => content,
    }
}

async fn load_graph_for_export(
    db: &DatabaseConnection,
    project_id: i32,
    dag_node_id: &str,
) -> anyhow::Result<(Graph, &'static str)> {
    let graph_data_service = GraphDataService::new(db.clone());
    let palette_layers = GraphService::new(db.clone())
        .get_all_resolved_layers(project_id)
        .await
        .unwrap_or_default();
    let palette_map: HashMap<String, Layer> = palette_layers
        .into_iter()
        .map(|layer| (layer.id.clone(), layer))
        .collect();

    if let Some(gd) = graph_data_service.get_by_dag_node(dag_node_id).await? {
        let (gd_model, nodes, edges) = graph_data_service.load_full(gd.id).await?;
        let graph = build_graph_from_graph_data(&gd_model, nodes, edges, Some(&palette_map));
        return Ok((graph, "graph_data"));
    }

    Err(anyhow!(
        "GraphData for node with id '{}' not found",
        dag_node_id
    ))
}

fn build_graph_from_graph_data(
    gd: &graph_data_model::Model,
    nodes: Vec<graph_data_nodes::Model>,
    edges: Vec<graph_data_edges::Model>,
    palette: Option<&HashMap<String, Layer>>,
) -> Graph {
    let normalize_hex = |value: &str| value.trim_start_matches('#').to_string();

    let graph_nodes: Vec<Node> = nodes
        .into_iter()
        .map(|n| Node {
            id: n.external_id,
            label: n.label.unwrap_or_default(),
            layer: n.layer.unwrap_or_default(),
            is_partition: n.is_partition,
            belongs_to: n.belongs_to,
            weight: n.weight.map(|w| w as i32).unwrap_or(1),
            comment: n.comment,
            dataset: n.source_dataset_id,
            attributes: n.attributes,
        })
        .collect();

    let graph_edges: Vec<Edge> = edges
        .into_iter()
        .map(|e| Edge {
            id: e.external_id,
            source: e.source,
            target: e.target,
            label: e.label.unwrap_or_default(),
            layer: e.layer.unwrap_or_default(),
            weight: e.weight.map(|w| w as i32).unwrap_or(1),
            comment: e.comment,
            dataset: e.source_dataset_id,
            attributes: e.attributes,
        })
        .collect();

    let mut layer_map: HashMap<String, Layer> = HashMap::new();
    for node in &graph_nodes {
        if node.layer.is_empty() || layer_map.contains_key(&node.layer) {
            continue;
        }

        let (bg_color, text_color, border_color) = node
            .attributes
            .as_ref()
            .and_then(|attrs| attrs.as_object())
            .map(|obj| {
                let bg = obj
                    .get("backgroundColor")
                    .or_else(|| obj.get("color"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let text = obj
                    .get("textColor")
                    .or_else(|| obj.get("labelColor"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let border = obj
                    .get("borderColor")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (bg, text, border)
            })
            .unwrap_or((None, None, None));

        let mut layer = Layer {
            id: node.layer.clone(),
            label: node.layer.clone(),
            background_color: bg_color
                .as_deref()
                .map(normalize_hex)
                .unwrap_or_else(|| "FFFFFF".to_string()),
            text_color: text_color
                .as_deref()
                .map(normalize_hex)
                .unwrap_or_else(|| "000000".to_string()),
            border_color: border_color
                .as_deref()
                .map(normalize_hex)
                .unwrap_or_else(|| "000000".to_string()),
            alias: None,
            dataset: node.dataset,
            attributes: node.attributes.clone(),
        };

        if let Some(palette) = palette {
            if let Some(p) = palette.get(&node.layer) {
                layer.background_color = normalize_hex(&p.background_color);
                layer.text_color = normalize_hex(&p.text_color);
                layer.border_color = normalize_hex(&p.border_color);
                layer.alias = p.alias.clone();
                layer.dataset = p.dataset;
            }
        }

        layer_map.insert(node.layer.clone(), layer);
    }

    let layers: Vec<Layer> = layer_map.into_values().collect();

    Graph {
        name: gd.name.clone(),
        nodes: graph_nodes,
        edges: graph_edges,
        layers,
        annotations: gd
            .annotations
            .as_ref()
            .and_then(|v| v.as_str().map(|s| s.to_string())),
    }
}

fn map_mermaid_theme(style: &str) -> String {
    match style {
        "dark" => "dark".to_string(),
        "none" => "".to_string(),
        _ => "default".to_string(),
    }
}

fn map_plantuml_theme(style: &str) -> String {
    match style {
        "dark" => "cerulean".to_string(),
        "none" => "".to_string(),
        _ => "plain".to_string(),
    }
}

async fn export_sequence_artefact_node(
    context: &GraphQLContext,
    artefact_node: &plan_dag_nodes::Model,
    project_id: i32,
    edges: &[plan_dag_edges::Model],
    all_nodes: &[plan_dag_nodes::Model],
) -> Result<ExportNodeOutputResult> {
    let stored_config: StoredSequenceArtefactNodeConfig =
        serde_json::from_str(&artefact_node.config_json).map_err(|e| {
            StructuredError::bad_request(format!("Failed to parse sequence artefact config: {}", e))
        })?;

    let render_target_str = stored_config
        .render_target
        .unwrap_or_else(|| "MermaidSequence".to_string());
    let render_target = map_sequence_render_target(&render_target_str)?;
    let output_path = stored_config.output_path;
    let use_story_layers = stored_config.use_story_layers.unwrap_or(true);
    let resolved_config =
        resolve_sequence_render_config(stored_config.render_config, use_story_layers);

    let upstream_story_node = edges
        .iter()
        .find(|edge| edge.target_node_id == artefact_node.id)
        .map(|edge| edge.source_node_id.clone())
        .ok_or_else(|| StructuredError::not_found("Upstream story", "sequence artefact"))?;

    let story_node = all_nodes
        .iter()
        .find(|node| node.id == upstream_story_node)
        .ok_or_else(|| StructuredError::not_found("Story node", &upstream_story_node))?;

    if story_node.node_type != "StoryNode" {
        return Err(StructuredError::bad_request(format!(
            "Sequence artefact must be connected to a Story node, found {}",
            story_node.node_type
        )));
    }

    let story_config: StoryNodeConfig =
        serde_json::from_str(&story_node.config_json).map_err(|e| {
            StructuredError::bad_request(format!(
                "Failed to parse story node config for {}: {}",
                story_node.id, e
            ))
        })?;
    let _story_id = story_config
        .story_id
        .ok_or_else(|| StructuredError::bad_request("Story node is not configured"))?;

    let project = projects::Entity::find_by_id(project_id)
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
        .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

    let extension = get_extension_for_format(render_target_str.as_str());
    let filename = output_path.unwrap_or_else(|| format!("{}.{}", project.name, extension));

    use layercake_core::database::entities::sequence_contexts;
    let stored_context = sequence_contexts::Entity::find()
        .filter(sequence_contexts::Column::NodeId.eq(&story_node.id))
        .one(&context.db)
        .await
        .map_err(|e| StructuredError::database("sequence_contexts::Entity::find", e))?
        .ok_or_else(|| StructuredError::internal("Story context not found"))?;
    let base_context: SequenceStoryContext = serde_json::from_str(&stored_context.context_json)
        .map_err(|e| StructuredError::internal(format!("Invalid story context data: {}", e)))?;
    let render_payload = apply_render_config(&base_context, resolved_config);

    let rendered = match render_target {
        SequenceArtefactRenderTarget::MermaidSequence => {
            to_mermaid_sequence::render(&render_payload)
                .map_err(|e| StructuredError::service("render_mermaid_sequence", e))?
        }
        SequenceArtefactRenderTarget::PlantUmlSequence => {
            to_plantuml_sequence::render(&render_payload)
                .map_err(|e| StructuredError::service("render_plantuml_sequence", e))?
        }
    };

    let encoded_content = base64::engine::general_purpose::STANDARD.encode(rendered.as_bytes());
    let mime_type = get_mime_type_for_format(render_target_str.as_str());

    Ok(ExportNodeOutputResult {
        success: true,
        message: format!(
            "Successfully exported {} as {}",
            filename, render_target_str
        ),
        content: encoded_content,
        filename,
        mime_type,
    })
}

#[Object]
impl PlanDagMutation {
    async fn update_plan_dag(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        plan_id: Option<i32>,
        plan_dag: PlanDagInput,
    ) -> Result<Option<PlanDag>> {
        let context = ctx.data::<GraphQLContext>()?;

        // Verify project exists
        let _project = projects::Entity::find_by_id(project_id)
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("projects::Entity::find_by_id", e))?
            .ok_or_else(|| StructuredError::not_found("Project", project_id))?;

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
        let plan_record_id = plan.id;

        // Clear existing Plan DAG nodes and edges for this plan
        plan_dag_nodes::Entity::delete_many()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan_record_id))
            .exec(&context.db)
            .await?;

        plan_dag_edges::Entity::delete_many()
            .filter(plan_dag_edges::Column::PlanId.eq(plan_record_id))
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
                crate::graphql::types::PlanDagNodeType::Projection => "ProjectionNode",
                crate::graphql::types::PlanDagNodeType::Story => "StoryNode",
                crate::graphql::types::PlanDagNodeType::SequenceArtefact => "SequenceArtefactNode",
            };

            let metadata_json = serde_json::to_string(&node.metadata)?;

            let dag_node = plan_dag_nodes::ActiveModel {
                id: Set(node_id),
                plan_id: Set(plan_record_id),
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
                plan_id: Set(plan_record_id),
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
            .filter(plan_dag_nodes::Column::PlanId.eq(plan_record_id))
            .all(&context.db)
            .await?;

        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan_record_id))
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
        plan_id: Option<i32>,
        node_id: String,
        #[graphql(name = "renderConfigOverride")] render_config_override: Option<
            GraphQLRenderConfig,
        >,
        preview_rows: Option<i32>,
    ) -> Result<ExportNodeOutputResult> {
        let context = ctx.data::<GraphQLContext>()?;

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

        let edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_edges::Entity::find", e))?;

        let all_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&context.db)
            .await
            .map_err(|e| StructuredError::database("plan_dag_nodes::Entity::find", e))?;

        if node_type.as_str() == "SequenceArtefactNode" {
            let result = export_sequence_artefact_node(
                &context,
                &artefact_node,
                project_id,
                &edges,
                &all_nodes,
            )
            .await?;
            return Ok(result);
        }

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
                    stored_config
                        .render_config
                        .map(|rc| rc.into_render_config()),
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
                    stored_config
                        .render_config
                        .map(|rc| rc.into_render_config()),
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

        let upstream_node_id = edges
            .iter()
            .find(|e| e.target_node_id == node_id)
            .map(|e| e.source_node_id.clone())
            .ok_or_else(|| StructuredError::not_found("Upstream graph", "artefact node"))?;

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

        // Load the built graph (graph_data-first, legacy graphs fallback)
        let (graph, _source) = load_graph_for_export(&context.db, project_id, &upstream_node_id)
            .await
            .map_err(|e| StructuredError::service("load_graph_for_export", e))?;

        let export_format = parse_export_format(render_target.as_str())?;

        let preview_limit = preview_rows.and_then(|value| {
            if value > 0 {
                Some(value as usize)
            } else {
                None
            }
        });

        let raw_content = context
            .app
            .export_service()
            .export_to_string(&graph, &export_format, Some(render_config.clone()))
            .map_err(Error::from)?;

        let content = apply_preview_limit(raw_content, &export_format, preview_limit);

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

        // Delete all computed graph_data generated by this project's plan nodes
        let delete_result = graph_data::Entity::delete_many()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::SourceType.eq("computed"))
            .exec(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data::Entity::delete_many", e))?;

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

        // Find all graph_data entries in processing state for this project
        let processing_graphs = graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::Status.eq("processing"))
            .all(&context.db)
            .await
            .map_err(|e| {
                StructuredError::database("graph_data::Entity::find (stop execution)", e)
            })?;

        let mut stopped_count = 0;

        // Update each graph_data to error state with a stop message
        for graph in processing_graphs {
            let mut active: graph_data::ActiveModel = graph.into();
            active.status = Set("error".to_string());
            active.error_message = Set(Some("Execution stopped by user".to_string()));
            active
                .update(&context.db)
                .await
                .map_err(|e| StructuredError::database("graph_data::Entity::update", e))?;
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
