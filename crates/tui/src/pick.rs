//! Choosing a saved dashboard: the list `sitroom --pick` shows before sitroom
//! starts (`choose`), and the TUI's `d` overlay (`draw`). Both take the same
//! keys: arrows or j/k move, Enter or a number opens, Esc or q closes.

use std::io::Write;

use anyhow::Result;
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::crossterm::{cursor, execute, queue, style, terminal};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use sitroom_contract::ConfigInfo;

use crate::help::centered;
use crate::mouse::{Click, Hits};
use crate::theme::Palette;

/// The last entry's label: no saved dashboard, the config file alone.
pub const CONFIG_ALONE: &str = "config.yaml alone";

/// One choice in the list.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The saved dashboard. `None`: `config.yaml` alone.
    pub name: Option<String>,
    /// Its `description:`.
    pub description: Option<String>,
}

impl Entry {
    fn label(&self) -> &str {
        self.name.as_deref().unwrap_or(CONFIG_ALONE)
    }
}

/// What a key did.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Open this dashboard (`None`: `config.yaml` alone).
    Open(Option<String>),
    /// Closed without choosing.
    Cancel,
}

/// The saved dashboards, then `config.yaml` alone, with one selected.
#[derive(Debug, Clone, PartialEq)]
pub struct Picker {
    pub entries: Vec<Entry>,
    pub selected: usize,
    /// The entry in use, marked in the list.
    pub current: usize,
}

impl Picker {
    /// The list for these dashboards (name and description), with the one in
    /// use (`None`: `config.yaml` alone) selected.
    pub fn new(
        saved: impl IntoIterator<Item = (String, Option<String>)>,
        current: Option<&str>,
    ) -> Self {
        let mut entries: Vec<Entry> = saved
            .into_iter()
            .map(|(name, description)| Entry {
                name: Some(name),
                description,
            })
            .collect();
        entries.push(Entry {
            name: None,
            description: None,
        });
        let current = entries
            .iter()
            .position(|e| e.name.as_deref() == current)
            .unwrap_or(entries.len() - 1);
        Self {
            entries,
            selected: current,
            current,
        }
    }

    /// The list from the settings a renderer has.
    pub fn from_config(config: &ConfigInfo) -> Self {
        Self::new(
            config
                .dashboards
                .iter()
                .map(|d| (d.name.clone(), d.description.clone())),
            config.dashboard.as_deref(),
        )
    }

    /// Move or choose. `None`: still choosing.
    pub fn key(&mut self, key: KeyEvent) -> Option<Outcome> {
        let last = self.entries.len() - 1;
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Some(Outcome::Cancel);
            }
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                self.selected = self.selected.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                self.selected = (self.selected + 1).min(last)
            }
            KeyCode::Home => self.selected = 0,
            KeyCode::End => self.selected = last,
            KeyCode::Char(c @ '1'..='9') => {
                let i = c as usize - '1' as usize;
                if i <= last {
                    return Some(self.open(i));
                }
            }
            KeyCode::Enter => return Some(self.open(self.selected)),
            KeyCode::Esc | KeyCode::Char('q') => return Some(Outcome::Cancel),
            _ => {}
        }
        None
    }

    /// Choose entry `i`.
    pub fn open(&self, i: usize) -> Outcome {
        Outcome::Open(self.entries.get(i).and_then(|e| e.name.clone()))
    }

    /// One line per entry: "▸ 1 work   Day job   (in use)".
    fn rows(&self) -> Vec<(String, String, bool)> {
        let width = self
            .entries
            .iter()
            .map(|e| e.label().len())
            .max()
            .unwrap_or(0);
        self.entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let marker = if i == self.selected { "▸" } else { " " };
                let number = if i < 9 {
                    (i + 1).to_string()
                } else {
                    " ".to_string()
                };
                let mut note = e.description.clone().unwrap_or_default();
                if i == self.current {
                    note = format!("{note}  (in use)").trim_start().to_string();
                }
                (
                    format!("{marker} {number} {:width$}", e.label()),
                    note,
                    i == self.selected,
                )
            })
            .collect()
    }
}

/// The key line under the list.
const KEYS: &str = "↑↓ move  Enter or 1-9 open  Esc close";

