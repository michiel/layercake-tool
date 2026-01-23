use std::{fs, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, ValueEnum};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::query_payloads::{
    EdgeDeletePayload, EdgeUpdatePayload, ExportRequest, NodeDeletePayload, NodeMovePayload,
    NodeUpdatePayload,
};
use layercake_core::{
    app_context::AppContext,
    database::connection::{establish_connection, get_database_url},
    services::cli_graphql_helpers::{CliContext, CliPlanEdgeInput, CliPlanNodeInput},
};

/// Query command arguments consumed by `layercake query`.
#[derive(Debug, Parser)]
pub struct QueryArgs {
    /// Data entity to work with (datasets, plans, nodes, edges, exports).
    #[clap(long, value_enum)]
    pub entity: QueryEntity,

    /// Action to execute for the entity.
    #[clap(long, value_enum)]
    pub action: QueryAction,

    /// Project identifier (required for project-scoped entities).
    #[clap(long)]
    pub project: Option<i32>,

    /// Plan identifier (optional when the default plan should be used).
    #[clap(long)]
    pub plan: Option<i32>,

    /// Inline JSON payload describing the request (mutually exclusive with --payload-file).
    #[clap(long, conflicts_with = "payload_file")]
    pub payload_json: Option<String>,

    /// Path to a file containing the JSON payload.
    #[clap(long, conflicts_with = "payload_json")]
    pub payload_file: Option<String>,

    /// Where to write export outputs (downloads).
    #[clap(long)]
    pub output_file: Option<String>,

    /// Pretty-print the JSON response for humans.
    #[clap(long)]
    pub pretty: bool,

    /// Override the database path (defaults to `layercake.db`).
    #[clap(long)]
    pub database: Option<String>,

    /// Optional session identifier (passed to helpers for future auth support).
    #[clap(long)]
    pub session: Option<String>,

    /// Validate without executing (dry-run mode) - Phase 1.6
    #[clap(long)]
    pub dry_run: bool,
}

/// Supported query entities.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QueryEntity {
    Datasets,
    Plans,
    Nodes,
    Edges,
    Exports,
    Schema,      // Phase 1.4
    Analysis,    // Phase 2.3
    Annotations, // Phase 2.4
}

impl QueryEntity {
    fn as_str(&self) -> &'static str {
        match self {
            QueryEntity::Datasets => "datasets",
            QueryEntity::Plans => "plans",
            QueryEntity::Nodes => "nodes",
            QueryEntity::Edges => "edges",
            QueryEntity::Exports => "exports",
            QueryEntity::Schema => "schema",
            QueryEntity::Analysis => "analysis",
            QueryEntity::Annotations => "annotations",
        }
    }
}

/// Supported query actions.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QueryAction {
    List,
    Get,
    Create,
    Update,
    Delete,
    Move,
    Download,
    Traverse, // Phase 1.3
    Batch,    // Phase 2.1
    Search,   // Phase 2.2
    Clone,    // Phase 2.5
}

impl QueryAction {
    fn as_str(&self) -> &'static str {
        match self {
            QueryAction::List => "list",
            QueryAction::Get => "get",
            QueryAction::Create => "create",
            QueryAction::Update => "update",
            QueryAction::Delete => "delete",
            QueryAction::Move => "move",
            QueryAction::Download => "download",
            QueryAction::Traverse => "traverse",
            QueryAction::Batch => "batch",
            QueryAction::Search => "search",
            QueryAction::Clone => "clone",
        }
    }
}

