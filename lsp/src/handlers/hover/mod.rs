//! Hover provider implementation for LSP.
//!
//! Resolves symbol under cursor and returns structured hover
//! information for Hub instances, types, and global fields.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn hover(server: &Backend, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db_val, ws_val) = server.read_db();
        let db = &db_val;
        let ws = &ws_val;
        return hover_impl(db, *ws, &symbol, &uri);
    }

    Ok(None)
}

pub fn hover_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    symbol: &str,
    uri: &Url,
) -> Result<Option<Hover>> {
    // 1. Try resolve as Hub Instance
    if let Some(instance) = crate::db::resolve_reference(db, ws, symbol.to_string()) {
        return hover_instance(db, ws, instance);
    }

    // 2. Try resolve as Hub Type (scoped)
    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file_name = path.file_name().map(|s| s.to_string_lossy().to_string());
        let file = ws
            .files(db)
            .into_iter()
            .find(|f| f.path(db) == path_str || file_name.as_deref() == Some(f.path(db).as_str()));
        if let Some(file) = file {
            if let Some(hub_type) = crate::db::resolve_type(db, ws, file, symbol.to_string()) {
                return hover_type(db, &hub_type);
            }

            // 3. Try resolve as Global Field
            if let Some(global_field) = resolve_global_field(db, file, symbol) {
                return hover_global_field(db, &global_field);
            }
        }
    }

    Ok(None)
}

fn resolve_global_field<'a>(
    db: &'a dyn crate::db::Db,
    file: crate::db::SourceFile,
    name: &str,
) -> Option<crate::db::GlobalField<'a>> {
    let result = crate::db::parse_hubgs(db, file);
    result
        .global_fields(db)
        .iter()
        .find(|f| f.name(db) == name)
        .cloned()
}

fn extract_source_snippet_type(
    db: &dyn crate::db::Db,
    hub_type: &crate::db::HubType<'_>,
) -> String {
    let file = hub_type.file(db);
    let contents = file.contents(db);
    let block_range = hub_type.block_range(db);
    super::documents::get_range_text(&contents, block_range.into())
}

pub fn format_hub_value(val: &crate::db::HubValue) -> String {
    match val {
        crate::db::HubValue::Text(s) => format!("\"{}\"", s),
        crate::db::HubValue::Array(items) => {
            let formatted: Vec<String> = items
                .iter()
                .map(|item| match item {
                    crate::db::HubValue::Number(n) => n.into_f64().to_string(),
                    crate::db::HubValue::Text(s) => format!("\"{}\"", s),
                    crate::db::HubValue::Boolean(b) => b.to_string(),
                    _ => item.to_string(),
                })
                .collect();
            format!("[{}]", formatted.join(", "))
        }
        other => other.to_string(),
    }
}

