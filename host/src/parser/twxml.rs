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
    let (title, author, meta, blocks) = tauwriter_twxml::load_and_parse_twxml(path)?;
    Ok((
        title,
        author,
        meta.into_iter().map(|(k, v)| (SharedString::from(k), SharedString::from(v))).collect(),
        blocks.into_iter().map(Block::from).collect(),
    ))
}

pub fn parse_twxml(
    xml_content: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(SharedString, SharedString)>,
    Vec<Block>,
)> {
    let (title, author, meta, blocks) = tauwriter_twxml::parse_twxml(xml_content)?;
    Ok((
        title,
        author,
        meta.into_iter().map(|(k, v)| (SharedString::from(k), SharedString::from(v))).collect(),
        blocks.into_iter().map(Block::from).collect(),
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
    let (title, author, meta, blocks) =
        tauwriter_twxml::parse_twxml_internal(xml_content, base_dir, visited)?;
    Ok((
        title,
        author,
        meta.into_iter().map(|(k, v)| (SharedString::from(k), SharedString::from(v))).collect(),
        blocks.into_iter().map(Block::from).collect(),
    ))
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    let raw_blocks: Vec<tauwriter_twxml::Block> = blocks.iter().map(tauwriter_twxml::Block::from).collect();
    tauwriter_twxml::blocks_to_markdown(&raw_blocks)
}
