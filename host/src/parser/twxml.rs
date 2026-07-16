//! TWXML document parsing — converts raw XML text into `renderer_schema::Block` trees.
//!
//! The host currently uses `roxmltree` for parsing because it returns typed
//! [`Block`](renderer_schema::Block) objects consumed by the WASM-free preview pipeline.
//! This is separate from `tauwriter_lsp::parser::twxml`, which extracts HubReferences
//! via tree-sitter for diagnostics/completion.  The two parsers serve different domains;
//! eventual unification would require a shared AST layer.

use gpui::SharedString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextRun {
    pub text: SharedString,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub hubref: Option<SharedString>,
    pub link: Option<SharedString>,
    pub footnote_ref: Option<SharedString>,
    pub id: Option<SharedString>,
    pub attributes: Vec<(SharedString, SharedString)>,
    pub range: Option<std::ops::Range<usize>>,
}

impl Default for TextRun {
    fn default() -> Self {
        Self {
            text: SharedString::default(),
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            code: false,
            superscript: false,
            subscript: false,
            hubref: None,
            link: None,
            footnote_ref: None,
            id: None,
            attributes: Vec::new(),
            range: None,
        }
    }
}

impl TextRun {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub runs: Vec<TextRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Block {
    Heading {
        level: usize,
        text: SharedString,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Paragraph {
        runs: Vec<TextRun>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    BlockQuote {
        runs: Vec<TextRun>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Aside {
        runs: Vec<TextRun>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    CodeBlock {
        language: SharedString,
        code: SharedString,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    DescriptionList {
        items: Vec<(SharedString, Vec<TextRun>)>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Table {
        headers: Vec<SharedString>,
        rows: Vec<Vec<Vec<TextRun>>>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    HorizontalRule {
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Image {
        src: SharedString,
        alt: Option<SharedString>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Audio {
        src: SharedString,
        alt: Option<SharedString>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Video {
        src: SharedString,
        alt: Option<SharedString>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Details {
        summary: SharedString,
        blocks: Vec<Block>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Footnote {
        id: SharedString,
        runs: Vec<TextRun>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Review {
        blocks: Vec<Block>,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
    },
    Include {
        src: SharedString,
        id: Option<SharedString>,
        attributes: Vec<(SharedString, SharedString)>,
        range: Option<std::ops::Range<usize>>,
        resolved_blocks: Option<Vec<Block>>,
    },
}

/// Load a TWXML file and parse it into (title, author, metadata, blocks).
pub fn load_and_parse_twxml(
    path: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(SharedString, SharedString)>,
    Vec<Block>,
)> {
    let xml_content = std::fs::read_to_string(path)?;
    let base_dir = std::path::Path::new(path).parent();
    let mut visited = std::collections::HashSet::new();
    if let Ok(abs_path) = std::path::Path::new(path).canonicalize() {
        visited.insert(abs_path);
    }
    parse_twxml_internal(&xml_content, base_dir, &mut visited)
}

/// Parse TWXML XML content into a document model.
pub fn parse_twxml(
    xml_content: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(SharedString, SharedString)>,
    Vec<Block>,
)> {
    let mut visited = std::collections::HashSet::new();
    parse_twxml_internal(xml_content, None, &mut visited)
}

pub fn parse_twxml_internal(
    xml_content: &str,
    base_dir: Option<&std::path::Path>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) -> anyhow::Result<(
    String,
    String,
    Vec<(SharedString, SharedString)>,
    Vec<Block>,
)> {
    let doc = roxmltree::Document::parse(xml_content)?;
    let root = doc.root_element();

    let mut title = String::new();
    let mut author = String::new();
    let mut metadata: Vec<(SharedString, SharedString)> = Vec::new();
    let mut blocks = Vec::new();

    for child in root.children() {
        if child.has_tag_name("meta") {
            let name = child.attribute("name").unwrap_or("");
            let content = child.attribute("content").unwrap_or("");
            let name_ss: SharedString = name.into();
            let content_ss: SharedString = content.into();
            metadata.push((name_ss, content_ss));
            if name == "title" {
                title = content.to_string();
            } else if name == "author" {
                author = content.to_string();
            }
        }
    }

    if let Some(body) = root.children().find(|c| c.has_tag_name("body")) {
        for child in body.children() {
            parse_node(child, 0, &mut blocks, base_dir, visited);
        }
    }

    Ok((title, author, metadata, blocks))
}

/// Recursively convert a TWXML element node into one or more `Block` entries.
const MAX_PARSE_DEPTH: usize = 128;

fn parse_node(
    node: roxmltree::Node,
    depth: usize,
    blocks: &mut Vec<Block>,
    base_dir: Option<&std::path::Path>,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    if depth > MAX_PARSE_DEPTH {
        log::warn!("TWXML document exceeds max nesting depth ({MAX_PARSE_DEPTH}); truncating.");
        return;
    }
    let range = Some(node.range());
    let id = node.attribute("id").map(SharedString::from);
    let attributes: Vec<(SharedString, SharedString)> = node
        .attributes()
        .map(|a| (SharedString::from(a.name()), SharedString::from(a.value())))
        .collect();

    if node.has_tag_name("section") {
        for child in node.children() {
            parse_node(child, depth + 1, blocks, base_dir, visited);
        }
    } else if node.has_tag_name("include") {
        let src_val = node.attribute("src").unwrap_or("");
        let src = SharedString::from(src_val);
        let mut resolved_blocks = None;
        if let Some(dir) = base_dir {
            let target_path = dir.join(src_val);
            if let Ok(abs_path) = target_path.canonicalize() {
                if !visited.contains(&abs_path) {
                    visited.insert(abs_path.clone());
                    if let Ok(content) = std::fs::read_to_string(&target_path) {
                        let sub_dir = target_path.parent();
                        let mut sub_visited = visited.clone();
                        if let Ok((_, _, _, sub_blocks)) =
                            parse_twxml_internal(&content, sub_dir, &mut sub_visited)
                        {
                            resolved_blocks = Some(sub_blocks);
                        }
                    }
                    visited.remove(&abs_path);
                } else {
                    log::warn!("Circular include detected: {}", target_path.display());
                }
            }
        }
        blocks.push(Block::Include {
            src,
            id,
            attributes,
            range,
            resolved_blocks,
        });
    } else if node.has_tag_name("heading") {
        let text = SharedString::from(collect_text(node));
        blocks.push(Block::Heading {
            level: depth + 1,
            text,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("paragraph") {
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::Paragraph {
            runs,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("blockquote") {
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::BlockQuote {
            runs,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("aside") {
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::Aside {
            runs,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("codeblock") {
        let language = SharedString::from(node.attribute("language").unwrap_or(""));
        let code = SharedString::from(node.text().unwrap_or(""));
        blocks.push(Block::CodeBlock {
            language,
            code,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("ul") || node.has_tag_name("ol") {
        let ordered = node.has_tag_name("ol");
        let mut items = Vec::new();
        for child in node.children() {
            if child.has_tag_name("li") {
                let checked = child
                    .attribute("checked")
                    .and_then(|v| v.parse::<bool>().ok());
                let runs = collect_and_normalize_runs(child);
                items.push(ListItem { checked, runs });
            }
        }
        blocks.push(Block::List {
            ordered,
            items,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("dl") {
        let mut items = Vec::new();
        let mut current_term = String::new();
        for child in node.children() {
            if child.has_tag_name("dt") {
                current_term = collect_text(child);
            } else if child.has_tag_name("dd") {
                let runs = collect_and_normalize_runs(child);
                items.push((SharedString::from(current_term.clone()), runs));
            }
        }
        blocks.push(Block::DescriptionList {
            items,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("table") {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        for child in node.children() {
            if child.has_tag_name("tr") {
                let mut row_cells = Vec::new();
                for cell in child.children() {
                    if cell.has_tag_name("th") {
                        headers.push(SharedString::from(collect_text(cell)));
                    } else if cell.has_tag_name("td") {
                        let runs = collect_and_normalize_runs(cell);
                        row_cells.push(runs);
                    }
                }
                if !row_cells.is_empty() {
                    rows.push(row_cells);
                }
            }
        }
        blocks.push(Block::Table {
            headers,
            rows,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("hr") {
        blocks.push(Block::HorizontalRule {
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("image") {
        let src = SharedString::from(node.attribute("src").unwrap_or(""));
        let alt = node.attribute("alt").map(SharedString::from);
        blocks.push(Block::Image {
            src,
            alt,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("audio") {
        let src = SharedString::from(node.attribute("src").unwrap_or(""));
        let alt = node.attribute("alt").map(SharedString::from);
        blocks.push(Block::Audio {
            src,
            alt,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("video") {
        let src = SharedString::from(node.attribute("src").unwrap_or(""));
        let alt = node.attribute("alt").map(SharedString::from);
        blocks.push(Block::Video {
            src,
            alt,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("details") {
        let mut summary = SharedString::from("Details");
        let mut details_blocks = Vec::new();
        for child in node.children() {
            if child.has_tag_name("summary") {
                summary = SharedString::from(collect_text(child));
            } else {
                parse_node(child, depth + 1, &mut details_blocks, base_dir, visited);
            }
        }
        blocks.push(Block::Details {
            summary,
            blocks: details_blocks,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("footnote") {
        let footnote_id = SharedString::from(node.attribute("id").unwrap_or(""));
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::Footnote {
            id: footnote_id,
            runs,
            attributes: attributes.clone(),
            range,
        });
    } else if node.has_tag_name("review") {
        let mut review_blocks = Vec::new();
        for child in node.children() {
            parse_node(child, depth + 1, &mut review_blocks, base_dir, visited);
        }
        blocks.push(Block::Review {
            blocks: review_blocks,
            id,
            attributes,
            range,
        });
    } else if node.is_element() {
        for child in node.children() {
            parse_node(child, depth, blocks, base_dir, visited);
        }
    }
}

/// Extract all text content from a node and its descendants.
fn collect_text(node: roxmltree::Node) -> String {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs, &Style::default());
    normalize_runs(&mut runs);
    runs.iter().map(|r| r.text.as_ref()).collect()
}

/// Helper to collect and normalize runs for a node.
fn collect_and_normalize_runs(node: roxmltree::Node) -> Vec<TextRun> {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs, &Style::default());
    normalize_runs(&mut runs);
    runs
}

/// A lightweight inline style accumulator used during `collect_runs`.
#[derive(Clone, Default)]
pub(crate) struct Style {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) code: bool,
    pub(crate) superscript: bool,
    pub(crate) subscript: bool,
    hubref: Option<SharedString>,
    link: Option<SharedString>,
    footnote_ref: Option<SharedString>,
    pub(crate) id: Option<SharedString>,
    pub(crate) attributes: Vec<(SharedString, SharedString)>,
}

/// Recursively collect styled `TextRun` segments, threading the inline style
/// through the subtree via [`Style`] accumulators.
fn collect_runs(node: roxmltree::Node, runs: &mut Vec<TextRun>, current_style: &Style) {
    for child in node.children() {
        let range = Some(child.range());
        if child.is_text() {
            let text = SharedString::from(child.text().unwrap_or(""));
            if !text.trim().is_empty() || text == " " {
                runs.push(TextRun {
                    text,
                    bold: current_style.bold,
                    italic: current_style.italic,
                    underline: current_style.underline,
                    strikethrough: current_style.strikethrough,
                    code: current_style.code,
                    superscript: current_style.superscript,
                    subscript: current_style.subscript,
                    hubref: current_style.hubref.clone(),
                    link: current_style.link.clone(),
                    footnote_ref: current_style.footnote_ref.clone(),
                    id: current_style.id.clone(),
                    attributes: current_style.attributes.clone(),
                    range,
                });
            }
        } else if child.is_element() {
            if child.has_tag_name("br") {
                runs.push(TextRun {
                    text: SharedString::from("\n"),
                    bold: current_style.bold,
                    italic: current_style.italic,
                    underline: current_style.underline,
                    strikethrough: current_style.strikethrough,
                    code: current_style.code,
                    superscript: current_style.superscript,
                    subscript: current_style.subscript,
                    hubref: current_style.hubref.clone(),
                    link: current_style.link.clone(),
                    footnote_ref: current_style.footnote_ref.clone(),
                    id: current_style.id.clone(),
                    attributes: current_style.attributes.clone(),
                    range,
                });
            } else if child.has_tag_name("fr") {
                let fr_id = SharedString::from(child.attribute("id").unwrap_or(""));
                let child_attrs: Vec<(SharedString, SharedString)> = child
                    .attributes()
                    .map(|a| (SharedString::from(a.name()), SharedString::from(a.value())))
                    .collect();
                runs.push(TextRun {
                    text: SharedString::from(format!("[{}]", fr_id)),
                    subscript: true,
                    footnote_ref: Some(fr_id.clone()),
                    id: Some(fr_id),
                    attributes: child_attrs,
                    range,
                    ..Default::default()
                });
            } else {
                let mut next_style = current_style.clone();
                let child_id = child.attribute("id").map(SharedString::from);
                let child_attrs: Vec<(SharedString, SharedString)> = child
                    .attributes()
                    .map(|a| (SharedString::from(a.name()), SharedString::from(a.value())))
                    .collect();
                next_style.id = child_id.or(next_style.id);
                if !child_attrs.is_empty() {
                    next_style.attributes = child_attrs;
                }

                if child.has_tag_name("bold") {
                    next_style.bold = true;
                } else if child.has_tag_name("italic") {
                    next_style.italic = true;
                } else if child.has_tag_name("underline") {
                    next_style.underline = true;
                } else if child.has_tag_name("strikethrough") {
                    next_style.strikethrough = true;
                } else if child.has_tag_name("code") {
                    next_style.code = true;
                } else if child.has_tag_name("super") {
                    next_style.superscript = true;
                } else if child.has_tag_name("sub") {
                    next_style.subscript = true;
                } else if child.has_tag_name("hubref") {
                    next_style.hubref = child.attribute("id").map(SharedString::from);
                } else if child.has_tag_name("link") {
                    next_style.link = child.attribute("href").map(SharedString::from);
                }
                collect_runs(child, runs, &next_style);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum NextElement {
    Char(char),
    LineBreak,
    None,
}

#[derive(Debug, PartialEq, Eq)]
enum PrevElement {
    Char(char),
    LineBreak,
    None,
}

fn find_next_semantic_element(
    runs: &[TextRun],
    start_run_idx: usize,
    start_char_idx: usize,
) -> NextElement {
    for (r_idx, run) in runs.iter().enumerate().skip(start_run_idx) {
        if run.text == "\n" {
            return NextElement::LineBreak;
        }
        let chars = run.text.chars();
        for (c_idx, c) in chars.enumerate() {
            if r_idx == start_run_idx && c_idx < start_char_idx {
                continue;
            }
            if !c.is_whitespace() {
                return NextElement::Char(c);
            }
        }
    }
    NextElement::None
}

fn find_prev_semantic_element(
    runs: &[TextRun],
    start_run_idx: usize,
    start_char_idx: usize,
) -> PrevElement {
    let mut r_idx = start_run_idx;
    let mut first = true;
    loop {
        let run = &runs[r_idx];
        if run.text == "\n" {
            return PrevElement::LineBreak;
        }
        let chars: Vec<char> = run.text.chars().collect();
        let start = if first {
            first = false;
            if start_char_idx > chars.len() {
                chars.len()
            } else {
                start_char_idx
            }
        } else {
            chars.len()
        };
        for c_idx in (0..start).rev() {
            let c = chars[c_idx];
            if !c.is_whitespace() {
                return PrevElement::Char(c);
            }
        }
        if r_idx == 0 {
            break;
        }
        r_idx -= 1;
    }
    PrevElement::None
}

fn normalize_runs(runs: &mut Vec<TextRun>) {
    let num_runs = runs.len();
    let mut new_texts = Vec::with_capacity(num_runs);

    for r_idx in 0..num_runs {
        let run = &runs[r_idx];
        if run.text == "\n" {
            new_texts.push(SharedString::from("\n"));
            continue;
        }

        let chars: Vec<char> = run.text.chars().collect();
        let mut normalized = String::new();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if c == '\n' {
                let prev_el = find_prev_semantic_element(runs, r_idx, i);
                let next_el = find_next_semantic_element(runs, r_idx, i + 1);

                // Skip the newline and all subsequent whitespaces in this run.
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }

                if let (PrevElement::Char(_), NextElement::Char(next_c)) = (prev_el, next_el) {
                    let is_punctuation =
                        ['.', ',', ':', ';', '!', '?', ')', ']', '}'].contains(&next_c);
                    if !is_punctuation {
                        normalized.push(' ');
                    }
                }
            } else {
                normalized.push(c);
                i += 1;
            }
        }
        new_texts.push(SharedString::from(normalized));
    }

    let mut updated_runs = Vec::with_capacity(num_runs);
    for (mut run, new_text) in runs.drain(..).zip(new_texts) {
        if !new_text.is_empty() {
            run.text = new_text;
            updated_runs.push(run);
        }
    }
    *runs = updated_runs;
}

/// Represents a node in the document outline graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutlineNode {
    pub id: String,
    pub name: String,
    pub kind: String, // "section", "heading", "paragraph", "hubref"
    pub start_offset: usize,
}

/// Parse the active twxml text using Tree-sitter and the outlines.scm query
/// to produce nodes and parent-child edges.
pub fn parse_document_outline(text: &str) -> (Vec<OutlineNode>, Vec<(usize, usize)>) {
    use tree_sitter::StreamingIterator;
    let language = match crate::load_twxml_language() {
        Some(lang) => lang,
        None => return (Vec::new(), Vec::new()),
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return (Vec::new(), Vec::new());
    }
    let tree = match parser.parse(text, None) {
        Some(t) => t,
        None => return (Vec::new(), Vec::new()),
    };

    let query_str = include_str!("../../../extension/languages/twxml/outlines.scm");
    let query = match tree_sitter::Query::new(&language, query_str) {
        Ok(q) => q,
        Err(_) => return (Vec::new(), Vec::new()),
    };

    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = query_cursor.matches(&query, tree.root_node(), text.as_bytes());

    let mut nodes = Vec::new();
    let mut ts_id_to_idx = std::collections::HashMap::new();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            if node.kind() == "element" || node.kind() == "self_closing_element" {
                let node_id = node.id();
                if ts_id_to_idx.contains_key(&node_id) {
                    continue;
                }

                let mut tag_name = String::new();
                if let Some(start_tag) = node.child(0) {
                    if start_tag.kind() == "start_tag" {
                        if let Some(name_node) = start_tag.child_by_field_name("name") {
                            tag_name = text[name_node.byte_range()].to_string();
                        }
                    }
                }
                if tag_name.is_empty() && node.kind() == "self_closing_element" {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        tag_name = text[name_node.byte_range()].to_string();
                    }
                }
                if tag_name.is_empty() {
                    tag_name = "element".to_string();
                }

                let mut display_name = String::new();
                if tag_name == "section" {
                    if let Some(start_tag) = node.child(0) {
                        display_name = get_attribute_value_str(start_tag, text, "alias")
                            .unwrap_or_else(|| "Section".to_string());
                    }
                } else if tag_name == "heading" {
                    display_name = collect_node_text(node, text);
                    if display_name.len() > 15 {
                        display_name =
                            format!("{}...", display_name.chars().take(12).collect::<String>());
                    }
                } else if tag_name == "paragraph" {
                    display_name = collect_node_text(node, text);
                    if display_name.len() > 15 {
                        display_name =
                            format!("{}...", display_name.chars().take(12).collect::<String>());
                    }
                } else if tag_name == "hubref" {
                    let start_tag = if node.kind() == "element" {
                        node.child(0).unwrap_or(node)
                    } else {
                        node
                    };
                    let id_val = get_attribute_value_str(start_tag, text, "id")
                        .unwrap_or_else(|| "hubref".to_string());
                    display_name = format!("Ref: {}", id_val);
                }

                if display_name.trim().is_empty() {
                    display_name = tag_name.clone();
                }

                let start_offset = node.start_byte();
                let idx = nodes.len();
                nodes.push((
                    node,
                    OutlineNode {
                        id: format!("{}_{}", tag_name, idx),
                        name: display_name,
                        kind: tag_name,
                        start_offset,
                    },
                ));
                ts_id_to_idx.insert(node_id, idx);
            }
        }
    }

    let mut edges = Vec::new();
    for (idx, (node, _)) in nodes.iter().enumerate() {
        let mut parent = node.parent();
        while let Some(p) = parent {
            if let Some(&parent_idx) = ts_id_to_idx.get(&p.id()) {
                edges.push((parent_idx, idx));
                break;
            }
            parent = p.parent();
        }
    }

    let final_nodes = nodes.into_iter().map(|(_, n)| n).collect();
    (final_nodes, edges)
}

fn get_attribute_value_str(
    tag_node: tree_sitter::Node,
    text: &str,
    attr_name: &str,
) -> Option<String> {
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let Some(name_node) = child.child(0) {
                let name = &text[name_node.byte_range()];
                if name == attr_name {
                    if let Some(val_node) = child.child(2) {
                        return Some(
                            text[val_node.byte_range()]
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string(),
                        );
                    }
                }
            }
        }
    }
    None
}

fn collect_node_text(node: tree_sitter::Node, text: &str) -> String {
    if node.kind() == "text" {
        return text[node.byte_range()].to_string();
    }
    let mut text_acc = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "start_tag" && child.kind() != "end_tag" {
            let t = collect_node_text(child, text);
            if !t.is_empty() {
                if !text_acc.is_empty() && !text_acc.ends_with(' ') {
                    text_acc.push(' ');
                }
                text_acc.push_str(&t);
            }
        }
    }
    text_acc
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    let mut md = String::new();
    render_blocks_to_markdown(blocks, &mut md);
    md
}

fn render_blocks_to_markdown(blocks: &[Block], md: &mut String) {
    for block in blocks {
        match block {
            Block::Heading { level, text, .. } => {
                let hashes = "#".repeat(*level);
                md.push_str(&format!("{} {}\n\n", hashes, text));
            }
            Block::Paragraph { runs, .. } => {
                md.push_str(&format!("{}\n\n", runs_to_markdown(runs)));
            }
            Block::BlockQuote { runs, .. } => {
                md.push_str(&format!("> {}\n\n", runs_to_markdown(runs)));
            }
            Block::Aside { runs, .. } => {
                md.push_str(&format!("> [!NOTE]\n> {}\n\n", runs_to_markdown(runs)));
            }
            Block::CodeBlock { language, code, .. } => {
                md.push_str(&format!("```{}\n{}\n```\n\n", language, code));
            }
            Block::List { ordered, items, .. } => {
                for (idx, item) in items.iter().enumerate() {
                    let prefix = if let Some(checked) = item.checked {
                        if checked {
                            "- [x] "
                        } else {
                            "- [ ] "
                        }
                    } else if *ordered {
                        &format!("{}. ", idx + 1)
                    } else {
                        "- "
                    };
                    md.push_str(&format!("{}{}\n", prefix, runs_to_markdown(&item.runs)));
                }
                md.push('\n');
            }
            Block::DescriptionList { items, .. } => {
                for (term, runs) in items {
                    md.push_str(&format!("**{}**: {}\n", term, runs_to_markdown(runs)));
                }
                md.push('\n');
            }
            Block::Table { headers, rows, .. } => {
                if !headers.is_empty() {
                    md.push_str("| ");
                    for h in headers {
                        md.push_str(h.as_ref());
                        md.push_str(" | ");
                    }
                    md.push('\n');
                    md.push_str("| ");
                    for _ in headers {
                        md.push_str("--- | ");
                    }
                    md.push('\n');
                }
                for row in rows {
                    md.push_str("| ");
                    for cell in row {
                        md.push_str(&runs_to_markdown(cell));
                        md.push_str(" | ");
                    }
                    md.push('\n');
                }
                md.push('\n');
            }
            Block::HorizontalRule { .. } => {
                md.push_str("---\n\n");
            }
            Block::Image { src, alt, .. } => {
                let alt_str = alt.as_ref().map(|s| s.as_ref()).unwrap_or("");
                md.push_str(&format!("![{}]({})\n\n", alt_str, src));
            }
            Block::Audio { src, alt, .. } => {
                let alt_str = alt.as_ref().map(|s| s.as_ref()).unwrap_or("Audio");
                md.push_str(&format!("![{}]({})\n\n", alt_str, src));
            }
            Block::Video { src, alt, .. } => {
                let alt_str = alt.as_ref().map(|s| s.as_ref()).unwrap_or("Video");
                md.push_str(&format!("![{}]({})\n\n", alt_str, src));
            }
            Block::Details {
                summary,
                blocks: inner,
                ..
            } => {
                md.push_str(&format!("<details><summary>{}</summary>\n\n", summary));
                render_blocks_to_markdown(inner, md);
                md.push_str("</details>\n\n");
            }
            Block::Footnote { id, runs, .. } => {
                md.push_str(&format!("[^{}]: {}\n", id, runs_to_markdown(runs)));
            }
            Block::Review { blocks: inner, .. } => {
                md.push_str("> [!WARNING]\n> **Review required**\n");
                let mut inner_md = String::new();
                render_blocks_to_markdown(inner, &mut inner_md);
                for line in inner_md.lines() {
                    if !line.is_empty() {
                        md.push_str(&format!("> {}\n", line));
                    }
                }
                md.push('\n');
            }
            Block::Include { src, .. } => {
                let path = std::path::Path::new(src.as_str());
                let stem = path.file_stem().map_or("", |s| s.to_str().unwrap_or(""));
                md.push_str(&format!("![[{}]]\n\n", stem));
            }
        }
    }
}

fn runs_to_markdown(runs: &[TextRun]) -> String {
    let mut s = String::new();
    for run in runs {
        let mut prefix = String::new();
        let mut suffix = String::new();

        if run.bold {
            prefix.push_str("**");
            suffix.insert_str(0, "**");
        }
        if run.italic {
            prefix.push_str("*");
            suffix.insert_str(0, "*");
        }
        if run.underline {
            prefix.push_str("<u>");
            suffix.insert_str(0, "</u>");
        }
        if run.strikethrough {
            prefix.push_str("~~");
            suffix.insert_str(0, "~~");
        }
        if run.code {
            prefix.push_str("`");
            suffix.insert_str(0, "`");
        }
        if run.superscript {
            prefix.push_str("<sup>");
            suffix.insert_str(0, "</sup>");
        }
        if run.subscript {
            prefix.push_str("<sub>");
            suffix.insert_str(0, "</sub>");
        }

        if let Some(ref fn_ref) = run.footnote_ref {
            s.push_str(&format!("[^{}]", fn_ref));
        } else if let Some(ref hub_id) = run.hubref {
            s.push_str(&format!("[[{}|{}{}{}]]", hub_id, prefix, run.text, suffix));
        } else if let Some(ref href) = run.link {
            s.push_str(&format!("[{}{}{}]( {})", prefix, run.text, suffix, href));
        } else {
            s.push_str(&format!("{}{}{}", prefix, run.text, suffix));
        }
    }
    s
}
