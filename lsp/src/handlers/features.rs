use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn semantic_tokens_full(
    server: &Backend,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    let uri = params.text_document.uri;

    let (db, ws) = server.lock_db().await;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = (*ws)
            .files(&*db)
            .into_iter()
            .find(|f| f.path(&*db) == path_str);

        if let Some(file) = file {
            return semantic_tokens_impl(&*db, file);
        }
    }

    Ok(None)
}

fn semantic_tokens_impl(
    db: &dyn crate::db::Db,
    file: crate::db::SourceFile,
) -> Result<Option<SemanticTokensResult>> {
    let tokens = crate::db::get_semantic_tokens(db, file);
    let mut last_line: u32 = 0;
    let mut last_char: u32 = 0;

    let data: Vec<tower_lsp::lsp_types::SemanticToken> = tokens
        .into_iter()
        .map(|t| {
            let delta_line = t.line - last_line;
            let delta_start = if t.line == last_line {
                t.character - last_char
            } else {
                t.character
            };

            last_line = t.line;
            last_char = t.character;

            tower_lsp::lsp_types::SemanticToken {
                delta_line,
                delta_start,
                length: t.length,
                token_type: t.token_type,
                token_modifiers_bitset: t.token_modifiers,
            }
        })
        .collect();

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data,
    })))
}

pub async fn folding_range(
    server: &Backend,
    params: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>> {
    let uri = params.text_document.uri;

    let (db, ws) = server.lock_db().await;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = (*ws)
            .files(&*db)
            .into_iter()
            .find(|f| f.path(&*db) == path_str);

        if let Some(file) = file {
            return folding_range_impl(&*db, file);
        }
    }

    Ok(None)
}

fn folding_range_impl(
    db: &dyn crate::db::Db,
    file: crate::db::SourceFile,
) -> Result<Option<Vec<FoldingRange>>> {
    let ranges = crate::db::get_folding_ranges(db, file);
    let folding_ranges: Vec<FoldingRange> = ranges
        .into_iter()
        .map(|r| FoldingRange {
            start_line: r.start.line,
            start_character: Some(r.start.character),
            end_line: r.end.line,
            end_character: Some(r.end.character),
            kind: Some(FoldingRangeKind::Region),
            ..Default::default()
        })
        .collect();

    Ok(Some(folding_ranges))
}

pub async fn formatting(
    server: &Backend,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;

    if let Some(content) = server.open_files.get(&uri) {
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
