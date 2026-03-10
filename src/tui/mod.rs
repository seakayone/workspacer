use std::io::{self, Write};

use anyhow::Result;
use crossterm::cursor::{Hide, MoveToColumn, MoveUp, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{self, ClearType};
use crossterm::{execute, queue};

/// A workspace entry with name and optional agent marker for display.
pub struct WorkspaceEntry {
    pub name: String,
    pub marker: String,
}

impl WorkspaceEntry {
    fn display(&self, name_width: usize) -> String {
        if self.marker.is_empty() {
            self.name.clone()
        } else {
            format!("{:<width$} {}", self.name, self.marker, width = name_width)
        }
    }
}

pub fn pick_workspace(items: Vec<WorkspaceEntry>) -> Result<Option<String>> {
    if items.is_empty() {
        anyhow::bail!("no workspaces found");
    }

    let name_width = items
        .iter()
        .map(|e| e.name.len())
        .max()
        .unwrap_or(0)
        .max("WORKSPACE".len());
    let header = format!("   {:<width$} {}", "WORKSPACE", "AGENT", width = name_width);
    let display_items: Vec<String> = items.iter().map(|e| e.display(name_width)).collect();
    let names: Vec<String> = items.into_iter().map(|e| e.name).collect();

    // Print header (not part of selectable items)
    let mut stderr = io::stderr();
    execute!(stderr, Hide, Print(&header), Print("\n"))?;

    terminal::enable_raw_mode()?;
    let result = run_inline_picker(&display_items, &names);
    terminal::disable_raw_mode()?;
    execute!(stderr, Show)?;

    // Clear the picker output and header
    let total_lines = display_items.len() + 1; // +1 for header
    if total_lines > 1 {
        execute!(stderr, MoveUp(total_lines as u16 - 1))?;
    }
    for _ in 0..total_lines {
        execute!(
            stderr,
            MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print("\n")
        )?;
    }
    execute!(stderr, MoveUp(total_lines as u16))?;

    result
}

fn render(items: &[String], selected: usize, first: bool) -> Result<()> {
    let mut stderr = io::stderr();
    let (cols, _) = terminal::size().unwrap_or((80, 24));
    let width = cols as usize;

    // Move cursor back up to overwrite previous render (skip on first draw)
    if !first {
        queue!(stderr, MoveUp(items.len() as u16))?;
    }

    for (_i, item) in items.iter().enumerate() {
        queue!(stderr, MoveToColumn(0), terminal::Clear(ClearType::CurrentLine))?;
        if _i == selected {
            let text = format!(" > {item}");
            // Pad to terminal width - 1 to avoid line wrap
            let padded = format!("{:<width$}", text, width = width.saturating_sub(1));
            queue!(
                stderr,
                SetAttribute(Attribute::Reverse),
                Print(padded),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(stderr, Print(format!("   {item}")))?;
        }
        queue!(stderr, Print("\n"))?;
    }

    stderr.flush()?;
    Ok(())
}

fn run_inline_picker(display_items: &[String], names: &[String]) -> Result<Option<String>> {
    let mut selected: usize = 0;

    render(display_items, selected, true)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => return Ok(None),
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                KeyCode::Enter => return Ok(Some(names[selected].clone())),
                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1) % display_items.len();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected.checked_sub(1).unwrap_or(display_items.len() - 1);
                }
                KeyCode::Char('h') | KeyCode::Home => {
                    selected = 0;
                }
                KeyCode::Char('l') | KeyCode::End => {
                    selected = display_items.len() - 1;
                }
                _ => continue,
            }
            render(display_items, selected, false)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceEntry;

    #[test]
    fn display_without_marker() {
        let entry = WorkspaceEntry {
            name: "my-ws".into(),
            marker: String::new(),
        };
        assert_eq!(entry.display(20), "my-ws");
    }

    #[test]
    fn display_with_marker_pads_name() {
        let entry = WorkspaceEntry {
            name: "my-ws".into(),
            marker: "\u{1F916}".into(), // 🤖
        };
        let result = entry.display(20);
        assert!(result.starts_with("my-ws"));
        assert!(result.ends_with("\u{1F916}"));
        // Name padded to width 20 + space + emoji
        assert_eq!(result.len(), 20 + 1 + "\u{1F916}".len());
    }

    #[test]
    fn display_entries_align_columns() {
        let entries = vec![
            WorkspaceEntry {
                name: "short".into(),
                marker: "\u{1F916}".into(),
            },
            WorkspaceEntry {
                name: "much-longer-name".into(),
                marker: "\u{1F4AC}".into(),
            },
        ];
        let width = entries.iter().map(|e| e.name.len()).max().unwrap();
        let displays: Vec<String> = entries.iter().map(|e| e.display(width)).collect();

        // Both markers should start at the same column
        let marker_pos_0 = displays[0].find('\u{1F916}').unwrap();
        let marker_pos_1 = displays[1].find('\u{1F4AC}').unwrap();
        assert_eq!(marker_pos_0, marker_pos_1);
    }
}
