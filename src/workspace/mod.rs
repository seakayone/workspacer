use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::agents;
use crate::config::{Config, Template};

pub fn branch_name(workspace: &str) -> String {
    workspace.to_string()
}

pub fn workspace_dir(config: &Config) -> PathBuf {
    config.workspace_dir.clone()
}

pub fn list(config: &Config) -> Result<Vec<String>> {
    let dir = workspace_dir(config);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut workspaces = Vec::new();
    for entry in
        fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    workspaces.push(name.to_string());
                }
            }
        }
    }
    workspaces.sort();
    Ok(workspaces)
}

fn wt_command(config: &Config, workspace: &str) -> Command {
    let mut cmd = Command::new("wt");
    cmd.env("WORKTRUNK_DIRECTIVE_FILE", "/dev/null")
        .env(
            "WORKTRUNK_WORKTREE_PATH",
            config.worktree_path_template(workspace),
        );
    cmd
}

pub fn create(config: &Config, name: &str, template: &Template) -> Result<PathBuf> {
    let branch = branch_name(name);
    let ws_dir = workspace_dir(config).join(name);

    for repo in &template.repos {
        eprintln!("Creating worktree for {} ...", repo.display());
        let status = wt_command(config, name)
            .args(["switch", "--create", "--no-cd", &branch])
            .arg("-C")
            .arg(repo)
            .status()
            .with_context(|| {
                format!(
                    "failed to run `wt switch --create {}` in {}",
                    branch,
                    repo.display()
                )
            })?;

        if !status.success() {
            anyhow::bail!(
                "wt switch --create {} failed in {} (exit code: {:?})",
                branch,
                repo.display(),
                status.code()
            );
        }
    }

    if config.generate_claude_config {
        agents::generate(&ws_dir, name, template)?;
    }

    eprintln!("Created workspace: {}", ws_dir.display());
    Ok(ws_dir)
}

/// Detect the current workspace name from a directory path.
/// Returns the workspace name if the path is inside the workspace_dir.
pub fn detect_workspace(config: &Config, cwd: &std::path::Path) -> Result<String> {
    let ws_root = workspace_dir(config);
    let relative = cwd
        .strip_prefix(&ws_root)
        .with_context(|| format!("current directory is not inside workspace dir {}", ws_root.display()))?;
    let name = relative
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .with_context(|| "could not determine workspace name from current directory")?;
    Ok(name.to_string())
}

pub fn add_repo(config: &Config, name: &str, repo: &std::path::Path) -> Result<()> {
    let branch = branch_name(name);
    let repo = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());

    eprintln!("Adding worktree for {} to workspace {name} ...", repo.display());
    let status = wt_command(config, name)
        .args(["switch", "--create", "--no-cd", &branch])
        .arg("-C")
        .arg(&repo)
        .status()
        .with_context(|| {
            format!(
                "failed to run `wt switch --create {}` in {}",
                branch,
                repo.display()
            )
        })?;

    if !status.success() {
        anyhow::bail!(
            "wt switch --create {} failed in {} (exit code: {:?})",
            branch,
            repo.display(),
            status.code()
        );
    }

    if config.generate_claude_config {
        let ws_dir = workspace_dir(config).join(name);
        agents::add_repo(&ws_dir, &repo)?;
    }

    eprintln!("Added {} to workspace {name}", repo.display());
    Ok(())
}

/// List repo directories inside a workspace.
pub fn list_repos(config: &Config, workspace: &str) -> Result<Vec<String>> {
    let ws_dir = workspace_dir(config).join(workspace);
    if !ws_dir.exists() {
        anyhow::bail!("workspace '{}' does not exist", workspace);
    }
    let mut repos = Vec::new();
    for entry in
        fs::read_dir(&ws_dir).with_context(|| format!("failed to read {}", ws_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') {
                    repos.push(name.to_string());
                }
            }
        }
    }
    repos.sort();
    Ok(repos)
}

/// Remove a single repo worktree from a workspace.
pub fn remove_repo(config: &Config, workspace: &str, repo_name: &str) -> Result<()> {
    let branch = branch_name(workspace);
    let ws_dir = workspace_dir(config).join(workspace);
    let repo_dir = ws_dir.join(repo_name);

    if !repo_dir.exists() {
        anyhow::bail!(
            "repo '{}' not found in workspace '{}'",
            repo_name,
            workspace
        );
    }

    eprintln!("Removing worktree for {repo_name} from workspace {workspace} ...");
    let status = wt_command(config, workspace)
        .args(["remove", &branch])
        .arg("-C")
        .arg(&repo_dir)
        .status()
        .with_context(|| {
            format!(
                "failed to run `wt remove {}` in {}",
                branch,
                repo_dir.display()
            )
        })?;

    if !status.success() {
        eprintln!(
            "warning: wt remove {} failed in {} (exit code: {:?})",
            branch,
            repo_dir.display(),
            status.code()
        );
    }

    // Remove the directory if it still exists
    if repo_dir.exists() {
        fs::remove_dir_all(&repo_dir)
            .with_context(|| format!("failed to remove {}", repo_dir.display()))?;
    }

    if config.generate_claude_config {
        agents::remove_repo(&ws_dir, repo_name)?;
    }

    eprintln!("Removed {repo_name} from workspace {workspace}");
    Ok(())
}

const AGENT_MARKER_FILE: &str = "agent-marker";
const CONFIG_DIR: &str = ".config/workspacer";

fn agent_marker_path(config: &Config, name: &str) -> PathBuf {
    workspace_dir(config).join(name).join(CONFIG_DIR).join(AGENT_MARKER_FILE)
}

/// Read the agent state marker for a workspace, if any.
pub fn agent_marker(config: &Config, name: &str) -> Option<String> {
    fs::read_to_string(agent_marker_path(config, name))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Set the agent state marker for a workspace.
pub fn set_agent_marker(config: &Config, name: &str, marker: &str) -> Result<()> {
    let path = agent_marker_path(config, name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, marker).with_context(|| format!("failed to write {}", path.display()))
}

/// Clear the agent state marker for a workspace.
pub fn clear_agent_marker(config: &Config, name: &str) -> Result<()> {
    let path = agent_marker_path(config, name);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))?;
    }
    Ok(())
}

pub fn remove(config: &Config, name: &str, template: &Template) -> Result<()> {
    let branch = branch_name(name);
    for repo in &template.repos {
        eprintln!("Removing worktree for {} ...", repo.display());
        let status = wt_command(config, name)
            .args(["remove", &branch])
            .arg("-C")
            .arg(repo)
            .status()
            .with_context(|| {
                format!(
                    "failed to run `wt remove {}` in {}",
                    branch,
                    repo.display()
                )
            })?;

        if !status.success() {
            eprintln!(
                "warning: wt remove {} failed in {} (exit code: {:?})",
                branch,
                repo.display(),
                status.code()
            );
        }
    }

    let dir = workspace_dir(config).join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .with_context(|| format!("failed to remove workspace dir {}", dir.display()))?;
    }

    eprintln!("Removed workspace: {name}");
    Ok(())
}
