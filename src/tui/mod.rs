use std::io::{self, Write};

use anyhow::Result;
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{self, ClearType};
use crossterm::{execute, queue};

pub fn pick_workspace(items: Vec<String>) -> Result<Option<String>> {
    if items.is_empty() {
        anyhow::bail!("no workspaces found");
    }

    terminal::enable_raw_mode()?;
    let result = run_inline_picker(&items);
    terminal::disable_raw_mode()?;

    // Clear the picker output after selection
    let mut stderr = io::stderr();
    if items.len() > 0 {
        // Move up to the first line we printed and clear everything
        if items.len() > 1 {
            execute!(stderr, MoveUp(items.len() as u16 - 1))?;
        }
        for _ in 0..items.len() {
            execute!(
                stderr,
                MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                Print("\n")
            )?;
        }
        // Move back up
        if items.len() > 0 {
            execute!(stderr, MoveUp(items.len() as u16))?;
        }
    }

    result
}

fn render(items: &[String], selected: usize, first: bool) -> Result<()> {
    let mut stderr = io::stderr();

    // Move cursor back up to overwrite previous render (skip on first draw)
    if !first && items.len() > 1 {
        queue!(stderr, MoveUp(items.len() as u16 - 1))?;
    }

    for (i, item) in items.iter().enumerate() {
        queue!(stderr, MoveToColumn(0), terminal::Clear(ClearType::CurrentLine))?;
        if i == selected {
            queue!(
                stderr,
                SetAttribute(Attribute::Reverse),
                Print(format!(" > {item} ")),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(stderr, Print(format!("   {item}")))?;
        }
        if i < items.len() - 1 {
            queue!(stderr, Print("\n"))?;
        }
    }

    stderr.flush()?;
    Ok(())
}

fn run_inline_picker(items: &[String]) -> Result<Option<String>> {
    let mut selected: usize = 0;

    render(items, selected, true)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                KeyCode::Enter => return Ok(Some(items[selected].clone())),
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1).min(items.len() - 1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.saturating_sub(1);
                }
                KeyCode::Char('h') | KeyCode::Home => {
                    selected = 0;
                }
                KeyCode::Char('l') | KeyCode::End => {
                    selected = items.len() - 1;
                }
                _ => continue,
            }
            render(items, selected, false)?;
        }
    }
}
