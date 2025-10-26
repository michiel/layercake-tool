mod common;
mod data_loader;
mod export;
mod generate_commands;
mod graph;
mod pipeline;
mod plan;
mod plan_execution;
mod update;

mod database;
mod server;
mod services;
mod utils;

#[cfg(feature = "graphql")]
mod graphql;

#[cfg(feature = "graphql")]
mod collaboration;

#[cfg(feature = "mcp")]
mod mcp;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing::Level;
use tracing_subscriber::EnvFilter;

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
    Serve {
        #[clap(short, long, default_value = "3000")]
        port: u16,
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
        #[clap(long)]
        cors_origin: Option<String>,
    },
    Db {
        #[clap(subcommand)]
        command: DbCommands,
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
        Commands::Serve {
            port,
            database,
            cors_origin,
        } => {
            info!("Starting server on port {}", port);
            server::start_server(port, &database, cors_origin.as_deref()).await?;
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
