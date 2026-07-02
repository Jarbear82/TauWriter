use ropey::Rope;
use tower_lsp::lsp_types::*;

use crate::Backend;
use salsa::prelude::*;

fn utf16_to_char_offset_in_line(line: ropey::RopeSlice<'_>, utf16_idx: usize) -> usize {
    let mut utf16_current = 0;
    let mut char_current = 0;
    for c in line.chars() {
        if utf16_current >= utf16_idx {
            break;
        }
        utf16_current += c.len_utf16();
        char_current += 1;
    }
    char_current
}

pub async fn did_open(server: &Backend, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri;
    let text = params.text_document.text;
    server.open_files.insert(uri.clone(), Rope::from_str(&text));

    if let Ok(path_buf) = uri.to_file_path() {
        let path_str = path_buf.to_string_lossy().to_string();
        let mut db = server.db.lock().unwrap();
        let ws = server.workspace_input;
        let mut files = ws.files(&*db).clone();
        if let Some(idx) = files.iter().position(|f| f.path(&*db) == path_str) {
            files[idx].set_contents(&mut *db).to(text.clone());
        } else {
            let source = crate::db::SourceFile::new(&mut *db, path_str, text.clone());
            files.push(source);
        }
        ws.set_files(&mut *db).to(files);
    }

    server.publish_diagnostics(uri).await;
}

pub(crate) fn edit_tree(
    tree: &mut tree_sitter::Tree,
    rope: &ropey::Rope,
    range: lsp_types::Range,
    new_text: &str,
    start_char: usize,
    end_char: usize,
    start_line_idx: usize,
    end_line_idx: usize,
) {
    let start_byte = rope.char_to_byte(start_char);
    let old_end_byte = rope.char_to_byte(end_char);
    let new_end_byte = start_byte + new_text.len();

    let start_row = range.start.line as usize;
    let start_line_byte = if start_line_idx < rope.len_lines() {
        rope.line_to_byte(start_line_idx)
    } else {
        rope.len_bytes()
    };
    let start_col = start_byte.saturating_sub(start_line_byte);

    let old_end_row = range.end.line as usize;
    let end_line_byte = if end_line_idx < rope.len_lines() {
        rope.line_to_byte(end_line_idx)
    } else {
        rope.len_bytes()
    };
    let old_end_col = old_end_byte.saturating_sub(end_line_byte);

    let new_lines_count = new_text.chars().filter(|&c| c == '\n').count();
    let new_end_row = start_row + new_lines_count;
    let new_end_col = if new_lines_count == 0 {
        start_col + new_text.len()
    } else {
        new_text.split('\n').last().unwrap_or("").len()
    };

    let edit = tree_sitter::InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position: tree_sitter::Point { row: start_row, column: start_col },
        old_end_position: tree_sitter::Point { row: old_end_row, column: old_end_col },
        new_end_position: tree_sitter::Point { row: new_end_row, column: new_end_col },
    };

    tree.edit(&edit);
}

