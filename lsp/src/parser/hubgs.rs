use crate::db::{
    Db, HubAssignment, HubFieldDef, HubImport, HubInstance, HubRoleDef, HubType, HubgsParseResult,
    SourceFile,
};
use tree_sitter::Parser;

pub fn parse_hubgs_ast(db: &dyn Db, file: SourceFile) -> HubgsParseResult<'_> {
    let mut instances = Vec::new();
    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut structs = Vec::new();
    let mut global_fields = Vec::new();
    let mut imports = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    let language = unsafe { super::tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    
    let old_entry = crate::get_tree_cache().get(&path).map(|t| t.clone());
    let tree = if let Some(ref entry) = old_entry {
        if entry.content_len == contents.len() && entry.content_hash == crate::calculate_hash(&contents) {
            if entry.needs_reparse {
                let t = parser.parse(&contents, Some(&entry.tree)).unwrap();
                crate::get_tree_cache().insert(
                    path,
                    crate::CachedTree {
                        tree: t.clone(),
                        content_len: contents.len(),
                        content_hash: crate::calculate_hash(&contents),
                        needs_reparse: false,
                    },
                );
                t
            } else {
                entry.tree.clone()
            }
        } else {
            let t = parser.parse(&contents, None).unwrap();
            crate::get_tree_cache().insert(
                path,
                crate::CachedTree {
                    tree: t.clone(),
                    content_len: contents.len(),
                    content_hash: crate::calculate_hash(&contents),
                    needs_reparse: false,
                },
            );
            t
        }
    } else {
        let t = parser.parse(&contents, None).unwrap();
        crate::get_tree_cache().insert(
            path,
            crate::CachedTree {
                tree: t.clone(),
                content_len: contents.len(),
                content_hash: crate::calculate_hash(&contents),
                needs_reparse: false,
            },
        );
        t
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
                if !id.is_empty() && !type_str.is_empty() && !id_node.is_missing() && !type_node.is_missing() {
                    global_fields.push(crate::db::GlobalField::new(
                        db,
                        id,
                        file,
                        super::ts_range_to_lsp(id_node.range()),
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
            if let Some(name_node) = enum_def.child(0) {
                let name = contents[name_node.byte_range()].to_string();
                if !name.is_empty() && !name_node.is_missing() {
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
                        super::ts_range_to_lsp(name_node.range()),
                        variants,
                    ));
                }
            }
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
            if let Some(name_node) = struct_def.child(0) {
                let name = contents[name_node.byte_range()].to_string();
                if !name.is_empty() && !name_node.is_missing() {
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
                        super::ts_range_to_lsp(name_node.range()),
                        field_names,
                    ));
                }
            }
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
            if let Some(name_node) = hub_def.child(0) {
                let name = contents[name_node.byte_range()].to_string();
                if !name.is_empty() && !name_node.is_missing() {
                    // ponytail: Extract parent types from optional EXTENDS clause
                    let mut ext_nodes: Vec<_> = hub_def
                        .children(&mut hub_def.walk())
                        .filter(|c| c.kind() == "extension_clause")
                        .collect();
                    let extends_parents: Vec<String> = ext_nodes
                        .drain(..)
                        .flat_map(|ext| {
                            (0..ext.child_count()).filter_map(move |i| {
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

                    let mut item_cursor = hub_def.walk();
                    for item in hub_def.children(&mut item_cursor) {
                        match item.kind() {
                            "hub_field" => {
                                if let Some(id_node) = item.child(0) {
                                    let (decorator, expression) =
                                        parse_field_decorators(&item, contents);
                                    fields.push(HubFieldDef {
                                        name: contents[id_node.byte_range()].to_string(),
                                        range: super::ts_range_to_lsp(id_node.range()),
                                        decorator,
                                        expression,
                                    });
                                }
                            }
                            "hub_role" => {
                                if let Some(_id_node) = item.child(0) {
                                    roles.push(parse_hub_role(&item, contents));
                                }
                            }
                            _ => {}
                        }
                    }

                    types.push(HubType::new(
                        db,
                        name,
                        file,
                        super::ts_range_to_lsp(name_node.range()),
                        super::ts_range_to_lsp(hub_def.range()),
                        fields,
                        roles,
                        extends_parents,
                    ));
                }
            }
        }
    }
}

fn parse_field_decorators(
    item: &tree_sitter::Node,
    contents: &str,
) -> (Option<String>, Option<String>) {
    let mut decorator = None;
    let mut expression = None;
    if let Some(eq_node) = item.child(1) {
        if eq_node.kind() == "=" {
            if let Some(dec_node) = item.child(2) {
                if dec_node.kind() == "decorator" {
                    if let Some(choice_node) = dec_node.child(0) {
                        decorator = Some(contents[choice_node.byte_range()].to_string());
                    }
                    if let Some(expr_node) = dec_node.child(2) {
                        expression = Some(contents[expr_node.byte_range()].to_string());
                    }
                }
            }
        }
    }
    (decorator, expression)
}

fn parse_hub_role(item: &tree_sitter::Node, contents: &str) -> HubRoleDef {
    let id_node = item.child(0).unwrap();
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
            if let Some(ref_node) = child.child_by_field_name("ref") {
                let name = contents[ref_node.byte_range()].to_string();
                if !name.is_empty() && !ref_node.is_missing() {
                    let type_name = if let Some(type_node) = child.child_by_field_name("type") {
                        contents[type_node.byte_range()].to_string()
                    } else {
                        "Unknown".to_string()
                    };

                    let mut assignments = Vec::new();
                    let mut metadata_display = None;
                    let mut metadata_background = None;
                    let mut metadata_background_range = None;
                    let mut block_cursor = child.walk();
                    for assignment in child.children(&mut block_cursor) {
                        let is_meta = assignment.kind() == "metadata_block";
                        let is_inst_meta = assignment.kind() == "instance_assignment" && assignment.child(0).map(|c| c.kind() == "metadata_block").unwrap_or(false);
                        if is_meta || is_inst_meta {
                            let meta_node = if is_meta { assignment } else { assignment.child(0).unwrap() };
                            let mut meta_cursor = meta_node.walk();
                            let named_children: Vec<tree_sitter::Node> = meta_node
                                .children(&mut meta_cursor)
                                .filter(|n| n.is_named())
                                .collect();
                            for chunk in named_children.chunks_exact(2) {
                                let key_node = chunk[0];
                                let val_node = chunk[1];
                                if key_node.kind() == "identifier" {
                                    let key = contents[key_node.byte_range()].to_string();
                                    if let Some(val) = node_to_hub_value(val_node, contents) {
                                        if key == "display" {
                                            metadata_display = Some(val.to_string());
                                        } else if key == "background" {
                                            metadata_background = Some(val.to_string());
                                            metadata_background_range = Some(super::ts_range_to_lsp(val_node.range()));
                                        }
                                    }
                                }
                            }
                        } else if assignment.kind() == "instance_assignment" {
                            if let Some(id_node) = assignment.child(0) {
                                let attr_name = contents[id_node.byte_range()].to_string();
                                if !attr_name.is_empty() && !id_node.is_missing() {
                                    if let Some(expr_node) = assignment.child(2) {
                                        if let Some(val) = node_to_hub_value(expr_node, contents) {
                                            assignments.push(HubAssignment {
                                                name: attr_name,
                                                range: super::ts_range_to_lsp(id_node.range()),
                                                value: val,
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
                            crate::db::HubValue::String(s) => Some(s.clone()),
                            _ => None,
                        });

                    instances.push(HubInstance::new(
                        db,
                        name,
                        type_name,
                        file,
                        super::ts_range_to_lsp(ref_node.range()),
                        super::ts_range_to_lsp(child.range()),
                        description,
                        assignments,
                        metadata_display,
                        metadata_background,
                        metadata_background_range,
                    ));
                }
            }
        }
    }
}

fn node_to_hub_value(node: tree_sitter::Node, contents: &str) -> Option<crate::db::HubValue> {
    match node.kind() {
        "identifier" => Some(crate::db::HubValue::Identifier(
            contents[node.byte_range()].to_string(),
        )),
        "number" => Some(crate::db::HubValue::Number(
            contents[node.byte_range()].to_string(),
        )),
        "string" | "template_string" => Some(crate::db::HubValue::String(
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
    let language = unsafe { super::tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

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

    if let Some(type_node) = node.child_by_field_name("type") {
        return Some(contents[type_node.byte_range()].to_string());
    }

    None
}

pub fn is_in_hub_definition(db: &dyn Db, file: SourceFile, pos: lsp_types::Position) -> bool {
    let contents = file.contents(db);
    let language = unsafe { super::tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

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

pub fn get_hubgs_completion_context(
    contents: &str,
    pos: lsp_types::Position,
) -> HubgsCompletionContext {
    let language = unsafe { super::tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return HubgsCompletionContext::None,
    };

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let node = match tree.root_node().descendant_for_point_range(ts_pos, ts_pos) {
        Some(n) => n,
        None => return HubgsCompletionContext::None,
    };

    // Check if cursor is after ALLOWS keyword in a hub_role
    let mut current = node;
    while current.kind() != "source_file" {
        if current.kind() == "hub_role" {
            let mut allows_node = None;
            let mut cursor = current.walk();
            for child in current.children(&mut cursor) {
                if child.kind() == "ALLOWS" || &contents[child.byte_range()] == "ALLOWS" {
                    allows_node = Some(child);
                    break;
                }
            }
            if let Some(an) = allows_node {
                if node.start_byte() > an.end_byte() {
                    return HubgsCompletionContext::AllowsList;
                }
            }
            break;
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    // Check for instance assignment context
    let mut current = node;
    let mut assignment_node = None;
    while current.kind() != "source_file" {
        if current.kind() == "instance_assignment" {
            assignment_node = Some(current);
            break;
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    if let Some(assign) = assignment_node {
        if let Some(id_node) = assign.child(0) {
            let role_name = contents[id_node.byte_range()].trim().to_string();

            let mut inst_block = assign;
            while inst_block.kind() != "instance_block" && inst_block.kind() != "source_file" {
                if let Some(p) = inst_block.parent() {
                    inst_block = p;
                } else {
                    break;
                }
            }

            if inst_block.kind() == "instance_block" {
                if let Some(type_node) = inst_block.child_by_field_name("type") {
                    let type_name = contents[type_node.byte_range()].trim().to_string();
                    return HubgsCompletionContext::InstanceAssignment {
                        type_name,
                        role_name,
                    };
                }
            }
        }
    }

    HubgsCompletionContext::None
}

pub enum HubgsCompletionContext {
    AllowsList,
    InstanceAssignment {
        type_name: String,
        role_name: String,
    },
    None,
}
