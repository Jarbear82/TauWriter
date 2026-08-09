use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::uuid::{generate_uuid_ref, generate_uuid_v4, generate_uuid_v7};
use crate::db::TWXML_TAG_INFO;
use crate::Backend;

/// Which UUID completions to offer based on cursor context.
#[derive(Debug, Clone, Copy, PartialEq)]
enum OfferUuid {
    /// Offer ref-style UUID (HubGS instance identifiers use _hex32 format).
    Ref,
    /// Offer standard UUID v4.
    V4,
    /// Offer standard UUID v7.
    V7,
    /// Offer both.
    Both,
    /// No UUID completions — the context doesn't expect a UUID value.
    None,
}

/// Combined result from context-specific completion handlers.
struct CompletionResult {
    response: CompletionResponse,
    offer_uuid: OfferUuid,
}

fn try_completion_context(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
    content: &str,
    position: Position,
) -> Option<CompletionResult> {
    let ctx = crate::parser::get_hubgs_completion_context(content, position);

    match ctx {
        crate::parser::HubgsCompletionContext::AllowsList => Some(CompletionResult {
            response: complete_allows_list(db, ws),
            offer_uuid: OfferUuid::None,
        }),
        crate::parser::HubgsCompletionContext::InstanceAssignment {
            type_name,
            role_name,
        } => Some(CompletionResult {
            response: CompletionResponse::Array(complete_role_instances(
                db, ws, file, &type_name, &role_name,
            )),
            offer_uuid: OfferUuid::Ref,
        }),
        crate::parser::HubgsCompletionContext::None => None,
    }
}

pub fn check_slash_uuid_completion(content: &str, position: Position) -> Option<Vec<CompletionItem>> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return None;
    }
    let line = lines[line_idx];
    let char_idx = position.character as usize;
    if char_idx > line.len() {
        return None;
    }
    let prefix = &line[..char_idx];

    let slash_pos = prefix.rfind('/')?;
    let typed_after_slash = &prefix[slash_pos + 1..];

    if !typed_after_slash.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }

    let edit_range = Range {
        start: Position {
            line: position.line,
            character: slash_pos as u32,
        },
        end: position,
    };

    let uuid_v4_val = generate_uuid_v4();
    let uuid_v7_val = generate_uuid_v7();

    let item_v4 = CompletionItem {
        label: "uuid_v4".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Generate a new UUID v4".to_string()),
        filter_text: Some(format!("/uuid_v4")),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: edit_range,
            new_text: uuid_v4_val.clone(),
        })),
        insert_text: Some(uuid_v4_val),
        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
        ..Default::default()
    };

    let item_v7 = CompletionItem {
        label: "uuid_v7".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Generate a new UUID v7".to_string()),
        filter_text: Some(format!("/uuid_v7")),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range: edit_range,
            new_text: uuid_v7_val.clone(),
        })),
        insert_text: Some(uuid_v7_val),
        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
        ..Default::default()
    };

    Some(vec![item_v4, item_v7])
}

