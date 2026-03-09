use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::config::{Config, Template};

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

fn wt_command(config: &Config) -> Command {
    let mut cmd = Command::new("wt");
    cmd.env("WORKTRUNK_DIRECTIVE_FILE", "/dev/null")
        .env("WORKTRUNK_WORKTREE_PATH", config.worktree_path_template());
    cmd
}

pub fn create(config: &Config, name: &str, template: &Template) -> Result<PathBuf> {
    let ws_dir = workspace_dir(config).join(name);

    for repo in &template.repos {
        eprintln!("Creating worktree for {} ...", repo.display());
        let status = wt_command(config)
            .args(["switch", "--create", "--no-cd", name])
            .arg("-C")
            .arg(repo)
            .status()
            .with_context(|| {
                format!(
                    "failed to run `wt switch --create {}` in {}",
                    name,
                    repo.display()
                )
            })?;

        if !status.success() {
            anyhow::bail!(
                "wt switch --create {} failed in {} (exit code: {:?})",
                name,
                repo.display(),
                status.code()
            );
        }
    }

    eprintln!("Created workspace: {}", ws_dir.display());
    Ok(ws_dir)
}

pub fn remove(config: &Config, name: &str, template: &Template) -> Result<()> {
    for repo in &template.repos {
        eprintln!("Removing worktree for {} ...", repo.display());
        let status = wt_command(config)
            .args(["remove", name])
            .arg("-C")
            .arg(repo)
            .status()
            .with_context(|| {
                format!(
                    "failed to run `wt remove {}` in {}",
                    name,
                    repo.display()
                )
            })?;

        if !status.success() {
            eprintln!(
                "warning: wt remove {} failed in {} (exit code: {:?})",
                name,
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
