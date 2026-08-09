use crate::db::{
    Db, HubAssignment, HubFieldDef, HubImport, HubInstance, HubRoleDef, HubType, HubgsParseResult,
    SourceFile,
};
use tree_sitter::Parser;

// user-review: Query string for hubgs grammar. Captures all top-level sections.
const HUBGS_QUERY: &str = r#"
    (hub_definition (identifier) @type_def)
    (instance_block (identifier) @inst_name (identifier) @inst_type)
    (hub_field (identifier) @field_name)
    (hub_role (identifier) @role_name)
    (instance_assignment (identifier) @assign_name)
    (enum_definition (identifier) @enum_name)
"#;

pub fn parse_hubgs_ast(db: &dyn Db, file: SourceFile) -> HubgsParseResult<'_> {
    let mut instances = Vec::new();
    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut structs = Vec::new();
    let mut global_fields = Vec::new();
    let mut imports = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    let language = match super::super::get_hubgs_language() {
        Some(lang) => lang,
        None => {
            return HubgsParseResult::new(
                db,
                instances,
                types,
                enums,
                structs,
                global_fields,
                imports,
            )
        }
    };

    let tree = {
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&language).is_err() {
            return HubgsParseResult::new(
                db,
                instances,
                types,
                enums,
                structs,
                global_fields,
                imports,
            );
        }
        match parser.parse(&contents, None) {
            Some(t) => t,
            None => {
                return HubgsParseResult::new(
                    db,
                    instances,
                    types,
                    enums,
                    structs,
                    global_fields,
                    imports,
                )
            }
        }
    };

    let mut cursor = tree.walk();
    for node in tree.root_node().children(&mut cursor) {
        if node.kind() == "imports_section" {
            parse_imports(&node, &contents, &mut imports);
        }
        if node.kind() == "definitions_section" {
            parse_definitions(
                db,
                file,
                &node,
                &contents,
                &mut global_fields,
                &mut enums,
                &mut structs,
                &mut types,
            );
        }
        if node.kind() == "instances_section" {
            parse_instances(db, file, &node, &contents, &mut instances);
        }
    }

    HubgsParseResult::new(db, instances, types, enums, structs, global_fields, imports)
}

fn parse_imports(node: &tree_sitter::Node, contents: &str, imports: &mut Vec<HubImport>) {
    let mut imp_cursor = node.walk();
    for stmt in node.children(&mut imp_cursor) {
        if stmt.kind() == "import_statement" {
            let mut type_names = Vec::new();
            let mut from_path = String::new();

            let mut stmt_cursor = stmt.walk();
            for child in stmt.children(&mut stmt_cursor) {
                match child.kind() {
                    "identifier" => {
                        type_names.push(contents[child.byte_range()].to_string());
                    }
                    "string" => {
                        from_path = contents[child.byte_range()]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                    }
                    _ => {}
                }
            }
            imports.push(HubImport {
                types: type_names,
                from: from_path,
            });
        }
    }
}

fn parse_definitions<'a>(
    db: &'a dyn Db,
    file: SourceFile,
    node: &tree_sitter::Node,
    contents: &str,
    global_fields: &mut Vec<crate::db::GlobalField<'a>>,
    enums: &mut Vec<crate::db::HubEnum<'a>>,
    structs: &mut Vec<crate::db::HubStruct<'a>>,
    types: &mut Vec<HubType<'a>>,
) {
    let mut def_cursor = node.walk();
    for block in node.children(&mut def_cursor) {
        match block.kind() {
            "fields_block" => parse_fields_block(db, file, &block, contents, global_fields),
            "enums_block" => parse_enums_block(db, file, &block, contents, enums),
            "structs_block" => parse_structs_block(db, file, &block, contents, structs),
            "hubs_block" => parse_hubs_block(db, file, &block, contents, types),
            _ => {}
        }
    }
}

