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
        let (db_val, ws_val) = server.read_db();
        let db = &db_val;
        let ws = &ws_val;
        let refs = crate::db::find_all_references(db, *ws, symbol);
        let locations = refs
            .into_iter()
            .filter_map(|r| {
                let path = r.file(&*db).path(&*db);
                Url::from_file_path(path).ok().map(|uri| Location {
                    uri,
                    range: r.range(&*db).into(),
                })
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
        let (db_val, ws_val) = server.read_db();
        let db = &db_val;
        let ws = &ws_val;
        return rename_impl(db, *ws, &symbol, &new_name);
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
        if let Ok(def_uri) = Url::from_file_path(instance.file(db).path(db)) {
            let def_edit = TextEdit {
                range: instance.range(db).into(),
                new_text: new_name.to_string(),
            };
            changes
                .entry(def_uri)
                .or_insert_with(Vec::new)
                .push(def_edit);
        }
    }

    let refs = crate::db::find_all_references(db, ws, symbol.to_string());
    for r in refs {
        if let Ok(ref_uri) = Url::from_file_path(r.file(db).path(db)) {
            let ref_edit = TextEdit {
                range: r.range(db).into(),
                new_text: new_name.to_string(),
            };
            changes
                .entry(ref_uri)
                .or_insert_with(Vec::new)
                .push(ref_edit);
        }
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
        let (db_val, ws_val) = server.read_db();
        let db = &db_val;
        let ws = &ws_val;
        return document_highlight_impl(db, *ws, &symbol, &uri);
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

pub async fn prepare_call_hierarchy(
    server: &Backend,
    params: CallHierarchyPrepareParams,
) -> Result<Option<Vec<CallHierarchyItem>>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db_val, ws_val) = server.read_db();
        let db = &db_val;
        let ws = &ws_val;

        // 1. Check if symbol is a Hub Instance
        if let Some(instance) = crate::db::resolve_reference(db, *ws, symbol.clone()) {
            if let Ok(target_uri) = Url::from_file_path(instance.file(db).path(db)) {
                let item = CallHierarchyItem {
                    name: symbol.clone(),
                    kind: SymbolKind::VARIABLE,
                    tags: None,
                    detail: Some(format!("Hub Instance ({})", instance.type_name(db))),
                    uri: target_uri,
                    range: instance.block_range(db).into(),
                    selection_range: instance.range(db).into(),
                    data: Some(serde_json::json!({
                        "type": "HubInstance",
                        "name": symbol,
                    })),
                };
                return Ok(Some(vec![item]));
            }
        }

        // 2. Check if symbol is a Hub Type
        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);
            if let Some(file) = file {
                if let Some(hub_type) = crate::db::resolve_type(db, *ws, file, symbol.clone()) {
                    if let Ok(target_uri) = Url::from_file_path(hub_type.file(db).path(db)) {
                        let item = CallHierarchyItem {
                            name: symbol.clone(),
                            kind: SymbolKind::CLASS,
                            tags: None,
                            detail: Some("Hub Type".to_string()),
                            uri: target_uri,
                            range: hub_type.block_range(db).into(),
                            selection_range: hub_type.range(db).into(),
                            data: Some(serde_json::json!({
                                "type": "HubType",
                                "name": symbol,
                            })),
                        };
                        return Ok(Some(vec![item]));
                    }
                }
            }
        }
    }

    Ok(None)
}

