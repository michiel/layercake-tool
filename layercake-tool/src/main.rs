mod common;
mod data_loader;
mod db;
mod export;
mod generate_commands;
mod graph;
mod graph_utils;
mod graphql_server;
mod plan;
mod plan_execution;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, error};
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
        #[clap(long, default_value = "layercake.db")]
        db_path: String,
    },
    Db {
        #[clap(subcommand)]
        command: DbCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DbCommands {
    Init { 
        #[clap(short, long, default_value = "layercake.db")]
        path: String 
    },
    Reset {
        #[clap(short, long, default_value = "layercake.db")]
        path: String
    },
    Import {
        #[clap(short, long)]
        plan: String,
        #[clap(short, long, default_value = "layercake.db")]
        db_path: String,
        #[clap(short, long, default_value = "Imported Project")]
        name: String,
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
        Commands::Serve { plan, port, persist, db_path } => {
            info!("Starting GraphQL server with plan: {}", plan);
            
            // Use memory if not persisting, otherwise use the provided db_path
            let db_path = if persist {
                db_path
            } else {
                ":memory:".to_string()
            };
            
            // Load the graph from the plan
            let plan_file_path = std::path::Path::new(&plan);
            let path_content = std::fs::read_to_string(plan_file_path)?;
            let plan_data: Plan = serde_yaml::from_str(&path_content)?;
            
            // Create graph and load data
            let mut graph = graph_utils::create_graph_from_plan(&plan_data);
            graph_utils::load_data_into_graph(&mut graph, &plan_data, plan_file_path)?;
            
            // Start the GraphQL server
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                graphql_server::serve_graph(plan_data, graph, port, &db_path).await
            })?;
        },
        Commands::Db { command } => match command {
            DbCommands::Init { path } => {
                info!("Initializing database at {}", path);
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(async {
                    match db::establish_connection(&path).await {
                        Ok(_) => {
                            info!("Database initialized successfully");
                            Ok(())
                        },
                        Err(e) => {
                            error!("Failed to initialize database: {}", e);
                            Err(anyhow::anyhow!("Database initialization failed"))
                        }
                    }
                })?;
            },
            DbCommands::Reset { path } => {
                info!("Resetting database at {}", path);
                // Delete the database file if it exists
                if std::path::Path::new(&path).exists() {
                    std::fs::remove_file(&path)?;
                    info!("Database file deleted");
                }
                
                // Reinitialize the database
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(async {
                    match db::establish_connection(&path).await {
                        Ok(_) => {
                            info!("Database reinitialized successfully");
                            Ok(())
                        },
                        Err(e) => {
                            error!("Failed to reinitialize database: {}", e);
                            Err(anyhow::anyhow!("Database reinitialization failed"))
                        }
                    }
                })?;
            },
            DbCommands::Import { plan, db_path, name } => {
                info!("Importing plan {} into database {}", plan, db_path);
                
                // Load the plan
                let plan_file_path = std::path::Path::new(&plan);
                let path_content = std::fs::read_to_string(plan_file_path)?;
                let plan_data: crate::plan::Plan = serde_yaml::from_str(&path_content)?;
                
                // Create graph from plan
                let mut graph = graph_utils::create_graph_from_plan(&plan_data);
                graph_utils::load_data_into_graph(&mut graph, &plan_data, plan_file_path)?;
                
                // Import into database
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(async {
                    // Connect to database
                    let db = db::establish_connection(&db_path).await?;
                    let repo = db::repository::ProjectRepository::new(db);
                    
                    // Create project
                    match repo.create_project(&name, Some(&format!("Imported from {}", plan)), &plan_data, &graph).await {
                        Ok(project_id) => {
                            info!("Project imported successfully with ID {}", project_id);
                            Ok(())
                        },
                        Err(e) => {
                            error!("Failed to import project: {}", e);
                            Err(anyhow::anyhow!("Project import failed"))
                        }
                    }
                })?;
            }
        }
    }

    Ok(())
}
