// Formatting related features

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn formatting(
    server: &Backend,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;

    if let Some(content) = server.open_files.get(&uri).map(|r| r.to_string()) {
        let file_type = if uri.as_str().ends_with(".twxml") {
            "twxml"
        } else if uri.as_str().ends_with(".hubgs") {
            "hubgs"
        } else {
            return Ok(None);
        };

        let new_text = crate::formatter::format_source(&content, file_type);
        let last_line_len = content.lines().last().map(|l| l.len()).unwrap_or(0) as u32;
        let line_count = content.lines().count() as u32;
        let end_line = if line_count > 0 { line_count - 1 } else { 0 };

        let edit = TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: end_line,
                    character: last_line_len,
                },
            },
            new_text,
        };

        return Ok(Some(vec![edit]));
    }

    Ok(None)
}

/// Auto-close TWXML tags when the user types `>`.
pub async fn on_type_formatting(
    server: &Backend,
    params: DocumentOnTypeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    if params.ch != ">" {
        return Ok(None);
    }

    let uri = params.text_document_position.text_document.uri;
    if !uri.as_str().ends_with(".twxml") {
        return Ok(None);
    }

    let position = params.text_document_position.position;

    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => return Ok(None),
    };

    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return Ok(None);
    }

    let line = lines[line_idx];

    let Some(tag_name) = extract_opening_tag_name(&line, position.character as usize) else {
        return Ok(None);
    };

    if tag_name == "metadata" {
        return Ok(None);
    }

    let closing = format!("</{}>", tag_name);

    let edit = TextEdit {
        range: Range {
            start: position,
            end: position,
        },
        new_text: closing,
    };

    Ok(Some(vec![edit]))
}

/// Extract the tag name from an opening tag at the end of `text`.
/// Returns `None` for closing tags, self-closing tags, comments,
/// or already-balanced tags on the same line.
fn extract_opening_tag_name(text: &str, cursor: usize) -> Option<String> {
    let text = if cursor <= text.len() {
        &text[..cursor]
    } else {
        text
    };

    let trimmed = text.trim_end();
    if !trimmed.ends_with('>') {
        return None;
    }

    let after_last_lt = trimmed.rfind('<')?;
    let between = &trimmed[after_last_lt..];

    // Skip comments: <!--
    if between.starts_with("<!--") {
        return None;
    }

    // Skip closing tags: </
    if between.starts_with("</") {
        return None;
    }

    // Skip self-closing: <.../>
    if between.ends_with("/>") {
        return None;
    }

    // Skip if line already has a matching closing tag after this opening tag
    let rest = &text[(after_last_lt + between.len())..];
    let tag_name_candidate = extract_name_from_tag(between)?;
    let closing_pattern = format!("</{}>", tag_name_candidate);
    if rest.starts_with(&closing_pattern) {
        return None;
    }

    Some(tag_name_candidate)
}

/// Extract just the tag name from a start tag string like `<section id="1">`.
fn extract_name_from_tag(tag: &str) -> Option<String> {
    let inner = tag.strip_prefix('<')?.strip_suffix('>')?;
    let name = inner
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()?
        .to_string();

    if name.is_empty() {
        return None;
    }

    Some(name)
}

/// Computes the format edit by comparing original and formatted text line-by-line.
/// Returns `None` if nothing changed, otherwise returns the TextEdit that replaces
/// the changed region with the new content while preserving prefix/suffix lines.
pub fn compute_format_edit(original: &str, formatted: &str) -> Option<TextEdit> {
    let orig_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<String> = formatted.lines().map(|s| s.to_string()).collect();

    // If both are empty, nothing to do
    if orig_lines.is_empty() && new_lines.is_empty() {
        return None;
    }

    // Compute common prefix length
    let mut prefix = 0;
    while prefix < orig_lines.len()
        && prefix < new_lines.len()
        && orig_lines[prefix] == new_lines[prefix]
    {
        prefix += 1;
    }

    // Compute common suffix length
    let mut suffix = 0;
    while suffix < orig_lines.len() - prefix
        && suffix < new_lines.len() - prefix
        && orig_lines[orig_lines.len() - 1 - suffix] == new_lines[new_lines.len() - 1 - suffix]
    {
        suffix += 1;
    }

    // If all lines are unchanged (prefix + suffix covers everything)
    if prefix >= orig_lines.len() || (prefix + suffix == orig_lines.len()) {
        return None;
    }

    let orig_start_line = prefix as u32;
    let orig_end_line = (orig_lines.len() - suffix) as u32;

    let start_pos = Position {
        line: orig_start_line,
        character: 0,
    };

    let end_pos = if orig_end_line == 0 {
        Position {
            line: 0,
            character: 0,
        }
    } else if orig_end_line as usize >= orig_lines.len() {
        let last_line = orig_lines.last().copied().unwrap_or("");
        Position {
            line: (orig_lines.len() - 1) as u32,
            character: last_line.len() as u32,
        }
    } else {
        Position {
            line: orig_end_line,
            character: 0,
        }
    };

    let replacement_lines = &new_lines[prefix..(new_lines.len() - suffix)];
    let mut new_text = replacement_lines.join("\n");
    // Only add trailing newline when: replacing multiline block OR original ends with \n
    if (replacement_lines.len() > 1 && (orig_end_line as usize) < orig_lines.len())
        || (original.ends_with('\n') && (orig_end_line as usize == orig_lines.len() || suffix > 0))
    {
        new_text.push('\n');
    }

    Some(TextEdit {
        range: Range {
            start: start_pos,
            end: end_pos,
        },
        new_text,
    })
}

pub async fn range_formatting(
    server: &Backend,
    params: DocumentRangeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;
    let range = params.range;

    if let Some(content) = server.open_files.get(&uri).map(|r| r.to_string()) {
        let file_type = if uri.as_str().ends_with(".twxml") {
            "twxml"
        } else if uri.as_str().ends_with(".hubgs") {
            "hubgs"
        } else {
            return Ok(None);
        };

        let formatted = crate::formatter::format_source(&content, file_type);
        if formatted == content {
            return Ok(Some(vec![]));
        }

        let edit = compute_format_edit(&content, &formatted);
        match edit {
            Some(edit) => {
                if edit.range.start.line <= range.end.line
                    && edit.range.end.line >= range.start.line
                {
                    return Ok(Some(vec![edit]));
                }
            }
            None => {}
        }

        return Ok(Some(vec![]));
    }

    Ok(None)
}

#[cfg(test)]
#[path = "formatting_tests.rs"]
mod formatting_test_module;