pub async fn completion(
    server: &Backend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    let content = server
        .open_files
        .get(&uri)
        .map(|r| r.to_string())
        .or_else(|| {
            let (db, ws) = server.read_db();
            if let Ok(path) = uri.to_file_path() {
                let path_str = path.to_string_lossy();
                ws.files(&db)
                    .into_iter()
                    .find(|f| f.path(&db) == path_str)
                    .map(|f| f.contents(&db))
            } else {
                None
            }
        });

    if let Some(content) = &content {
        if let Some(slash_items) = check_slash_uuid_completion(content, position) {
            return Ok(Some(CompletionResponse::Array(slash_items)));
        }
    }

    let (db_val, ws_val) = server.read_db();
    let db_ref = &db_val;
    let ws_ref = ws_val;

    let (items, offer_uuid) = if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws_ref
            .files(db_ref)
            .into_iter()
            .find(|f| f.path(db_ref) == path_str);

        if let Some(file) = file {
            let content = file.contents(db_ref);

            if path_str.ends_with(".twxml") {
                match handle_twxml_completion(db_ref, ws_ref, &content, position) {
                    Some(r) => (r.response.unwrap_array(), r.offer_uuid),
                    None => default_fallback_items(db_ref, ws_ref),
                }
            } else if path_str.ends_with(".hubgs") {
                match handle_hubgs_completion(db_ref, ws_ref, file, &content, position) {
                    Some(r) => (r.response.unwrap_array(), r.offer_uuid),
                    None => default_fallback_items(db_ref, ws_ref),
                }
            } else {
                default_fallback_items(db_ref, ws_ref)
            }
        } else {
            default_fallback_items(db_ref, ws_ref)
        }
    } else {
        default_fallback_items(db_ref, ws_ref)
    };

    let mut items = apply_uuid_filter(items, offer_uuid);

    Ok(Some(CompletionResponse::Array(items)))
}

/// Helper: unwrap a CompletionResponse to Vec<CompletionItem>.
trait ResponseExt {
    fn unwrap_array(self) -> Vec<CompletionItem>;
}

impl ResponseExt for CompletionResponse {
    fn unwrap_array(self) -> Vec<CompletionItem> {
        match self {
            CompletionResponse::Array(arr) => arr,
            CompletionResponse::List(list) => list.items,
        }
    }
}

/// Default fallback items (used when no context-specific handler matches).
fn default_fallback_items(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
) -> (Vec<CompletionItem>, OfferUuid) {
    let instances = crate::db::all_hub_instances(db, ws);
    let items = instances
        .into_iter()
        .map(|i| instance_completion_item(db, ws, i))
        .collect();
    (items, OfferUuid::None)
}