/// Show the list under the cursor and wait for a choice. Needs a terminal on
/// stdin and stderr; the list is drawn on stderr and erased afterwards.
pub fn choose(mut picker: Picker) -> Result<Outcome> {
    let mut out = std::io::stderr();
    terminal::enable_raw_mode()?;
    let result = (|| -> Result<Outcome> {
        let mut drawn = 0u16;
        loop {
            if drawn > 0 {
                queue!(out, cursor::MoveUp(drawn))?;
            }
            queue!(
                out,
                cursor::MoveToColumn(0),
                terminal::Clear(terminal::ClearType::FromCursorDown)
            )?;
            let mut lines = vec!["Open which dashboard?".to_string()];
            for (row, note, selected) in picker.rows() {
                let line = format!("{row}  {note}");
                lines.push(if selected {
                    format!("{}", style::Stylize::bold(line.trim_end()))
                } else {
                    line.trim_end().to_string()
                });
            }
            lines.push(format!("{}", style::Stylize::dim(KEYS)));
            for line in &lines {
                queue!(out, style::Print(line), style::Print("\r\n"))?;
            }
            out.flush()?;
            drawn = lines.len() as u16;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && let Some(outcome) = picker.key(key)
            {
                execute!(
                    out,
                    cursor::MoveUp(drawn),
                    cursor::MoveToColumn(0),
                    terminal::Clear(terminal::ClearType::FromCursorDown)
                )?;
                return Ok(outcome);
            }
        }
    })();
    terminal::disable_raw_mode()?;
    result
}

/// The TUI's dashboard overlay. Each row is a click target.
pub fn draw(f: &mut Frame, p: &Palette, picker: &Picker, hits: &mut Hits) {
    let area = centered(f.area(), 60, 60);
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title(" Dashboards ");
    let inner = block.inner(area);
    f.render_widget(block, area);
    hits.add(area, Click::Inside);

    let mut lines = vec![];
    for (i, (row, note, selected)) in picker.rows().into_iter().enumerate() {
        let style = if selected {
            Style::new().fg(p.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new()
        };
        lines.push(Line::from(vec![
            Span::styled(row, style),
            Span::raw("  "),
            Span::styled(note, Style::new().fg(p.muted)),
        ]));
        let y = inner.y + i as u16;
        if y < inner.bottom().saturating_sub(1) {
            hits.add(
                Rect {
                    y,
                    height: 1,
                    ..inner
                },
                Click::Dashboard(i),
            );
        }
    }
    let keys = Rect {
        y: inner.bottom().saturating_sub(1),
        height: 1,
        ..inner
    };
    f.render_widget(Paragraph::new(lines), inner);
    f.render_widget(
        Paragraph::new(Line::styled(KEYS, Style::new().fg(p.muted))),
        keys,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picker(current: Option<&str>) -> Picker {
        Picker::new(
            [
                ("default".to_string(), Some("Start here".to_string())),
                ("work".to_string(), None),
            ],
            current,
        )
    }

    fn key(p: &mut Picker, code: KeyCode) -> Option<Outcome> {
        p.key(KeyEvent::from(code))
    }

    #[test]
    fn starts_on_the_dashboard_in_use() {
        assert_eq!(picker(Some("work")).selected, 1);
        // Nothing in use: config.yaml alone, the last entry.
        let p = picker(None);
        assert_eq!(p.selected, 2);
        assert_eq!(p.entries[2].label(), CONFIG_ALONE);
    }

    #[test]
    fn keys_move_and_choose() {
        let mut p = picker(Some("default"));
        assert_eq!(key(&mut p, KeyCode::Up), None);
        assert_eq!(p.selected, 0);
        key(&mut p, KeyCode::Down);
        key(&mut p, KeyCode::Char('j'));
        key(&mut p, KeyCode::Down);
        assert_eq!(p.selected, 2, "stops at the last entry");
        assert_eq!(key(&mut p, KeyCode::Enter), Some(Outcome::Open(None)));
        assert_eq!(
            key(&mut p, KeyCode::Char('2')),
            Some(Outcome::Open(Some("work".into())))
        );
        assert_eq!(key(&mut p, KeyCode::Char('9')), None, "no ninth entry");
        assert_eq!(key(&mut p, KeyCode::Esc), Some(Outcome::Cancel));
        assert_eq!(
            p.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Outcome::Cancel)
        );
    }

    #[test]
    fn rows_mark_the_selected_and_the_one_in_use() {
        let mut p = picker(Some("default"));
        key(&mut p, KeyCode::Down);
        let rows = p.rows();
        assert_eq!(rows[0].0, format!("  1 {:17}", "default"));
        assert_eq!(rows[0].1, "Start here  (in use)");
        assert_eq!(rows[1].0, format!("▸ 2 {:17}", "work"));
        assert!(rows[1].2);
        assert_eq!(rows[2].0, format!("  3 {CONFIG_ALONE}"));
    }
}
