use std::{
    io::{self, BufRead},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use atty::Stream;
use clap::Parser;
use rustyline::{error::ReadlineError, DefaultEditor};
use serde_json::{json, Value};

use layercake_core::{
    app_context::AppContext,
    database::connection::{establish_connection, get_database_url},
    services::cli_graphql_helpers::CliContext,
};

use crate::query::{execute_query_action, QueryAction, QueryArgs, QueryEntity};

/// Arguments when launching `layercake repl`.
#[derive(Debug, Parser)]
pub struct ReplArgs {
    /// Database path to use (defaults to layercake.db)
    #[arg(long)]
    pub database: Option<String>,

    /// Optional session identifier (mirrors `--session` on `layercake query`)
    #[arg(long)]
    pub session: Option<String>,

    /// Optional initial project context.
    #[arg(long)]
    pub project: Option<i32>,

    /// Optional initial plan context.
    #[arg(long)]
    pub plan: Option<i32>,
}

/// REPL session state.
struct ReplState {
    project_id: Option<i32>,
    plan_id: Option<i32>,
}

impl ReplState {
    fn new(project: Option<i32>, plan: Option<i32>) -> Self {
        Self {
            project_id: project,
            plan_id: plan,
        }
    }

    fn prompt(&self) -> String {
        format!(
            "layercake[project={} plan={}]> ",
            self.project_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            self.plan_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        )
    }

    fn set_project(&mut self, project_id: i32) {
        self.project_id = Some(project_id);
        self.plan_id = None;
    }

    fn set_plan(&mut self, plan_id: i32) {
        self.plan_id = Some(plan_id);
    }
}

/// Repl command execution request.
enum ReplCommand {
    Exit,
    Help,
    Noop,
    SetProject(i32),
    SetPlan(i32),
    Execute {
        entity: QueryEntity,
        action: QueryAction,
        payload: Option<Value>,
        output_file: Option<String>,
    },
}

/// Start the interactive REPL shell.
pub async fn run_repl(args: ReplArgs) -> Result<()> {
    let database_url = get_database_url(args.database.as_deref());
    let db = establish_connection(&database_url).await?;
    let app = Arc::new(AppContext::new(db.clone()));
    let mut ctx = CliContext::new(Arc::clone(&app));

    if let Some(session_id) = args.session.clone() {
        ctx = ctx.with_session(session_id);
    }

    let mut state = ReplState::new(args.project, args.plan);

    println!("Layercake REPL (type `help` for commands, `exit` to quit).");

    if atty::is(Stream::Stdin) {
        run_interactive(&mut ctx, &mut state).await?;
    } else {
        run_batch(&mut ctx, &mut state).await?;
    }

    Ok(())
}

async fn run_interactive(ctx: &mut CliContext, state: &mut ReplState) -> Result<()> {
    let mut editor = DefaultEditor::new()?;

    loop {
        let prompt = state.prompt();
        match editor.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = editor.add_history_entry(trimmed);
                if process_command(ctx, state, trimmed).await? {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("(interrupt)");
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                return Err(anyhow!("reading REPL input: {err}"));
            }
        }
    }

    Ok(())
}

async fn run_batch(ctx: &mut CliContext, state: &mut ReplState) -> Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if process_command(ctx, state, trimmed).await? {
            break;
        }
    }
    Ok(())
}

async fn process_command(ctx: &mut CliContext, state: &mut ReplState, line: &str) -> Result<bool> {
    match parse_command(line) {
        Ok(ReplCommand::Exit) => {
            emit_repl_response(line, state, "ok", None, Some("exiting REPL"));
            return Ok(true);
        }
        Ok(ReplCommand::Help) => {
            emit_repl_response(
                line,
                state,
                "info",
                None,
                Some("Examples: `set project=1`, `list nodes`, `create node { ... }`, `download export { ... } --output /tmp/file`"),
            );
            return Ok(false);
        }
        Ok(ReplCommand::Noop) => return Ok(false),
        Ok(ReplCommand::SetProject(project_id)) => {
            state.set_project(project_id);
            emit_repl_response(
                line,
                state,
                "ok",
                Some(json!({"project": project_id})),
                Some("project context set, plan reset"),
            );
            return Ok(false);
        }
        Ok(ReplCommand::SetPlan(plan_id)) => {
            state.set_plan(plan_id);
            emit_repl_response(
                line,
                state,
                "ok",
                Some(json!({"plan": plan_id})),
                Some("plan context updated"),
            );
            return Ok(false);
        }
        Ok(ReplCommand::Execute {
            entity,
            action,
            payload,
            output_file,
        }) => {
            let args = QueryArgs {
                entity,
                action,
                project: state.project_id,
                plan: state.plan_id,
                payload_json: None,
                payload_file: None,
                output_file,
                pretty: false,
                database: None,
                session: None,
                dry_run: false,
            };

            match execute_query_action(ctx, &args, payload).await {
                Ok(result) => {
                    emit_repl_response(line, state, "ok", Some(result), None);
                }
                Err(err) => {
                    emit_repl_response(line, state, "error", None, Some(&err.to_string()));
                }
            }

            return Ok(false);
        }
        Err(err) => {
            emit_repl_response(line, state, "error", None, Some(&err.to_string()));
            return Ok(false);
        }
    }
}

