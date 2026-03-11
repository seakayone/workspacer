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

    let mut stderr = io::stderr();
    execute!(stderr, Hide)?;

    terminal::enable_raw_mode()?;
    let mut last_rendered_lines: u16 = 0;
    let result = run_inline_picker(&display_items, &names, &header, &mut last_rendered_lines);
    terminal::disable_raw_mode()?;
    execute!(stderr, Show)?;

    // Clear the picker output
    if last_rendered_lines > 0 {
        execute!(stderr, MoveUp(last_rendered_lines))?;
    }
    for _ in 0..last_rendered_lines {
        execute!(
            stderr,
            MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print("\n")
        )?;
    }
    if last_rendered_lines > 0 {
        execute!(stderr, MoveUp(last_rendered_lines))?;
    }

    result
}

fn render(
    header: &str,
    filter: &str,
    display_items: &[String],
    filtered_indices: &[usize],
    selected: usize,
    prev_lines: u16,
) -> Result<u16> {
    let mut stderr = io::stderr();
    let (cols, _) = terminal::size().unwrap_or((80, 24));
    let width = cols as usize;

    // Move cursor back up to overwrite previous render
    if prev_lines > 0 {
        queue!(stderr, MoveUp(prev_lines))?;
    }

    // Header line
    queue!(
        stderr,
        MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        Print(header),
        Print("\n"),
    )?;

    // Filter prompt line
    queue!(
        stderr,
        MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
    )?;
    if filter.is_empty() {
        queue!(
            stderr,
            SetAttribute(Attribute::Dim),
            Print(" / type to filter"),
            SetAttribute(Attribute::Reset),
        )?;
    } else {
        queue!(stderr, Print(format!(" / {filter}")))?;
    }
    queue!(stderr, Print("\n"))?;

    // Filtered items
    for (i, &idx) in filtered_indices.iter().enumerate() {
        queue!(
            stderr,
            MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )?;
        if i == selected {
            let text = format!(" > {}", display_items[idx]);
            let padded = format!("{:<width$}", text, width = width.saturating_sub(1));
            queue!(
                stderr,
                SetAttribute(Attribute::Reverse),
                Print(padded),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(stderr, Print(format!("   {}", display_items[idx])))?;
        }
        queue!(stderr, Print("\n"))?;
    }

    // Clear any leftover lines from previous longer render
    let current_lines = 2 + filtered_indices.len() as u16; // header + filter + items
    for _ in current_lines..prev_lines {
        queue!(
            stderr,
            MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
            Print("\n"),
        )?;
    }

    stderr.flush()?;

    Ok(current_lines.max(prev_lines))
}

fn filter_indices(names: &[String], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        (0..names.len()).collect()
    } else {
        let lower = filter.to_lowercase();
        (0..names.len())
            .filter(|&i| names[i].to_lowercase().contains(&lower))
            .collect()
    }
}

fn run_inline_picker(
    display_items: &[String],
    names: &[String],
    header: &str,
    last_rendered_lines: &mut u16,
) -> Result<Option<String>> {
    let mut selected: usize = 0;
    let mut filter = String::new();
    let mut filtered = filter_indices(names, &filter);

    let prev = render(header, &filter, display_items, &filtered, selected, 0)?;
    *last_rendered_lines = prev;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('c')
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    return Ok(None)
                }
                KeyCode::Esc => return Ok(None),
                KeyCode::Enter => {
                    return Ok(filtered.get(selected).map(|&idx| names[idx].clone()));
                }
                KeyCode::Down => {
                    if !filtered.is_empty() {
                        selected = (selected + 1) % filtered.len();
                    }
                }
                KeyCode::Up => {
                    if !filtered.is_empty() {
                        selected =
                            selected.checked_sub(1).unwrap_or(filtered.len() - 1);
                    }
                }
                KeyCode::Backspace => {
                    filter.pop();
                    filtered = filter_indices(names, &filter);
                    selected = 0;
                }
                KeyCode::Char(c) => {
                    filter.push(c);
                    filtered = filter_indices(names, &filter);
                    selected = 0;
                }
                _ => continue,
            }
            let prev =
                render(header, &filter, display_items, &filtered, selected, *last_rendered_lines)?;
            *last_rendered_lines = prev;
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
    fn filter_indices_empty_filter_returns_all() {
        let names: Vec<String> = vec!["alpha".into(), "beta".into(), "gamma".into()];
        assert_eq!(super::filter_indices(&names, ""), vec![0, 1, 2]);
    }

    #[test]
    fn filter_indices_matches_substring_case_insensitive() {
        let names: Vec<String> = vec!["Alpha".into(), "beta".into(), "Gamma".into()];
        assert_eq!(super::filter_indices(&names, "alph"), vec![0]);
        assert_eq!(super::filter_indices(&names, "ETA"), vec![1]);
    }

    #[test]
    fn filter_indices_no_match_returns_empty() {
        let names: Vec<String> = vec!["alpha".into(), "beta".into()];
        assert_eq!(super::filter_indices(&names, "xyz"), Vec::<usize>::new());
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