pub async fn did_change(server: &Backend, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;

    let mut path_str = String::new();
    if let Ok(path_buf) = uri.to_file_path() {
        path_str = path_buf.to_string_lossy().to_string();
    }

    let mut content = String::new();
    if let Some(mut rope_ref) = server.open_files.get_mut(&uri) {
        for change in params.content_changes {
            if let Some(range) = change.range {
                let start_line_idx = range.start.line as usize;
                let start_char = if start_line_idx < rope_ref.len_lines() {
                    rope_ref.line_to_char(start_line_idx)
                        + utf16_to_char_offset_in_line(rope_ref.line(start_line_idx), range.start.character as usize)
                } else {
                    rope_ref.len_chars()
                };

                let end_line_idx = range.end.line as usize;
                let end_char = if end_line_idx < rope_ref.len_lines() {
                    rope_ref.line_to_char(end_line_idx)
                        + utf16_to_char_offset_in_line(rope_ref.line(end_line_idx), range.end.character as usize)
                } else {
                    rope_ref.len_chars()
                };

                if !path_str.is_empty() {
                    if let Some(mut tree_entry) = crate::get_tree_cache().get_mut(&path_str) {
                        edit_tree(
                            &mut tree_entry.value_mut().tree,
                            &rope_ref,
                            range,
                            &change.text,
                            start_char,
                            end_char,
                            start_line_idx,
                            end_line_idx,
                        );

                        rope_ref.remove(start_char..end_char);
                        rope_ref.insert(start_char, &change.text);

                        let new_str = rope_ref.to_string();
                        tree_entry.value_mut().content_len = new_str.len();
                        tree_entry.value_mut().content_hash = crate::calculate_hash(&new_str);
                        tree_entry.value_mut().needs_reparse = true;
                        continue;
                    }
                }

                rope_ref.remove(start_char..end_char);
                rope_ref.insert(start_char, &change.text);
            } else {
                // Full document replacement fallback (rare; client opted into full sync)
                *rope_ref = Rope::from_str(&change.text);
                if !path_str.is_empty() {
                    crate::get_tree_cache().remove(&path_str);
                }
            }
        }
        content = rope_ref.to_string();
    }

    if let Ok(path_buf) = uri.to_file_path() {
        let path_str = path_buf.to_string_lossy().to_string();
        let mut db = server.db.lock().unwrap();
        let ws = server.workspace_input;
        let mut files = ws.files(&*db).clone();
        if let Some(idx) = files.iter().position(|f| f.path(&*db) == path_str) {
            files[idx].set_contents(&mut *db).to(content);
        } else {
            let source = crate::db::SourceFile::new(&mut *db, path_str, content);
            files.push(source);
        }
        ws.set_files(&mut *db).to(files);
    }

    server.publish_diagnostics(uri.clone()).await;

    if uri.as_str().ends_with(".twxml") {
        handle_twxml_change(server, &uri).await;
    } else if uri.as_str().ends_with(".hubgs") {
        handle_hubgs_change(server, &uri).await;
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

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

    let mut symbols = Vec::new();

    // 1. Hub Instances
    let instances = crate::db::all_hub_instances(&*db, *ws);
    for inst in instances {
        let name = inst.name(&*db);
        if name.to_lowercase().contains(&query) {
            let path = inst.file(&*db).path(&*db);
            if let Ok(uri) = Url::from_file_path(path) {
                symbols.push(SymbolInformation {
                    name,
                    kind: SymbolKind::VARIABLE,
                    #[allow(deprecated)]
                    deprecated: None,
                    tags: None,
                    location: Location {
                        uri,
                        range: inst.range(&*db).into(),
                    },
                    container_name: Some("Instances".to_string()),
                });
            }
        }
    }

    // 2. Hub Types
    let types = crate::db::all_hub_types(&*db, *ws);
    for t in types {
        let name = t.name(&*db);
        if name.to_lowercase().contains(&query) {
            let path = t.file(&*db).path(&*db);
            if let Ok(uri) = Url::from_file_path(path) {
                symbols.push(SymbolInformation {
                    name,
                    kind: SymbolKind::CLASS,
                    #[allow(deprecated)]
                    deprecated: None,
                    tags: None,
                    location: Location {
                        uri,
                        range: t.range(&*db).into(),
                    },
                    container_name: Some("Types".to_string()),
                });
            }
        }
    }

    Ok(Some(symbols))
}

pub async fn document_symbol(
    server: &Backend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
    let uri = params.text_document.uri;

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

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
    let (db, ws) = server.read_db();
    let uri_clone = uri.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut edits = Vec::new();
        if let Ok(path_buf) = uri_clone.to_file_path() {
            let path = path_buf.to_string_lossy().to_string();
            if let Some(file) = ws.files(&db).into_iter().find(|f| f.path(&db) == path) {
                    let refs = crate::db::parse_twxml(&db, file);
                    for r in refs {
                        if r.is_reviewed(&db) {
                            if let (Some(ref text_val), Some(ref field_name)) =
                                (r.text(&db), r.field(&db))
                            {
                                let name = r.name(&db);
                                if let Some(instance) =
                                    crate::db::resolve_reference(&db, ws, name.clone())
                                {
                                    if let Some(eval_val) = crate::db::compute_field_value(
                                        &db,
                                        ws,
                                        instance,
                                        field_name.clone(),
                                    ) {
                                        let canonical_str = eval_val.to_string();
                                        if canonical_str == *text_val {
                                            let review_range = r.tag_range(&db);
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
    let (db, ws) = server.read_db();
    let open_files_clone = server.open_files.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let mut changes = std::collections::HashMap::new();
        {
            for file in ws.files(&db) {
                let path = file.path(&db);
                if path.ends_with(".twxml") {
                    if let Ok(file_uri) = Url::from_file_path(&path) {
                        let content = open_files_clone
                            .get(&file_uri)
                            .map(|r| r.to_string())
                            .unwrap_or_else(|| file.contents(&db));

                        let refs = crate::db::parse_twxml(&db, file);
                        let mut edits = Vec::new();
                        for r in refs {
                            if r.is_reviewed(&db) {
                                continue;
                            }
                            if let (Some(ref text_val), Some(ref field_name)) =
                                (r.text(&db), r.field(&db))
                            {
                                let name = r.name(&db);
                                if let Some(instance) =
                                    crate::db::resolve_reference(&db, ws, name.clone())
                                {
                                    if let Some(eval_val) = crate::db::compute_field_value(
                                        &db,
                                        ws,
                                        instance,
                                        field_name.clone(),
                                    ) {
                                        let canonical_str = eval_val.to_string();
                                        if canonical_str != *text_val {
                                            let tag_range = r.tag_range(&db);
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
        let start_byte = crate::utf16_idx_to_byte_idx(line, range.start.character as usize);
        let end_byte = crate::utf16_idx_to_byte_idx(line, range.end.character as usize);
        if start_byte <= line.len() && end_byte <= line.len() {
            return line[start_byte..end_byte].to_string();
        }
    } else {
        let mut result = Vec::new();
        let first_line = lines[start_line];
        let start_byte = crate::utf16_idx_to_byte_idx(first_line, range.start.character as usize);
        if start_byte <= first_line.len() {
            result.push(&first_line[start_byte..]);
        }
        for i in (start_line + 1)..end_line {
            result.push(lines[i]);
        }
        let last_line = lines[end_line];
        let end_byte = crate::utf16_idx_to_byte_idx(last_line, range.end.character as usize);
        if end_byte <= last_line.len() {
            result.push(&last_line[..end_byte]);
        }
        return result.join("\n");
    }
    String::new()
}

pub async fn did_change_watched_files(server: &Backend, params: DidChangeWatchedFilesParams) {
    let mut files_updated = false;
    let mut affected_files = Vec::new();
    let mut deleted_uris = Vec::new();

    {
        let mut db = server.db.lock().unwrap();
        let ws = server.workspace_input;
        let mut files = ws.files(&*db).clone();

        for event in params.changes {
            if let Ok(path) = event.uri.to_file_path() {
                let path_str = path.to_string_lossy().to_string();
                match event.typ {
                    FileChangeType::CREATED | FileChangeType::CHANGED => {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            if let Some(idx) = files.iter().position(|f| f.path(&*db) == path_str) {
                                files[idx].set_contents(&mut *db).to(contents.clone());
                            } else {
                                let source = crate::db::SourceFile::new(&mut *db, path_str.clone(), contents.clone());
                                files.push(source);
                            }
                            if server.open_files.contains_key(&event.uri) {
                                server.open_files.insert(event.uri.clone(), Rope::from_str(&contents));
                            }
                            files_updated = true;
                            affected_files.push(event.uri);
                        }
                    }
                    FileChangeType::DELETED => {
                        if let Some(idx) = files.iter().position(|f| f.path(&*db) == path_str) {
                            files.remove(idx);
                            server.open_files.remove(&event.uri);
                            files_updated = true;
                            deleted_uris.push(event.uri.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        if files_updated {
            ws.set_files(&mut *db).to(files);
        }
    }

    for uri in deleted_uris {
        server.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    for uri in affected_files {
        server.publish_diagnostics(uri).await;
    }

    let open_uris: Vec<Url> = server.open_files.iter().map(|kv| kv.key().clone()).collect();
    for uri in open_uris {
        server.publish_diagnostics(uri).await;
    }
}

pub async fn did_create_files(server: &Backend, params: CreateFilesParams) {
    let mut files_updated = false;
    let mut affected_files = Vec::new();

    {
        let mut db = server.db.lock().unwrap();
        let ws = server.workspace_input;
        let mut files = ws.files(&*db).clone();

        for f in params.files {
            if let Ok(uri) = Url::parse(&f.uri) {
                if let Ok(path) = uri.to_file_path() {
                    let path_str = path.to_string_lossy().to_string();
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        if !files.iter().any(|file| file.path(&*db) == path_str) {
                            let source = crate::db::SourceFile::new(&mut *db, path_str, contents);
                            files.push(source);
                            files_updated = true;
                            affected_files.push(uri);
                        }
                    }
                }
            }
        }

        if files_updated {
            ws.set_files(&mut *db).to(files);
        }
    }

    for uri in affected_files {
        server.publish_diagnostics(uri).await;
    }
}

pub async fn did_rename_files(server: &Backend, params: RenameFilesParams) {
    let mut files_updated = false;
    let mut affected_files = Vec::new();
    let mut deleted_uris = Vec::new();

    {
        let mut db = server.db.lock().unwrap();
        let ws = server.workspace_input;
        let mut files = ws.files(&*db).clone();

        for f in params.files {
            let old_uri_opt = Url::parse(&f.old_uri).ok();
            let new_uri_opt = Url::parse(&f.new_uri).ok();

            if let (Some(old_uri), Some(new_uri)) = (old_uri_opt, new_uri_opt) {
                let old_path = old_uri.to_file_path().ok();
                let new_path = new_uri.to_file_path().ok();

                if let (Some(old_p), Some(new_p)) = (old_path, new_path) {
                    let old_path_str = old_p.to_string_lossy().to_string();
                    let new_path_str = new_p.to_string_lossy().to_string();

                    if let Some(idx) = files.iter().position(|file| file.path(&*db) == old_path_str) {
                        files.remove(idx);
                        server.open_files.remove(&old_uri);
                        deleted_uris.push(old_uri);
                        files_updated = true;
                    }

                    if let Ok(contents) = std::fs::read_to_string(&new_p) {
                        let source = crate::db::SourceFile::new(&mut *db, new_path_str, contents);
                        files.push(source);
                        affected_files.push(new_uri);
                        files_updated = true;
                    }
                }
            }
        }

        if files_updated {
            ws.set_files(&mut *db).to(files);
        }
    }

    for uri in deleted_uris {
        server.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    for uri in affected_files {
        server.publish_diagnostics(uri).await;
    }
}

pub async fn did_delete_files(server: &Backend, params: DeleteFilesParams) {
    let mut files_updated = false;
    let mut deleted_uris = Vec::new();

    {
        let mut db = server.db.lock().unwrap();
        let ws = server.workspace_input;
        let mut files = ws.files(&*db).clone();

        for f in params.files {
            if let Ok(uri) = Url::parse(&f.uri) {
                if let Ok(path) = uri.to_file_path() {
                    let path_str = path.to_string_lossy().to_string();
                    if let Some(idx) = files.iter().position(|file| file.path(&*db) == path_str) {
                        files.remove(idx);
                        server.open_files.remove(&uri);
                        deleted_uris.push(uri);
                        files_updated = true;
                    }
                }
            }
        }

        if files_updated {
            ws.set_files(&mut *db).to(files);
        }
    }

    for uri in deleted_uris {
        server.client.publish_diagnostics(uri, Vec::new(), None).await;
    }
}

