//! TWXML document parsing — converts raw XML text into `renderer_schema::Block` trees.
//!
//! The host currently uses `roxmltree` for parsing because it returns typed
//! [`Block`](renderer_schema::Block) objects consumed by the WASM-free preview pipeline.
//! This is separate from `tauwriter_lsp::parser::twxml`, which extracts HubReferences
//! via tree-sitter for diagnostics/completion.  The two parsers serve different domains;
//! eventual unification would require a shared AST layer.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub hubref: Option<String>,
    pub link: Option<String>,
}

impl Default for TextRun {
    fn default() -> Self {
        Self {
            text: String::new(),
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            code: false,
            superscript: false,
            subscript: false,
            hubref: None,
            link: None,
        }
    }
}

impl TextRun {
    pub fn new(text: impl Into<String>) -> Self {
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
    Heading { level: usize, text: String },
    Paragraph { runs: Vec<TextRun> },
    BlockQuote { runs: Vec<TextRun> },
    Aside { runs: Vec<TextRun> },
    CodeBlock { language: String, code: String },
    List { ordered: bool, items: Vec<ListItem> },
    DescriptionList { items: Vec<(String, Vec<TextRun>)> },
    Table { headers: Vec<String>, rows: Vec<Vec<Vec<TextRun>>> },
    HorizontalRule,
    Image { src: String, alt: String },
    Audio { src: String, alt: String },
    Video { src: String, alt: String },
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
    hubref: Option<String>,
    link: Option<String>,
}

/// Load a TWXML file and parse it into (title, author, blocks).
pub fn load_and_parse_twxml(path: &str) -> anyhow::Result<(String, String, Vec<Block>)> {
    let xml_content = std::fs::read_to_string(path)?;
    parse_twxml(&xml_content)
}

/// Parse TWXML XML content into a document model.
pub fn parse_twxml(xml_content: &str) -> anyhow::Result<(String, String, Vec<Block>)> {
    let doc = roxmltree::Document::parse(xml_content)?;
    let root = doc.root_element();

    let mut title = String::new();
    let mut author = String::new();
    let mut blocks = Vec::new();

    for child in root.children() {
        if child.has_tag_name("meta") {
            let name = child.attribute("name").unwrap_or("");
            let content = child.attribute("content").unwrap_or("");
            if name == "title" {
                title = content.to_string();
            } else if name == "author" {
                author = content.to_string();
            }
        }
    }

    if let Some(body) = root.children().find(|c| c.has_tag_name("body")) {
        for child in body.children() {
            parse_node(child, 0, &mut blocks);
        }
    }

    Ok((title, author, blocks))
}

/// Recursively convert a TWXML element node into one or more `Block` entries.
fn parse_node(node: roxmltree::Node, depth: usize, blocks: &mut Vec<Block>) {
    if node.has_tag_name("section") {
        for child in node.children() {
            parse_node(child, depth + 1, blocks);
        }
    } else if node.has_tag_name("heading") {
        let text = collect_text(node);
        blocks.push(Block::Heading {
            level: depth + 1,
            text,
        });
    } else if node.has_tag_name("paragraph") {
        let mut runs = Vec::new();
        collect_runs(node, &mut runs, &Style::default());
        blocks.push(Block::Paragraph { runs });
    } else if node.has_tag_name("blockquote") {
        let mut runs = Vec::new();
        collect_runs(node, &mut runs, &Style::default());
        blocks.push(Block::BlockQuote { runs });
    } else if node.has_tag_name("aside") {
        let mut runs = Vec::new();
        collect_runs(node, &mut runs, &Style::default());
        blocks.push(Block::Aside { runs });
    } else if node.has_tag_name("codeblock") {
        let language = node.attribute("language").unwrap_or("").to_string();
        let code = node.text().unwrap_or("").to_string();
        blocks.push(Block::CodeBlock { language, code });
    } else if node.has_tag_name("ul") || node.has_tag_name("ol") {
        let ordered = node.has_tag_name("ol");
        let mut items = Vec::new();
        for child in node.children() {
            if child.has_tag_name("li") {
                let checked = child
                    .attribute("checked")
                    .and_then(|v| v.parse::<bool>().ok());
                let mut runs = Vec::new();
                collect_runs(child, &mut runs, &Style::default());
                items.push(ListItem { checked, runs });
            }
        }
        blocks.push(Block::List { ordered, items });
    } else if node.has_tag_name("dl") {
        let mut items = Vec::new();
        let mut current_term = String::new();
        for child in node.children() {
            if child.has_tag_name("dt") {
                current_term = collect_text(child);
            } else if child.has_tag_name("dd") {
                let mut runs = Vec::new();
                collect_runs(child, &mut runs, &Style::default());
                items.push((current_term.clone(), runs));
            }
        }
        blocks.push(Block::DescriptionList { items });
    } else if node.has_tag_name("table") {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        for child in node.children() {
            if child.has_tag_name("tr") {
                let mut row_cells = Vec::new();
                for cell in child.children() {
                    if cell.has_tag_name("th") {
                        headers.push(collect_text(cell));
                    } else if cell.has_tag_name("td") {
                        let mut runs = Vec::new();
                        collect_runs(cell, &mut runs, &Style::default());
                        row_cells.push(runs);
                    }
                }
                if !row_cells.is_empty() {
                    rows.push(row_cells);
                }
            }
        }
        blocks.push(Block::Table { headers, rows });
    } else if node.has_tag_name("hr") {
        blocks.push(Block::HorizontalRule);
    } else if node.has_tag_name("image") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").unwrap_or("").to_string();
        blocks.push(Block::Image { src, alt });
    } else if node.has_tag_name("audio") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").unwrap_or("").to_string();
        blocks.push(Block::Audio { src, alt });
    } else if node.has_tag_name("video") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").unwrap_or("").to_string();
        blocks.push(Block::Video { src, alt });
    } else if node.has_tag_name("details") {
        for child in node.children() {
            parse_node(child, depth, blocks);
        }
    } else if node.is_element() {
        for child in node.children() {
            parse_node(child, depth, blocks);
        }
    }
}

