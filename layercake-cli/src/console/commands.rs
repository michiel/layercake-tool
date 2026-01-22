#![cfg(feature = "console")]

use clap::{Parser, Subcommand};

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
    /// Manage runtime system settings.
    #[command(name = "settings")]
    Settings {
        #[command(subcommand)]
        command: SettingsCommand,
    },
    /// Print exit instructions.
    #[command(name = "exit")]
    Exit,
}

/// Nested commands under `settings`.
#[derive(Subcommand, Debug)]
pub enum SettingsCommand {
    /// List all configurable settings
    #[command(name = "list")]
    List,
    /// Show details for a specific setting
    #[command(name = "show")]
    Show {
        /// Setting key (e.g. PROJECT_DEFAULT_LAYER)
        key: String,
    },
    /// Update a setting value
    #[command(name = "set")]
    Set {
        /// Setting key (e.g. PROJECT_DEFAULT_LAYER)
        key: String,
        /// New value to persist
        value: String,
    },
}
