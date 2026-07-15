use anyhow::Result;
use clap::{Parser, Subcommand};
use layercake_code_analysis::cli::CodeAnalysisArgs;
use tracing::info;
use tracing::Level;
use tracing_subscriber::EnvFilter;

use layercake_core::{common, generate_commands, plan, plan_execution, update};
use layercake_server::server;

#[cfg(feature = "console")]
mod console;

mod doc;
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
    #[clap(alias = "ca")]
    CodeAnalysis(CodeAnalysisArgs),
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
    /// Output documentation guides
    Guide {
        #[clap(subcommand)]
        command: GuideCommands,
    },
    /// Print embedded agent-facing documentation (workflows and commands)
    Doc {
        #[clap(subcommand)]
        command: DocCommands,
    },
    /// Inspect the GraphQL API surface
    Schema {
        #[clap(subcommand)]
        command: SchemaCommands,
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
}

#[derive(Subcommand, Debug)]
enum GenerateCommands {
    Template { name: String },
    Sample { sample: String, dir: String },
}

#[derive(Subcommand, Debug)]
enum GuideCommands {
    /// Output the AI agent guide for the query interface
    Agent,
    /// Output the graph model documentation
    Model,
}

#[derive(Subcommand, Debug)]
enum DocCommands {
    /// List all available workflow and command docs
    List,
    /// Print a workflow doc (docs-tool/workflow/<name>.md)
    Workflow { name: String },
    /// Print a command doc (docs-tool/command/<name>.md)
    Command { name: String },
}

#[derive(Subcommand, Debug)]
enum SchemaCommands {
    /// Print the GraphQL schema (SDL by default; --json for introspection)
    Dump {
        /// Emit introspection JSON instead of SDL
        #[clap(long)]
        json: bool,
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
        },
        Commands::CodeAnalysis(args) => {
            layercake_code_analysis::cli::run(args)?;
        }
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
        Commands::Guide { command } => match command {
            GuideCommands::Agent => {
                const AGENT_GUIDE: &str = include_str!("../../LAYERCAKE_AGENT_GUIDE.md");
                print!("{}", AGENT_GUIDE);
            }
            GuideCommands::Model => {
                const MODEL_GUIDE: &str = include_str!("../../LAYERCAKE_MODEL_GUIDE.md");
                print!("{}", MODEL_GUIDE);
            }
        },
        Commands::Doc { command } => match command {
            DocCommands::List => doc::print_list(),
            DocCommands::Workflow { name } => doc::print_doc("workflow", &name)?,
            DocCommands::Command { name } => doc::print_doc("command", &name)?,
        },
        Commands::Schema { command } => match command {
            SchemaCommands::Dump { json } => schema_dump::dump(json).await?,
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
