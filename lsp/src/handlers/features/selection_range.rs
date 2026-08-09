// Selection ranges for TWXML and HubGS files

use std::path::Path;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser;
use crate::Backend;

pub async fn selection_range(
    server: &Backend,
    params: SelectionRangeParams,
) -> Result<Option<Vec<SelectionRange>>> {
    let uri = params.text_document.uri;
    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => return Ok(None),
    };

    let language = if uri.as_str().ends_with(".twxml") {
        match parser::get_language("twxml") {
            Some(lang) => lang,
            None => return Ok(None),
        }
    } else if uri.as_str().ends_with(".hubgs") {
        match parser::get_language("hubgs") {
            Some(lang) => lang,
            None => return Ok(None),
        }
    } else {
        return Ok(None);
    };

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok();
    let tree = match parser.parse(&content, None) {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut selection_ranges = Vec::new();

    for pos in params.positions {
        let lines: Vec<&str> = content.lines().collect();
        let mut byte_idx = 0;
        for i in 0..(pos.line as usize) {
            if i < lines.len() {
                byte_idx += lines[i].len() + 1;
            }
        }
        if (pos.line as usize) < lines.len() {
            byte_idx +=
                crate::utf16_idx_to_byte_idx(lines[pos.line as usize], pos.character as usize);
        }

        let mut node = tree
            .root_node()
            .descendant_for_byte_range(byte_idx, byte_idx);

        let mut path = Vec::new();
        while let Some(n) = node {
            path.push(n);
            node = n.parent();
        }

        let mut current_range: Option<SelectionRange> = None;
        for n in path.iter().rev() {
            let range = crate::parser::ts_range_to_lsp(n.range()).into();
            current_range = Some(SelectionRange {
                range,
                parent: current_range.map(Box::new),
            });
        }

        if let Some(r) = current_range {
            selection_ranges.push(r);
        }
    }

    Ok(Some(selection_ranges))
}