/// Run the `layercake query` command.
pub async fn run_query_command(args: QueryArgs) -> Result<()> {
    let database_url = get_database_url(args.database.as_deref());
    let db = establish_connection(&database_url).await?;
    let app = Arc::new(AppContext::new(db.clone()));
    let mut ctx = CliContext::new(Arc::clone(&app));

    if let Some(session_id) = args.session.clone() {
        ctx = ctx.with_session(session_id);
    }

    let payload = read_payload(&args)?;

    // Phase 1.6: Validation phase
    let validation_result = validate_action(&args, &payload);

    if args.dry_run {
        // Dry-run mode: return validation results without executing
        let result = json!({
            "valid": validation_result.is_valid,
            "errors": validation_result.errors,
            "warnings": validation_result.warnings,
        });
        emit_response(&args, "ok", Some(result), None)?;
        return Ok(());
    }

    if !validation_result.is_valid {
        let error_msg = format!(
            "Validation failed: {}",
            validation_result.errors.join("; ")
        );
        emit_response(&args, "error", None, Some(&error_msg))?;
        bail!(error_msg);
    }

    match execute_query_action(&ctx, &args, payload).await {
        Ok(result) => {
            emit_response(&args, "ok", Some(result), None)?;
            Ok(())
        }
        Err(err) => {
            let message = err.to_string();
            emit_response(&args, "error", None, Some(&message))?;
            Err(err)
        }
    }
}

