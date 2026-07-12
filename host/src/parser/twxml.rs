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
    pub footnote_ref: Option<String>,
    pub id: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub range: Option<std::ops::Range<usize>>,
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
            footnote_ref: None,
            id: None,
            attributes: Vec::new(),
            range: None,
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
    Heading { level: usize, text: String, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Paragraph { runs: Vec<TextRun>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    BlockQuote { runs: Vec<TextRun>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Aside { runs: Vec<TextRun>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    CodeBlock { language: String, code: String, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    List { ordered: bool, items: Vec<ListItem>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    DescriptionList { items: Vec<(String, Vec<TextRun>)>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Table { headers: Vec<String>, rows: Vec<Vec<Vec<TextRun>>>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    HorizontalRule { id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Image { src: String, alt: String, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Audio { src: String, alt: String, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Video { src: String, alt: String, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Details { summary: String, blocks: Vec<Block>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Footnote { id: String, runs: Vec<TextRun>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
    Review { blocks: Vec<Block>, id: Option<String>, attributes: Vec<(String, String)>, range: Option<std::ops::Range<usize>> },
}


/// Load a TWXML file and parse it into (title, author, metadata, blocks).
pub fn load_and_parse_twxml(path: &str) -> anyhow::Result<(String, String, Vec<(String, String)>, Vec<Block>)> {
    let xml_content = std::fs::read_to_string(path)?;
    parse_twxml(&xml_content)
}

/// Parse TWXML XML content into a document model.
pub fn parse_twxml(xml_content: &str) -> anyhow::Result<(String, String, Vec<(String, String)>, Vec<Block>)> {
    let doc = roxmltree::Document::parse(xml_content)?;
    let root = doc.root_element();

    let mut title = String::new();
    let mut author = String::new();
    let mut metadata = Vec::new();
    let mut blocks = Vec::new();

    for child in root.children() {
        if child.has_tag_name("meta") {
            let name = child.attribute("name").unwrap_or("").to_string();
            let content = child.attribute("content").unwrap_or("").to_string();
            metadata.push((name.clone(), content.clone()));
            if name == "title" {
                title = content;
            } else if name == "author" {
                author = content;
            }
        }
    }

    if let Some(body) = root.children().find(|c| c.has_tag_name("body")) {
        for child in body.children() {
            parse_node(child, 0, &mut blocks);
        }
    }

    Ok((title, author, metadata, blocks))
}

/// Recursively convert a TWXML element node into one or more `Block` entries.
fn parse_node(node: roxmltree::Node, depth: usize, blocks: &mut Vec<Block>) {
    let range = Some(node.range());
    let id = node.attribute("id").map(|s| s.to_string());
    let attributes: Vec<(String, String)> = node.attributes()
        .map(|a| (a.name().to_string(), a.value().to_string()))
        .collect();

    if node.has_tag_name("section") {
        for child in node.children() {
            parse_node(child, depth + 1, blocks);
        }
    } else if node.has_tag_name("heading") {
        let text = collect_text(node);
        blocks.push(Block::Heading {
            level: depth + 1,
            text,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("paragraph") {
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::Paragraph { runs, id, attributes, range });
    } else if node.has_tag_name("blockquote") {
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::BlockQuote { runs, id, attributes, range });
    } else if node.has_tag_name("aside") {
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::Aside { runs, id, attributes, range });
    } else if node.has_tag_name("codeblock") {
        let language = node.attribute("language").unwrap_or("").to_string();
        let code = node.text().unwrap_or("").to_string();
        blocks.push(Block::CodeBlock { language, code, id, attributes, range });
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
        blocks.push(Block::List { ordered, items, id, attributes, range });
    } else if node.has_tag_name("dl") {
        let mut items = Vec::new();
        let mut current_term = String::new();
        for child in node.children() {
            if child.has_tag_name("dt") {
                current_term = collect_text(child);
            } else if child.has_tag_name("dd") {
                let runs = collect_and_normalize_runs(child);
                items.push((current_term.clone(), runs));
            }
        }
        blocks.push(Block::DescriptionList { items, id, attributes, range });
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
                        let runs = collect_and_normalize_runs(cell);
                        row_cells.push(runs);
                    }
                }
                if !row_cells.is_empty() {
                    rows.push(row_cells);
                }
            }
        }
        blocks.push(Block::Table { headers, rows, id, attributes, range });
    } else if node.has_tag_name("hr") {
        blocks.push(Block::HorizontalRule { id, attributes, range });
    } else if node.has_tag_name("image") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").unwrap_or("").to_string();
        blocks.push(Block::Image { src, alt, id, attributes, range });
    } else if node.has_tag_name("audio") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").unwrap_or("").to_string();
        blocks.push(Block::Audio { src, alt, id, attributes, range });
    } else if node.has_tag_name("video") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").unwrap_or("").to_string();
        blocks.push(Block::Video { src, alt, id, attributes, range });
    } else if node.has_tag_name("details") {
        let mut summary = "Details".to_string();
        let mut details_blocks = Vec::new();
        for child in node.children() {
            if child.has_tag_name("summary") {
                summary = collect_text(child);
            } else {
                parse_node(child, depth + 1, &mut details_blocks);
            }
        }
        blocks.push(Block::Details { summary, blocks: details_blocks, id, attributes, range });
    } else if node.has_tag_name("footnote") {
        let footnote_id = node.attribute("id").unwrap_or("").to_string();
        let runs = collect_and_normalize_runs(node);
        blocks.push(Block::Footnote { id: footnote_id, runs, attributes: attributes.clone(), range });
    } else if node.has_tag_name("review") {
        let mut review_blocks = Vec::new();
        for child in node.children() {
            parse_node(child, depth + 1, &mut review_blocks);
        }
        blocks.push(Block::Review { blocks: review_blocks, id, attributes, range });
    } else if node.is_element() {
        for child in node.children() {
            parse_node(child, depth, blocks);
        }
    }
}

/// Extract all text content from a node and its descendants.
fn collect_text(node: roxmltree::Node) -> String {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs, &Style::default());
    normalize_runs(&mut runs);
    runs.into_iter().map(|r| r.text).collect()
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
    hubref: Option<String>,
    link: Option<String>,
    footnote_ref: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) attributes: Vec<(String, String)>,
}


/// Recursively collect styled `TextRun` segments, threading the inline style
/// through the subtree via [`Style`] accumulators.
fn collect_runs(node: roxmltree::Node, runs: &mut Vec<TextRun>, current_style: &Style) {
    for child in node.children() {
        let range = Some(child.range());
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
                    footnote_ref: current_style.footnote_ref.clone(),
                    id: current_style.id.clone(),
                    attributes: current_style.attributes.clone(),
                    range,
                });
            }
        } else if child.is_element() {
            if child.has_tag_name("br") {
                runs.push(TextRun {
                    text: "\n".to_string(),
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
                let fr_id = child.attribute("id").unwrap_or("").to_string();
                let child_attrs: Vec<(String, String)> = child.attributes()
                    .map(|a| (a.name().to_string(), a.value().to_string()))
                    .collect();
                runs.push(TextRun {
                    text: format!("[{}]", fr_id),
                    subscript: true,
                    footnote_ref: Some(fr_id.clone()),
                    id: Some(fr_id),
                    attributes: child_attrs,
                    range,
                    ..Default::default()
                });
            } else {
                let mut next_style = current_style.clone();
                let child_id = child.attribute("id").map(|s| s.to_string());
                let child_attrs: Vec<(String, String)> = child.attributes()
                    .map(|a| (a.name().to_string(), a.value().to_string()))
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
                    next_style.hubref = child.attribute("id").map(|s| s.to_string());
                } else if child.has_tag_name("link") {
                    next_style.link = child.attribute("href").map(|s| s.to_string());
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

fn find_next_semantic_element(runs: &[TextRun], start_run_idx: usize, start_char_idx: usize) -> NextElement {
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

fn find_prev_semantic_element(runs: &[TextRun], start_run_idx: usize, start_char_idx: usize) -> PrevElement {
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
            new_texts.push("\n".to_string());
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
                    let is_punctuation = ['.', ',', ':', ';', '!', '?', ')', ']', '}'].contains(&next_c);
                    if !is_punctuation {
                        normalized.push(' ');
                    }
                }
            } else {
                normalized.push(c);
                i += 1;
            }
        }
        new_texts.push(normalized);
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
        let (title, author, _metadata, blocks) = parse_twxml(xml).unwrap();
        assert_eq!(title, "Test Document");
        assert_eq!(author, "Test Author");
        assert_eq!(blocks.len(), 2);

        match &blocks[0] {
            Block::Heading { level, text, .. } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Chapter 1");
            }
            _ => panic!("Expected Heading"),
        }

        match &blocks[1] {
            Block::Paragraph { runs, .. } => {
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

    #[test]
    fn test_parser_whitespace_normalization_and_br_handling_succeeds() {
        let xml = r#"
        <document>
          <body>
            <paragraph>
              This is a very long paragraph that has
              been split across multiple lines for source
              readability.
              It also contains  intentional  double spaces.
              And punctuation,
              like this!
              Here is a line break:<br/>
              And another line break:<br />
              And some <bold>bold
              text</bold> on multiple lines.
            </paragraph>
          </body>
        </document>
        "#;
        let (_, _, _metadata, blocks) = parse_twxml(xml).unwrap();
        assert_eq!(blocks.len(), 1);

        match &blocks[0] {
            Block::Paragraph { runs, .. } => {
                let text_content: String = runs.iter().map(|r| r.text.as_str()).collect();
                // Check if newlines are removed, double spaces preserved, br tags are \n, and punctuation has no extra space.
                let expected = "This is a very long paragraph that has been split across multiple lines for source readability. It also contains  intentional  double spaces. And punctuation, like this! Here is a line break:\nAnd another line break:\nAnd some bold text on multiple lines.";
                assert_eq!(text_content, expected);

                // Verify that bold style run was correctly normalized and preserved.
                let bold_run = runs.iter().find(|r| r.bold).unwrap();
                assert_eq!(bold_run.text, "bold text");
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parser_extracts_correct_character_ranges() {
        let xml = "<document><body><heading>Chapter 1</heading><paragraph>Hello <bold>world</bold>!</paragraph></body></document>";
        let (_, _, _metadata, blocks) = parse_twxml(xml).unwrap();
        assert_eq!(blocks.len(), 2);

        // Heading block
        match &blocks[0] {
            Block::Heading { text, range, .. } => {
                assert_eq!(text, "Chapter 1");
                let range = range.as_ref().unwrap();
                assert_eq!(&xml[range.clone()], "<heading>Chapter 1</heading>");
            }
            _ => panic!("Expected Heading"),
        }

        // Paragraph block
        match &blocks[1] {
            Block::Paragraph { runs, range, .. } => {
                let range = range.as_ref().unwrap();
                assert_eq!(&xml[range.clone()], "<paragraph>Hello <bold>world</bold>!</paragraph>");

                assert_eq!(runs.len(), 3);
                // "Hello " text run
                let r0 = runs[0].range.as_ref().unwrap();
                assert_eq!(&xml[r0.clone()], "Hello ");

                // "<bold>world</bold>" -> text run "world"
                let r1 = runs[1].range.as_ref().unwrap();
                assert_eq!(&xml[r1.clone()], "world");

                // "!" text run
                let r2 = runs[2].range.as_ref().unwrap();
                assert_eq!(&xml[r2.clone()], "!");
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parser_handles_details_footnotes_and_reviews_properly() {
        let xml = r#"
        <document>
          <body>
            <details>
              <summary>Show details</summary>
              <paragraph>Hidden paragraph.</paragraph>
            </details>
            <paragraph>Inline footnote reference: <fr id="99"/>.</paragraph>
            <footnote id="99">
              Footnote content.
            </footnote>
            <review>
              <paragraph>To be reviewed.</paragraph>
            </review>
          </body>
        </document>
        "#;
        let (_, _, _metadata, blocks) = parse_twxml(xml).unwrap();
        assert_eq!(blocks.len(), 4);

        // 1. Details block
        match &blocks[0] {
            Block::Details { summary, blocks: inner_blocks, .. } => {
                assert_eq!(summary, "Show details");
                assert_eq!(inner_blocks.len(), 1);
                match &inner_blocks[0] {
                    Block::Paragraph { runs, .. } => {
                        assert_eq!(runs[0].text, "Hidden paragraph.");
                    }
                    _ => panic!("Expected paragraph inside details"),
                }
            }
            _ => panic!("Expected Details block"),
        }

        // 2. Paragraph with fr tag
        match &blocks[1] {
            Block::Paragraph { runs, .. } => {
                assert_eq!(runs.len(), 3);
                assert_eq!(runs[0].text, "Inline footnote reference: ");
                assert_eq!(runs[1].text, "[99]");
                assert_eq!(runs[1].footnote_ref.as_deref(), Some("99"));
                assert!(runs[1].subscript);
                assert_eq!(runs[2].text, ".");
            }
            _ => panic!("Expected Paragraph block"),
        }

        // 3. Footnote definition block
        match &blocks[2] {
            Block::Footnote { id, runs, .. } => {
                assert_eq!(id, "99");
                assert_eq!(runs[0].text, "Footnote content.");
            }
            _ => panic!("Expected Footnote block"),
        }

        // 4. Review block
        match &blocks[3] {
            Block::Review { blocks: inner_blocks, .. } => {
                assert_eq!(inner_blocks.len(), 1);
                match &inner_blocks[0] {
                    Block::Paragraph { runs, .. } => {
                        assert_eq!(runs[0].text, "To be reviewed.");
                    }
                    _ => panic!("Expected paragraph inside review"),
                }
            }
            _ => panic!("Expected Review block"),
        }
    }
}

