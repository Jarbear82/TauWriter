use crate::db::{
    Db, HubAssignment, HubFieldDef, HubImport, HubInstance, HubReference, HubRoleDef, HubType,
    HubgsParseResult, LspPosition, LspRange, SemanticToken, SourceFile,
};
use tree_sitter::{Language, Parser};

// Actual C-linked languages
extern "C" {
    pub fn tree_sitter_hubgs() -> Language;
    pub fn tree_sitter_twxml() -> Language;
}

pub fn compute_semantic_tokens(db: &dyn Db, file: SourceFile) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    if path.ends_with(".hubgs") {
        let language = unsafe { tree_sitter_hubgs() };
        let mut parser = Parser::new();
        parser.set_language(language).ok();
        let tree = parser.parse(&contents, None).unwrap();

        let query_str = r#"
            (hub_definition (identifier) @type_def)
            (instance_block (identifier) @inst_name (identifier) @inst_type)
            (hub_field (identifier) @field_name)
            (hub_role (identifier) @role_name)
            (instance_assignment (identifier) @assign_name)
            (enum_definition (identifier) @enum_name)
        "#;
        let query = tree_sitter::Query::new(language, query_str).unwrap();
        let mut query_cursor = tree_sitter::QueryCursor::new();
        let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

        for m in matches {
            for capture in m.captures {
                let name = &query.capture_names()[capture.index as usize];
                let node = capture.node;
                let range = node.range();
                let (token_type, modifiers) = match name.as_str() {
                    "type_def" => (0, 3),                 // CLASS, DEFINITION | DECLARATION
                    "inst_name" => (2, 2),                // VARIABLE, DEFINITION
                    "inst_type" => (0, 0),                // CLASS
                    "field_name" | "role_name" => (1, 3), // PROPERTY, DEFINITION | DECLARATION
                    "assign_name" => (1, 0),              // PROPERTY
                    "enum_name" => (3, 3),                // ENUM, DEFINITION | DECLARATION
                    _ => continue,
                };
                tokens.push(SemanticToken {
                    line: range.start_point.row as u32,
                    character: range.start_point.column as u32,
                    length: (range.end_byte - range.start_byte) as u32,
                    token_type,
                    token_modifiers: modifiers,
                });
            }
        }
    } else if path.ends_with(".twxml") {
        let language = unsafe { tree_sitter_twxml() };
        let mut parser = Parser::new();
        parser.set_language(language).ok();
        let tree = parser.parse(&contents, None).unwrap();

        let query_str = r#"
            (element
                (start_tag
                    (tag_name) @tag_name (#eq? @tag_name "hubref")
                    (attribute
                        (attribute_name) @attr_name (#eq? @attr_name "id")
                        (attribute_value) @attr_value
                    )
                )
            )
        "#;
        let query = tree_sitter::Query::new(language, query_str).unwrap();
        let mut query_cursor = tree_sitter::QueryCursor::new();
        let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

        for m in matches {
            if let Some(node) = m.nodes_for_capture_index(2).next() {
                let range = node.range();
                tokens.push(SemanticToken {
                    line: range.start_point.row as u32,
                    character: range.start_point.column as u32 + 1, // Skip quote
                    length: (range.end_byte - range.start_byte - 2) as u32, // Subtract quotes
                    token_type: 2,                                  // VARIABLE
                    token_modifiers: 0,
                });
            }
        }
    }

    tokens.sort_by(|a, b| {
        if a.line != b.line {
            a.line.cmp(&b.line)
        } else {
            a.character.cmp(&b.character)
        }
    });
    tokens
}

pub fn compute_folding_ranges(db: &dyn Db, file: SourceFile) -> Vec<LspRange> {
    let mut ranges = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    let language = if path.ends_with(".hubgs") {
        unsafe { tree_sitter_hubgs() }
    } else if path.ends_with(".twxml") {
        unsafe { tree_sitter_twxml() }
    } else {
        return ranges;
    };

    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let mut stack = vec![tree.root_node()];

    while let Some(node) = stack.pop() {
        let range = node.range();
        if range.start_point.row != range.end_point.row {
            let is_foldable = match node.kind() {
                "imports_section"
                | "definitions_section"
                | "fields_block"
                | "enums_block"
                | "hubs_block"
                | "hub_definition"
                | "instances_section"
                | "instance_block"
                | "element" => true,
                _ => false,
            };

            if is_foldable {
                ranges.push(ts_range_to_lsp(range));
            }
        }

        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            stack.push(child);
        }
    }

    ranges
}

