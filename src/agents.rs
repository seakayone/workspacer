use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::Template;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ClaudeSettings {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    additional_directories: Vec<PathBuf>,
}

pub fn generate(workspace_dir: &Path, workspace_name: &str, template: &Template) -> Result<()> {
    let agents_path = workspace_dir.join("AGENTS.md");
    let claude_path = workspace_dir.join("CLAUDE.md");

    let mut content = format!("# Workspace: {workspace_name}\n\n");
    content.push_str("## Repos\n\n");

    for repo in &template.repos {
        let repo_name = repo
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        content.push_str(&format!("- `{repo_name}/` — {}\n", repo.display()));
    }

    content.push_str(&format!(
        "\n## Working across repos\n\n\
         All repos are checked out on branch `{workspace_name}`.\n\
         Make changes across repos as needed for the task.\n"
    ));

    fs::write(&agents_path, &content)
        .with_context(|| format!("failed to write {}", agents_path.display()))?;

    // Create CLAUDE.md as a symlink to AGENTS.md
    if claude_path.exists() || claude_path.is_symlink() {
        fs::remove_file(&claude_path).ok();
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink("AGENTS.md", &claude_path)
        .with_context(|| format!("failed to symlink {}", claude_path.display()))?;

    #[cfg(not(unix))]
    fs::copy(&agents_path, &claude_path)
        .with_context(|| format!("failed to copy to {}", claude_path.display()))?;

    generate_claude_settings(workspace_dir, template)?;

    eprintln!("Generated AGENTS.md, CLAUDE.md symlink, and .claude/settings.local.json");
    Ok(())
}

fn repo_directories(workspace_dir: &Path, template: &Template) -> Vec<PathBuf> {
    template
        .repos
        .iter()
        .filter_map(|repo| {
            repo.file_name()
                .and_then(|n| n.to_str())
                .map(|name| workspace_dir.join(name))
        })
        .collect()
}

fn generate_claude_settings(workspace_dir: &Path, template: &Template) -> Result<()> {
    let claude_dir = workspace_dir.join(".claude");
    fs::create_dir_all(&claude_dir)
        .with_context(|| format!("failed to create {}", claude_dir.display()))?;

    let settings_path = claude_dir.join("settings.local.json");
    let settings = ClaudeSettings {
        additional_directories: repo_directories(workspace_dir, template),
    };

    let contents = serde_json::to_string_pretty(&settings)
        .context("failed to serialize Claude settings")?;
    fs::write(&settings_path, contents)
        .with_context(|| format!("failed to write {}", settings_path.display()))?;

    Ok(())
}

/// Append a repo entry to an existing AGENTS.md.
pub fn add_repo(workspace_dir: &Path, repo: &Path) -> Result<()> {
    let agents_path = workspace_dir.join("AGENTS.md");
    if !agents_path.exists() {
        return Ok(());
    }

    let repo_name = repo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let entry = format!("- `{repo_name}/` — {}\n", repo.display());

    let content = fs::read_to_string(&agents_path)
        .with_context(|| format!("failed to read {}", agents_path.display()))?;

    if content.contains(&format!("`{repo_name}/`")) {
        return Ok(());
    }

    // Insert before the "## Working across repos" section, or append to end
    let new_content = if let Some(pos) = content.find("\n## Working across repos") {
        let (before, after) = content.split_at(pos);
        format!("{before}{entry}{after}")
    } else {
        format!("{content}{entry}")
    };

    fs::write(&agents_path, &new_content)
        .with_context(|| format!("failed to write {}", agents_path.display()))?;

    eprintln!("Updated AGENTS.md with {repo_name}");

    add_repo_to_claude_settings(workspace_dir, repo)?;

    Ok(())
}

fn add_repo_to_claude_settings(workspace_dir: &Path, repo: &Path) -> Result<()> {
    let claude_dir = workspace_dir.join(".claude");
    let settings_path = claude_dir.join("settings.local.json");

    let mut settings = if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path)
            .with_context(|| format!("failed to read {}", settings_path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse {}", settings_path.display()))?
    } else {
        fs::create_dir_all(&claude_dir)
            .with_context(|| format!("failed to create {}", claude_dir.display()))?;
        ClaudeSettings::default()
    };

    let repo_name = repo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let repo_dir = workspace_dir.join(repo_name);

    if !settings.additional_directories.contains(&repo_dir) {
        settings.additional_directories.push(repo_dir);
        let contents = serde_json::to_string_pretty(&settings)
            .context("failed to serialize Claude settings")?;
        fs::write(&settings_path, contents)
            .with_context(|| format!("failed to write {}", settings_path.display()))?;
        eprintln!("Updated .claude/settings.local.json with {repo_name}");
    }

    Ok(())
}
