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

    if config.generate_agents_md {
        agents::generate(&ws_dir, name, template)?;
    }

    eprintln!("Created workspace: {}", ws_dir.display());
    Ok(ws_dir)
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
