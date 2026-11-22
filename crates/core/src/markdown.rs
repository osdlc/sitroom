//! Markdown in a text widget (`Widget::Text { markdown: true }`), as styled
//! lines for the renderers that draw text themselves: the TUI and the native
//! app. The web view renders the same text with `@osdlc/markdown-render`.
//!
//! The text is parsed with comrak: CommonMark and GitHub Flavored Markdown
//! (tables, task lists, strikethrough, autolinks, alerts) and Obsidian's
//! additions (wikilinks, callouts, `==highlights==`, block ids). As in
//! Obsidian, a single line break is kept as a line break. `%% comments %%`
//! are shown as text: a text widget is a module's output, not a vault note,
//! and doubled percent signs in it (a format string, say) are not comments.
//!
//! A link shows its URL after its text, since these renderers can't follow
//! it. Lists get bullets or numbers, quotes and callouts a bar, task items a box,
//! and blocks a blank line between them. Images show their alt text.
//!
//! As in the web view, HTML tags are hidden and the text between them is
//! kept, so `Vec<String>` shows as `Vec` (write `` `Vec<String>` ``), and
//! front matter is not read: a text widget's text is a module's output, not
//! a note with properties.

use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};

/// One line to draw.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Line {
    pub kind: LineKind,
    pub spans: Vec<Span>,
    /// What starts each row after the first when the line wraps: the
    /// spaces under a list marker, a quote's bar. Draw it muted.
    pub indent: String,
}

/// What a line is, for its style as a whole.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineKind {
    #[default]
    Text,
    /// `#` to `######`: the level, 1 to 6.
    Heading(u8),
    /// A line of a code block.
    Code,
    /// A thematic break (`---`): draw a rule across the width.
    Rule,
}

/// A run of text with one style.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub mark: Mark,
    /// The link's target, for a link or a wikilink.
    pub link: Option<String>,
}

/// How a span is styled. Several can be on at once.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mark {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub strike: bool,
    pub highlight: bool,
    /// List markers, quote bars and other decoration: drawn dimmer.
    pub muted: bool,
}

impl Line {
    /// The line's text without styles.
    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
}

/// The parser options: GFM and Obsidian's additions.
fn options() -> Options<'static> {
    let mut o = Options::default();
    let e = &mut o.extension;
    e.strikethrough = true;
    e.table = true;
    e.autolink = true;
    e.tasklist = true;
    e.alerts = true;
    e.ofm_callouts = true;
    e.ofm_block_ids = true;
    e.highlight = true;
    e.wikilinks_title_after_pipe = true;
    o
}

/// The text as lines to draw.
pub fn lines(text: &str) -> Vec<Line> {
    let arena = Arena::new();
    let root = parse_document(&arena, text, &options());
    let mut w = Writer::default();
    w.blocks(root);
    w.out
}

/// One part of a line's prefix, from a container: a list item's marker on
/// its first line and spaces after, or a quote's bar on every line.
struct Prefix {
    first: String,
    rest: String,
    used: bool,
}

#[derive(Default)]
struct Writer {
    out: Vec<Line>,
    prefix: Vec<Prefix>,
    /// The line being written, if one is open.
    line: Option<Line>,
}

impl Writer {
    /// Start a line, with the containers' prefixes.
    fn open(&mut self, kind: LineKind) {
        self.close();
        let mut spans = vec![];
        let indent = self.indent();
        let text: String = self
            .prefix
            .iter_mut()
            .map(|p| {
                let text = if p.used { &p.rest } else { &p.first };
                p.used = true;
                text.clone()
            })
            .collect();
        if !text.is_empty() {
            spans.push(Span {
                text,
                mark: Mark {
                    muted: true,
                    ..Mark::default()
                },
                link: None,
            });
        }
        self.line = Some(Line {
            kind,
            spans,
            indent,
        });
    }

    /// The containers' prefixes after their first line.
    fn indent(&self) -> String {
        self.prefix.iter().map(|p| p.rest.as_str()).collect()
    }

    fn close(&mut self) {
        if let Some(line) = self.line.take() {
            self.out.push(line);
        }
    }

    fn push(&mut self, text: &str, mark: Mark, link: Option<&str>) {
        if text.is_empty() {
            return;
        }
        if self.line.is_none() {
            self.open(LineKind::Text);
        }
        let line = self.line.as_mut().expect("opened above");
        match line.spans.last_mut() {
            Some(last) if last.mark == mark && last.link.as_deref() == link => {
                last.text.push_str(text)
            }
            _ => line.spans.push(Span {
                text: text.into(),
                mark,
                link: link.map(Into::into),
            }),
        }
    }

