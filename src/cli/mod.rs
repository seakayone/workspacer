use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Color, Styles};
use clap::{ColorChoice, Parser, Subcommand};

fn help_styles() -> Styles {
    Styles::styled()
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
}

#[derive(Debug, Parser)]
#[command(name = "ws", about = "Manage related git worktrees in a workspace", color = ColorChoice::Always, styles = help_styles())]
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

    /// Manage repos in a workspace
    #[command(subcommand)]
    Repo(RepoCommands),

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

    /// Manage agent state for a workspace
    #[command(subcommand)]
    Agent(AgentCommands),

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

#[derive(Debug, Subcommand)]
pub enum RepoCommands {
    /// Add a repo worktree to the current workspace
    Add {
        /// Path to the repo to add
        repo: PathBuf,
    },

    /// Remove a repo worktree from the current workspace
    #[command(alias = "rm")]
    Remove {
        /// Name of the repo directory to remove
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentCommands {
    /// Set the agent state marker (e.g. 🤖 or 💬)
    Set {
        /// The marker to display (e.g. 🤖)
        marker: String,
    },

    /// Clear the agent state marker
    Clear,
}
