//! Fixed text from the config, in markdown: instructions, a legend, a
//! reminder. Use with `builtin: note`:
//!
//! ```yaml
//! modules:
//!   welcome:
//!     builtin: note
//!     config:
//!       title: Start here
//!       text: |
//!         # Keys
//!         - `?` help
//!         - `q` quit
//! ```
//!
//! For text kept in a file, and shown again when the file changes, use
//! `textfile`.

use anyhow::{Result, bail};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use sitroom_contract::{CatalogEntry, Manifest, Size, Status, View, Widget};
use sitroom_core::host::Module;

use crate::common::manifest;

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct Config {
    /// The text: markdown (headings, lists, `**bold**`, `` `code` ``, links,
    /// tables), with single line breaks kept.
    text: String,
    /// The tile's title. Default: "Note".
    title: Option<String>,
    /// Show `text` as it is, not as markdown.
    plain: bool,
}

/// Fixed text from the config.
#[derive(Default)]
pub struct Note {
    view: Option<View>,
}

#[async_trait]
impl Module for Note {
    async fn init(&mut self, config: serde_json::Value) -> Result<Manifest> {
        let config: Config = match config.is_null() {
            true => Config::default(),
            false => serde_json::from_value(config)?,
        };
        if config.text.trim().is_empty() {
            bail!("the note module needs `config: {{ text: ... }}`");
        }
        let title = config.title.unwrap_or_else(|| "Note".into());
        self.view = Some(view(&config.text, !config.plain));
        // The text only changes with the config, which starts the module
        // again: render rarely.
        Ok(manifest("note", &title, 3_600_000, 1_000, Size::Medium, 0))
    }

    async fn render(&mut self) -> Result<View> {
        match &self.view {
            Some(view) => Ok(view.clone()),
            None => bail!("the note module was not started"),
        }
    }
}

fn view(text: &str, markdown: bool) -> View {
    let text = text.trim_end();
    let lines = text.lines().count();
    View {
        status: Status::Ok,
        summary: match lines {
            1 => "1 line".into(),
            n => format!("{n} lines"),
        },
        root: Widget::Text {
            text: text.into(),
            markdown,
        },
    }
}

const DEMO: &str = "\
# Start here
- **Arrows** or `hjkl` move between tiles
- `?` lists every key
- [The tutorial](https://osdlc.github.io/sitroom/tutorial/)";

/// Catalog entry for this module, with demo data.
pub fn catalog() -> CatalogEntry {
    CatalogEntry {
        description: "Fixed text from the config, in markdown: instructions, a legend or a \
                      reminder. Add it several times with `builtin: note`. For text in a \
                      file, use textfile."
            .into(),
        tags: super::strings(&["notes", "text", "local"]),
        config_schema: super::schema::<Config>(),
        ..super::entry("note", "Note", Size::Medium, view(DEMO, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shows_the_text_with_its_title() {
        let mut m = Note::default();
        let manifest = m
            .init(serde_json::json!({ "title": "Keys", "text": "# Keys\n- `q` quit\n\n" }))
            .await
            .unwrap();
        assert_eq!(manifest.name, "Keys");
        let view = m.render().await.unwrap();
        assert_eq!(view.summary, "2 lines");
        assert_eq!(
            view.root,
            Widget::Text {
                text: "# Keys\n- `q` quit".into(),
                markdown: true
            }
        );
        let mut m = Note::default();
        let manifest = m
            .init(serde_json::json!({ "text": "a*b", "plain": true }))
            .await
            .unwrap();
        assert_eq!(manifest.name, "Note");
        assert_eq!(
            m.render().await.unwrap().root,
            Widget::Text {
                text: "a*b".into(),
                markdown: false
            }
        );
    }

    #[tokio::test]
    async fn needs_text() {
        for config in [
            serde_json::Value::Null,
            serde_json::json!({ "text": "  " }),
            serde_json::json!({ "txt": "x" }),
        ] {
            assert!(Note::default().init(config).await.is_err());
        }
    }
}