    /// A blank line between blocks, keeping the quote bars.
    fn gap(&mut self) {
        self.close();
        let indent = self.indent();
        let text = indent.trim_end();
        let spans = match text.is_empty() {
            true => vec![],
            false => vec![Span {
                text: text.into(),
                mark: Mark {
                    muted: true,
                    ..Mark::default()
                },
                link: None,
            }],
        };
        self.out.push(Line {
            kind: LineKind::Text,
            spans,
            indent,
        });
    }

    /// Each child of a container as a block, with a blank line between them
    /// (none between the items of a tight list, or inside a tight item).
    fn blocks<'a>(&mut self, node: &'a AstNode<'a>) {
        let tight = match &node.data().value {
            NodeValue::List(l) => l.tight,
            NodeValue::Item(_) | NodeValue::TaskItem(_) => node
                .parent()
                .is_some_and(|p| matches!(&p.data().value, NodeValue::List(l) if l.tight)),
            _ => false,
        };
        let mut first = true;
        let mut number = match &node.data().value {
            NodeValue::List(l) => l.start,
            _ => 1,
        };
        for child in node.children() {
            if !shown(child) {
                continue;
            }
            if !first && !tight {
                self.gap();
            }
            first = false;
            self.block(child, number);
            number += 1;
        }
    }

    /// One block. `number`: its place in an ordered list.
    fn block<'a>(&mut self, node: &'a AstNode<'a>, number: usize) {
        let value = node.data().value.clone();
        match &value {
            NodeValue::Paragraph | NodeValue::TableCell => {
                self.open(LineKind::Text);
                self.inlines(node, Mark::default(), None);
                self.close();
            }
            NodeValue::Heading(h) => {
                self.open(LineKind::Heading(h.level));
                self.inlines(
                    node,
                    Mark {
                        bold: true,
                        ..Mark::default()
                    },
                    None,
                );
                self.close();
            }
            NodeValue::CodeBlock(c) => self.literal(&c.literal, LineKind::Code),
            NodeValue::MathBlock(m) => self.literal(&m.literal, LineKind::Code),
            NodeValue::HtmlBlock(h) => {
                let text = without_tags(&h.literal);
                let text: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
                self.literal(&text.join("\n"), LineKind::Text);
            }
            NodeValue::ThematicBreak(_) => {
                self.open(LineKind::Rule);
                self.close();
            }
            NodeValue::List(_) => self.blocks(node),
            NodeValue::Item(_) | NodeValue::TaskItem(_) => {
                let marker = match &value {
                    NodeValue::TaskItem(t) => match t.symbol {
                        None => "☐ ".to_string(),
                        Some(_) => "☑ ".to_string(),
                    },
                    _ => match node.parent().map(|p| p.data().value.clone()) {
                        Some(NodeValue::List(l)) if l.list_type == ListType::Ordered => {
                            format!("{number}. ")
                        }
                        _ => "• ".to_string(),
                    },
                };
                self.contain(node, marker.clone(), " ".repeat(marker.chars().count()));
            }
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                self.contain(node, "│ ".into(), "│ ".into())
            }
            NodeValue::Alert(a) => {
                let title = a
                    .title
                    .clone()
                    .unwrap_or_else(|| a.alert_type.default_title().into());
                self.titled(node, &title);
            }
            NodeValue::Callout(c) => {
                // The title is the CalloutTitle child when it has text, else
                // the type ("note", "tip"), capitalized.
                let mut chars = c.callout_type.chars();
                let title: String = chars
                    .next()
                    .map(|f| f.to_uppercase().chain(chars).collect())
                    .unwrap_or_default();
                self.titled(node, &title);
            }
            NodeValue::CalloutTitle => {}
            NodeValue::Table(_) => {
                for (i, row) in node.children().enumerate() {
                    self.open(LineKind::Text);
                    for (j, cell) in row.children().enumerate() {
                        if j > 0 {
                            self.push(
                                " │ ",
                                Mark {
                                    muted: true,
                                    ..Mark::default()
                                },
                                None,
                            );
                        }
                        let mark = Mark {
                            bold: i == 0,
                            ..Mark::default()
                        };
                        self.inlines(cell, mark, None);
                    }
                    self.close();
                }
            }
            NodeValue::FootnoteDefinition(f) => {
                let marker = format!("[^{}] ", f.name);
                self.contain(node, marker.clone(), " ".repeat(marker.chars().count()));
            }
            _ => self.blocks(node),
        }
    }

    /// A container whose lines start with `first`, then `rest`.
    fn contain<'a>(&mut self, node: &'a AstNode<'a>, first: String, rest: String) {
        self.close();
        self.prefix.push(Prefix {
            first,
            rest,
            used: false,
        });
        // An empty item still shows its marker.
        if !node.children().any(shown) {
            self.open(LineKind::Text);
            self.close();
        }
        self.blocks(node);
        self.close();
        self.prefix.pop();
    }

    /// An alert or callout: a bar, a bold title, then its blocks.
    fn titled<'a>(&mut self, node: &'a AstNode<'a>, fallback: &str) {
        self.close();
        self.prefix.push(Prefix {
            first: "│ ".into(),
            rest: "│ ".into(),
            used: false,
        });
        self.open(LineKind::Text);
        let bold = Mark {
            bold: true,
            ..Mark::default()
        };
        let title = node
            .children()
            .find(|c| matches!(c.data().value, NodeValue::CalloutTitle));
        match title {
            Some(t) if t.children().next().is_some() => self.inlines(t, bold, None),
            _ => self.push(fallback, bold, None),
        }
        self.close();
        let body = node
            .children()
            .filter(|c| shown(c) && !matches!(c.data().value, NodeValue::CalloutTitle));
        for (i, child) in body.enumerate() {
            if i > 0 {
                self.gap();
            }
            self.block(child, 1);
        }
        self.close();
        self.prefix.pop();
    }

    /// Text kept line by line, such as a code block.
    fn literal(&mut self, text: &str, kind: LineKind) {
        if text.is_empty() {
            return;
        }
        let mark = Mark {
            code: kind == LineKind::Code,
            ..Mark::default()
        };
        for line in text.strip_suffix('\n').unwrap_or(text).split('\n') {
            self.open(kind);
            self.push(line, mark, None);
            self.close();
        }
    }

    /// The inline children of `node`, in `mark`, inside `link` if any.
    fn inlines<'a>(&mut self, node: &'a AstNode<'a>, mark: Mark, link: Option<&str>) {
        for child in node.children() {
            let value = child.data().value.clone();
            match value {
                NodeValue::Text(t) => self.push(&t, mark, link),
                NodeValue::Code(c) => self.push(&c.literal, Mark { code: true, ..mark }, link),
                NodeValue::Math(m) => self.push(&m.literal, Mark { code: true, ..mark }, link),
                // A tag: hidden, as the web view hides it.
                NodeValue::HtmlInline(_) => {}
                NodeValue::Raw(h) => self.push(&h, mark, link),
                NodeValue::EscapedTag(t) => self.push(t, mark, link),
                // The next line keeps this one's kind: a heading's second
                // line is still the heading.
                NodeValue::SoftBreak | NodeValue::LineBreak => {
                    let kind = self.line.as_ref().map_or(LineKind::Text, |l| l.kind);
                    self.open(kind);
                }
                NodeValue::Strong(_) => self.inlines(child, Mark { bold: true, ..mark }, link),
                NodeValue::Emph(_) => self.inlines(
                    child,
                    Mark {
                        italic: true,
                        ..mark
                    },
                    link,
                ),
                NodeValue::Strikethrough(_) => self.inlines(
                    child,
                    Mark {
                        strike: true,
                        ..mark
                    },
                    link,
                ),
                NodeValue::Highlight => self.inlines(
                    child,
                    Mark {
                        highlight: true,
                        ..mark
                    },
                    link,
                ),
                NodeValue::Link(l) => {
                    self.inlines(child, mark, Some(&l.url));
                    // A terminal can't follow the link, so show where it goes,
                    // unless the text already says (an autolink, which for
                    // `www.x.org` gets an `http://` in front).
                    let text = plain_text(child);
                    let said = [l.url.as_str()].into_iter().chain(
                        ["mailto:", "http://", "https://"]
                            .iter()
                            .filter_map(|scheme| l.url.strip_prefix(scheme)),
                    );
                    if !said.into_iter().any(|u| u == text) {
                        self.push(
                            &format!(" ({})", l.url),
                            Mark {
                                muted: true,
                                ..mark
                            },
                            None,
                        );
                    }
                }
                NodeValue::WikiLink(l) => self.inlines(child, mark, Some(&l.url)),
                NodeValue::Image(_) => self.inlines(
                    child,
                    Mark {
                        muted: true,
                        ..mark
                    },
                    link,
                ),
                NodeValue::FootnoteReference(f) => self.push(
                    &format!("[^{}]", f.name),
                    Mark {
                        muted: true,
                        ..mark
                    },
                    link,
                ),
                NodeValue::BlockId(_) => {}
                _ => self.inlines(child, mark, link),
            }
        }
    }
}

