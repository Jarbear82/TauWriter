use crate::db::{
    Db, HubAssignment, HubFieldDef, HubImport, HubInstance, HubRoleDef, HubType, HubgsParseResult,
    SourceFile, HubEnum, HubStruct, GlobalField,
};

pub fn parse_hubgs_ast(db: &dyn Db, file: SourceFile) -> HubgsParseResult<'_> {
    let contents = file.contents(db);

    let output = match tauwriter_hubgs::parse_hubgs_full(&contents) {
        Ok(out) => out,
        Err(_) => {
            return HubgsParseResult::new(
                db,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        }
    };

    let types = output
        .types_ast
        .into_iter()
        .map(|t| {
            HubType::new(
                db,
                t.name,
                file,
                span_to_lsp(t.range),
                span_to_lsp(t.block_range),
                t.fields
                    .into_iter()
                    .map(|f| HubFieldDef {
                        name: f.name,
                        decorator: f.decorator,
                        expression: f.expression,
                        is_display: f.is_display,
                        is_background: f.is_background,
                        range: span_to_lsp(f.range),
                    })
                    .collect(),
                t.roles
                    .into_iter()
                    .map(|r| HubRoleDef {
                        name: r.name,
                        direction: r.direction,
                        multiplicity: r.multiplicity,
                        allowed_types: r.allowed_types,
                    })
                    .collect(),
                t.extends_parents,
                t.constraints,
            )
        })
        .collect();

    let instances = output
        .instances_ast
        .into_iter()
        .map(|i| {
            HubInstance::new(
                db,
                i.id,
                i.type_name,
                file,
                span_to_lsp(i.name_range),
                span_to_lsp(i.block_range),
                i.description,
                i.assignments
                    .into_iter()
                    .map(|a| HubAssignment {
                        name: a.name,
                        range: span_to_lsp(a.range),
                        value: convert_hub_value(a.value),
                        value_range: span_to_lsp(a.value_range),
                    })
                    .collect(),
            )
        })
        .collect();

    let enums = output
        .enums_ast
        .into_iter()
        .map(|e| {
            HubEnum::new(
                db,
                e.name,
                file,
                span_to_lsp(e.range),
                e.variants,
            )
        })
        .collect();

    let structs = output
        .structs_ast
        .into_iter()
        .map(|s| {
            HubStruct::new(
                db,
                s.name,
                file,
                span_to_lsp(s.range),
                s.field_names,
            )
        })
        .collect();

    let global_fields = output
        .global_fields_ast
        .into_iter()
        .map(|g| {
            GlobalField::new(
                db,
                g.name,
                file,
                span_to_lsp(g.range),
                g.type_name,
            )
        })
        .collect();

    let imports = output
        .imports
        .into_iter()
        .map(|imp| HubImport {
            types: imp.types,
            from: imp.from,
        })
        .collect();

    HubgsParseResult::new(db, instances, types, enums, structs, global_fields, imports)
}

fn span_to_lsp(span: tauwriter_hubgs::SpanRange) -> crate::db::LspRange {
    crate::db::LspRange {
        start: crate::db::LspPosition {
            line: span.start.line,
            character: span.start.character,
        },
        end: crate::db::LspPosition {
            line: span.end.line,
            character: span.end.character,
        },
    }
}

fn convert_hub_value(val: tauwriter_hubgs::HubValue) -> crate::db::HubValue {
    match val {
        tauwriter_hubgs::HubValue::Identifier(s) => crate::db::HubValue::Identifier(s),
        tauwriter_hubgs::HubValue::Number(n) => crate::db::HubValue::Number(crate::db::RawF64::from_f64(n)),
        tauwriter_hubgs::HubValue::Text(s) => crate::db::HubValue::Text(s),
        tauwriter_hubgs::HubValue::Boolean(b) => crate::db::HubValue::Boolean(b),
        tauwriter_hubgs::HubValue::Array(vals) => {
            crate::db::HubValue::Array(vals.into_iter().map(convert_hub_value).collect())
        }
    }
}

pub fn get_hub_type_at_position(
    db: &dyn Db,
    file: SourceFile,
    pos: crate::db::LspPosition,
) -> Option<String> {
    let contents = file.contents(db);
    let language = super::super::get_hubgs_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(&contents, None)?;

    let point = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };
    let mut curr = tree.root_node().descendant_for_point_range(point, point);
    while let Some(node) = curr {
        if node.kind() == "hub_definition" {
            let id_node = node.child(0)?;
            return Some(contents[id_node.byte_range()].to_string());
        } else if node.kind() == "instance_block" {
            let type_node = node.child_by_field_name("type")?;
            return Some(contents[type_node.byte_range()].to_string());
        }
        curr = node.parent();
    }
    None
}

pub fn is_in_hub_definition(
    db: &dyn Db,
    file: SourceFile,
    pos: crate::db::LspPosition,
) -> bool {
    let contents = file.contents(db);
    let language = match super::super::get_hubgs_language() {
        Some(lang) => lang,
        None => return false,
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let tree = match parser.parse(&contents, None) {
        Some(t) => t,
        None => return false,
    };

    let point = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };
    let mut curr = tree.root_node().descendant_for_point_range(point, point);
    while let Some(node) = curr {
        if node.kind() == "hub_definition" {
            return true;
        }
        curr = node.parent();
    }
    false
}

