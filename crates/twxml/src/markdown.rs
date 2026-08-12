use crate::types::{Block, TextRun};

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
                            "- [x] ".to_string()
                        } else {
                            "- [ ] ".to_string()
                        }
                    } else if *ordered {
                        format!("{}. ", idx + 1)
                    } else {
                        "- ".to_string()
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
                        md.push_str(h);
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
                let alt_str = alt.as_deref().unwrap_or("");
                md.push_str(&format!("![{}]({})\n\n", alt_str, src));
            }
            Block::Audio { src, alt, .. } => {
                let alt_str = alt.as_deref().unwrap_or("Audio");
                md.push_str(&format!("![{}]({})\n\n", alt_str, src));
            }
            Block::Video { src, alt, .. } => {
                let alt_str = alt.as_deref().unwrap_or("Video");
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
            prefix.push('*');
            suffix.insert(0, '*');
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
            prefix.push('`');
            suffix.insert(0, '`');
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
