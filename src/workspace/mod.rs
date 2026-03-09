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

pub fn create(config: &Config, name: &str, template: &Template) -> Result<PathBuf> {
    let ws_dir = workspace_dir(config).join(name);
    fs::create_dir_all(&ws_dir)
        .with_context(|| format!("failed to create workspace dir {}", ws_dir.display()))?;

    for repo in &template.repos {
        println!("Creating worktree for {} ...", repo.display());
        let status = Command::new("wt")
            .args(["switch", "--create", name])
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

    println!("Created workspace: {}", ws_dir.display());
    Ok(ws_dir)
}

pub fn remove(config: &Config, name: &str, template: &Template) -> Result<()> {
    for repo in &template.repos {
        println!("Removing worktree for {} ...", repo.display());
        let status = Command::new("wt")
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

    println!("Removed workspace: {name}");
    Ok(())
}