/// The text inside an inline node, without styles.
fn plain_text<'a>(node: &'a AstNode<'a>) -> String {
    node.descendants()
        .filter_map(|n| match &n.data().value {
            NodeValue::Text(t) => Some(t.to_string()),
            NodeValue::Code(c) => Some(c.literal.clone()),
            _ => None,
        })
        .collect()
}

/// `html` with its tags (`<...>`) taken out.
fn without_tags(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(end) => rest = &rest[start + end + 1..],
            None => {
                rest = &rest[start..];
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Blocks that draw nothing: block ids.
fn shown<'a>(node: &'a AstNode<'a>) -> bool {
    !matches!(node.data().value, NodeValue::BlockId(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each line's text, with `*` around bold spans and `` ` `` around code.
    fn show(text: &str) -> Vec<String> {
        lines(text)
            .iter()
            .map(|l| {
                let mut s = match l.kind {
                    LineKind::Heading(n) => format!("h{n} "),
                    LineKind::Code => "code ".into(),
                    LineKind::Rule => "rule".into(),
                    LineKind::Text => String::new(),
                };
                for span in &l.spans {
                    let t = &span.text;
                    s.push_str(&match (span.mark.bold, span.mark.code, &span.link) {
                        (_, _, Some(url)) => format!("[{t}]({url})"),
                        (true, _, _) => format!("*{t}*"),
                        (_, true, _) => format!("`{t}`"),
                        _ => t.clone(),
                    });
                }
                s
            })
            .collect()
    }

    #[test]
    fn headings_paragraphs_and_line_breaks() {
        assert_eq!(
            show("# Start here\n\nOne line\nand the next, **bold** and `code`."),
            [
                "h1 *Start here*",
                "",
                "One line",
                "and the next, *bold* and `code`."
            ]
        );
    }

    #[test]
    fn lists_tasks_and_quotes() {
        assert_eq!(
            show("- one\n- two\n  - nested\n\n3. three\n4. four\n\n- [ ] todo\n- [x] done"),
            [
                "• one",
                "• two",
                "  • nested",
                "",
                "3. three",
                "4. four",
                "",
                "☐ todo",
                "☑ done",
            ]
        );
        assert_eq!(
            show("> quoted\n> more\n>\n> second"),
            ["│ quoted", "│ more", "│", "│ second"]
        );
    }

    #[test]
    fn links_code_blocks_rules_and_callouts() {
        assert_eq!(
            show("See [the site](https://example.com), <https://a.org> and [[Note|a note]]."),
            [
                "See [the site](https://example.com) (https://example.com), \
                 [https://a.org](https://a.org) and [a note](Note)."
            ]
        );
        assert_eq!(
            show("```sh\nsitroom serve\nsitroom tui\n```\n\n---"),
            ["code `sitroom serve`", "code `sitroom tui`", "", "rule"]
        );
        assert_eq!(
            show("> [!tip]\n> Press `?` for help."),
            ["│ *Tip*", "│ Press `?` for help."]
        );
        assert_eq!(
            show("> [!note]\n> one\n>\n> two"),
            ["│ *Note*", "│ one", "│", "│ two"]
        );
        assert_eq!(
            show("see www.example.com"),
            ["see [www.example.com](http://www.example.com)"]
        );
    }

    #[test]
    fn wrapped_rows_keep_the_containers_prefixes() {
        let indents: Vec<String> = lines("- item\n  > quoted")
            .into_iter()
            .map(|l| l.indent)
            .collect();
        assert_eq!(indents, ["  ", "  │ "]);
        assert_eq!(
            show("Release\nnotes\n====="),
            ["h1 *Release*", "h1 *notes*"]
        );
    }

    #[test]
    fn html_tags_are_hidden_and_comments_are_text() {
        assert_eq!(
            show("returns Vec<String> on success\n\n<div>\n<b>hi</b>\n</div>"),
            ["returns Vec on success", "", "hi"]
        );
        assert_eq!(
            show("Text %%kept%% shown ==marked=="),
            ["Text %%kept%% shown marked"]
        );
        assert_eq!(lines(""), []);
    }
}
