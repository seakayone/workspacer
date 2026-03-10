use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use workspacer::agents;
use workspacer::config::{Config, Template};
use workspacer::workspace;

fn empty_config(tmp: &TempDir) -> Config {
    Config {
        workspace_dir: tmp.path().to_path_buf(),
        templates: BTreeMap::new(),
        generate_claude_config: false,
    }
}

fn config_with_template(tmp: &TempDir, repos: Vec<PathBuf>) -> Config {
    let mut templates = BTreeMap::new();
    templates.insert("default".to_string(), Template { repos });
    Config {
        workspace_dir: tmp.path().to_path_buf(),
        templates,
        generate_claude_config: false,
    }
}

// --- list ---

#[test]
fn list_empty_workspace_dir() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);
    let result = workspace::list(&config).unwrap();
    assert!(result.is_empty());
}

#[test]
fn list_nonexistent_workspace_dir() {
    let config = Config {
        workspace_dir: "/tmp/workspacer_does_not_exist_12345".into(),
        templates: BTreeMap::new(),
        generate_claude_config: false,
    };
    let result = workspace::list(&config).unwrap();
    assert!(result.is_empty());
}

#[test]
fn list_returns_sorted_directories() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);

    fs::create_dir(tmp.path().join("charlie")).unwrap();
    fs::create_dir(tmp.path().join("alpha")).unwrap();
    fs::create_dir(tmp.path().join("bravo")).unwrap();

    let workspaces = workspace::list(&config).unwrap();
    assert_eq!(workspaces, vec!["alpha", "bravo", "charlie"]);
}

#[test]
fn list_excludes_hidden_directories() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);

    fs::create_dir(tmp.path().join("visible")).unwrap();
    fs::create_dir(tmp.path().join(".hidden")).unwrap();
    fs::create_dir(tmp.path().join(".also-hidden")).unwrap();

    let workspaces = workspace::list(&config).unwrap();
    assert_eq!(workspaces, vec!["visible"]);
}

// --- config serialization ---

#[test]
fn config_serialization_roundtrip() {
    let config = config_with_template(
        &TempDir::new().unwrap(),
        vec!["/repo/one".into(), "/repo/two".into()],
    );

    let toml_str = toml::to_string_pretty(&config).unwrap();
    let loaded: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(loaded.templates.len(), 1);
    let tmpl = loaded.templates.get("default").unwrap();
    assert_eq!(tmpl.repos.len(), 2);
    assert_eq!(tmpl.repos[0].to_str().unwrap(), "/repo/one");
}

#[test]
fn config_deserializes_without_templates_field() {
    let toml_str = r#"workspace_dir = "/some/path""#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.templates.is_empty());
}

// --- worktree path template ---

#[test]
fn worktree_path_template_uses_workspace_dir() {
    let config = Config {
        workspace_dir: "/my/workspaces".into(),
        templates: BTreeMap::new(),
        generate_claude_config: false,
    };
    let tmpl = config.worktree_path_template("feature-a");
    assert_eq!(tmpl, "/my/workspaces/feature-a/{{ repo }}");
}

// --- resolve_template ---

#[test]
fn resolve_template_auto_selects_single() {
    let tmp = TempDir::new().unwrap();
    let config = config_with_template(&tmp, vec!["/repo".into()]);

    let (name, tmpl) = config.resolve_template(None).unwrap();
    assert_eq!(name, "default");
    assert_eq!(tmpl.repos.len(), 1);
}

#[test]
fn resolve_template_by_name() {
    let tmp = TempDir::new().unwrap();
    let config = config_with_template(&tmp, vec!["/repo".into()]);

    let (name, _) = config.resolve_template(Some("default")).unwrap();
    assert_eq!(name, "default");
}

#[test]
fn resolve_template_fails_when_empty() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);

    let result = config.resolve_template(None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no templates"));
}

#[test]
fn resolve_template_fails_when_ambiguous() {
    let tmp = TempDir::new().unwrap();
    let mut templates = BTreeMap::new();
    templates.insert("a".into(), Template { repos: vec![] });
    templates.insert("b".into(), Template { repos: vec![] });
    let config = Config {
        workspace_dir: tmp.path().to_path_buf(),
        templates,
        generate_claude_config: false,
    };

    let result = config.resolve_template(None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("--template"));
}

#[test]
fn resolve_template_fails_for_unknown_name() {
    let tmp = TempDir::new().unwrap();
    let config = config_with_template(&tmp, vec![]);

    let result = config.resolve_template(Some("nope"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// --- detect_workspace ---

#[test]
fn detect_workspace_from_workspace_root() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);
    let ws_dir = tmp.path().join("my-feature");
    fs::create_dir(&ws_dir).unwrap();

    let name = workspace::detect_workspace(&config, &ws_dir).unwrap();
    assert_eq!(name, "my-feature");
}

#[test]
fn detect_workspace_from_subdirectory() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);
    let sub_dir = tmp.path().join("my-feature").join("repo-a");
    fs::create_dir_all(&sub_dir).unwrap();

    let name = workspace::detect_workspace(&config, &sub_dir).unwrap();
    assert_eq!(name, "my-feature");
}

