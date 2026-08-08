use crate::types::{Block, ListItem, TextRun};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn load_and_parse_twxml(
    path: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(String, String)>,
    Vec<Block>,
)> {
    let xml_content = std::fs::read_to_string(path)?;
    let base_dir = Path::new(path).parent();
    let mut visited = HashSet::new();
    if let Ok(abs_path) = Path::new(path).canonicalize() {
        visited.insert(abs_path);
    }
    parse_twxml_internal(&xml_content, base_dir, &mut visited)
}

pub fn parse_twxml(
    xml_content: &str,
) -> anyhow::Result<(
    String,
    String,
    Vec<(String, String)>,
    Vec<Block>,
)> {
    let mut visited = HashSet::new();
    parse_twxml_internal(xml_content, None, &mut visited)
}

pub fn parse_twxml_internal(
    xml_content: &str,
    base_dir: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
) -> anyhow::Result<(
    String,
    String,
    Vec<(String, String)>,
    Vec<Block>,
)> {
    let doc = roxmltree::Document::parse(xml_content)?;
    let root = doc.root_element();

    let mut title = String::new();
    let mut author = String::new();
    let mut metadata: Vec<(String, String)> = Vec::new();
    let mut blocks = Vec::new();

    for child in root.children() {
        if child.has_tag_name("meta") {
            let name = child.attribute("name").unwrap_or("");
            let content = child.attribute("content").unwrap_or("");
            metadata.push((name.to_string(), content.to_string()));
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

const MAX_PARSE_DEPTH: usize = 128;

fn parse_node(
    node: roxmltree::Node,
    depth: usize,
    blocks: &mut Vec<Block>,
    base_dir: Option<&Path>,
    visited: &mut HashSet<PathBuf>,
) {
    if depth > MAX_PARSE_DEPTH {
        return;
    }
    let range = Some(node.range());
    let id = node.attribute("id").map(|s| s.to_string());
    let attributes: Vec<(String, String)> = node
        .attributes()
        .map(|a| (a.name().to_string(), a.value().to_string()))
        .collect();

    if node.has_tag_name("section") {
        for child in node.children() {
            parse_node(child, depth + 1, blocks, base_dir, visited);
        }
    } else if node.has_tag_name("include") {
        let src_val = node.attribute("src").unwrap_or("");
        let src = src_val.to_string();
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
        let language = node.attribute("language").unwrap_or("").to_string();
        let code = node.text().unwrap_or("").to_string();
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
                items.push((current_term.clone(), runs));
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
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").map(|s| s.to_string());
        blocks.push(Block::Image {
            src,
            alt,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("audio") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").map(|s| s.to_string());
        blocks.push(Block::Audio {
            src,
            alt,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("video") {
        let src = node.attribute("src").unwrap_or("").to_string();
        let alt = node.attribute("alt").map(|s| s.to_string());
        blocks.push(Block::Video {
            src,
            alt,
            id,
            attributes,
            range,
        });
    } else if node.has_tag_name("details") {
        let mut summary = "Details".to_string();
        let mut details_blocks = Vec::new();
        for child in node.children() {
            if child.has_tag_name("summary") {
                summary = collect_text(child);
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
        let footnote_id = node.attribute("id").unwrap_or("").to_string();
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

fn collect_text(node: roxmltree::Node) -> String {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs, &Style::default());
    normalize_runs(&mut runs);
    runs.iter().map(|r| r.text.as_str()).collect()
}

fn collect_and_normalize_runs(node: roxmltree::Node) -> Vec<TextRun> {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs, &Style::default());
    normalize_runs(&mut runs);
    runs
}

#[derive(Clone, Default)]
struct Style {
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    code: bool,
    superscript: bool,
    subscript: bool,
    hubref: Option<String>,
    link: Option<String>,
    footnote_ref: Option<String>,
    id: Option<String>,
    attributes: Vec<(String, String)>,
}

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
                let child_attrs: Vec<(String, String)> = child
                    .attributes()
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
                let child_attrs: Vec<(String, String)> = child
                    .attributes()
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

enum NextElement {
    Char(char),
    LineBreak,
    None,
}

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
