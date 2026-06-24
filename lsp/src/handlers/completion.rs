use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

fn try_completion_context(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
    content: &str,
    position: Position,
) -> Option<Result<Option<CompletionResponse>>> {
    let ctx = crate::parser::get_hubgs_completion_context(content, position);

    match ctx {
        crate::parser::HubgsCompletionContext::AllowsList => Some(complete_allows_list(db, ws)),
        crate::parser::HubgsCompletionContext::InstanceAssignment {
            type_name,
            role_name,
        } => Some(complete_role_instances(
            db, ws, file, &type_name, &role_name,
        )),
        crate::parser::HubgsCompletionContext::None => None,
    }
}

pub async fn completion(
    server: &Backend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    let (db, ws) = server.lock_db().await;
    let db_ref = &*db;
    let ws_ref = *ws;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws
            .files(db_ref)
            .into_iter()
            .find(|f| f.path(db_ref) == path_str);

        if let Some(file) = file {
            let content = file.contents(db_ref);

            if path_str.ends_with(".twxml") {
                return handle_twxml_completion(db_ref, ws_ref, &content, position);
            }
            if path_str.ends_with(".hubgs") {
                return handle_hubgs_completion(db_ref, ws_ref, file, &content, position);
            }
        }
    }

    // Fallback: list all hub instances
    let instances = crate::db::all_hub_instances(db_ref, ws_ref);
    let items: Vec<CompletionItem> = instances
        .into_iter()
        .map(|i| CompletionItem {
            label: i.name(db_ref),
            kind: Some(CompletionItemKind::REFERENCE),
            detail: Some("Hub Instance".to_string()),
            ..Default::default()
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}

fn handle_twxml_completion(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    content: &str,
    position: Position,
) -> Result<Option<CompletionResponse>> {
    let ctx = crate::parser::get_twxml_completion_context(content, position);

    match ctx {
        crate::parser::TwxmlCompletionContext::HubrefId => {
            let instances = crate::db::all_hub_instances(db, ws);
            let items: Vec<CompletionItem> = instances
                .into_iter()
                .map(|i| CompletionItem {
                    label: i.name(db),
                    kind: Some(CompletionItemKind::REFERENCE),
                    detail: Some("Hub Instance".to_string()),
                    ..Default::default()
                })
                .collect();
            Ok(Some(CompletionResponse::Array(items)))
        }
        crate::parser::TwxmlCompletionContext::HubrefField { id_val } => {
            complete_hub_fields(db, ws, &id_val)
        }
        crate::parser::TwxmlCompletionContext::None => Ok(None),
    }
}

fn complete_hub_fields(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    id_val: &str,
) -> Result<Option<CompletionResponse>> {
    if let Some(instance) = crate::db::resolve_reference(db, ws, id_val.to_string()) {
        let type_name = instance.type_name(db);
        if let Some(hub_type) = crate::db::resolve_type(db, ws, instance.file(db), type_name) {
            let mut items = Vec::new();
            for field in hub_type.fields(db) {
                items.push(CompletionItem {
                    label: field.name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("Field".to_string()),
                    ..Default::default()
                });
            }
            for role in hub_type.roles(db) {
                items.push(CompletionItem {
                    label: role.name.clone(),
                    kind: Some(CompletionItemKind::INTERFACE),
                    detail: Some("Role".to_string()),
                    ..Default::default()
                });
            }
            return Ok(Some(CompletionResponse::Array(items)));
        }
    }
    Ok(None)
}

fn handle_hubgs_completion(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
    content: &str,
    position: Position,
) -> Result<Option<CompletionResponse>> {
    if let Some(result) = try_completion_context(db, ws, file, content, position) {
        return result;
    }

    // Try field/role completion on current type at position
    if let Some(type_name) = crate::db::get_hub_type_at_position(db, file, position.into()) {
        if let Some(hub_type) = crate::db::resolve_type(db, ws, file, type_name) {
            let items = complete_fields_and_roles(db, &hub_type);
            return Ok(Some(CompletionResponse::Array(items)));
        }
    }

    // Inside a hub definition — offer global fields
    if crate::db::is_in_hub_definition(db, file, position.into()) {
        let globals = complete_global_fields(db, ws);
        return Ok(Some(CompletionResponse::Array(globals)));
    }

    Ok(None)
}

fn complete_allows_list(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
) -> Result<Option<CompletionResponse>> {
    let types = crate::db::all_hub_types(db, ws);
    let items: Vec<CompletionItem> = types
        .into_iter()
        .map(|t| CompletionItem {
            label: t.name(db),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("Hub Type".to_string()),
            ..Default::default()
        })
        .collect();
    Ok(Some(CompletionResponse::Array(items)))
}

fn complete_role_instances(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
    type_name: &str,
    role_name: &str,
) -> Result<Option<CompletionResponse>> {
    if let Some(hub_type) = crate::db::resolve_type(db, ws, file, type_name.to_string()) {
        if let Some(role) = hub_type.roles(db).iter().find(|r| r.name == role_name) {
            let instances = crate::db::all_hub_instances(db, ws);
            let items: Vec<CompletionItem> = instances
                .into_iter()
                .filter(|i| role.allowed_types.contains(&i.type_name(db)))
                .map(|i| CompletionItem {
                    label: i.name(db),
                    kind: Some(CompletionItemKind::REFERENCE),
                    detail: Some(format!("Hub Instance ({})", i.type_name(db))),
                    ..Default::default()
                })
                .collect();
            return Ok(Some(CompletionResponse::Array(items)));
        }
    }
    Ok(None)
}

fn complete_fields_and_roles(
    db: &dyn crate::db::Db,
    hub_type: &crate::db::HubType<'_>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for field in hub_type.fields(db) {
        items.push(CompletionItem {
            label: field.name.clone(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some("Field".to_string()),
            ..Default::default()
        });
    }
    for role in hub_type.roles(db) {
        items.push(CompletionItem {
            label: role.name.clone(),
            kind: Some(CompletionItemKind::INTERFACE),
            detail: Some("Role".to_string()),
            ..Default::default()
        });
    }
    items
}

fn complete_global_fields(db: &dyn crate::db::Db, ws: crate::db::Workspace) -> Vec<CompletionItem> {
    let global_fields = crate::db::all_global_fields(db, ws);
    global_fields
        .into_iter()
        .map(|gf| CompletionItem {
            label: gf.name(db),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some(format!("Global Field ({})", gf.type_name(db))),
            ..Default::default()
        })
        .collect()
}
