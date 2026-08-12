//! TWXML parser adapter for TauWriter host — delegates core parsing, AST building,
//! outline extraction, and markdown generation to the `tauwriter-twxml` crate while
//! adapting strings to GPUI's `SharedString`.

use gpui::SharedString;
use serde::{Deserialize, Serialize};

pub use tauwriter_twxml::parse_document_outline;
pub use tauwriter_twxml::OutlineNode;

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

impl From<tauwriter_twxml::TextRun> for TextRun {
    fn from(r: tauwriter_twxml::TextRun) -> Self {
        Self {
            text: SharedString::from(r.text),
            bold: r.bold,
            italic: r.italic,
            underline: r.underline,
            strikethrough: r.strikethrough,
            code: r.code,
            superscript: r.superscript,
            subscript: r.subscript,
            hubref: r.hubref.map(SharedString::from),
            link: r.link.map(SharedString::from),
            footnote_ref: r.footnote_ref.map(SharedString::from),
            id: r.id.map(SharedString::from),
            attributes: convert_attrs(r.attributes),
            range: r.range,
        }
    }
}

impl From<&TextRun> for tauwriter_twxml::TextRun {
    fn from(r: &TextRun) -> Self {
        Self {
            text: r.text.to_string(),
            bold: r.bold,
            italic: r.italic,
            underline: r.underline,
            strikethrough: r.strikethrough,
            code: r.code,
            superscript: r.superscript,
            subscript: r.subscript,
            hubref: r.hubref.as_ref().map(|s| s.to_string()),
            link: r.link.as_ref().map(|s| s.to_string()),
            footnote_ref: r.footnote_ref.as_ref().map(|s| s.to_string()),
            id: r.id.as_ref().map(|s| s.to_string()),
            attributes: r.attributes.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            range: r.range.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub runs: Vec<TextRun>,
}

impl From<tauwriter_twxml::ListItem> for ListItem {
    fn from(item: tauwriter_twxml::ListItem) -> Self {
        Self {
            checked: item.checked,
            runs: item.runs.into_iter().map(TextRun::from).collect(),
        }
    }
}

impl From<&ListItem> for tauwriter_twxml::ListItem {
    fn from(item: &ListItem) -> Self {
        Self {
            checked: item.checked,
            runs: item.runs.iter().map(tauwriter_twxml::TextRun::from).collect(),
        }
    }
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

impl Block {
    pub fn range(&self) -> Option<std::ops::Range<usize>> {
        match self {
            Block::Heading { range, .. }
            | Block::Paragraph { range, .. }
            | Block::BlockQuote { range, .. }
            | Block::Aside { range, .. }
            | Block::CodeBlock { range, .. }
            | Block::List { range, .. }
            | Block::DescriptionList { range, .. }
            | Block::Table { range, .. }
            | Block::HorizontalRule { range, .. }
            | Block::Image { range, .. }
            | Block::Audio { range, .. }
            | Block::Video { range, .. }
            | Block::Details { range, .. }
            | Block::Footnote { range, .. }
            | Block::Review { range, .. }
            | Block::Include { range, .. } => range.clone(),
        }
    }
}

fn convert_attrs(attrs: Vec<(String, String)>) -> Vec<(SharedString, SharedString)> {
    attrs
        .into_iter()
        .map(|(k, v)| (SharedString::from(k), SharedString::from(v)))
        .collect()
}

fn convert_attrs_back(attrs: &[(SharedString, SharedString)]) -> Vec<(String, String)> {
    attrs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

impl From<tauwriter_twxml::Block> for Block {
    fn from(b: tauwriter_twxml::Block) -> Self {
        match b {
            tauwriter_twxml::Block::Heading { level, text, id, attributes, range } => Block::Heading {
                level,
                text: SharedString::from(text),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Paragraph { runs, id, attributes, range } => Block::Paragraph {
                runs: runs.into_iter().map(TextRun::from).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::BlockQuote { runs, id, attributes, range } => Block::BlockQuote {
                runs: runs.into_iter().map(TextRun::from).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Aside { runs, id, attributes, range } => Block::Aside {
                runs: runs.into_iter().map(TextRun::from).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::CodeBlock { language, code, id, attributes, range } => Block::CodeBlock {
                language: SharedString::from(language),
                code: SharedString::from(code),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::List { ordered, items, id, attributes, range } => Block::List {
                ordered,
                items: items.into_iter().map(ListItem::from).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::DescriptionList { items, id, attributes, range } => Block::DescriptionList {
                items: items.into_iter().map(|(t, r)| (SharedString::from(t), r.into_iter().map(TextRun::from).collect())).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Table { headers, rows, id, attributes, range } => Block::Table {
                headers: headers.into_iter().map(SharedString::from).collect(),
                rows: rows.into_iter().map(|row| row.into_iter().map(|cell| cell.into_iter().map(TextRun::from).collect()).collect()).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::HorizontalRule { id, attributes, range } => Block::HorizontalRule {
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Image { src, alt, id, attributes, range } => Block::Image {
                src: SharedString::from(src),
                alt: alt.map(SharedString::from),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Audio { src, alt, id, attributes, range } => Block::Audio {
                src: SharedString::from(src),
                alt: alt.map(SharedString::from),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Video { src, alt, id, attributes, range } => Block::Video {
                src: SharedString::from(src),
                alt: alt.map(SharedString::from),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Details { summary, blocks, id, attributes, range } => Block::Details {
                summary: SharedString::from(summary),
                blocks: blocks.into_iter().map(Block::from).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Footnote { id, runs, attributes, range } => Block::Footnote {
                id: SharedString::from(id),
                runs: runs.into_iter().map(TextRun::from).collect(),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Review { blocks, id, attributes, range } => Block::Review {
                blocks: blocks.into_iter().map(Block::from).collect(),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
            },
            tauwriter_twxml::Block::Include { src, id, attributes, range, resolved_blocks } => Block::Include {
                src: SharedString::from(src),
                id: id.map(SharedString::from),
                attributes: convert_attrs(attributes),
                range,
                resolved_blocks: resolved_blocks.map(|b| b.into_iter().map(Block::from).collect()),
            },
        }
    }
}

impl From<&Block> for tauwriter_twxml::Block {
    fn from(b: &Block) -> Self {
        match b {
            Block::Heading { level, text, id, attributes, range } => tauwriter_twxml::Block::Heading {
                level: *level,
                text: text.to_string(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Paragraph { runs, id, attributes, range } => tauwriter_twxml::Block::Paragraph {
                runs: runs.iter().map(tauwriter_twxml::TextRun::from).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::BlockQuote { runs, id, attributes, range } => tauwriter_twxml::Block::BlockQuote {
                runs: runs.iter().map(tauwriter_twxml::TextRun::from).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Aside { runs, id, attributes, range } => tauwriter_twxml::Block::Aside {
                runs: runs.iter().map(tauwriter_twxml::TextRun::from).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::CodeBlock { language, code, id, attributes, range } => tauwriter_twxml::Block::CodeBlock {
                language: language.to_string(),
                code: code.to_string(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::List { ordered, items, id, attributes, range } => tauwriter_twxml::Block::List {
                ordered: *ordered,
                items: items.iter().map(tauwriter_twxml::ListItem::from).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::DescriptionList { items, id, attributes, range } => tauwriter_twxml::Block::DescriptionList {
                items: items.iter().map(|(t, r)| (t.to_string(), r.iter().map(tauwriter_twxml::TextRun::from).collect())).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Table { headers, rows, id, attributes, range } => tauwriter_twxml::Block::Table {
                headers: headers.iter().map(|h| h.to_string()).collect(),
                rows: rows.iter().map(|row| row.iter().map(|cell| cell.iter().map(tauwriter_twxml::TextRun::from).collect()).collect()).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::HorizontalRule { id, attributes, range } => tauwriter_twxml::Block::HorizontalRule {
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Image { src, alt, id, attributes, range } => tauwriter_twxml::Block::Image {
                src: src.to_string(),
                alt: alt.as_ref().map(|s| s.to_string()),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Audio { src, alt, id, attributes, range } => tauwriter_twxml::Block::Audio {
                src: src.to_string(),
                alt: alt.as_ref().map(|s| s.to_string()),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Video { src, alt, id, attributes, range } => tauwriter_twxml::Block::Video {
                src: src.to_string(),
                alt: alt.as_ref().map(|s| s.to_string()),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Details { summary, blocks, id, attributes, range } => tauwriter_twxml::Block::Details {
                summary: summary.to_string(),
                blocks: blocks.iter().map(tauwriter_twxml::Block::from).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Footnote { id, runs, attributes, range } => tauwriter_twxml::Block::Footnote {
                id: id.to_string(),
                runs: runs.iter().map(tauwriter_twxml::TextRun::from).collect(),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Review { blocks, id, attributes, range } => tauwriter_twxml::Block::Review {
                blocks: blocks.iter().map(tauwriter_twxml::Block::from).collect(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
            },
            Block::Include { src, id, attributes, range, resolved_blocks } => tauwriter_twxml::Block::Include {
                src: src.to_string(),
                id: id.as_ref().map(|s| s.to_string()),
                attributes: convert_attrs_back(attributes),
                range: range.clone(),
                resolved_blocks: resolved_blocks.as_ref().map(|b| b.iter().map(tauwriter_twxml::Block::from).collect()),
            },
        }
    }
}

pub fn load_and_parse_twxml(
    path: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(SharedString, SharedString)>,
    Vec<Block>,
)> {
    let res = tauwriter_twxml::load_and_parse_twxml(path)?;
    Ok((
        res.title,
        res.author,
        res.metadata.into_iter().map(|(k, v)| (SharedString::from(k), SharedString::from(v))).collect(),
        res.blocks.into_iter().map(Block::from).collect(),
    ))
}

#[allow(dead_code)]
pub fn parse_twxml(
    xml_content: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(SharedString, SharedString)>,
    Vec<Block>,
)> {
    let res = tauwriter_twxml::parse_twxml(xml_content)?;
    Ok((
        res.title,
        res.author,
        res.metadata.into_iter().map(|(k, v)| (SharedString::from(k), SharedString::from(v))).collect(),
        res.blocks.into_iter().map(Block::from).collect(),
    ))
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
    let res = tauwriter_twxml::parse_twxml_internal(xml_content, base_dir, visited)?;
    Ok((
        res.title,
        res.author,
        res.metadata.into_iter().map(|(k, v)| (SharedString::from(k), SharedString::from(v))).collect(),
        res.blocks.into_iter().map(Block::from).collect(),
    ))
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    let raw_blocks: Vec<tauwriter_twxml::Block> = blocks.iter().map(tauwriter_twxml::Block::from).collect();
    tauwriter_twxml::blocks_to_markdown(&raw_blocks)
}

/// Generate default TWXML tag skeleton templates for slash menu insertion.
pub fn generate_block_skeleton(kind: &str) -> &'static str {
    match kind {
        "heading" | "h1" | "h2" | "h3" => "<heading>Heading Title</heading>",
        "section" => "<section>\n<heading>Section Title</heading>\n<paragraph>Section content</paragraph>\n</section>",
        "code" | "codeblock" => "<codeblock language=\"rust\">\nfn main() {}\n</codeblock>",
        "aside" => "<aside type=\"note\">\n<paragraph>Note callout</paragraph>\n</aside>",
        "details" => "<details>\n<summary>Details title</summary>\n<paragraph>Collapsible content</paragraph>\n</details>",
        "list" => "<ul>\n<li>List item</li>\n</ul>",
        "table" => "<table>\n<tr><th>Header 1</th><th>Header 2</th></tr>\n<tr><td>Cell 1</td><td>Cell 2</td></tr>\n</table>",
        "hubref" => "<paragraph><hubref id=\"new_instance\" /></paragraph>",
        _ => "<paragraph>New paragraph</paragraph>",
    }
}

/// Extract plain text content from a Block AST node.
pub fn extract_plain_text_from_block(block: &Block) -> String {
    match block {
        Block::Heading { text, .. } => text.to_string(),
        Block::Paragraph { runs, .. }
        | Block::BlockQuote { runs, .. }
        | Block::Aside { runs, .. } => runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join(""),
        Block::CodeBlock { code, .. } => code.to_string(),
        Block::List { items, .. } => items
            .iter()
            .flat_map(|it| it.runs.iter().map(|r| r.text.as_str()))
            .collect::<Vec<_>>()
            .join(" "),
        Block::Details { summary, .. } => summary.to_string(),
        _ => String::new(),
    }
}

/// Convert an existing Block AST node to a new TWXML markup tag format.
#[allow(dead_code)]
pub fn convert_block_to_twxml(block: &Block, target_type: &str) -> String {
    let text = extract_plain_text_from_block(block);
    let safe_text = if text.trim().is_empty() {
        "Block content"
    } else {
        text.trim()
    };

    match target_type {
        "heading" | "h1" | "h2" | "h3" => format!("<heading>{}</heading>", safe_text),
        "section" => format!("<section>\n<heading>{}</heading>\n</section>", safe_text),
        "blockquote" => format!("<blockquote>{}</blockquote>", safe_text),
        "code" => format!("<codeblock language=\"rust\">{}</codeblock>", safe_text),
        "list" => format!("<ul>\n<li>{}</li>\n</ul>", safe_text),
        "aside" => format!("<aside type=\"note\">\n<paragraph>{}</paragraph>\n</aside>", safe_text),
        _ => format!("<paragraph>{}</paragraph>", safe_text),
    }
}

/// Result of detecting a markdown typing shortcut trigger at line start.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum MarkdownTriggerResult {
    Heading(String),
    Section(String),
    BlockQuote(String),
    UnorderedList(String),
    OrderedList(String),
    CodeBlock(String),
    NoMatch,
}

/// Detects if a block text input starts with a markdown trigger prefix (e.g. `# `, `## `, `> `, `- `, `1. `, ```).
#[allow(dead_code)]
pub fn detect_markdown_prefix_trigger(input_text: &str) -> MarkdownTriggerResult {
    let trimmed = input_text.trim_start();
    if let Some(rest) = trimmed.strip_prefix("# ") {
        MarkdownTriggerResult::Heading(rest.trim_end().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("## ") {
        MarkdownTriggerResult::Section(rest.trim_end().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("### ") {
        MarkdownTriggerResult::Section(rest.trim_end().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("> ") {
        MarkdownTriggerResult::BlockQuote(rest.trim_end().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
    {
        MarkdownTriggerResult::UnorderedList(rest.trim_end().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("1. ") {
        MarkdownTriggerResult::OrderedList(rest.trim_end().to_string())
    } else if let Some(rest) = trimmed.strip_prefix("```") {
        MarkdownTriggerResult::CodeBlock(rest.trim_end().to_string())
    } else {
        MarkdownTriggerResult::NoMatch
    }
}

/// Detects if the current input caret word starts with `@` or `#` for HubRef LSP completion trigger.
#[allow(dead_code)]
pub fn detect_hubref_completion_trigger(input_text: &str, caret_offset: usize) -> Option<&str> {
    let safe_offset = caret_offset.min(input_text.len());
    let prefix = &input_text[..safe_offset];
    let last_word = prefix.split_whitespace().last()?;
    if last_word.starts_with('@') || last_word.starts_with('#') {
        Some(&last_word[1..])
    } else {
        None
    }
}

/// Reorders a block from src_range to target_range in document text markup.
pub fn reorder_blocks_in_document(
    doc_text: &str,
    src_range: std::ops::Range<usize>,
    target_range: std::ops::Range<usize>,
) -> String {
    if src_range == target_range || src_range.start >= doc_text.len() || target_range.start >= doc_text.len() {
        return doc_text.to_string();
    }

    let src_text = doc_text[src_range.clone()].to_string();
    let mut new_doc = doc_text.to_string();

    if src_range.start < target_range.start {
        // Moving downwards: remove src first, then insert at target (offset adjusted)
        new_doc.replace_range(src_range.clone(), "");
        let insert_pos = target_range.end.saturating_sub(src_range.len());
        let insert_pos = insert_pos.min(new_doc.len());
        new_doc.insert_str(insert_pos, &src_text);
    } else {
        // Moving upwards: insert before target_range.start, then remove src (offset adjusted)
        let insert_pos = target_range.start;
        new_doc.insert_str(insert_pos, &src_text);
        let remove_start = src_range.start + src_text.len();
        let remove_end = src_range.end + src_text.len();
        let remove_start = remove_start.min(new_doc.len());
        let remove_end = remove_end.min(new_doc.len());
        new_doc.replace_range(remove_start..remove_end, "");
    }

    new_doc
}

/// Converts table headers and rows into TWXML <table> markup.
pub fn table_to_twxml(headers: &[SharedString], rows: &[Vec<Vec<TextRun>>]) -> String {
    let mut out = String::from("<table>\n  <headers>\n");
    for h in headers {
        out.push_str(&format!("    <header>{}</header>\n", h));
    }
    out.push_str("  </headers>\n  <rows>\n");
    for r in rows {
        out.push_str("    <row>\n");
        for cell in r {
            out.push_str("      <cell>");
            for run in cell {
                let text = run.text.to_string();
                if run.bold {
                    out.push_str(&format!("<bold>{}</bold>", text));
                } else if run.italic {
                    out.push_str(&format!("<italic>{}</italic>", text));
                } else if run.code {
                    out.push_str(&format!("<code>{}</code>", text));
                } else if run.underline {
                    out.push_str(&format!("<u>{}</u>", text));
                } else {
                    out.push_str(&text);
                }
            }
            out.push_str("</cell>\n");
        }
        out.push_str("    </row>\n");
    }
    out.push_str("  </rows>\n</table>");
    out
}

/// Adds a new empty row to a table matrix.
pub fn table_add_row(col_count: usize, rows: &mut Vec<Vec<Vec<TextRun>>>, at_idx: usize) {
    let new_row = vec![vec![]; col_count.max(1)];
    let insert_pos = at_idx.min(rows.len());
    rows.insert(insert_pos, new_row);
}

/// Deletes a row from a table matrix if more than 1 row remains.
pub fn table_delete_row(rows: &mut Vec<Vec<Vec<TextRun>>>, at_idx: usize) {
    if rows.len() > 1 && at_idx < rows.len() {
        rows.remove(at_idx);
    }
}

/// Adds a new column to a table matrix.
pub fn table_add_column(
    headers: &mut Vec<SharedString>,
    rows: &mut Vec<Vec<Vec<TextRun>>>,
    at_idx: usize,
) {
    let insert_pos = at_idx.min(headers.len());
    headers.insert(insert_pos, "Header".into());
    for row in rows.iter_mut() {
        let row_pos = at_idx.min(row.len());
        row.insert(row_pos, vec![]);
    }
}

/// Deletes a column from a table matrix if more than 1 column remains.
pub fn table_delete_column(
    headers: &mut Vec<SharedString>,
    rows: &mut Vec<Vec<Vec<TextRun>>>,
    at_idx: usize,
) {
    if headers.len() > 1 && at_idx < headers.len() {
        headers.remove(at_idx);
        for row in rows.iter_mut() {
            if at_idx < row.len() {
                row.remove(at_idx);
            }
        }
    }
}

/// Normalizes multiline block text into a single continuous string for single-line block card editing.
/// Collapses formatting newlines and multiline indentation while preserving internal word spaces.
pub fn normalize_block_text_for_editing(text: &str) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    lines.join(" ")
}

/// Wraps plain text selection in TWXML inline formatting markup (bold, italic, code, underline, hubref).
pub fn wrap_text_in_inline_format(text: &str, format_kind: &str, target_id: Option<&str>) -> String {
    let trimmed = text.trim();
    let safe_text = if trimmed.is_empty() { "text" } else { trimmed };
    match format_kind {
        "bold" | "b" => format!("<bold>{}</bold>", safe_text),
        "italic" | "i" => format!("<italic>{}</italic>", safe_text),
        "underline" | "u" => format!("<u>{}</u>", safe_text),
        "code" => format!("<code>{}</code>", safe_text),
        "hubref" => {
            let id = target_id.unwrap_or("target_id");
            format!("<hubref id=\"{}\">{}</hubref>", id, safe_text)
        }
        _ => safe_text.to_string(),
    }
}