fn parse_fields_block<'a>(
    db: &'a dyn Db,
    file: SourceFile,
    block: &tree_sitter::Node,
    contents: &str,
    global_fields: &mut Vec<crate::db::GlobalField<'a>>,
) {
    let mut field_cursor = block.walk();
    for field_def in block.children(&mut field_cursor) {
        if field_def.kind() == "field_definition" {
            if let (Some(id_node), Some(type_node)) = (field_def.child(0), field_def.child(2)) {
                let id = contents[id_node.byte_range()].to_string();
                let type_str = contents[type_node.byte_range()].to_string();
                if !id.is_empty()
                    && !type_str.is_empty()
                    && !id_node.is_missing()
                    && !type_node.is_missing()
                {
                    global_fields.push(crate::db::GlobalField::new(
                        db,
                        id,
                        file,
                        super::super::ts_range_to_lsp(id_node.range()),
                        type_str,
                    ));
                }
            }
        }
    }
}

fn parse_enums_block<'a>(
    db: &'a dyn Db,
    file: SourceFile,
    block: &tree_sitter::Node,
    contents: &str,
    enums: &mut Vec<crate::db::HubEnum<'a>>,
) {
    let mut enum_cursor = block.walk();
    for enum_def in block.children(&mut enum_cursor) {
        if enum_def.kind() == "enum_definition" {
            let name_node = match enum_def.child(0) {
                Some(n) if !n.is_missing() => n,
                _ => continue,
            };
            let name = contents[name_node.byte_range()].to_string();
            if name.is_empty() {
                continue;
            }

            let mut variants = Vec::new();
            let mut var_cursor = enum_def.walk();
            for var_node in enum_def.children(&mut var_cursor) {
                if var_node.kind() == "identifier" && var_node.id() != name_node.id() {
                    let var_name = contents[var_node.byte_range()].to_string();
                    if !var_name.is_empty() && !var_node.is_missing() {
                        variants.push(var_name);
                    }
                }
            }
            enums.push(crate::db::HubEnum::new(
                db,
                name,
                file,
                super::super::ts_range_to_lsp(name_node.range()),
                variants,
            ));
        }
    }
}

fn parse_structs_block<'a>(
    db: &'a dyn Db,
    file: SourceFile,
    block: &tree_sitter::Node,
    contents: &str,
    structs: &mut Vec<crate::db::HubStruct<'a>>,
) {
    let mut struct_cursor = block.walk();
    for struct_def in block.children(&mut struct_cursor) {
        if struct_def.kind() == "struct_definition" {
            let name_node = match struct_def.child(0) {
                Some(n) if !n.is_missing() => n,
                _ => continue,
            };
            let name = contents[name_node.byte_range()].to_string();
            if name.is_empty() {
                continue;
            }

            let mut field_names = Vec::new();
            let mut field_cursor = struct_def.walk();
            for field_node in struct_def.children(&mut field_cursor) {
                if field_node.kind() == "identifier" && field_node.id() != name_node.id() {
                    let f_name = contents[field_node.byte_range()].to_string();
                    if !f_name.is_empty() && !field_node.is_missing() {
                        field_names.push(f_name);
                    }
                }
            }
            structs.push(crate::db::HubStruct::new(
                db,
                name,
                file,
                super::super::ts_range_to_lsp(name_node.range()),
                field_names,
            ));
        }
    }
}

