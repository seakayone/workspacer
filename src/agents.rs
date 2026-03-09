use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Template;

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

    eprintln!("Generated AGENTS.md and CLAUDE.md symlink");
    Ok(())
}
