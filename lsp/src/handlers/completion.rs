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
        crate::parser::TwxmlCompletionContext::Tag { parent } => {
            complete_twxml_tags(parent.as_deref())
        }
        crate::parser::TwxmlCompletionContext::None => Ok(None),
    }
}

/// Suggest TWXML structural tags based on the current parent context.
fn complete_twxml_tags(parent: Option<&str>) -> Result<Option<CompletionResponse>> {
    // ponytail: full nesting rules not yet implemented — suggest all known tags.
    // Upgrade path: build a parent->allowed_children map from validation rules.
    let all_tags: [(&str, CompletionItemKind, &str); 38] = [
        // Structural
        ("document", CompletionItemKind::CLASS, "TWXML Document"),
        ("body", CompletionItemKind::CLASS, "Body Block"),
        ("meta", CompletionItemKind::CLASS, "Meta Tag"),
        // Content blocks
        ("section", CompletionItemKind::CLASS, "Section"),
        ("heading", CompletionItemKind::CLASS, "Heading"),
        ("paragraph", CompletionItemKind::CLASS, "Paragraph"),
        ("aside", CompletionItemKind::CLASS, "Aside"),
        ("blockquote", CompletionItemKind::CLASS, "Blockquote"),
        ("codeblock", CompletionItemKind::CLASS, "Code Block"),
        // Lists
        ("ul", CompletionItemKind::CLASS, "Unordered List"),
        ("ol", CompletionItemKind::CLASS, "Ordered List"),
        ("li", CompletionItemKind::CLASS, "List Item"),
        ("dl", CompletionItemKind::CLASS, "Definition List"),
        ("dt", CompletionItemKind::CLASS, "Definition Term"),
        ("dd", CompletionItemKind::CLASS, "Definition Description"),
        // Interactive
        ("details", CompletionItemKind::CLASS, "Details"),
        ("summary", CompletionItemKind::CLASS, "Summary"),
        // Tables
        ("table", CompletionItemKind::CLASS, "Table"),
        ("tr", CompletionItemKind::CLASS, "Table Row"),
        ("th", CompletionItemKind::CLASS, "Table Header"),
        ("td", CompletionItemKind::CLASS, "Table Cell"),
        // Inline
        ("hubref", CompletionItemKind::REFERENCE, "Hub Reference"),
        ("link", CompletionItemKind::REFERENCE, "Link"),
        ("image", CompletionItemKind::VALUE, "Image"),
        ("audio", CompletionItemKind::VALUE, "Audio"),
        ("video", CompletionItemKind::VALUE, "Video"),
        ("code", CompletionItemKind::VALUE, "Inline Code"),
        ("bold", CompletionItemKind::VALUE, "Bold"),
        ("italic", CompletionItemKind::VALUE, "Italic"),
        ("underline", CompletionItemKind::VALUE, "Underline"),
        ("strikethrough", CompletionItemKind::VALUE, "Strikethrough"),
        ("super", CompletionItemKind::VALUE, "Superscript"),
        ("sub", CompletionItemKind::VALUE, "Subscript"),
        // Special
        ("br", CompletionItemKind::VALUE, "Line Break"),
        ("hr", CompletionItemKind::VALUE, "Horizontal Rule"),
        ("fr", CompletionItemKind::REFERENCE, "Footnote Reference"),
        ("footnote", CompletionItemKind::CLASS, "Footnote"),
        ("review", CompletionItemKind::CLASS, "Review"),
    ];

    let items: Vec<CompletionItem> = all_tags
        .into_iter()
        .filter(|(name, _, _)| {
            // Don't suggest document inside nested content
            if parent.is_some() && *name == "document" {
                return false;
            }
            // Don't suggest body inside another body
            if parent == Some("body") && *name == "body" {
                return false;
            }
            true
        })
        .map(|(name, kind, detail)| CompletionItem {
            label: name.to_string(),
            kind: Some(kind),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
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
            // ponytail: Use polymorphic field/role lookups to respect EXTENDS inheritance
            let all_fields = crate::db::polymorphic::hub_type_all_fields(db, ws, &hub_type);
            let all_roles = crate::db::polymorphic::hub_type_all_roles(db, ws, &hub_type);
            for field in all_fields {
                items.push(CompletionItem {
                    label: field.name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("Field".to_string()),
                    ..Default::default()
                });
            }
            for role in all_roles {
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
            let items = complete_fields_and_roles(db, ws, &hub_type);
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
            // ponytail: Polymorphic completion - child instances satisfy parent roles
            use crate::db::polymorphic::hub_type_allows;
            let items: Vec<CompletionItem> = instances
                .into_iter()
                .filter(|i| {
                    if let Some(inst_type) =
                        crate::db::resolve_type(db, ws, i.file(db), i.type_name(db).clone())
                    {
                        hub_type_allows(db, ws, &inst_type, &role.allowed_types)
                    } else {
                        role.allowed_types.contains(&i.type_name(db))
                    }
                })
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
    ws: crate::db::Workspace,
    hub_type: &crate::db::HubType<'_>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    // ponytail: Use polymorphic field/role lookups to respect EXTENDS inheritance
    let all_fields = crate::db::polymorphic::hub_type_all_fields(db, ws, &hub_type);
    let all_roles = crate::db::polymorphic::hub_type_all_roles(db, ws, &hub_type);
    for field in all_fields {
        items.push(CompletionItem {
            label: field.name.clone(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some("Field".to_string()),
            ..Default::default()
        });
    }
    for role in all_roles {
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
