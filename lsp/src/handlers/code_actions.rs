//! Code action provider for Hub reference quick-fixes.
//!
//! Generates actions to sync or preserve `<hubref>` content
//! at the cursor position.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn code_action(
    server: &Backend,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let uri = params.text_document.uri;
    let position = params.range.start;

    if let Some(content) = server.open_files.get(&uri).map(|r| r.to_string()) {
        if let Some((review_range, _hubref_range, id_val, field_val, current_text)) =
            crate::parser::find_review_at_position(&content, position.into())
        {
            let (db_val, ws_val) = server.read_db();
            let db = &db_val;
            let ws = &ws_val;

            if let Some(instance) = crate::db::resolve_reference(db, *ws, id_val.clone()) {
                if let Ok(Some(eval_val)) =
                    crate::db::compute_field_value(db, *ws, instance, field_val.clone())
                {
                    return code_action_impl(
                        &uri,
                        review_range.into(),
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
    review_range: lsp_types::Range,
    id_val: &str,
    field_val: &str,
    current_text: &str,
    eval_val: crate::db::HubValue,
) -> Result<Option<CodeActionResponse>> {
    let canonical_str = eval_val.to_string();

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
