use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn inlay_hints(
    server: &Backend,
    params: InlayHintParams,
) -> Result<Option<Vec<InlayHint>>> {
    let uri = params.text_document.uri;

    let (db, ws) = server.lock_db().await;

    let mut hints = Vec::new();

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();

        // For .hubgs files, show type hints on instance assignments
        if path_str.ends_with(".hubgs") {
            let file = (*ws)
                .files(&*db)
                .into_iter()
                .find(|f| f.path(&*db) == path_str);

            if let Some(file) = file {
                let result = crate::db::parse_hubgs(&*db, file);

                for inst in result.instances(&*db) {
                    let type_name = inst.type_name(&*db);
                    let inst_range = inst.range(&*db);

                    // Show the type as an inlay hint after the instance name
                    hints.push(InlayHint {
                        position: Position {
                            line: inst_range.end.line,
                            character: inst_range.end.character,
                        },
                        label: InlayHintLabel::String(format!(": {}", type_name)),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: None,
                        padding_left: Some(true),
                        padding_right: None,
                        data: None,
                    });
                }
            }
        }
    }

    Ok(Some(hints))
}
