use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing::Level;
use tracing_subscriber::EnvFilter;

use layercake_core::{common, generate_commands, plan, plan_execution, update};
use layercake_server::server;

#[cfg(feature = "console")]
mod console;

mod api;
mod db_info;
mod doc;
mod doctor;
mod query;
mod query_payloads;
mod schema_dump;
mod schema_introspection;
use query::QueryArgs;
mod repl;
use repl::ReplArgs;

#[cfg(feature = "console")]
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(short, long, global = true)]
    log_level: Option<String>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[clap(short, long)]
        plan: String,
        #[clap(short, long)]
        watch: bool,
    },
    Init {
        #[clap(short, long)]
        plan: String,
    },
    Generate {
        #[clap(subcommand)]
        command: GenerateCommands,
    },
    Repl(ReplArgs),
    Query(QueryArgs),
    Serve {
        /// Address to bind. Defaults to loopback (local-only); use 0.0.0.0 to
        /// expose the server on the network when self-hosting.
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        #[clap(short, long, default_value = "3000")]
        port: u16,
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
        #[clap(long)]
        cors_origin: Option<String>,
        /// Open the web UI in the default browser once the server is ready.
        #[clap(long)]
        open: bool,
    },
    Db {
        #[clap(subcommand)]
        command: DbCommands,
    },
    #[cfg(feature = "console")]
    Console {
        /// Optional database path; defaults to layercake.db
        #[clap(long)]
        database: Option<String>,
    },
    Update {
        /// Check for updates without installing
        #[clap(short, long)]
        check: bool,
        /// Force update even if already up to date
        #[clap(short, long)]
        force: bool,
        /// Include pre-release versions
        #[clap(short, long)]
        pre_release: bool,
        /// Create backup before updating
        #[clap(short, long)]
        backup: bool,
        /// Rollback to previous version
        #[clap(short, long)]
        rollback: bool,
        /// Show what would be done without making changes
        #[clap(long)]
        dry_run: bool,
    },
    /// Print embedded agent-facing documentation (workflows, commands, guides)
    Doc {
        #[clap(subcommand)]
        command: DocCommands,
    },
    /// Inspect the GraphQL API surface
    Schema {
        #[clap(subcommand)]
        command: SchemaCommands,
    },
    /// Talk to a running server over HTTP (info / call)
    Api {
        #[clap(subcommand)]
        command: ApiCommands,
    },
    /// Scan a project for structural health problems
    Doctor {
        /// Project id to scan
        #[clap(long)]
        project: i32,
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
        /// Emit machine-readable JSON
        #[clap(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DbCommands {
    Init {
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
    },
    Migrate {
        #[clap(subcommand)]
        direction: server::MigrateDirection,
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
    },
    /// Show the database file location and size (filesystem-only, safe while a
    /// server has the DB open)
    Info {
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
        /// Emit machine-readable JSON
        #[clap(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum GenerateCommands {
    Template { name: String },
    Sample { sample: String, dir: String },
}

#[derive(Subcommand, Debug)]
enum DocCommands {
    /// List all available docs (workflows, commands, guides)
    List,
    /// Print a workflow doc (docs-tool/workflow/<name>.md)
    Workflow { name: String },
    /// Print a command doc (docs-tool/command/<name>.md)
    Command { name: String },
    /// Print a reference guide (docs-tool/guide/<name>.md), e.g. agent, model
    Guide { name: String },
}

#[derive(Subcommand, Debug)]
enum SchemaCommands {
    /// Print the GraphQL schema (SDL by default; --json for introspection)
    Dump {
        /// Emit introspection JSON instead of SDL
        #[clap(long)]
        json: bool,
        /// Only the Mutation root type
        #[clap(long)]
        only_mutations: bool,
        /// Only input object types
        #[clap(long)]
        only_inputs: bool,
    },
    /// Print a single type's SDL (type / input / enum / …)
    Type {
        /// The type name, e.g. SequenceEdgeRefInput
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum ApiCommands {
    /// Print endpoints and headers for a running server
    Info {
        /// Full base URL (overrides --host/--port), e.g. http://127.0.0.1:3000
        #[clap(long)]
        url: Option<String>,
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        #[clap(short, long, default_value = "3000")]
        port: u16,
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
        #[clap(long)]
        json: bool,
    },
    /// POST a GraphQL operation to a running server and print the JSON response
    Call {
        /// The GraphQL query or mutation
        #[clap(long)]
        query: String,
        /// Variables as inline JSON or @path/to/file.json
        #[clap(long)]
        variables: Option<String>,
        /// Full base URL (overrides --host/--port)
        #[clap(long)]
        url: Option<String>,
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        #[clap(short, long, default_value = "3000")]
        port: u16,
        /// Value for the x-layercake-session header
        #[clap(long)]
        session: Option<String>,
    },
    /// Join a project's collaboration session so the caller appears as a user
    /// in the multi-user UI (holds the connection until Ctrl-C)
    Join {
        /// Project id to join
        #[clap(long)]
        project: i32,
        /// Display name shown to other collaborators
        #[clap(long)]
        name: String,
        /// Stable user id (defaults to an agent-<pid> id)
        #[clap(long)]
        id: Option<String>,
        /// Avatar colour (hex)
        #[clap(long)]
        color: Option<String>,
        /// Mark this presence as an agent (shows an Agent badge in the UI)
        #[clap(long)]
        agent: bool,
        /// Full base URL (overrides --host/--port)
        #[clap(long)]
        url: Option<String>,
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        #[clap(short, long, default_value = "3000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    setup_logging(&args.log_level);

    match args.command {
        Commands::Run { plan, watch } => {
            info!("Running plan: {}", plan);
            plan_execution::execute_plan(plan, watch)?;
        }
        Commands::Init { plan } => {
            info!("Initializing plan: {}", plan);
            let plan_file_path = plan;
            let plan = plan::Plan::default();
            let serialized_plan = serde_yaml::to_string(&plan)?;
            common::write_string_to_file(&plan_file_path, &serialized_plan)?;
        }
        Commands::Generate { command } => match command {
            GenerateCommands::Template { name } => {
                info!("Generating template: {}", name);
                generate_commands::generate_template(name);
            }
            GenerateCommands::Sample { sample, dir } => {
                info!("Generating sample: {} in {}", sample, dir);
                generate_commands::generate_sample(sample, dir);
            }
        },
        Commands::Repl(repl_args) => {
            repl::run_repl(repl_args).await?;
        }
        Commands::Query(query_args) => {
            query::run_query_command(query_args).await?;
        }
        Commands::Serve {
            host,
            port,
            database,
            cors_origin,
            open,
        } => {
            info!("Starting server on {}:{}", host, port);
            server::start_server(&host, port, &database, cors_origin.as_deref(), open).await?;
        }
        Commands::Db { command } => match command {
            DbCommands::Init { database } => {
                info!("Initializing database: {}", database);
                server::migrate_database(&database, server::MigrateDirection::Up).await?;
            }
            DbCommands::Migrate {
                direction,
                database,
            } => {
                info!("Running database migration: {:?}", direction);
                server::migrate_database(&database, direction).await?;
            }
            DbCommands::Info { database, json } => {
                db_info::run(&database, json)?;
            }
        },
        #[cfg(feature = "console")]
        Commands::Console { database } => {
            console::run_console(console::ConsoleOptions { database }).await?;
        }
        Commands::Update {
            check,
            force,
            pre_release,
            backup,
            rollback,
            dry_run,
        } => {
            let update_cmd = update::command::UpdateCommand {
                check_only: check,
                force,
                pre_release,
                version: None,
                install_dir: None,
                backup,
                rollback,
                dry_run,
                skip_verify: false,
            };
            update_cmd.execute().await?;
        }
        Commands::Doc { command } => match command {
            DocCommands::List => doc::print_list(),
            DocCommands::Workflow { name } => doc::print_doc("workflow", &name)?,
            DocCommands::Command { name } => doc::print_doc("command", &name)?,
            DocCommands::Guide { name } => doc::print_doc("guide", &name)?,
        },
        Commands::Schema { command } => match command {
            SchemaCommands::Dump {
                json,
                only_mutations,
                only_inputs,
            } => schema_dump::dump(json, only_mutations, only_inputs).await?,
            SchemaCommands::Type { name } => schema_dump::print_type(&name)?,
        },
        Commands::Doctor {
            project,
            database,
            json,
        } => {
            doctor::run(project, Some(&database), json).await?;
        }
        Commands::Api { command } => match command {
            ApiCommands::Info {
                url,
                host,
                port,
                database,
                json,
            } => api::info(url.as_deref(), &host, port, &database, json).await?,
            ApiCommands::Call {
                query,
                variables,
                url,
                host,
                port,
                session,
            } => {
                api::call(
                    &query,
                    variables.as_deref(),
                    url.as_deref(),
                    &host,
                    port,
                    session.as_deref(),
                )
                .await?
            }
            ApiCommands::Join {
                project,
                name,
                id,
                color,
                agent,
                url,
                host,
                port,
            } => {
                api::join(
                    project,
                    name,
                    id,
                    color,
                    agent,
                    url.as_deref(),
                    &host,
                    port,
                )
                .await?
            }
        },
    }

    Ok(())
}

fn setup_logging(log_level: &Option<String>) {
    let log_level = match log_level
        .as_ref()
        .unwrap_or(&"info".to_string())
        .to_lowercase()
        .as_str()
    {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("handlebars=off,{}", log_level)))
        .without_time()
        .init();
}