#[test]
fn detect_workspace_fails_outside_workspace_dir() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);
    let outside = PathBuf::from("/tmp");

    let result = workspace::detect_workspace(&config, &outside);
    assert!(result.is_err());
}

// --- agents ---

#[test]
fn generate_claude_config_creates_files() {
    let tmp = TempDir::new().unwrap();
    let ws_dir = tmp.path().join("my-feature");
    fs::create_dir(&ws_dir).unwrap();

    let template = Template {
        repos: vec!["/path/to/repo-a".into(), "/path/to/repo-b".into()],
    };

    agents::generate(&ws_dir, "my-feature", &template).unwrap();

    let agents_path = ws_dir.join("AGENTS.md");
    let claude_path = ws_dir.join("CLAUDE.md");

    assert!(agents_path.exists());
    assert!(claude_path.is_symlink());

    let content = fs::read_to_string(&agents_path).unwrap();
    assert!(content.contains("# Workspace: my-feature"));
    assert!(content.contains("`repo-a/`"));
    assert!(content.contains("`repo-b/`"));
    assert!(content.contains("branch `my-feature`"));

    // Symlink should resolve to same content
    let claude_content = fs::read_to_string(&claude_path).unwrap();
    assert_eq!(content, claude_content);

    // .claude/settings.local.json should list repo directories
    let settings_path = ws_dir.join(".claude/settings.local.json");
    assert!(settings_path.exists());
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    let dirs = settings["additionalDirectories"].as_array().unwrap();
    assert_eq!(dirs.len(), 2);
    assert!(dirs[0].as_str().unwrap().ends_with("/repo-a"));
    assert!(dirs[1].as_str().unwrap().ends_with("/repo-b"));
}

// --- list_repos ---

#[test]
fn list_repos_returns_sorted_directories() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);
    let ws_dir = tmp.path().join("my-ws");
    fs::create_dir(&ws_dir).unwrap();
    fs::create_dir(ws_dir.join("repo-c")).unwrap();
    fs::create_dir(ws_dir.join("repo-a")).unwrap();
    fs::create_dir(ws_dir.join("repo-b")).unwrap();
    // hidden dirs should be excluded
    fs::create_dir(ws_dir.join(".config")).unwrap();

    let repos = workspace::list_repos(&config, "my-ws").unwrap();
    assert_eq!(repos, vec!["repo-a", "repo-b", "repo-c"]);
}

#[test]
fn list_repos_fails_for_nonexistent_workspace() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);

    let result = workspace::list_repos(&config, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn list_repos_empty_workspace() {
    let tmp = TempDir::new().unwrap();
    let config = empty_config(&tmp);
    let ws_dir = tmp.path().join("empty-ws");
    fs::create_dir(&ws_dir).unwrap();

    let repos = workspace::list_repos(&config, "empty-ws").unwrap();
    assert!(repos.is_empty());
}

// --- remove_repo (agents) ---

#[test]
fn remove_repo_updates_agents_md() {
    let tmp = TempDir::new().unwrap();
    let ws_dir = tmp.path().join("my-feature");
    fs::create_dir(&ws_dir).unwrap();

    let template = Template {
        repos: vec!["/path/to/repo-a".into(), "/path/to/repo-b".into()],
    };
    agents::generate(&ws_dir, "my-feature", &template).unwrap();

    // Verify both repos are in AGENTS.md
    let content = fs::read_to_string(ws_dir.join("AGENTS.md")).unwrap();
    assert!(content.contains("`repo-a/`"));
    assert!(content.contains("`repo-b/`"));

    // Remove repo-a
    agents::remove_repo(&ws_dir, "repo-a").unwrap();

    let content = fs::read_to_string(ws_dir.join("AGENTS.md")).unwrap();
    assert!(!content.contains("`repo-a/`"));
    assert!(content.contains("`repo-b/`"));
}

#[test]
fn remove_repo_updates_claude_settings() {
    let tmp = TempDir::new().unwrap();
    let ws_dir = tmp.path().join("my-feature");
    fs::create_dir(&ws_dir).unwrap();

    let template = Template {
        repos: vec!["/path/to/repo-a".into(), "/path/to/repo-b".into()],
    };
    agents::generate(&ws_dir, "my-feature", &template).unwrap();

    // Verify both repos in settings
    let settings_path = ws_dir.join(".claude/settings.local.json");
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(settings["additionalDirectories"].as_array().unwrap().len(), 2);

    // Remove repo-a
    agents::remove_repo(&ws_dir, "repo-a").unwrap();

    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    let dirs = settings["additionalDirectories"].as_array().unwrap();
    assert_eq!(dirs.len(), 1);
    assert!(dirs[0].as_str().unwrap().ends_with("/repo-b"));
}

#[test]
fn remove_repo_noop_when_no_agents_md() {
    let tmp = TempDir::new().unwrap();
    let ws_dir = tmp.path().join("my-feature");
    fs::create_dir(&ws_dir).unwrap();

    // Should not error when no AGENTS.md exists
    agents::remove_repo(&ws_dir, "repo-a").unwrap();
}