fn parse_hubs_block<'a>(
    db: &'a dyn Db,
    file: SourceFile,
    block: &tree_sitter::Node,
    contents: &str,
    types: &mut Vec<HubType<'a>>,
) {
    let mut hub_cursor = block.walk();
    for hub_def in block.children(&mut hub_cursor) {
        if hub_def.kind() == "hub_definition" {
            let name_node = match hub_def.child(0) {
                Some(n) if !n.is_missing() => n,
                _ => continue,
            };
            let name = contents[name_node.byte_range()].to_string();
            if name.is_empty() {
                continue;
            }

            // user-review: Extract parent types from optional EXTENDS clause
            let mut ext_nodes: Vec<_> = hub_def
                .children(&mut hub_def.walk())
                .filter(|c| c.kind() == "extension_clause")
                .collect();
            let extends_parents: Vec<String> = ext_nodes
                .drain(..)
                .flat_map(|ext| {
                    (0..ext.child_count() as u32).filter_map(move |i| {
                        ext.child(i).and_then(|child| {
                            if child.kind() == "identifier" {
                                Some(contents[child.byte_range()].to_string())
                            } else {
                                None
                            }
                        })
                    })
                })
                .collect();

            let mut fields = Vec::new();
            let mut roles = Vec::new();
            let mut constraints = Vec::new();

            let mut item_cursor = hub_def.walk();
            for item in hub_def.children(&mut item_cursor) {
                match item.kind() {
                    "hub_field" => {
                        if let Some(id_node) = item.child(0) {
                            let (decorator, expression) = parse_field_decorators(&item, contents);
                            let mut is_display = false;
                            let mut is_background = false;
                            let mut attr_cursor = item.walk();
                            for child in item.children(&mut attr_cursor) {
                                if child.kind() == "field_attribute" {
                                    let attr_text = contents[child.byte_range()].trim().to_string();
                                    if attr_text == "@display" {
                                        is_display = true;
                                    } else if attr_text == "@background" {
                                        is_background = true;
                                    }
                                }
                            }
                            fields.push(HubFieldDef {
                                name: contents[id_node.byte_range()].to_string(),
                                range: super::super::ts_range_to_lsp(id_node.range()),
                                decorator,
                                expression,
                                is_display,
                                is_background,
                            });
                        }
                    }
                    "hub_role" => {
                        if item.child(0).is_some() {
                            roles.push(parse_hub_role(&item, contents));
                        }
                    }
                    "constraints_block" => {
                        let mut c_cursor = item.walk();
                        for child in item.children(&mut c_cursor) {
                            let kind = child.kind();
                            if kind != "@constraints"
                                && kind != "["
                                && kind != "]"
                                && kind != ","
                                && !child.is_missing()
                            {
                                let expr_text = contents[child.byte_range()].to_string();
                                if !expr_text.is_empty() {
                                    constraints.push(expr_text);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            types.push(HubType::new(
                db,
                name,
                file,
                super::super::ts_range_to_lsp(name_node.range()),
                super::super::ts_range_to_lsp(hub_def.range()),
                fields,
                roles,
                extends_parents,
                constraints,
            ));
        }
    }
}

/// Extract decorator name and expression from a hub_field node.
fn parse_field_decorators(
    item: &tree_sitter::Node,
    contents: &str,
) -> (Option<String>, Option<String>) {
    let mut decorator = None;
    let mut expression = None;
    let mut cursor = item.walk();
    for child in item.children(&mut cursor) {
        if child.kind() == "decorator" {
            if let Some(choice_node) = child.child(0) {
                decorator = Some(contents[choice_node.byte_range()].to_string());
            }
            if let Some(expr_node) = child.child(2) {
                expression = Some(contents[expr_node.byte_range()].to_string());
            }
            break;
        }
    }
    (decorator, expression)
}

/// Build a HubRoleDef from its tree-sitter AST node.
fn parse_hub_role(item: &tree_sitter::Node, contents: &str) -> HubRoleDef {
    let id_node = match item.child(0) {
        Some(n) => n,
        None => {
            return HubRoleDef {
                name: String::new(),
                direction: String::new(),
                multiplicity: String::new(),
                allowed_types: Vec::new(),
            }
        }
    };
    let role_name = contents[id_node.byte_range()].to_string();
    let direction = item
        .child(1)
        .map(|n| contents[n.byte_range()].to_string())
        .unwrap_or_default();
    let multiplicity = item
        .child(3)
        .map(|n| contents[n.byte_range()].to_string())
        .unwrap_or_default();

    let mut allowed_types = Vec::new();
    {
        let mut list_cursor = item.walk();
        for child in item.children(&mut list_cursor) {
            if child.kind() == "identifier" && child.id() != id_node.id() {
                allowed_types.push(contents[child.byte_range()].to_string());
            }
        }
    }

    HubRoleDef {
        name: role_name,
        direction,
        multiplicity,
        allowed_types,
    }
}

fn parse_instances<'a>(
    db: &'a dyn Db,
    file: SourceFile,
    node: &tree_sitter::Node,
    contents: &str,
    instances: &mut Vec<HubInstance<'a>>,
) {
    let mut section_cursor = node.walk();
    for child in node.children(&mut section_cursor) {
        if child.kind() == "instance_block" {
            let ref_node = match child.child_by_field_name("ref") {
                Some(n) if !n.is_missing() => n,
                _ => continue,
            };
            let name = contents[ref_node.byte_range()].to_string();
            if name.is_empty() {
                continue;
            }

            let type_name = child
                .child_by_field_name("type")
                .map(|n| contents[n.byte_range()].to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let mut assignments = Vec::new();
            let mut block_cursor = child.walk();
            for assignment in child.children(&mut block_cursor) {
                if assignment.kind() == "instance_assignment" {
                    if let Some(id_node) = assignment.child(0) {
                        let attr_name = contents[id_node.byte_range()].to_string();
                        if !attr_name.is_empty() && !id_node.is_missing() {
                            if let Some(expr_node) = assignment.child(2) {
                                if let Some(val) = node_to_hub_value(expr_node, contents) {
                                    assignments.push(HubAssignment {
                                        name: attr_name,
                                        range: super::super::ts_range_to_lsp(id_node.range()),
                                        value: val,
                                        value_range: super::super::ts_range_to_lsp(
                                            expr_node.range(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Extract description from assignments if present
            let description = assignments
                .iter()
                .find(|a| a.name == "description")
                .and_then(|a| match &a.value {
                    crate::db::HubValue::Text(s) => Some(s.clone()),
                    _ => None,
                });

            instances.push(HubInstance::new(
                db,
                name,
                type_name,
                file,
                super::super::ts_range_to_lsp(ref_node.range()),
                super::super::ts_range_to_lsp(child.range()),
                description,
                assignments,
            ));
        }
    }
}

/// Convert a tree-sitter expression node to a HubValue.
fn node_to_hub_value(node: tree_sitter::Node, contents: &str) -> Option<crate::db::HubValue> {
    match node.kind() {
        "identifier" => Some(crate::db::HubValue::Identifier(
            contents[node.byte_range()].to_string(),
        )),
        "number" => match contents[node.byte_range()].parse::<f64>() {
            Ok(n) => Some(crate::db::HubValue::Number(crate::db::RawF64::from_f64(n))),
            Err(_) => None,
        },
        "string" | "template_string" => Some(crate::db::HubValue::Text(
            contents[node.byte_range()]
                .trim_matches('"')
                .trim_matches('\'')
                .trim_matches('`')
                .to_string(),
        )),
        "boolean" => Some(crate::db::HubValue::Boolean(
            &contents[node.byte_range()] == "true",
        )),
        "array" => {
            let mut values = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(val) = node_to_hub_value(child, contents) {
                    values.push(val);
                }
            }
            Some(crate::db::HubValue::Array(values))
        }
        "_expression" | "parenthesized_expression" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if !["(", ")", "[", "]", "{", "}", ",", "."].contains(&child.kind()) {
                    if let Some(val) = node_to_hub_value(child, contents) {
                        return Some(val);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

pub fn get_hub_type_at_position(
    db: &dyn Db,
    file: SourceFile,
    pos: lsp_types::Position,
) -> Option<String> {
    let contents = file.contents(db);
    let language = super::super::get_hubgs_language()?;

    let mut parser = Parser::new();
    if let Err(_) = parser.set_language(&language) {
        return None;
    }
    let tree = parser.parse(&contents, None)?;

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let mut node = tree
        .root_node()
        .descendant_for_point_range(ts_pos, ts_pos)?;

    while node.kind() != "instance_block" {
        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            return None;
        }
    }

    node.child_by_field_name("type")
        .map(|n| contents[n.byte_range()].to_string())
}

pub fn is_in_hub_definition(db: &dyn Db, file: SourceFile, pos: lsp_types::Position) -> bool {
    let contents = file.contents(db);
    let Some(language) = super::super::get_hubgs_language() else {
        return false;
    };

    let mut parser = Parser::new();
    if let Err(_) = parser.set_language(&language) {
        return false;
    }
    let tree = match parser.parse(&contents, None) {
        Some(t) => t,
        None => return false,
    };

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let mut node = match tree.root_node().descendant_for_point_range(ts_pos, ts_pos) {
        Some(n) => n,
        None => return false,
    };

    while node.kind() != "hub_definition" {
        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            return false;
        }
    }
    true
}
