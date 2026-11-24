//! `sitroom widgets`: search the catalog of everything that can be added.

use std::path::PathBuf;

use anyhow::Result;
use sitroom_contract::catalog::search;
use sitroom_contract::{CatalogEntry, EntryKind, Stability};

pub fn run(
    config_paths: &[PathBuf],
    query: &str,
    tag: Option<&str>,
    stability: Option<Stability>,
    json: bool,
) -> Result<()> {
    let (entries, errors) = sitroom_modules::runtime::catalog(config_paths);
    for err in &errors {
        crate::warn(&format!("skipped a plugin package: {err}"));
    }
    let found = search(&entries, query, tag, stability);
    if json {
        // The full entries, demo views included.
        return crate::print_json("widgets", &serde_json::json!({ "entries": found }));
    }
    if found.is_empty() {
        println!("nothing matches");
        return Ok(());
    }
    print!("{}", table(&found));
    Ok(())
}

/// One line per entry (ID, kind, stability tier, name, tags), then its
/// description indented.
fn table(entries: &[&CatalogEntry]) -> String {
    let id_width = entries.iter().map(|e| e.id.len()).max().unwrap_or(0);
    let name_width = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);
    let mut out = String::new();
    for e in entries {
        let kind = match e.kind {
            EntryKind::Builtin => "builtin",
            EntryKind::Plugin if e.installed() => "plugin",
            // In the marketplace index, not installed.
            EntryKind::Plugin => "market",
        };
        let line = format!(
            "{:id_width$}  {kind:7}  {:12}  {:name_width$}  {}",
            e.id,
            e.stability.as_str(),
            e.name,
            e.tags.join(", ")
        );
        out.push_str(line.trim_end());
        out.push('\n');
        out.push_str(&format!("    {}\n", e.description));
    }
    out
}

#[cfg(all(test, feature = "module-github-prs"))]
mod tests {
    use super::*;

    #[test]
    fn table_lists_matches_with_descriptions() {
        // Built-ins only: the full catalog also reads packages installed on this machine.
        let entries = sitroom_modules::catalog();
        let found = search(&entries, "requests", Some("github"), None);
        let text = table(&found);
        assert!(
            text.starts_with("github-prs  builtin  experimental  Pull requests"),
            "{text}"
        );
        assert!(text.contains("\n    Open pull requests"), "{text}");
        assert!(!text.contains("cpu"), "{text}");
    }
}