pub async fn incoming_calls(
    server: &Backend,
    params: CallHierarchyIncomingCallsParams,
) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
    let item = params.item;
    let data = match item.data {
        Some(d) => d,
        None => return Ok(None),
    };

    let sym_type = data.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let sym_name = data.get("name").and_then(|n| n.as_str()).unwrap_or("");

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

    let mut calls = Vec::new();

    if sym_type == "HubType" {
        let instances = crate::db::all_hub_instances(db, *ws);
        for inst in instances {
            if inst.type_name(db) == sym_name {
                if let Ok(inst_uri) = Url::from_file_path(inst.file(db).path(db)) {
                    let from_item = CallHierarchyItem {
                        name: inst.name(db),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        detail: Some(format!("Hub Instance ({})", sym_name)),
                        uri: inst_uri,
                        range: inst.block_range(db).into(),
                        selection_range: inst.range(db).into(),
                        data: Some(serde_json::json!({
                            "type": "HubInstance",
                            "name": inst.name(db),
                        })),
                    };
                    calls.push(CallHierarchyIncomingCall {
                        from: from_item,
                        from_ranges: vec![inst.range(db).into()],
                    });
                }
            }
        }
    } else if sym_type == "HubInstance" {
        let refs = crate::db::find_all_references(db, *ws, sym_name.to_string());
        let mut grouped_calls: std::collections::HashMap<String, (CallHierarchyItem, Vec<Range>)> = std::collections::HashMap::new();

        for r in refs {
            let ref_file = r.file(db);
            let ref_path = ref_file.path(db);
            if let Ok(ref_uri) = Url::from_file_path(&ref_path) {
                let mut caller_name = ref_path.split('/').last().unwrap_or("file").to_string();
                let mut caller_kind = SymbolKind::FILE;
                let mut caller_range = Range::default();
                let mut caller_selection_range = Range::default();
                let mut caller_detail = Some("File".to_string());
                let mut caller_data = None;

                if ref_path.ends_with(".hubgs") {
                    let parse_res = crate::db::parse_hubgs(db, ref_file);
                    let r_start = r.range(db).start;
                    let instances = parse_res.instances(db);
                    let is_inside = instances.iter().find(|inst| {
                        let b_range = inst.block_range(db);
                        let after_start = r_start.line > b_range.start.line
                            || (r_start.line == b_range.start.line && r_start.character >= b_range.start.character);
                        let before_end = r_start.line < b_range.end.line
                            || (r_start.line == b_range.end.line && r_start.character <= b_range.end.character);
                        after_start && before_end
                    });
                    if let Some(inst) = is_inside {
                        caller_name = inst.name(db);
                        caller_kind = SymbolKind::VARIABLE;
                        caller_range = inst.block_range(db).into();
                        caller_selection_range = inst.range(db).into();
                        caller_detail = Some(format!("Hub Instance ({})", inst.type_name(db)));
                        caller_data = Some(serde_json::json!({
                            "type": "HubInstance",
                            "name": inst.name(db),
                        }));
                    }
                }

                if caller_range == Range::default() {
                    caller_range = Range::new(Position::new(0, 0), Position::new(0, 0));
                    caller_selection_range = Range::new(Position::new(0, 0), Position::new(0, 0));
                }

                let key = format!("{}:{}", ref_uri, caller_name);
                let entry = grouped_calls.entry(key).or_insert_with(|| {
                    (
                        CallHierarchyItem {
                            name: caller_name,
                            kind: caller_kind,
                            tags: None,
                            detail: caller_detail,
                            uri: ref_uri,
                            range: caller_range,
                            selection_range: caller_selection_range,
                            data: caller_data,
                        },
                        Vec::new(),
                    )
                });
                entry.1.push(r.range(db).into());
            }
        }

        for (_, (from, from_ranges)) in grouped_calls {
            calls.push(CallHierarchyIncomingCall { from, from_ranges });
        }
    }

    Ok(Some(calls))
}

pub async fn outgoing_calls(
    server: &Backend,
    params: CallHierarchyOutgoingCallsParams,
) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
    let item = params.item;
    let data = match item.data {
        Some(d) => d,
        None => return Ok(None),
    };

    let sym_type = data.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let sym_name = data.get("name").and_then(|n| n.as_str()).unwrap_or("");

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

    let mut calls = Vec::new();

    if sym_type == "HubInstance" {
        if let Some(instance) = crate::db::resolve_reference(db, *ws, sym_name.to_string()) {
            let type_name = instance.type_name(db);
            if let Some(hub_type) = crate::db::resolve_type(db, *ws, instance.file(db), type_name.clone()) {
                if let Ok(type_uri) = Url::from_file_path(hub_type.file(db).path(db)) {
                    let to_item = CallHierarchyItem {
                        name: type_name.clone(),
                        kind: SymbolKind::CLASS,
                        tags: None,
                        detail: Some("Hub Type".to_string()),
                        uri: type_uri,
                        range: hub_type.block_range(db).into(),
                        selection_range: hub_type.range(db).into(),
                        data: Some(serde_json::json!({
                            "type": "HubType",
                            "name": type_name,
                        })),
                    };
                    calls.push(CallHierarchyOutgoingCall {
                        to: to_item,
                        from_ranges: vec![instance.range(db).into()],
                    });
                }
            }
        }
    }

    Ok(Some(calls))
}

