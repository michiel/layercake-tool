mod common;
mod data_loader;
mod export;
mod generate_commands;
mod graph;
mod graphql_server;
mod plan;
mod plan_execution;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::plan::Plan;

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
        #[clap(short, long)]
        plan: String,
        #[clap(long, default_value = "3000")]
        port: u16,
        #[clap(long)]
        persist: bool,
    },
}

#[derive(Subcommand, Debug)]
enum GenerateCommands {
    Template { name: String },
    Sample { sample: String, dir: String },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let log_level = match args
        .log_level
        .unwrap_or("info".to_string())
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

    // tracing_subscriber::fmt().with_max_level(log_level).init();
    tracing_subscriber::fmt()
        // .with_max_level(log_level)
        .with_env_filter(
            EnvFilter::new(format!("handlebars=off,{}", log_level)), // Exclude handlebars logs
        )
        .without_time() // This line removes the timestamp from the logging output
        .init();

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
            plan,
            port,
            persist,
        } => {
            info!("Starting GraphQL server with plan: {}", plan);

            // Load the graph from the plan
            let plan_file_path = std::path::Path::new(&plan);
            let path_content = std::fs::read_to_string(plan_file_path)?;
            let plan: Plan = serde_yaml::from_str(&path_content)?;

            // Create graph and load data
            let mut graph = plan_execution::create_graph_from_plan(&plan);
            plan_execution::load_data_into_graph(&mut graph, &plan, plan_file_path)?;

            // Start the GraphQL server
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async { graphql_server::serve_graph(graph, port, persist).await })?;
        }
    }

    Ok(())
}