/// Conditionally append UUID completions based on context.
fn apply_uuid_filter(mut items: Vec<CompletionItem>, offer: OfferUuid) -> Vec<CompletionItem> {
    match offer {
        OfferUuid::Ref => {
            let uuid_ref = generate_uuid_ref();
            items.push(CompletionItem {
                label: "uuid-ref".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Insert a valid HubGS ref UUID (prefixed, no hyphens)".to_string()),
                insert_text: Some(uuid_ref),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
        }
        OfferUuid::V4 => {
            let uuid_str = generate_uuid_v4();
            items.push(CompletionItem {
                label: "uuid-v4".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Insert a new standard UUID v4".to_string()),
                insert_text: Some(uuid_str),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
        }
        OfferUuid::V7 => {
            let uuid_str = generate_uuid_v7();
            items.push(CompletionItem {
                label: "uuid-v7".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Insert a new standard UUID v7".to_string()),
                insert_text: Some(uuid_str),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
        }
        OfferUuid::Both => {
            let uuid_v4 = generate_uuid_v4();
            let uuid_ref = generate_uuid_ref();
            items.push(CompletionItem {
                label: "uuid-v4".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Insert a new standard UUID v4".to_string()),
                insert_text: Some(uuid_v4),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: "uuid-ref".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Insert a valid HubGS ref UUID (prefixed, no hyphens)".to_string()),
                insert_text: Some(uuid_ref),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                ..Default::default()
            });
        }
        OfferUuid::None => {}
    }
    items
}

/// Handle completion requests for TWXML files.
fn handle_twxml_completion(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    content: &str,
    position: Position,
) -> Option<CompletionResult> {
    let ctx = crate::parser::get_twxml_completion_context(content, position);

    match ctx {
        crate::parser::TwxmlCompletionContext::HubrefId => Some(CompletionResult {
            response: CompletionResponse::Array(complete_hubref_id_instances(db, ws)),
            offer_uuid: OfferUuid::Ref,
        }),
        crate::parser::TwxmlCompletionContext::HubrefField { id_val } => {
            complete_hub_fields(db, ws, &id_val).map(|items| CompletionResult {
                response: CompletionResponse::Array(items),
                offer_uuid: OfferUuid::None,
            })
        }
        crate::parser::TwxmlCompletionContext::Tag { parent } => Some(CompletionResult {
            response: complete_twxml_tags(parent.as_deref()),
            offer_uuid: OfferUuid::None,
        }),
        crate::parser::TwxmlCompletionContext::None => None,
    }
}

/// Complete hubref instance suggestions for TWXML.
fn complete_hubref_id_instances(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
) -> Vec<CompletionItem> {
    let instances = crate::db::all_hub_instances(db, ws);
    instances
        .into_iter()
        .map(|i| instance_completion_item(db, ws, i))
        .collect()
}

/// Build a completion item for a HubGS instance.
fn instance_completion_item(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    i: crate::db::HubInstance<'_>,
) -> CompletionItem {
    let display = crate::db::resolution::hub_instance_metadata_display(db, ws, i);
    let detail = if let Some(disp) = display {
        format!("Hub Instance ({}) - {}", i.type_name(db), disp)
    } else {
        format!("Hub Instance ({})", i.type_name(db))
    };
    CompletionItem {
        label: i.name(db),
        kind: Some(CompletionItemKind::REFERENCE),
        detail: Some(detail),
        ..Default::default()
    }
}

/// Suggest TWXML structural tags based on the current parent context.
fn complete_twxml_tags(parent: Option<&str>) -> CompletionResponse {
    // Filter out root-level tags that don't belong inside a parent.
    let items: Vec<CompletionItem> = TWXML_TAG_INFO
        .iter()
        .filter(|(name, _kind, _detail)| {
            if parent.is_some() && *name == "document" {
                return false;
            }
            if parent == Some("body") && *name == "body" {
                return false;
            }
            true
        })
        .map(|(name, kind, detail)| CompletionItem {
            label: name.to_string(),
            kind: Some(*kind),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect();

    CompletionResponse::Array(items)
}

fn complete_hub_fields(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    id_val: &str,
) -> Option<Vec<CompletionItem>> {
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
            return Some(items);
        }
    }
    None
}

fn handle_hubgs_completion(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
    content: &str,
    position: Position,
) -> Option<CompletionResult> {
    if let Some(result) = try_completion_context(db, ws, file, content, position) {
        return Some(result);
    }

    // Try field/role completion on current type at position
    if let Some(type_name) = crate::db::get_hub_type_at_position(db, file, position.into()) {
        if let Some(hub_type) = crate::db::resolve_type(db, ws, file, type_name) {
            let items = complete_fields_and_roles(db, ws, &hub_type);
            return Some(CompletionResult {
                response: CompletionResponse::Array(items),
                offer_uuid: OfferUuid::None,
            });
        }
    }

    // Inside a hub definition — offer global fields
    if crate::db::is_in_hub_definition(db, file, position.into()) {
        let globals = complete_global_fields(db, ws);
        return Some(CompletionResult {
            response: CompletionResponse::Array(globals),
            offer_uuid: OfferUuid::None,
        });
    }

    None
}

fn complete_allows_list(db: &dyn crate::db::Db, ws: crate::db::Workspace) -> CompletionResponse {
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
    CompletionResponse::Array(items)
}

fn complete_role_instances(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
    type_name: &str,
    role_name: &str,
) -> Vec<CompletionItem> {
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
                .map(|i| {
                    let detail = if let Some(disp) =
                        crate::db::resolution::hub_instance_metadata_display(db, ws, i)
                    {
                        format!("Hub Instance ({}) - {}", i.type_name(db), disp)
                    } else {
                        format!("Hub Instance ({})", i.type_name(db))
                    };
                    CompletionItem {
                        label: i.name(db),
                        kind: Some(CompletionItemKind::REFERENCE),
                        detail: Some(detail),
                        ..Default::default()
                    }
                })
                .collect();
            return items;
        }
    }
    Vec::new()
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
