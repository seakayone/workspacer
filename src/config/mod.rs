use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub repos: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub workspace_dir: PathBuf,
    #[serde(default)]
    pub templates: BTreeMap<String, Template>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace_dir: dirs::home_dir()
                .expect("could not determine home directory")
                .join("workspaces"),
            templates: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("could not determine home directory")
            .join(".config")
            .join("workspacer")
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_file();
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config from {}", path.display()))?;
        let config: Config = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse config from {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create config dir {}", dir.display()))?;
        let path = Self::config_file();
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)
            .with_context(|| format!("failed to write config to {}", path.display()))?;
        Ok(())
    }

    /// Build the WORKTRUNK_WORKTREE_PATH value that places worktrees
    /// inside workspace_dir/<workspace>/<repo>/
    pub fn worktree_path_template(&self) -> String {
        let dir = self.workspace_dir.display();
        format!("{dir}/{{{{ branch | sanitize }}}}/{{{{ repo }}}}")
    }

    pub fn resolve_template<'a>(&'a self, name: Option<&'a str>) -> Result<(&'a str, &'a Template)> {
        match name {
            Some(n) => {
                let tmpl = self
                    .templates
                    .get(n)
                    .with_context(|| format!("template '{}' not found", n))?;
                Ok((n, tmpl))
            }
            None => {
                if self.templates.len() == 1 {
                    let (k, v) = self.templates.iter().next().unwrap();
                    Ok((k.as_str(), v))
                } else if self.templates.is_empty() {
                    anyhow::bail!("no templates configured. Add one with:\n  ws template add <name> --repo /path/to/repo");
                } else {
                    anyhow::bail!(
                        "multiple templates exist, specify one with --template: {}",
                        self.templates.keys().cloned().collect::<Vec<_>>().join(", ")
                    );
                }
            }
        }
    }
}
