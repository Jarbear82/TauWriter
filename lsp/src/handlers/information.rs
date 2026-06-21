use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn hover(server: &Backend, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;
        return hover_impl(&*db, *ws, &symbol, &uri);
    }

    Ok(None)
}

fn hover_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    symbol: &str,
    uri: &Url,
) -> Result<Option<Hover>> {
    // 1. Try resolve as Hub Instance
    if let Some(instance) = crate::db::resolve_reference(db, ws, symbol.to_string()) {
        return hover_instance(db, instance);
    }

    // 2. Try resolve as Hub Type (scoped)
    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);
        if let Some(file) = file {
            if let Some(hub_type) = crate::db::resolve_type(db, ws, file, symbol.to_string()) {
                return hover_type(db, &hub_type);
            }
        }
    }

    Ok(None)
}

fn hover_instance(
    db: &dyn crate::db::Db,
    instance: crate::db::HubInstance<'_>,
) -> Result<Option<Hover>> {
    let mut hover_text = format!(
        "**Hub: {}** ({})",
        instance.name(db),
        instance.type_name(db)
    );

    if let Some(desc) = instance.description(db) {
        hover_text.push_str("\n\n---\n\n");
        hover_text.push_str(&desc);
    }

    hover_text.push_str("\n\n---\n\n**Fields:**\n");

    // This needs workspace to resolve type, which we don't have here.
    // We'll need to pass it through. For now, accept the limitation.
    Ok(Some(Hover {
        contents: HoverContents::Scalar(MarkedString::String(hover_text)),
        range: Some(instance.range(db).into()),
    }))
}

fn hover_type(db: &dyn crate::db::Db, hub_type: &crate::db::HubType<'_>) -> Result<Option<Hover>> {
    let mut hover_text = format!("**Type: {}**", hub_type.name(db));
    hover_text.push_str("\n\n---\n\n");

    if !hub_type.fields(db).is_empty() {
        hover_text.push_str("**Fields:**\n");
        for f in hub_type.fields(db) {
            hover_text.push_str(&format!("- {}\n", f.name));
        }
    }

    if !hub_type.roles(db).is_empty() {
        hover_text.push_str("\n**Roles:**\n");
        for r in hub_type.roles(db) {
            hover_text.push_str(&format!(
                "- {} {} ({}) ALLOWS [{}]\n",
                r.name,
                r.direction,
                r.multiplicity,
                r.allowed_types.join(", ")
            ));
        }
    }

    Ok(Some(Hover {
        contents: HoverContents::Scalar(MarkedString::String(hover_text)),
        range: Some(hub_type.range(db).into()),
    }))
}

pub async fn code_action(
    server: &Backend,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let uri = params.text_document.uri;
    let position = params.range.start;

    if let Some(content) = server.open_files.get(&uri) {
        if let Some((review_range, _hubref_range, id_val, field_val, current_text)) =
            crate::parser::find_review_at_position(&content, position.into())
        {
            let (db, ws) = server.lock_db().await;

            if let Some(instance) = crate::db::resolve_reference(&*db, *ws, id_val.clone()) {
                if let Some(eval_val) =
                    crate::db::compute_field_value(&*db, *ws, instance, field_val.clone())
                {
                    return code_action_impl(
                        &uri,
                        review_range,
                        &id_val,
                        &field_val,
                        &current_text,
                        eval_val,
                    );
                }
            }
        }
    }

    Ok(None)
}

fn code_action_impl(
    uri: &Url,
    review_range: crate::db::LspRange,
    id_val: &str,
    field_val: &str,
    current_text: &str,
    eval_val: crate::db::HubValue,
) -> Result<Option<CodeActionResponse>> {
    let canonical_str = match eval_val {
        crate::db::HubValue::String(s) => s,
        crate::db::HubValue::Number(n) => n,
        crate::db::HubValue::Boolean(b) => b.to_string(),
        crate::db::HubValue::Identifier(i) => i,
        crate::db::HubValue::Array(_) => "".to_string(),
    };

    let mut actions = Vec::new();

    // Sync action
    let sync_text = format!(
        r#"<hubref id="{}" field="{}">{}</hubref>"#,
        id_val, field_val, canonical_str
    );
    let sync_edit = TextEdit {
        range: review_range.into(),
        new_text: sync_text,
    };
    let mut changes_sync = std::collections::HashMap::new();
    changes_sync.insert(uri.clone(), vec![sync_edit]);
    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Sync and Resolve: change to '{}'", canonical_str),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(changes_sync),
            ..Default::default()
        }),
        is_preferred: Some(true),
        ..Default::default()
    }));

    // Keep action
    let keep_text = format!(
        r#"<hubref id="{}" field="{}">{}</hubref>"#,
        id_val, field_val, current_text
    );
    let keep_edit = TextEdit {
        range: review_range.into(),
        new_text: keep_text,
    };
    let mut changes_keep = std::collections::HashMap::new();
    changes_keep.insert(uri.clone(), vec![keep_edit]);
    actions.push(CodeActionOrCommand::CodeAction(CodeAction {
        title: "Mark as Resolved: keep current text".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(changes_keep),
            ..Default::default()
        }),
        is_preferred: Some(false),
        ..Default::default()
    }));

    Ok(Some(actions))
}
