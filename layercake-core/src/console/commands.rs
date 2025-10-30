#![cfg(feature = "console")]

use clap::{Parser, Subcommand};

use super::chat::ChatProvider;

/// Top-level parser executed for each REPL input.
#[derive(Parser, Debug)]
#[command(
    name = "",
    disable_help_flag = false,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
pub struct ConsoleReplCommand {
    #[command(subcommand)]
    pub command: ConsoleCommand,
}

/// Supported commands within the console REPL.
#[derive(Subcommand, Debug)]
pub enum ConsoleCommand {
    /// List all known projects.
    #[command(name = "list-projects")]
    ListProjects,
    /// Select a project and update the prompt context.
    #[command(name = "use-project")]
    UseProject {
        /// Project identifier.
        project_id: i32,
    },
    /// List graphs for the active project (or an explicit one).
    #[command(name = "list-graphs")]
    ListGraphs {
        /// Optional explicit project identifier.
        #[arg(long)]
        project: Option<i32>,
    },
    /// Show a summary of a specific graph.
    #[command(name = "show-graph")]
    ShowGraph {
        /// Graph identifier.
        graph_id: i32,
    },
    /// Start an interactive chat session.
    #[command(name = "chat")]
    Chat {
        /// Override provider just for this chat session.
        #[arg(long)]
        provider: Option<ChatProvider>,
    },
    /// Print exit instructions.
    #[command(name = "exit")]
    Exit,
}
