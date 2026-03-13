use anyhow::{Context, Result};
use clap::Parser;
use crossterm::style::{Attribute, SetAttribute};

use workspacer::cli::{AgentCommands, Cli, Commands, RepoCommands, TemplateCommands};
use workspacer::config::{Config, Template};
use workspacer::{tui, workspace};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Repo(cmd) => {
            let cwd = std::env::current_dir().context("failed to get current directory")?;
            let ws_name = workspace::detect_workspace(&config, &cwd)?;
            match cmd {
                RepoCommands::Add { repo } => {
                    workspace::add_repo(&config, &ws_name, &repo)?;
                }
                RepoCommands::Remove { name } => {
                    workspace::remove_repo(&config, &ws_name, &name)?;
                }
            }
        }
        Commands::New { name, template } => {
            let (tmpl_name, tmpl) = config.resolve_template(template.as_deref())?;
            eprintln!("Using template: {tmpl_name}");
            let tmpl = tmpl.clone();
            workspace::create(&config, &name, &tmpl)?;
        }
        Commands::Switch { name } => {
            let target = match name {
                Some(n) => n,
                None => {
                    let workspaces = workspace::list(&config)?;
                    let entries: Vec<tui::WorkspaceEntry> = workspaces
                        .iter()
                        .map(|ws| tui::WorkspaceEntry {
                            name: ws.clone(),
                            marker: workspace::agent_marker(&config, ws).unwrap_or_default(),
                        })
                        .collect();
                    match tui::pick_workspace(entries)? {
                        Some(picked) => picked,
                        None => {
                            eprintln!("No workspace selected.");
                            return Ok(());
                        }
                    }
                }
            };
            let dir = workspace::workspace_dir(&config).join(&target);
            if !dir.exists() {
                anyhow::bail!("workspace '{}' does not exist", target);
            }
            // Print path to stdout so the shell wrapper can cd to it.
            println!("{}", dir.display());
        }
        Commands::List => {
            let workspaces = workspace::list(&config)?;
            if workspaces.is_empty() {
                println!("No workspaces found.");
            } else {
                let name_width = workspaces
                    .iter()
                    .map(|ws| ws.len())
                    .max()
                    .unwrap_or(0)
                    .max("WORKSPACE".len());
                println!(
                    "{bold}{:<width$} {}{reset}",
                    "WORKSPACE",
                    "AGENT",
                    width = name_width,
                    bold = SetAttribute(Attribute::Bold),
                    reset = SetAttribute(Attribute::Reset),
                );
                for ws in &workspaces {
                    let marker = workspace::agent_marker(&config, ws)
                        .unwrap_or_default();
                    println!(
                        "{dim}{:<width$} {}{reset}",
                        ws,
                        marker,
                        width = name_width,
                        dim = SetAttribute(Attribute::Dim),
                        reset = SetAttribute(Attribute::Reset),
                    );
                }
            }
        }
        Commands::Remove { name, template } => {
            let name = match name {
                Some(n) => n,
                None => {
                    let cwd = std::env::current_dir().context("failed to get current directory")?;
                    workspace::detect_workspace(&config, &cwd)?
                }
            };
            let (tmpl_name, tmpl) = config.resolve_template(template.as_deref())?;
            eprintln!("Using template: {tmpl_name}");
            let tmpl = tmpl.clone();
            workspace::remove(&config, &name, &tmpl)?;
        }
        Commands::Agent(cmd) => {
            let cwd = std::env::current_dir().context("failed to get current directory")?;
            let ws_name = workspace::detect_workspace(&config, &cwd)?;
            match cmd {
                AgentCommands::Set { marker } => {
                    workspace::set_agent_marker(&config, &ws_name, &marker)?;
                }
                AgentCommands::Clear => {
                    workspace::clear_agent_marker(&config, &ws_name)?;
                }
            }
        }
        Commands::Template(cmd) => handle_template(&mut config, cmd)?,
        Commands::ShellInit => {
            print!("{}", include_str!("shell/init.sh"));
        }
        Commands::Complete { kind } => match kind.as_str() {
            "workspaces" => {
                for ws in workspace::list(&config)? {
                    println!("{ws}");
                }
            }
            "templates" => {
                for name in config.templates.keys() {
                    println!("{name}");
                }
            }
            "repos" => {
                let cwd = std::env::current_dir().context("failed to get current directory")?;
                if let Ok(ws_name) = workspace::detect_workspace(&config, &cwd) {
                    for repo in workspace::list_repos(&config, &ws_name)? {
                        println!("{repo}");
                    }
                }
            }
            _ => {}
        },
        Commands::Config {
            workspace_dir,
            generate_claude_config,
        } => {
            let mut changed = false;

            if let Some(dir) = workspace_dir {
                config.workspace_dir = dir;
                changed = true;
                println!(
                    "Updated workspace directory to: {}",
                    config.workspace_dir.display()
                );
            }

            if let Some(val) = generate_claude_config {
                config.generate_claude_config = val;
                changed = true;
                println!("Generate Claude config: {val}");
            }

            if changed {
                config.save()?;
            } else {
                println!("Config file: {}", Config::config_file().display());
                println!("Workspace dir: {}", config.workspace_dir.display());
                println!("Generate Claude config: {}", config.generate_claude_config);
                println!(
                    "Templates: {}",
                    if config.templates.is_empty() {
                        "(none)".to_string()
                    } else {
                        config
                            .templates
                            .keys()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
            }
        }
    }

    Ok(())
}

fn handle_template(config: &mut Config, cmd: TemplateCommands) -> Result<()> {
    match cmd {
        TemplateCommands::List => {
            if config.templates.is_empty() {
                println!("No templates configured.");
            } else {
                for (name, tmpl) in &config.templates {
                    println!("  {name} ({} repos)", tmpl.repos.len());
                }
            }
        }
        TemplateCommands::Add { name, repo } => {
            let entry = config
                .templates
                .entry(name.clone())
                .or_insert_with(|| Template { repos: vec![] });
            for r in repo {
                let r = r.canonicalize().unwrap_or(r);
                if entry.repos.contains(&r) {
                    println!("  already in '{}': {}", name, r.display());
                } else {
                    println!("  added to '{}': {}", name, r.display());
                    entry.repos.push(r);
                }
            }
            config.save()?;
        }
        TemplateCommands::Remove { name, repo } => {
            if repo.is_empty() {
                if config.templates.remove(&name).is_some() {
                    println!("Removed template: {name}");
                } else {
                    anyhow::bail!("template '{}' not found", name);
                }
            } else {
                let entry = config
                    .templates
                    .get_mut(&name)
                    .with_context(|| format!("template '{}' not found", name))?;
                for r in repo {
                    let r = r.canonicalize().unwrap_or(r);
                    if let Some(pos) = entry.repos.iter().position(|x| *x == r) {
                        entry.repos.remove(pos);
                        println!("  removed from '{}': {}", name, r.display());
                    } else {
                        println!("  not in '{}': {}", name, r.display());
                    }
                }
            }
            config.save()?;
        }
        TemplateCommands::Show { name } => {
            let tmpl = config
                .templates
                .get(&name)
                .with_context(|| format!("template '{}' not found", name))?;
            println!("Template: {name}");
            for repo in &tmpl.repos {
                let path: &std::path::Path = repo;
                println!("  {}", path.display());
            }
        }
    }
    Ok(())
}
