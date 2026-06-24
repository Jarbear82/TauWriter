use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn did_open(server: &Backend, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri;
    let text = params.text_document.text;
    server.open_files.insert(uri.clone(), text);
    server.publish_diagnostics(uri).await;
}

pub async fn did_change(server: &Backend, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    if let Some(change) = params.content_changes.first() {
        server.open_files.insert(uri.clone(), change.text.clone());
        server.publish_diagnostics(uri.clone()).await;

        if uri.as_str().ends_with(".twxml") {
            handle_twxml_change(server, &uri).await;
        } else if uri.as_str().ends_with(".hubgs") {
            handle_hubgs_change(server, &uri).await;
        }
    }
}

pub async fn did_close(server: &Backend, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri;
    server.open_files.remove(&uri);
}

// Symbol handlers (workspace + document level)
pub async fn symbol(
    server: &Backend,
    params: WorkspaceSymbolParams,
) -> Result<Option<Vec<SymbolInformation>>, tower_lsp::jsonrpc::Error> {
    let query = params.query.to_lowercase();

    let (db, ws) = server.lock_db().await;

    let mut symbols = Vec::new();

    // 1. Hub Instances
    let instances = crate::db::all_hub_instances(&*db, *ws);
    for inst in instances {
        let name = inst.name(&*db);
        if name.to_lowercase().contains(&query) {
            let path = inst.file(&*db).path(&*db);
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::VARIABLE,
                #[allow(deprecated)]
                deprecated: None,
                tags: None,
                location: Location {
                    uri: Url::from_file_path(path).unwrap(),
                    range: inst.range(&*db).into(),
                },
                container_name: Some("Instances".to_string()),
            });
        }
    }

    // 2. Hub Types
    let types = crate::db::all_hub_types(&*db, *ws);
    for t in types {
        let name = t.name(&*db);
        if name.to_lowercase().contains(&query) {
            let path = t.file(&*db).path(&*db);
            symbols.push(SymbolInformation {
                name,
                kind: SymbolKind::CLASS,
                #[allow(deprecated)]
                deprecated: None,
                tags: None,
                location: Location {
                    uri: Url::from_file_path(path).unwrap(),
                    range: t.range(&*db).into(),
                },
                container_name: Some("Types".to_string()),
            });
        }
    }

    Ok(Some(symbols))
}

pub async fn document_symbol(
    server: &Backend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
    let uri = params.text_document.uri;

    let (db, ws) = server.lock_db().await;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = (*ws)
            .files(&*db)
            .into_iter()
            .find(|f| f.path(&*db) == path_str);

        if let Some(file) = file {
            let mut symbols = Vec::new();

            #[allow(deprecated)]
            if path_str.ends_with(".hubgs") {
                let result = crate::db::parse_hubgs(&*db, file);

                for inst in result.instances(&*db) {
                    symbols.push(SymbolInformation {
                        name: inst.name(&*db),
                        kind: SymbolKind::VARIABLE,
                        #[allow(deprecated)]
                        deprecated: None,
                        tags: None,
                        location: Location {
                            uri: uri.clone(),
                            range: inst.range(&*db).into(),
                        },
                        container_name: Some("Instances".to_string()),
                    });
                }

                for t in result.types(&*db) {
                    symbols.push(SymbolInformation {
                        name: t.name(&*db),
                        kind: SymbolKind::CLASS,
                        #[allow(deprecated)]
                        deprecated: None,
                        tags: None,
                        location: Location {
                            uri: uri.clone(),
                            range: t.range(&*db).into(),
                        },
                        container_name: Some("Types".to_string()),
                    });
                }
            }

            return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
        }
    }

    Ok(None)
}

