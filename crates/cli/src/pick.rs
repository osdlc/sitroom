//! `--pick` (and bare `sitroom`): choose a saved dashboard before starting,
//! edit one, or make a new one. See docs/dashboards.md.

use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::Result;
use sitroom_contract::NewDashboardFrom;
use sitroom_core::config::dashboards::{Registry, create};
use sitroom_core::config::layers::Sources;
use sitroom_tui::PickOutcome;

use crate::{Cli, dashboards, warn};

/// What was chosen.
pub struct Picked {
    /// Open the web view's screen editor: `e` in `serve --pick`, or a new
    /// blank dashboard there.
    pub edit: bool,
}

/// Show the list and set `--dashboard` (or `--no-dashboard`) from the
/// choice. `web`: for `serve`, where `e` opens the screen editor in the
/// browser; in the TUI it opens the file in `$VISUAL` or `$EDITOR` and shows
/// the list again. `None`: quit without starting. With no terminal it prints
/// the list and returns `None`; with no saved dashboards there is nothing to
/// choose, so it starts as it would without `--pick`.
pub fn run(cli: &mut Cli, web: bool) -> Result<Option<Picked>> {
    let sources = Sources {
        machine: cli.config.clone(),
        dashboard: cli.dashboard.clone(),
        no_default: cli.no_dashboard,
        ..Default::default()
    };
    let registry = sources.registry()?;
    let (saved, errors) = registry.list();
    for err in &errors {
        warn(err);
    }
    if saved.is_empty() {
        return Ok(Some(Picked { edit: false }));
    }
    if !(io::stdin().is_terminal() && io::stderr().is_terminal()) {
        dashboards::list(&sources, false)?;
        return Ok(None);
    }
    let mut picker = sitroom_tui::Picker::new(
        saved.into_iter().map(|s| (s.name, s.description)),
        sources.dashboard_in_use().as_deref(),
    )
    .with_new();
    picker.editable = true;
    loop {
        match sitroom_tui::pick_dashboard(&mut picker)? {
            PickOutcome::Open(name) => return open(cli, name, false),
            PickOutcome::Edit(name) if web => return open(cli, name, true),
            PickOutcome::Edit(name) => {
                let path = match &name {
                    Some(name) => registry.path(name)?,
                    None => machine_path(cli)?,
                };
                sitroom_tui::open_editor(&path.to_string_lossy())?;
            }
            PickOutcome::New => match new_dashboard(cli, &sources, &registry) {
                Ok(Some((name, from))) => {
                    return open(cli, Some(name), web && from == NewDashboardFrom::Blank);
                }
                // Back to the list.
                Ok(None) => {}
                Err(err) => eprintln!("sitroom: {err:#}"),
            },
            PickOutcome::Cancel => return Ok(None),
        }
    }
}

/// Start with this dashboard (`None`: `config.yaml` alone).
fn open(cli: &mut Cli, name: Option<String>, edit: bool) -> Result<Option<Picked>> {
    cli.no_dashboard = name.is_none();
    cli.dashboard = name;
    Ok(Some(Picked { edit }))
}

/// `config.yaml`, or `--config`.
fn machine_path(cli: &Cli) -> Result<std::path::PathBuf> {
    match &cli.config {
        Some(path) => Ok(path.clone()),
        None => sitroom_core::config::default_path(),
    }
}

/// Ask for a name and what to start from, then save the dashboard. `None`:
/// no name given, back to the list.
fn new_dashboard(
    cli: &Cli,
    sources: &Sources,
    registry: &Registry,
) -> Result<Option<(String, NewDashboardFrom)>> {
    let in_use = sources.dashboard_in_use();
    let Some(name) = ask("Name for the new dashboard (Enter alone goes back): ")? else {
        return Ok(None);
    };
    let current = in_use.as_deref().unwrap_or("config.yaml");
    eprintln!("Start from:");
    eprintln!("  1 the starter template: ready-made screens to change");
    eprintln!("  2 blank: one empty screen");
    eprintln!("  3 a copy of {current}");
    let from = loop {
        match ask("Choose 1, 2 or 3 [1]: ")?.as_deref() {
            None | Some("1") => break NewDashboardFrom::Starter,
            Some("2") => break NewDashboardFrom::Blank,
            Some("3") => break NewDashboardFrom::Copy,
            Some(_) => {}
        }
    };
    // A copy takes the file settings are saved to, or the demo when there is
    // no file yet.
    let copy = || {
        let path = match &in_use {
            Some(name) => registry.path(name)?,
            None => machine_path(cli)?,
        };
        Ok(match path.exists() {
            true => std::fs::read_to_string(&path)?,
            false => sitroom_core::config::DEFAULT_CONFIG.to_string(),
        })
    };
    let name = create(registry, &name, from, copy)?;
    eprintln!("saved {}", registry.path(&name)?.display());
    Ok(Some((name, from)))
}

/// One line from stdin, trimmed. `None`: an empty line or end of input.
fn ask(prompt: &str) -> Result<Option<String>> {
    eprint!("{prompt}");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    let line = line.trim();
    Ok((!line.is_empty()).then(|| line.to_string()))
}