pub fn get_hub_type_at_position(db: &dyn Db, file: SourceFile, pos: LspPosition) -> Option<String> {
    let contents = file.contents(db);
    let language = unsafe { tree_sitter_hubgs() };
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

    // Walk up to find instance_block
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

pub fn is_in_hub_definition(db: &dyn Db, file: SourceFile, pos: LspPosition) -> bool {
    let contents = file.contents(db);
    let language = unsafe { tree_sitter_hubgs() };
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

pub fn parse_hubgs_ast(db: &dyn Db, file: SourceFile) -> HubgsParseResult<'_> {
    let mut instances = Vec::new();
    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut structs = Vec::new();
    let mut global_fields = Vec::new();
    let mut imports = Vec::new();
    let contents = file.contents(db);

    let language = unsafe { tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let mut cursor = tree.walk();
    for node in tree.root_node().children(&mut cursor) {
        if node.kind() == "imports_section" {
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

        if node.kind() == "definitions_section" {
            let mut def_cursor = node.walk();
            for block in node.children(&mut def_cursor) {
                match block.kind() {
                    "fields_block" => {
                        let mut field_cursor = block.walk();
                        for field_def in block.children(&mut field_cursor) {
                            if field_def.kind() == "field_definition" {
                                if let (Some(id_node), Some(type_node)) =
                                    (field_def.child(0), field_def.child(2))
                                {
                                    global_fields.push(crate::db::GlobalField::new(
                                        db,
                                        contents[id_node.byte_range()].to_string(),
                                        file,
                                        ts_range_to_lsp(id_node.range()),
                                        contents[type_node.byte_range()].to_string(),
                                    ));
                                }
                            }
                        }
                    }
                    "enums_block" => {
                        let mut enum_cursor = block.walk();
                        for enum_def in block.children(&mut enum_cursor) {
                            if enum_def.kind() == "enum_definition" {
                                if let Some(name_node) = enum_def.child(0) {
                                    let mut variants = Vec::new();
                                    let mut var_cursor = enum_def.walk();
                                    for var_node in enum_def.children(&mut var_cursor) {
                                        if var_node.kind() == "identifier"
                                            && var_node.id() != name_node.id()
                                        {
                                            variants
                                                .push(contents[var_node.byte_range()].to_string());
                                        }
                                    }
                                    enums.push(crate::db::HubEnum::new(
                                        db,
                                        contents[name_node.byte_range()].to_string(),
                                        file,
                                        ts_range_to_lsp(name_node.range()),
                                        variants,
                                    ));
                                }
                            }
                        }
                    }
                    "structs_block" => {
                        let mut struct_cursor = block.walk();
                        for struct_def in block.children(&mut struct_cursor) {
                            if struct_def.kind() == "struct_definition" {
                                if let Some(name_node) = struct_def.child(0) {
                                    let mut field_names = Vec::new();
                                    let mut field_cursor = struct_def.walk();
                                    for field_node in struct_def.children(&mut field_cursor) {
                                        if field_node.kind() == "identifier"
                                            && field_node.id() != name_node.id()
                                        {
                                            field_names.push(
                                                contents[field_node.byte_range()].to_string(),
                                            );
                                        }
                                    }
                                    structs.push(crate::db::HubStruct::new(
                                        db,
                                        contents[name_node.byte_range()].to_string(),
                                        file,
                                        ts_range_to_lsp(name_node.range()),
                                        field_names,
                                    ));
                                }
                            }
                        }
                    }
                    "hubs_block" => {
                        let mut hub_cursor = block.walk();
                        for hub_def in block.children(&mut hub_cursor) {
                            if hub_def.kind() == "hub_definition" {
                                if let Some(name_node) = hub_def.child(0) {
                                    let name = contents[name_node.byte_range()].to_string();
                                    let mut fields = Vec::new();
                                    let mut roles = Vec::new();

                                    let mut item_cursor = hub_def.walk();
                                    for item in hub_def.children(&mut item_cursor) {
                                        match item.kind() {
                                            "hub_field" => {
                                                if let Some(id_node) = item.child(0) {
                                                    let mut decorator = None;
                                                    let mut expression = None;
                                                    if let Some(eq_node) = item.child(1) {
                                                        if eq_node.kind() == "=" {
                                                            if let Some(dec_node) = item.child(2) {
                                                                if dec_node.kind() == "decorator" {
                                                                    if let Some(choice_node) =
                                                                        dec_node.child(0)
                                                                    {
                                                                        decorator = Some(
                                                                            contents[choice_node
                                                                                .byte_range()]
                                                                            .to_string(),
                                                                        );
                                                                    }
                                                                    if let Some(expr_node) =
                                                                        dec_node.child(2)
                                                                    {
                                                                        expression = Some(
                                                                            contents[expr_node
                                                                                .byte_range()]
                                                                            .to_string(),
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    fields.push(HubFieldDef {
                                                        name: contents[id_node.byte_range()]
                                                            .to_string(),
                                                        range: ts_range_to_lsp(id_node.range()),
                                                        decorator,
                                                        expression,
                                                    });
                                                }
                                            }

                                            "hub_role" => {
                                                if let Some(id_node) = item.child(0) {
                                                    let role_name =
                                                        contents[id_node.byte_range()].to_string();
                                                    let direction = item
                                                        .child(1)
                                                        .map(|n| {
                                                            contents[n.byte_range()].to_string()
                                                        })
                                                        .unwrap_or_default();
                                                    let multiplicity = item
                                                        .child(3)
                                                        .map(|n| {
                                                            contents[n.byte_range()].to_string()
                                                        })
                                                        .unwrap_or_default();
                                                    let mut allowed_types = Vec::new();
                                                    if let Some(allows_list) = item.child(7) {
                                                        let mut list_cursor = allows_list.walk();
                                                        for type_id in
                                                            allows_list.children(&mut list_cursor)
                                                        {
                                                            if type_id.kind() == "identifier" {
                                                                allowed_types.push(
                                                                    contents[type_id.byte_range()]
                                                                        .to_string(),
                                                                );
                                                            }
                                                        }
                                                    }
                                                    roles.push(HubRoleDef {
                                                        name: role_name,
                                                        direction,
                                                        multiplicity,
                                                        allowed_types,
                                                    });
                                                }
                                            }
                                            _ => {}
                                        }
                                    }

                                    types.push(HubType::new(
                                        db,
                                        name,
                                        file,
                                        ts_range_to_lsp(name_node.range()),
                                        fields,
                                        roles,
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if node.kind() == "instances_section" {
            let mut section_cursor = node.walk();
            for child in node.children(&mut section_cursor) {
                if child.kind() == "instance_block" {
                    if let Some(ref_node) = child.child_by_field_name("ref") {
                        let name = contents[ref_node.byte_range()].to_string();
                        let type_name = if let Some(type_node) = child.child_by_field_name("type") {
                            contents[type_node.byte_range()].to_string()
                        } else {
                            "Unknown".to_string()
                        };

                        let mut description = None;
                        let mut assignments = Vec::new();
                        let mut block_cursor = child.walk();
                        for assignment in child.children(&mut block_cursor) {
                            if assignment.kind() == "instance_assignment" {
                                if let Some(id_node) = assignment.child(0) {
                                    let attr_name = contents[id_node.byte_range()].to_string();
                                    if attr_name == "description" {
                                        if let Some(expr_node) = assignment.child(2) {
                                            if expr_node.kind() == "string" {
                                                description = Some(
                                                    contents[expr_node.byte_range()]
                                                        .trim_matches('"')
                                                        .trim_matches('\'')
                                                        .to_string(),
                                                );
                                            }
                                        }
                                    } else if let Some(expr_node) = assignment.child(2) {
                                        if let Some(val) = node_to_hub_value(expr_node, &contents) {
                                            assignments.push(HubAssignment {
                                                name: attr_name,
                                                range: ts_range_to_lsp(id_node.range()),
                                                value: val,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        instances.push(HubInstance::new(
                            db,
                            name,
                            type_name,
                            file,
                            ts_range_to_lsp(ref_node.range()),
                            description,
                            assignments,
                        ));
                    }
                }
            }
        }
    }

    HubgsParseResult::new(db, instances, types, enums, structs, global_fields, imports)
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
            // Recurse into first non-punctuation child
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

pub fn parse_twxml_ast(db: &dyn Db, file: SourceFile) -> Vec<HubReference<'_>> {
    let mut refs = Vec::new();
    let contents = file.contents(db);

    let language = unsafe { tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let query_str = r#"
        (element
            (start_tag
                (tag_name) @tag_name (#eq? @tag_name "hubref")
                (attribute
                    (attribute_name) @attr_name (#eq? @attr_name "id")
                    (attribute_value) @attr_value
                )
            )
        )
    "#;
    let query = tree_sitter::Query::new(language, query_str).unwrap();
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    for m in matches {
        if let Some(node) = m.nodes_for_capture_index(2).next() {
            let val = contents[node.byte_range()]
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            refs.push(HubReference::new(
                db,
                val,
                file,
                ts_range_to_lsp(node.range()),
            ));
        }
    }

    refs
}

pub fn get_all_twxml_tags(db: &dyn Db, file: SourceFile) -> Vec<crate::db::TwxmlTag<'_>> {
    let mut tags = Vec::new();
    let contents = file.contents(db);

    let language = unsafe { tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let query_str = "(tag_name) @tag";
    let query = tree_sitter::Query::new(language, query_str).unwrap();
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    for m in matches {
        for capture in m.captures {
            let node = capture.node; // This is the tag_name node
            let tag_name = contents[node.byte_range()].to_string();

            if let Some(start_tag_node) = node.parent() {
                if start_tag_node.kind() == "start_tag" {
                    if let Some(element_node) = start_tag_node.parent() {
                        if element_node.kind() == "element" {
                            let mut parent_name = None;
                            if let Some(parent_element_node) = element_node.parent() {
                                if parent_element_node.kind() == "element" {
                                    if let Some(p_start_tag) = parent_element_node.child(0) {
                                        if p_start_tag.kind() == "start_tag" {
                                            if let Some(p_tag_name_node) =
                                                p_start_tag.child_by_field_name("name")
                                            {
                                                parent_name = Some(
                                                    contents[p_tag_name_node.byte_range()]
                                                        .to_string(),
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            tags.push(crate::db::TwxmlTag::new(
                                db,
                                tag_name.clone(),
                                file,
                                ts_range_to_lsp(node.range()),
                                parent_name,
                            ));
                        }
                    }
                }
            }
        }
    }

    tags
}

fn ts_range_to_lsp(range: tree_sitter::Range) -> LspRange {
    LspRange {
        start: crate::db::LspPosition {
            line: range.start_point.row as u32,
            character: range.start_point.column as u32,
        },
        end: crate::db::LspPosition {
            line: range.end_point.row as u32,
            character: range.end_point.column as u32,
        },
    }
}
