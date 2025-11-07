#![cfg(feature = "console")]

//! Interactive console subsystem providing the `layercake console` REPL.
//!
//! The console currently exposes basic project and graph navigation commands
//! together with a conversational `chat` command that integrates with the MCP
//! tooling surface. The REPL is powered by `clap-repl`, reusing the same clap
//! syntax the CLI already understands.

mod commands;
mod context;
mod output;
mod prompt;

pub mod chat;

use anyhow::Result;
use clap::Parser;
use clap_repl::{ClapEditor, ReadCommandOutput};
use commands::{ConsoleCommand, ConsoleReplCommand};
use context::ConsoleContext;
use output::print_banner;
use prompt::ConsolePrompt;

use crate::database::{
    connection::{establish_connection, get_database_url},
    migrations::Migrator,
};
use sea_orm_migration::MigratorTrait;

/// CLI options for launching the console REPL.
#[derive(Debug, Clone, Parser)]
pub struct ConsoleOptions {
    /// Path to the sqlite database. Defaults to `layercake.db`.
    #[arg(long)]
    pub database: Option<String>,
}

/// Entry point invoked from the CLI `console` subcommand.
pub async fn run_console(options: ConsoleOptions) -> Result<()> {
    let database_url = get_database_url(options.database.as_deref());
    let db = establish_connection(&database_url).await?;

    // Ensure migrations are applied so the console has all necessary tables.
    Migrator::up(&db, None).await?;

    let mut context = ConsoleContext::bootstrap(db).await?;
    print_banner();

    let mut editor = ClapEditor::<ConsoleReplCommand>::builder()
        .with_prompt(Box::new(ConsolePrompt::from(&context)))
        .build();

    loop {
        match editor.read_command() {
            ReadCommandOutput::Command(cmd) => {
                if let Err(err) = dispatch_command(&mut context, cmd.command).await {
                    eprintln!("error: {err:?}");
                }
                editor.set_prompt(Box::new(ConsolePrompt::from(&context)));
            }
            ReadCommandOutput::EmptyLine => {}
            ReadCommandOutput::ClapError(err) => {
                err.print()?;
            }
            ReadCommandOutput::ShlexError => {
                eprintln!("error: unable to parse input");
            }
            ReadCommandOutput::ReedlineError(err) => {
                eprintln!("fatal: {err}");
                break;
            }
            ReadCommandOutput::CtrlC => {
                println!("(interrupt)");
            }
            ReadCommandOutput::CtrlD => {
                println!("Goodbye!");
                break;
            }
        }
    }

    Ok(())
}

async fn dispatch_command(context: &mut ConsoleContext, command: ConsoleCommand) -> Result<()> {
    use ConsoleCommand::*;

    match command {
        ListProjects => context.list_projects().await?,
        UseProject { project_id } => context.select_project(project_id).await?,
        ListGraphs { project } => context.list_graphs(project).await?,
        ShowGraph { graph_id } => context.show_graph(graph_id).await?,
        Chat { provider } => context.start_chat(provider).await?,
        Settings { command } => context.handle_settings_command(command).await?,
        Exit => {
            println!("Type Ctrl+D to exit the console.");
        }
    }

    Ok(())
}
