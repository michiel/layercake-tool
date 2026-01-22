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
}

/// Supported query entities.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QueryEntity {
    Datasets,
    Plans,
    Nodes,
    Edges,
    Exports,
}

impl QueryEntity {
    fn as_str(&self) -> &'static str {
        match self {
            QueryEntity::Datasets => "datasets",
            QueryEntity::Plans => "plans",
            QueryEntity::Nodes => "nodes",
            QueryEntity::Edges => "edges",
            QueryEntity::Exports => "exports",
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
            let snapshot = ctx
                .load_plan_dag(project_id, args.plan)
                .await
                .context("loading plan DAG")?;
            Ok(serde_json::to_value(snapshot)?)
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
    }

    response["result"] = data.unwrap_or_else(|| Value::Null);
    print_json(&response, args.pretty)?;
    Ok(())
}

fn print_json(value: &Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", value);
    }
    Ok(())
}