async fn handle_twxml_change(server: &Backend, uri: &Url) {
    let self_client = server.client.clone();
    let db_clone = server.db.clone();
    let ws_clone = server.workspace_input.clone();
    let uri_clone = uri.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut edits = Vec::new();
        {
            let db = db_clone.lock().await;
            let ws = *ws_clone.lock().await;
            let path = uri_clone
                .to_file_path()
                .unwrap()
                .to_string_lossy()
                .to_string();
            if let Some(file) = ws.files(&*db).into_iter().find(|f| f.path(&*db) == path) {
                let refs = crate::db::parse_twxml(&*db, file);
                for r in refs {
                    if r.is_reviewed(&*db) {
                        if let (Some(ref text_val), Some(ref field_name)) =
                            (r.text(&*db), r.field(&*db))
                        {
                            let name = r.name(&*db);
                            if let Some(instance) =
                                crate::db::resolve_reference(&*db, ws, name.clone())
                            {
                                if let Some(eval_val) = crate::db::compute_field_value(
                                    &*db,
                                    ws,
                                    instance,
                                    field_name.clone(),
                                ) {
                                    let canonical_str = eval_val.to_string();
                                    if canonical_str == *text_val {
                                        let review_range = r.tag_range(&*db);
                                        let keep_text = format!(
                                            r#"<hubref id="{}" field="{}">{}</hubref>"#,
                                            name, field_name, text_val
                                        );
                                        edits.push(TextEdit {
                                            range: review_range.into(),
                                            new_text: keep_text,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if !edits.is_empty() {
            let mut changes = std::collections::HashMap::new();
            changes.insert(uri_clone, edits);
            let edit = WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            };
            self_client.apply_edit(edit).await.ok();
        }
    });
}

async fn handle_hubgs_change(server: &Backend, _uri: &Url) {
    let self_client = server.client.clone();
    let db_clone = server.db.clone();
    let ws_clone = server.workspace_input.clone();
    let open_files_clone = server.open_files.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let mut changes = std::collections::HashMap::new();
        {
            let db = db_clone.lock().await;
            let ws = *ws_clone.lock().await;
            for file in ws.files(&*db) {
                let path = file.path(&*db);
                if path.ends_with(".twxml") {
                    let file_uri = Url::from_file_path(&path).unwrap();
                    let content = open_files_clone
                        .get(&file_uri)
                        .map(|x| x.clone())
                        .unwrap_or_else(|| file.contents(&*db));

                    let refs = crate::db::parse_twxml(&*db, file);
                    let mut edits = Vec::new();
                    for r in refs {
                        if r.is_reviewed(&*db) {
                            continue;
                        }
                        if let (Some(ref text_val), Some(ref field_name)) =
                            (r.text(&*db), r.field(&*db))
                        {
                            let name = r.name(&*db);
                            if let Some(instance) =
                                crate::db::resolve_reference(&*db, ws, name.clone())
                            {
                                if let Some(eval_val) = crate::db::compute_field_value(
                                    &*db,
                                    ws,
                                    instance,
                                    field_name.clone(),
                                ) {
                                    let canonical_str = eval_val.to_string();
                                    if canonical_str != *text_val {
                                        let tag_range = r.tag_range(&*db);
                                        let original_text =
                                            get_range_text(&content, tag_range.into());
                                        if !original_text.is_empty() {
                                            let new_text =
                                                format!("<review>{}</review>", original_text);
                                            edits.push(TextEdit {
                                                range: tag_range.into(),
                                                new_text,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !edits.is_empty() {
                        changes.insert(file_uri, edits);
                    }
                }
            }
        }
        if !changes.is_empty() {
            let edit = WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            };
            self_client.apply_edit(edit).await.ok();
        }
    });
}

pub fn get_range_text(contents: &str, range: lsp_types::Range) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;

    if start_line >= lines.len() || end_line >= lines.len() {
        return String::new();
    }

    if start_line == end_line {
        let line = lines[start_line];
        let start_char = range.start.character as usize;
        let end_char = range.end.character as usize;
        if start_char <= line.len() && end_char <= line.len() {
            return line[start_char..end_char].to_string();
        }
    } else {
        let mut result = Vec::new();
        let first_line = lines[start_line];
        let start_char = range.start.character as usize;
        if start_char <= first_line.len() {
            result.push(&first_line[start_char..]);
        }
        for i in (start_line + 1)..end_line {
            result.push(lines[i]);
        }
        let last_line = lines[end_line];
        let end_char = range.end.character as usize;
        if end_char <= last_line.len() {
            result.push(&last_line[..end_char]);
        }
        return result.join("\n");
    }
    String::new()
}