fn hover_instance(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    instance: crate::db::HubInstance<'_>,
) -> Result<Option<Hover>> {
    let mut md = crate::handlers::markdown::MarkdownContent::new();

    // Header: Type first, then name
    md.heading(
        2,
        &format!("{}: {} (Hub)", instance.type_name(db), instance.name(db)),
    );

    // Resolve the type to get fields and roles info
    let hub_type = {
        let file = instance.file(db);
        crate::db::resolve_type(db, ws, file, instance.type_name(db).clone())
    };

    // Collect all role names from the type definition
    let role_names: Vec<String> = if let Some(ref ht) = hub_type {
        ht.roles(db).iter().map(|r| r.name.clone()).collect()
    } else {
        Vec::new()
    };

    // Fields section - show non-role field values
    let assignments = instance.assignments(db);
    let field_assignments: Vec<_> = assignments
        .iter()
        .filter(|a| !role_names.contains(&a.name))
        .collect();

    if !field_assignments.is_empty() {
        md.separator();
        md.heading(3, "Fields:");
        for assignment in field_assignments.iter() {
            md.bold_list_item(&assignment.name, &format_hub_value(&assignment.value));
        }
    }

    // Roles section - show relationship info with counts and linked targets
    if let Some(ref ht) = hub_type {
        let roles = ht.roles(db);
        if !roles.is_empty() {
            md.separator();
            md.heading(3, "Roles:");
            for role in roles.iter() {
                // Find the assignment value for this role
                let role_value = assignments.iter().find(|a| a.name == role.name);

                md.bold(&format!("{} ({})", role.name, role.multiplicity));

                if let Some(val) = role_value {
                    match &val.value {
                        crate::db::HubValue::Array(arr) => {
                            md.text(&format!("Count: {}", arr.len()));
                            for item in arr.iter() {
                                if let crate::db::HubValue::Identifier(ref target) = item {
                                    // Resolve to instance and build clickable file URI
                                    if let Some(target_inst) =
                                        crate::db::resolve_reference(db, ws, target.clone())
                                    {
                                        if let Ok(uri) =
                                            Url::from_file_path(target_inst.file(db).path(db))
                                        {
                                            md.link_with_uri(target, &uri.to_string());
                                        } else {
                                            md.text_item(target);
                                        }
                                    } else {
                                        md.text_item(target);
                                    }
                                }
                            }
                        }
                        _ => {
                            // Single value role - try to link if it resolves
                            if let crate::db::HubValue::Identifier(ref target) = &val.value {
                                if let Some(target_inst) =
                                    crate::db::resolve_reference(db, ws, target.clone())
                                {
                                    md.text(&format!("Count: 1"));
                                    if let Ok(uri) =
                                        Url::from_file_path(target_inst.file(db).path(db))
                                    {
                                        md.link_with_uri(target, &uri.to_string());
                                    } else {
                                        md.text_item(target);
                                    }
                                } else {
                                    md.text(&format!("Value: {}", format_hub_value(&val.value)));
                                }
                            } else {
                                md.text(&format!("Value: {}", format_hub_value(&val.value)));
                            }
                        }
                    }
                } else {
                    md.text("Count: 0");
                }
            }
        }
    }

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.to_string(),
        }),
        range: Some(instance.range(db).into()),
    }))
}

fn hover_type(db: &dyn crate::db::Db, hub_type: &crate::db::HubType<'_>) -> Result<Option<Hover>> {
    let mut md = crate::handlers::markdown::MarkdownContent::new();

    // Header
    md.heading(2, &format!("Type: {}", hub_type.name(db)));

    // Fields section with types resolved from global fields
    let fields = hub_type.fields(db);
    if !fields.is_empty() {
        md.separator();
        md.heading(3, "Fields:");
        for f in fields.iter() {
            md.bold_list_item(&f.name, "(from global field def)");
        }
    }

    // Roles section
    let roles = hub_type.roles(db);
    if !roles.is_empty() {
        md.separator();
        md.heading(3, "Roles:");
        for r in roles.iter() {
            let allows_str = if r.allowed_types.is_empty() {
                "(no allows list)".to_string()
            } else {
                format!("ALLOWS [{}]", r.allowed_types.join(", "))
            };
            md.bold_list_item(
                &format!("{} {} ({})", r.name, r.direction, r.multiplicity),
                &allows_str,
            );
        }
    }

    // Source code snippet
    let snippet = extract_source_snippet_type(db, hub_type);
    if !snippet.is_empty() {
        md.separator();
        md.code_block(&snippet, "hubgs");
    }

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.to_string(),
        }),
        range: Some(hub_type.range(db).into()),
    }))
}

fn hover_global_field(
    db: &dyn crate::db::Db,
    global_field: &crate::db::GlobalField<'_>,
) -> Result<Option<Hover>> {
    let mut md = crate::handlers::markdown::MarkdownContent::new();

    md.heading(2, &format!("Field: {}", global_field.name(db)));
    md.separator();
    md.bold_list_item("Type", &global_field.type_name(db));

    // Source snippet
    let file = global_field.file(db);
    let contents = file.contents(db);
    let snippet = super::documents::get_range_text(&contents, global_field.range(db).into());
    if !snippet.is_empty() {
        md.separator();
        md.code_block(&snippet, "hubgs");
    }

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.to_string(),
        }),
        range: Some(global_field.range(db).into()),
    }))
}
