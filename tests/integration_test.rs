use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

use workspacer::config::{Config, Template};
use workspacer::workspace;

fn empty_config(tmp: &TempDir) -> Config {
    Config {
        workspace_dir: tmp.path().to_path_buf(),
        templates: BTreeMap::new(),
    }
}

fn config_with_template(tmp: &TempDir, repos: Vec<PathBuf>) -> Config {
    let mut templates = BTreeMap::new();
    templates.insert("default".to_string(), Template { repos });
    Config {
        workspace_dir: tmp.path().to_path_buf(),
        templates,
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

    let json = serde_json::to_string_pretty(&config).unwrap();
    let loaded: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.templates.len(), 1);
    let tmpl = loaded.templates.get("default").unwrap();
    assert_eq!(tmpl.repos.len(), 2);
    assert_eq!(tmpl.repos[0].to_str().unwrap(), "/repo/one");
}

#[test]
fn config_deserializes_without_templates_field() {
    let json = r#"{ "workspace_dir": "/some/path" }"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert!(config.templates.is_empty());
}

// --- worktree path template ---

#[test]
fn worktree_path_template_uses_workspace_dir() {
    let config = Config {
        workspace_dir: "/my/workspaces".into(),
        templates: BTreeMap::new(),
    };
    let tmpl = config.worktree_path_template();
    assert_eq!(tmpl, "/my/workspaces/{{ branch | sanitize }}/{{ repo }}");
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