/// Extract all text content from a node and its descendants.
fn collect_text(node: roxmltree::Node) -> String {
    let mut text = String::new();
    for desc in node.descendants() {
        if desc.is_text() {
            text.push_str(desc.text().unwrap_or(""));
        }
    }
    text
}

/// Recursively collect styled `TextRun` segments, threading the inline style
/// through the subtree via [`Style`] accumulators.
fn collect_runs(node: roxmltree::Node, runs: &mut Vec<TextRun>, current_style: &Style) {
    for child in node.children() {
        if child.is_text() {
            let text = child.text().unwrap_or("").to_string();
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
                });
            }
        } else if child.is_element() {
            let mut next_style = current_style.clone();
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
                next_style.hubref = child.attribute("id").map(|s| s.to_string());
            } else if child.has_tag_name("link") {
                next_style.link = child.attribute("href").map(|s| s.to_string());
            }
            collect_runs(child, runs, &next_style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_twxml_basic_structures_match() {
        let xml = r#"
        <document>
          <meta name="title" content="Test Document"/>
          <meta name="author" content="Test Author"/>
          <body>
            <heading>Chapter 1</heading>
            <paragraph>Hello <bold>world</bold>!</paragraph>
          </body>
        </document>
        "#;
        let (title, author, blocks) = parse_twxml(xml).unwrap();
        assert_eq!(title, "Test Document");
        assert_eq!(author, "Test Author");
        assert_eq!(blocks.len(), 2);

        match &blocks[0] {
            Block::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Chapter 1");
            }
            _ => panic!("Expected Heading"),
        }

        match &blocks[1] {
            Block::Paragraph { runs } => {
                assert_eq!(runs.len(), 3);
                assert_eq!(runs[0].text, "Hello ");
                assert_eq!(runs[0].bold, false);
                assert_eq!(runs[1].text, "world");
                assert_eq!(runs[1].bold, true);
                assert_eq!(runs[2].text, "!");
                assert_eq!(runs[2].bold, false);
            }
            _ => panic!("Expected Paragraph"),
        }
    }
}