fn emit_repl_response(
    command: &str,
    state: &ReplState,
    status: &str,
    result: Option<Value>,
    message: Option<&str>,
) {
    let mut response = json!({
        "command": command,
        "status": status,
        "project": state.project_id,
        "plan": state.plan_id,
        "result": result.unwrap_or_else(|| Value::Null),
    });

    if let Some(msg) = message {
        response["message"] = json!(msg);
    }

    println!("{}", response);
}

fn parse_command(line: &str) -> Result<ReplCommand> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(ReplCommand::Noop);
    }

    let lower = trimmed.to_ascii_lowercase();

    if lower == "exit" || lower == "quit" {
        return Ok(ReplCommand::Exit);
    }

    if lower == "help" {
        return Ok(ReplCommand::Help);
    }

    if lower.starts_with("set project") {
        let rest = trimmed["set project".len()..].trim_start();
        let project_id = parse_int_argument(rest, "set project")?;
        return Ok(ReplCommand::SetProject(project_id));
    }

    if lower.starts_with("set plan") {
        let rest = trimmed["set plan".len()..].trim_start();
        let plan_id = parse_int_argument(rest, "set plan")?;
        return Ok(ReplCommand::SetPlan(plan_id));
    }

    if lower == "list datasets" {
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Datasets,
            action: QueryAction::List,
            payload: None,
            output_file: None,
        });
    }

    if lower == "list plans" {
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Plans,
            action: QueryAction::List,
            payload: None,
            output_file: None,
        });
    }

    if lower == "list nodes" {
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Nodes,
            action: QueryAction::List,
            payload: None,
            output_file: None,
        });
    }

    if lower.starts_with("create node") {
        let payload_text = trimmed["create node".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "create node")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Nodes,
            action: QueryAction::Create,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("update node") {
        let payload_text = trimmed["update node".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "update node")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Nodes,
            action: QueryAction::Update,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("delete node") {
        let payload_text = trimmed["delete node".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "delete node")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Nodes,
            action: QueryAction::Delete,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("move node") {
        let payload_text = trimmed["move node".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "move node")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Nodes,
            action: QueryAction::Move,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("create edge") {
        let payload_text = trimmed["create edge".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "create edge")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Edges,
            action: QueryAction::Create,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("update edge") {
        let payload_text = trimmed["update edge".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "update edge")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Edges,
            action: QueryAction::Update,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("delete edge") {
        let payload_text = trimmed["delete edge".len()..].trim();
        let json_payload = parse_json_payload(payload_text, "delete edge")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Edges,
            action: QueryAction::Delete,
            payload: Some(json_payload),
            output_file: None,
        });
    }

    if lower.starts_with("download export") {
        let rest = trimmed["download export".len()..].trim();
        let (payload_text, output_file) = split_payload_and_output(rest)?;
        let json_payload = parse_json_payload(payload_text, "download export")?;
        return Ok(ReplCommand::Execute {
            entity: QueryEntity::Exports,
            action: QueryAction::Download,
            payload: Some(json_payload),
            output_file,
        });
    }

    Err(anyhow!("unrecognized command: {}", line))
}

fn parse_int_argument(value: &str, context: &str) -> Result<i32> {
    let trimmed = value.trim();
    let payload = if let Some(stripped) = trimmed.strip_prefix('=') {
        stripped.trim()
    } else {
        trimmed
    };
    let token = payload
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("{} requires a numeric value", context))?;
    token
        .parse::<i32>()
        .context(format!("failed to parse {} value", context))
}

fn parse_json_payload(text: &str, context: &str) -> Result<Value> {
    if text.is_empty() {
        return Err(anyhow!("{} requires a JSON payload", context));
    }
    serde_json::from_str(text).context(format!("invalid JSON for {}", context))
}

fn split_payload_and_output(text: &str) -> Result<(&str, Option<String>)> {
    if let Some(idx) = text.find("--output") {
        let (before, after_flag) = text.split_at(idx);
        let payload = before.trim();
        let mut after = after_flag["--output".len()..].trim_start();
        if let Some(stripped) = after.strip_prefix('=') {
            after = stripped.trim_start();
        }
        let path = after
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("missing path after --output"))?;
        let remaining = after[path.len()..].trim_start();
        if !remaining.is_empty() {
            return Err(anyhow!("unexpected text after --output path"));
        }
        Ok((payload, Some(path.to_string())))
    } else {
        Ok((text.trim(), None))
    }
}