pub async fn execute_query_action(
    ctx: &CliContext,
    args: &QueryArgs,
    payload: Option<Value>,
) -> Result<Value> {
    match (args.entity, args.action) {
        (QueryEntity::Datasets, QueryAction::List) => {
            let project_id = require_project_id(args)?;
            let items = ctx
                .list_datasets(project_id)
                .await
                .context("listing datasets")?;
            Ok(serde_json::to_value(items)?)
        }
        (QueryEntity::Datasets, QueryAction::Get) => {
            let payload = require_payload(payload, "datasets get")?;
            let data_set_id: i32 = extract_value(&payload, "id")?;
            let result = ctx
                .get_dataset(data_set_id)
                .await
                .context("loading dataset")?;
            Ok(serde_json::to_value(result)?)
        }
        (QueryEntity::Plans, QueryAction::List) => {
            let project_id = require_project_id(args)?;
            let items = ctx
                .list_plans(Some(project_id))
                .await
                .context("listing plans")?;
            Ok(serde_json::to_value(items)?)
        }
        (QueryEntity::Plans, QueryAction::Get) => {
            let payload = require_payload(payload, "plans get")?;
            let plan_id: i32 = extract_value(&payload, "id")?;
            let result = ctx.get_plan(plan_id).await.context("loading plan")?;
            Ok(serde_json::to_value(result)?)
        }
        (QueryEntity::Nodes, QueryAction::List) => {
            let project_id = require_project_id(args)?;

            // Phase 1.1: Check if filters are provided
            if let Some(ref payload_value) = payload {
                // Try to parse as filter payload
                if let Ok(filter) = serde_json::from_value::<crate::query_payloads::NodeFilterPayload>(
                    payload_value.clone(),
                ) {
                    let bounds = filter
                        .bounds
                        .map(|b| (b.min_x, b.max_x, b.min_y, b.max_y));

                    let nodes = ctx
                        .list_nodes_filtered(
                            project_id,
                            args.plan,
                            filter.node_type,
                            filter.label_pattern,
                            bounds,
                        )
                        .await
                        .context("listing filtered nodes")?;

                    return Ok(serde_json::to_value(nodes)?);
                }
            }

            // Default behaviour: load full DAG
            let snapshot = ctx
                .load_plan_dag(project_id, args.plan)
                .await
                .context("loading plan DAG")?;
            Ok(serde_json::to_value(snapshot)?)
        }
        (QueryEntity::Nodes, QueryAction::Get) => {
            // Phase 1.2: Get single node
            let project_id = require_project_id(args)?;
            let payload_value = require_payload(payload, "nodes get")?;
            let get_payload: crate::query_payloads::NodeGetPayload =
                serde_json::from_value(payload_value).context("parsing node get payload")?;

            let node = ctx
                .get_node(project_id, args.plan, get_payload.node_id)
                .await
                .context("getting node")?;

            Ok(serde_json::to_value(node)?)
        }
        (QueryEntity::Nodes, QueryAction::Traverse) => {
            // Phase 1.3: Graph traversal
            let project_id = require_project_id(args)?;
            let payload_value = require_payload(payload, "nodes traverse")?;
            let traverse: crate::query_payloads::TraversePayload =
                serde_json::from_value(payload_value).context("parsing traverse payload")?;

            if traverse.find_path.unwrap_or(false) && traverse.end_node.is_some() {
                // Path finding mode
                let path = ctx
                    .find_path(
                        project_id,
                        args.plan,
                        traverse.start_node,
                        traverse.end_node.unwrap(),
                    )
                    .await
                    .context("finding path")?;

                let result = json!({
                    "paths": path.map(|p| vec![p]).unwrap_or_default(),
                });
                Ok(result)
            } else {
                // Traversal mode
                let direction = traverse.direction.unwrap_or_else(|| "downstream".to_string());
                let max_depth = traverse.max_depth.unwrap_or(usize::MAX);

                let (nodes, edges) = ctx
                    .traverse_from_node(project_id, args.plan, traverse.start_node, direction, max_depth)
                    .await
                    .context("traversing graph")?;

                let result = json!({
                    "nodes": nodes,
                    "edges": edges,
                    "depth": max_depth,
                });

                Ok(result)
            }
        }
        (QueryEntity::Nodes, QueryAction::Create) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "nodes create")?;
            let node_input: CliPlanNodeInput =
                serde_json::from_value(payload).context("parsing node input")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let node = ctx
                .create_plan_node(project_id, plan_id, node_input)
                .await
                .context("creating plan node")?;
            Ok(serde_json::to_value(node)?)
        }
        (QueryEntity::Nodes, QueryAction::Update) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "nodes update")?;
            let update: NodeUpdatePayload =
                serde_json::from_value(payload).context("parsing node update")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let node = ctx
                .update_plan_node(project_id, plan_id, update.node_id, update.update)
                .await
                .context("updating plan node")?;
            Ok(serde_json::to_value(node)?)
        }
        (QueryEntity::Nodes, QueryAction::Delete) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "nodes delete")?;
            let delete: NodeDeletePayload =
                serde_json::from_value(payload).context("parsing node delete payload")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let node = ctx
                .delete_plan_node(project_id, plan_id, delete.node_id)
                .await
                .context("deleting plan node")?;
            Ok(serde_json::to_value(node)?)
        }
        (QueryEntity::Nodes, QueryAction::Move) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "nodes move")?;
            let movement: NodeMovePayload =
                serde_json::from_value(payload).context("parsing node move payload")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let node = ctx
                .move_plan_node(project_id, plan_id, movement.node_id, movement.position)
                .await
                .context("moving plan node")?;
            Ok(serde_json::to_value(node)?)
        }
        (QueryEntity::Edges, QueryAction::Create) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "edges create")?;
            let edge_input: CliPlanEdgeInput =
                serde_json::from_value(payload).context("parsing edge input")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let edge = ctx
                .create_plan_edge(project_id, plan_id, edge_input)
                .await
                .context("creating edge")?;
            Ok(serde_json::to_value(edge)?)
        }
        (QueryEntity::Edges, QueryAction::Update) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "edges update")?;
            let update: EdgeUpdatePayload =
                serde_json::from_value(payload).context("parsing edge update")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let edge = ctx
                .update_plan_edge(project_id, plan_id, update.edge_id, update.update)
                .await
                .context("updating edge")?;
            Ok(serde_json::to_value(edge)?)
        }
        (QueryEntity::Edges, QueryAction::Delete) => {
            let project_id = require_project_id(args)?;
            let payload = require_payload(payload, "edges delete")?;
            let delete: EdgeDeletePayload =
                serde_json::from_value(payload).context("parsing edge delete payload")?;
            let plan_id = ctx
                .resolve_plan_id(project_id, args.plan)
                .await
                .context("resolving plan")?;
            let edge = ctx
                .delete_plan_edge(project_id, plan_id, delete.edge_id)
                .await
                .context("deleting edge")?;
            Ok(serde_json::to_value(edge)?)
        }
        (QueryEntity::Exports, QueryAction::Download) => {
            let payload = require_payload(payload, "exports download")?;
            let request: ExportRequest =
                serde_json::from_value(payload).context("parsing export request")?;
            let format_value = serde_json::to_value(&request.format)?;
            let format_clone = request.format.clone();
            let content = ctx
                .preview_graph_export(
                    request.graph_id,
                    format_clone,
                    request.render_config,
                    request.max_rows,
                )
                .await
                .context("rendering export")?;

            let mut response = json!({
                "graphId": request.graph_id,
                "format": format_value,
                "content": content,
            });

            if let Some(output_path) = &args.output_file {
                fs::write(output_path, content.as_bytes()).context("writing export file")?;
                response["filePath"] = json!(output_path);
            }

            Ok(response)
        }
        (QueryEntity::Schema, QueryAction::Get) => {
            // Phase 1.4: Schema introspection - get schema for specific type
            let payload_value = payload.unwrap_or_else(|| json!({}));
            let schema_type = payload_value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("node");
            let node_type = payload_value.get("nodeType").and_then(|v| v.as_str());

            let schema = match schema_type {
                "node" => {
                    let desc = crate::schema_introspection::get_node_create_schema(node_type);
                    serde_json::to_value(desc)?
                }
                "edge" => {
                    let desc = crate::schema_introspection::get_edge_create_schema();
                    serde_json::to_value(desc)?
                }
                _ => json!({"error": "Unknown schema type. Valid types: 'node', 'edge'"}),
            };

            Ok(schema)
        }
        (QueryEntity::Schema, QueryAction::List) => {
            // Phase 1.4: Schema introspection - list available options
            let payload_value = payload.unwrap_or_else(|| json!({}));
            let list_type = payload_value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("entities");

            let result = match list_type {
                "nodeTypes" => {
                    let node_types = crate::schema_introspection::get_node_types();
                    json!({"nodeTypes": node_types})
                }
                "formats" => {
                    let formats = crate::schema_introspection::get_export_formats();
                    json!({"formats": formats})
                }
                "actions" => {
                    let entity = payload_value.get("entity").and_then(|v| v.as_str());
                    if let Some(entity_name) = entity {
                        let actions = crate::schema_introspection::get_available_actions(entity_name);
                        json!({"entity": entity_name, "actions": actions})
                    } else {
                        json!({"error": "Missing 'entity' parameter"})
                    }
                }
                _ => {
                    // Default: list all entities
                    json!({
                        "entities": ["datasets", "plans", "nodes", "edges", "exports", "schema"],
                        "hint": "Use --payload-json '{\"type\":\"nodeTypes\"}' to list node types, or '{\"type\":\"actions\",\"entity\":\"nodes\"}' for available actions"
                    })
                }
            };

            Ok(result)
        }
        (QueryEntity::Nodes, QueryAction::Batch) => {
            // Phase 2.1: Batch operations (simplified implementation)
            let project_id = require_project_id(args)?;
            let plan_id = ctx.resolve_plan_id(project_id, args.plan).await?;
            let payload_value = require_payload(payload, "nodes batch")?;
            let batch: crate::query_payloads::BatchPayload =
                serde_json::from_value(payload_value).context("parsing batch payload")?;

            use std::collections::HashMap;
            let mut id_mapping: HashMap<String, String> = HashMap::new();
            let mut completed = 0;
            let mut errors: Vec<String> = Vec::new();

            for operation in batch.operations {
                let result = match operation.op.as_str() {
                    "createNode" => {
                        // Parse node input
                        match serde_json::from_value::<layercake_core::services::cli_graphql_helpers::CliPlanNodeInput>(operation.data) {
                            Ok(input) => {
                                match ctx.create_plan_node(project_id, plan_id, input).await {
                                    Ok(node) => {
                                        if let Some(temp_id) = operation.id {
                                            id_mapping.insert(temp_id, node.node.id.clone());
                                        }
                                        Ok(())
                                    }
                                    Err(e) => Err(format!("Failed to create node: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Invalid node input: {}", e)),
                        }
                    }
                    "createEdge" => {
                        // Parse edge input and resolve temporary IDs
                        match serde_json::from_value::<layercake_core::services::cli_graphql_helpers::CliPlanEdgeInput>(operation.data.clone()) {
                            Ok(mut input) => {
                                // Resolve temporary IDs
                                if input.source.starts_with('$') {
                                    if let Some(actual_id) = id_mapping.get(&input.source[1..]) {
                                        input.source = actual_id.clone();
                                    }
                                }
                                if input.target.starts_with('$') {
                                    if let Some(actual_id) = id_mapping.get(&input.target[1..]) {
                                        input.target = actual_id.clone();
                                    }
                                }

                                match ctx.create_plan_edge(project_id, plan_id, input).await {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(format!("Failed to create edge: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Invalid edge input: {}", e)),
                        }
                    }
                    _ => Err(format!("Unknown operation: {}", operation.op)),
                };

                match result {
                    Ok(_) => completed += 1,
                    Err(e) => {
                        errors.push(e);
                        if batch.atomic.unwrap_or(false) {
                            // In atomic mode, stop on first error
                            break;
                        }
                    }
                }
            }

            let result = json!({
                "success": errors.is_empty(),
                "operationsCompleted": completed,
                "idMapping": id_mapping,
                "errors": errors,
            });

            Ok(result)
        }
        (QueryEntity::Nodes, QueryAction::Search) => {
            // Phase 2.2: Search and discovery
            let project_id = require_project_id(args)?;
            let payload_value = require_payload(payload, "nodes search")?;
            let search: crate::query_payloads::SearchPayload =
                serde_json::from_value(payload_value).context("parsing search payload")?;

            let nodes = if let Some(edge_filter) = search.edge_filter {
                // Edge filter mode
                ctx.find_nodes_by_edge_filter(project_id, args.plan, edge_filter)
                    .await
                    .context("searching by edge filter")?
            } else {
                // Text search mode
                let fields = search.fields.unwrap_or_else(|| vec!["label".to_string()]);
                ctx.search_nodes(project_id, args.plan, search.query, fields)
                    .await
                    .context("searching nodes")?
            };

            Ok(serde_json::to_value(nodes)?)
        }
        (QueryEntity::Analysis, QueryAction::Get) => {
            // Phase 2.3: Graph analysis operations
            let project_id = require_project_id(args)?;
            let plan_id = ctx.resolve_plan_id(project_id, args.plan).await?;
            let payload_value = require_payload(payload, "analysis get")?;
            let analysis_type = payload_value
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing analysis type"))?;

            match analysis_type {
                "stats" => {
                    let stats = ctx
                        .app
                        .plan_dag_service()
                        .analyze_plan_stats(project_id, Some(plan_id))
                        .await
                        .context("analyzing plan stats")?;
                    Ok(serde_json::to_value(stats)?)
                }
                "bottlenecks" => {
                    let threshold = payload_value
                        .get("threshold")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(5) as usize;
                    let bottlenecks = ctx
                        .app
                        .plan_dag_service()
                        .find_bottlenecks(project_id, Some(plan_id), threshold)
                        .await
                        .context("finding bottlenecks")?;
                    Ok(serde_json::to_value(bottlenecks)?)
                }
                "cycles" => {
                    let cycles = ctx
                        .app
                        .plan_dag_service()
                        .detect_cycles(project_id, Some(plan_id))
                        .await
                        .context("detecting cycles")?;
                    Ok(serde_json::to_value(json!({"cycles": cycles}))?)
                }
                _ => bail!("Unknown analysis type: {}", analysis_type),
            }
        }
        (QueryEntity::Nodes, QueryAction::Clone) => {
            // Phase 2.5: Clone operations
            let project_id = require_project_id(args)?;
            let plan_id = ctx.resolve_plan_id(project_id, args.plan).await?;
            let payload_value = require_payload(payload, "nodes clone")?;
            let clone_payload: crate::query_payloads::ClonePayload =
                serde_json::from_value(payload_value).context("parsing clone payload")?;

            let cloned_node = ctx
                .app
                .plan_dag_service()
                .clone_node(
                    project_id,
                    Some(plan_id),
                    &clone_payload.node_id,
                    clone_payload.position,
                    clone_payload.update_label,
                )
                .await
                .context("cloning node")?;

            use layercake_core::services::cli_graphql_helpers::CliPlanDagNode;
            let cli_node = CliPlanDagNode::new(project_id, plan_id, cloned_node);
            Ok(serde_json::to_value(cli_node)?)
        }
        (QueryEntity::Annotations, QueryAction::Create) => {
            // Phase 2.4: Create annotation
            let project_id = require_project_id(args)?;
            let payload_value = require_payload(payload, "annotations create")?;
            let create_payload: crate::query_payloads::AnnotationCreatePayload =
                serde_json::from_value(payload_value).context("parsing annotation create payload")?;

            let annotation = ctx
                .app
                .plan_dag_service()
                .create_annotation(
                    project_id,
                    args.plan,
                    create_payload.target_id,
                    create_payload.target_type,
                    create_payload.key,
                    create_payload.value,
                )
                .await
                .context("creating annotation")?;

            Ok(serde_json::to_value(annotation)?)
        }
        (QueryEntity::Annotations, QueryAction::List) => {
            // Phase 2.4: List annotations
            let project_id = require_project_id(args)?;
            let payload_value = require_payload(payload, "annotations list")?;
            let list_payload: crate::query_payloads::AnnotationListPayload =
                serde_json::from_value(payload_value).context("parsing annotation list payload")?;

            let annotations = if let Some(target_id) = list_payload.target_id {
                // List annotations for specific target
                ctx.app
                    .plan_dag_service()
                    .list_annotations(project_id, args.plan, target_id)
                    .await
                    .context("listing annotations")?
            } else {
                // List all annotations for plan (optionally filtered by key)
                ctx.app
                    .plan_dag_service()
                    .list_plan_annotations(project_id, args.plan, list_payload.key)
                    .await
                    .context("listing plan annotations")?
            };

            Ok(serde_json::to_value(annotations)?)
        }
        (QueryEntity::Annotations, QueryAction::Get) => {
            // Phase 2.4: Get specific annotation
            let payload_value = require_payload(payload, "annotations get")?;
            let get_payload: crate::query_payloads::AnnotationGetPayload =
                serde_json::from_value(payload_value).context("parsing annotation get payload")?;

            let annotation = ctx
                .app
                .plan_dag_service()
                .get_annotation(get_payload.id)
                .await
                .context("getting annotation")?;

            Ok(serde_json::to_value(annotation)?)
        }
        (QueryEntity::Annotations, QueryAction::Update) => {
            // Phase 2.4: Update annotation
            let payload_value = require_payload(payload, "annotations update")?;
            let update_payload: crate::query_payloads::AnnotationUpdatePayload =
                serde_json::from_value(payload_value).context("parsing annotation update payload")?;

            let annotation = ctx
                .app
                .plan_dag_service()
                .update_annotation(update_payload.id, update_payload.value)
                .await
                .context("updating annotation")?;

            Ok(serde_json::to_value(annotation)?)
        }
        (QueryEntity::Annotations, QueryAction::Delete) => {
            // Phase 2.4: Delete annotation
            let payload_value = require_payload(payload, "annotations delete")?;
            let delete_payload: crate::query_payloads::AnnotationDeletePayload =
                serde_json::from_value(payload_value).context("parsing annotation delete payload")?;

            let annotation = ctx
                .app
                .plan_dag_service()
                .delete_annotation(delete_payload.id)
                .await
                .context("deleting annotation")?;

            Ok(serde_json::to_value(annotation)?)
        }
        _ => bail!(
            "action '{}' is not available for entity '{}'",
            args.action.as_str(),
            args.entity.as_str()
        ),
    }
}

fn require_project_id(args: &QueryArgs) -> Result<i32> {
    args.project
        .context("`--project` is required for this entity")
}

fn require_payload(value: Option<Value>, context: &str) -> Result<Value> {
    value.ok_or_else(|| anyhow!("{} requires --payload-json or --payload-file", context))
}

fn extract_value<T: DeserializeOwned>(payload: &Value, key: &str) -> Result<T> {
    payload
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("payload is missing '{}'", key))
        .and_then(|value| {
            serde_json::from_value(value).context(format!("failed to parse '{}'", key))
        })
}

fn read_payload(args: &QueryArgs) -> Result<Option<Value>> {
    if let Some(json_text) = &args.payload_json {
        let value = serde_json::from_str(json_text).context("invalid --payload-json")?;
        return Ok(Some(value));
    }

    if let Some(path) = &args.payload_file {
        let contents =
            fs::read_to_string(path).context("failed to read --payload-file contents")?;
        let value = serde_json::from_str(&contents).context("invalid JSON in payload file")?;
        return Ok(Some(value));
    }

    Ok(None)
}

/// Phase 1.6: Validation result structure
#[derive(Debug)]
struct ValidationResult {
    is_valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// Phase 1.6: Validate action and payload before execution
fn validate_action(args: &QueryArgs, payload: &Option<Value>) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    match (args.entity, args.action) {
        (QueryEntity::Nodes, QueryAction::Create) => {
            if let Some(payload_value) = payload {
                // Validate node creation payload
                if payload_value.get("nodeType").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'nodeType'".to_string());
                    result.errors.push(
                        "Use --payload-json '{\"nodeType\":\"GraphNode\",\"position\":{\"x\":100,\"y\":200},\"metadata\":{\"label\":\"Label\"},\"config\":{}}'".to_string()
                    );
                }
                if payload_value.get("position").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'position'".to_string());
                }
                if payload_value.get("metadata").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'metadata'".to_string());
                }
                if payload_value.get("config").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'config'".to_string());
                }

                // Validate position structure
                if let Some(position) = payload_value.get("position") {
                    if position.get("x").is_none() || position.get("y").is_none() {
                        result.is_valid = false;
                        result
                            .errors
                            .push("Position must have 'x' and 'y' fields".to_string());
                    }
                }
            } else {
                result.is_valid = false;
                result
                    .errors
                    .push("Payload is required for create action".to_string());
            }
        }
        (QueryEntity::Nodes, QueryAction::Update) => {
            if let Some(payload_value) = payload {
                if payload_value.get("nodeId").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'nodeId'".to_string());
                }
            } else {
                result.is_valid = false;
                result
                    .errors
                    .push("Payload is required for update action".to_string());
            }
        }
        (QueryEntity::Nodes, QueryAction::Delete) => {
            if let Some(payload_value) = payload {
                if payload_value.get("nodeId").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'nodeId'".to_string());
                }
            } else {
                result.is_valid = false;
                result
                    .errors
                    .push("Payload is required for delete action".to_string());
            }
        }
        (QueryEntity::Nodes, QueryAction::Get) => {
            if let Some(payload_value) = payload {
                if payload_value.get("nodeId").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'nodeId'".to_string());
                }
            } else {
                result.is_valid = false;
                result.errors.push("Payload is required for get action".to_string());
            }
        }
        (QueryEntity::Edges, QueryAction::Create) => {
            if let Some(payload_value) = payload {
                if payload_value.get("source").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'source'".to_string());
                }
                if payload_value.get("target").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'target'".to_string());
                }
                if payload_value.get("metadata").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'metadata'".to_string());
                }
            } else {
                result.is_valid = false;
                result
                    .errors
                    .push("Payload is required for create action".to_string());
            }
        }
        (QueryEntity::Nodes, QueryAction::Traverse) => {
            if let Some(payload_value) = payload {
                if payload_value.get("startNode").is_none() {
                    result.is_valid = false;
                    result
                        .errors
                        .push("Missing required field 'startNode'".to_string());
                }
            } else {
                result.is_valid = false;
                result
                    .errors
                    .push("Payload is required for traverse action".to_string());
            }
        }
        _ => {
            // No specific validation for other actions
        }
    }

    // Check for project requirement
    if matches!(
        args.entity,
        QueryEntity::Nodes | QueryEntity::Edges | QueryEntity::Datasets
    ) && args.project.is_none()
    {
        result.is_valid = false;
        result
            .errors
            .push("--project parameter is required for this entity".to_string());
    }

    result
}

