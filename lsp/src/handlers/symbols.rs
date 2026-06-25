use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn references(
    server: &Backend,
    params: ReferenceParams,
) -> Result<Option<Vec<Location>>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;
        let refs = crate::db::find_all_references(&*db, *ws, symbol);
        let locations = refs
            .into_iter()
            .map(|r| {
                let path = r.file(&*db).path(&*db);
                Location {
                    uri: Url::from_file_path(path).unwrap(),
                    range: r.range(&*db).into(),
                }
            })
            .collect();
        return Ok(Some(locations));
    }

    Ok(None)
}

pub async fn rename(server: &Backend, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let new_name = params.new_name;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;
        return rename_impl(&*db, *ws, &symbol, &new_name);
    }

    Ok(None)
}

fn rename_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    symbol: &str,
    new_name: &str,
) -> Result<Option<WorkspaceEdit>> {
    let mut changes = std::collections::HashMap::new();

    if let Some(instance) = crate::db::resolve_reference(db, ws, symbol.to_string()) {
        let def_uri = Url::from_file_path(instance.file(db).path(db)).unwrap();
        let def_edit = TextEdit {
            range: instance.range(db).into(),
            new_text: new_name.to_string(),
        };
        changes
            .entry(def_uri)
            .or_insert_with(Vec::new)
            .push(def_edit);
    }

    let refs = crate::db::find_all_references(db, ws, symbol.to_string());
    for r in refs {
        let ref_uri = Url::from_file_path(r.file(db).path(db)).unwrap();
        let ref_edit = TextEdit {
            range: r.range(db).into(),
            new_text: new_name.to_string(),
        };
        changes
            .entry(ref_uri)
            .or_insert_with(Vec::new)
            .push(ref_edit);
    }

    Ok(Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    }))
}

pub async fn document_highlight(
    server: &Backend,
    params: DocumentHighlightParams,
) -> Result<Option<Vec<DocumentHighlight>>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;
        return document_highlight_impl(&*db, *ws, &symbol, &uri);
    }

    Ok(None)
}

fn document_highlight_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    symbol: &str,
    uri: &Url,
) -> Result<Option<Vec<DocumentHighlight>>> {
    let mut highlights = Vec::new();

    // 1. If it's a definition in this file, highlight it
    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        if path_str.ends_with(".hubgs") {
            if let Some(file) = ws.files(db).into_iter().find(|f| f.path(db) == path_str) {
                let result = crate::db::parse_hubgs(db, file);
                if let Some(inst) = result.instances(db).iter().find(|i| i.name(db) == symbol) {
                    highlights.push(DocumentHighlight {
                        range: inst.range(db).into(),
                        kind: Some(DocumentHighlightKind::WRITE),
                    });
                }
                if let Some(t) = result.types(db).iter().find(|t| t.name(db) == symbol) {
                    highlights.push(DocumentHighlight {
                        range: t.range(db).into(),
                        kind: Some(DocumentHighlightKind::WRITE),
                    });
                }
            }
        }
    }

    // 2. Find all references and filter by this file
    let refs = crate::db::find_all_references(db, ws, symbol.to_string());
    for r in refs {
        let r_path = r.file(db).path(db);
        if let Ok(uri_path) = uri.to_file_path() {
            if r_path == uri_path.to_string_lossy().to_string() {
                highlights.push(DocumentHighlight {
                    range: r.range(db).into(),
                    kind: Some(DocumentHighlightKind::READ),
                });
            }
        }
    }

    Ok(Some(highlights))
}
