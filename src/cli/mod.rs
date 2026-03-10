use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ws", about = "Manage related git worktrees in a workspace")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new workspace with worktrees for repos defined in a template
    New {
        /// Name for the new workspace (e.g. feature branch name)
        name: String,

        /// Template to use (auto-selected if only one exists)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Add a repo worktree to the current workspace
    Add {
        /// Path to the repo to add
        repo: PathBuf,
    },

    /// Switch to an existing workspace
    Switch {
        /// Name of the workspace to switch to (opens TUI picker if omitted)
        name: Option<String>,
    },

    /// List all workspaces
    #[command(alias = "ls")]
    List,

    /// Remove a workspace and its worktrees
    #[command(alias = "rm")]
    Remove {
        /// Name of the workspace to remove (detected from current directory if omitted)
        name: Option<String>,

        /// Template to use for worktree cleanup (auto-selected if only one exists)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Manage templates
    #[command(subcommand)]
    Template(TemplateCommands),

    /// Print shell integration (eval "$(ws shell-init)")
    ShellInit,

    /// Output completions for shell integration (hidden)
    #[command(hide = true)]
    Complete {
        /// What to complete: "workspaces" or "templates"
        kind: String,
    },

    /// Show or update configuration
    Config {
        /// Set the workspace directory
        #[arg(long)]
        workspace_dir: Option<PathBuf>,

        /// Enable or disable Claude config generation in new workspaces
        #[arg(long)]
        generate_claude_config: Option<bool>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommands {
    /// List all templates
    #[command(alias = "ls")]
    List,

    /// Add a new template or add repos to an existing one
    Add {
        /// Template name
        name: String,

        /// Repo paths to include
        #[arg(short, long, required = true)]
        repo: Vec<PathBuf>,
    },

    /// Remove a template or remove repos from one
    #[command(alias = "rm")]
    Remove {
        /// Template name
        name: String,

        /// Remove specific repos (removes entire template if omitted)
        #[arg(short, long)]
        repo: Vec<PathBuf>,
    },

    /// Show details of a template
    Show {
        /// Template name
        name: String,
    },
}