fn emit_response(
    args: &QueryArgs,
    status: &str,
    data: Option<Value>,
    message: Option<&str>,
) -> Result<()> {
    let mut response = json!({
        "status": status,
        "entity": args.entity.as_str(),
        "action": args.action.as_str(),
        "project": args.project,
        "plan": args.plan,
    });

    if let Some(msg) = message {
        response["message"] = json!(msg);

        // Phase 1.5: Add context and suggestions for errors
        if status == "error" {
            response["context"] = json!({
                "entity": args.entity.as_str(),
                "action": args.action.as_str(),
            });

            let suggestions = generate_error_suggestions(msg, args);
            if !suggestions.is_empty() {
                response["suggestions"] = json!(suggestions);
            }
        }
    }

    response["result"] = data.unwrap_or_else(|| Value::Null);
    print_json(&response, args.pretty)?;
    Ok(())
}

/// Phase 1.5: Generate helpful suggestions based on error messages
fn generate_error_suggestions(error: &str, args: &QueryArgs) -> Vec<String> {
    let mut suggestions = Vec::new();
    let error_lower = error.to_lowercase();

    // Not found errors
    if error_lower.contains("not found") {
        if error_lower.contains("node") {
            suggestions.push(format!(
                "Use 'layercake query --entity nodes --action list --project {} --plan {}' to see all available nodes",
                args.project.unwrap_or(0),
                args.plan.unwrap_or(0)
            ));
            suggestions.push(
                "Double-check the node ID - it should look like 'graph_abc123' or 'dataset_def456'"
                    .to_string(),
            );
        } else {
            suggestions.push(format!(
                "Use 'layercake query --entity {} --action list' to see all available items",
                args.entity.as_str()
            ));
        }
    }

    // Missing payload errors
    if error_lower.contains("requires") && error_lower.contains("payload") {
        suggestions.push(
            "This action requires --payload-json or --payload-file with the request data"
                .to_string(),
        );
        suggestions.push(format!(
            "Use 'layercake query --entity schema --action get --payload-json '{{\"type\":\"{}\"}}' for examples",
            match args.entity {
                QueryEntity::Nodes => "node",
                QueryEntity::Edges => "edge",
                _ => "node",
            }
        ));
    }

    // Missing project errors
    if error_lower.contains("project") && error_lower.contains("required") {
        suggestions.push("Add --project <project-id> to your command".to_string());
        suggestions.push("Use 'layercake list-projects' to see available projects".to_string());
    }

    // Parsing errors
    if error_lower.contains("parsing") || error_lower.contains("invalid json") {
        suggestions.push("Check that your JSON payload is valid and properly quoted".to_string());
        suggestions.push(
            "Use single quotes around the JSON: --payload-json '{\"key\":\"value\"}'"
                .to_string(),
        );
    }

    // Missing fields
    if error_lower.contains("missing") {
        if error_lower.contains("nodeid") || error_lower.contains("node_id") {
            suggestions.push("Add the nodeId field: '{\"nodeId\":\"graph_abc123\"}'".to_string());
        }
        if error_lower.contains("nodetype") || error_lower.contains("node_type") {
            suggestions.push(
                "Add the nodeType field, e.g. '{\"nodeType\":\"GraphNode\"}'"
                    .to_string(),
            );
            suggestions.push(
                "Use 'layercake query --entity schema --action list --payload-json '{\"type\":\"nodeTypes\"}' for available types"
                    .to_string(),
            );
        }
        if error_lower.contains("position") {
            suggestions.push(
                "Add the position field: '{\"position\":{\"x\":100,\"y\":200}}'"
                    .to_string(),
            );
        }
    }

    // Action not available
    if error_lower.contains("not available for entity") {
        suggestions.push(format!(
            "Use 'layercake query --entity schema --action list --payload-json '{{\"type\":\"actions\",\"entity\":\"{}\"}}' to see available actions",
            args.entity.as_str()
        ));
    }

    // Database errors
    if error_lower.contains("database error") {
        suggestions.push("Check that the database file exists and is accessible".to_string());
        suggestions.push(
            "Try specifying the database path: --database ./layercake.db".to_string(),
        );
    }

    suggestions
}

fn print_json(value: &Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", value);
    }
    Ok(())
}
