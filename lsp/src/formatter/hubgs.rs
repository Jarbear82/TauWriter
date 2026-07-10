use crate::parser;
use tree_sitter::Parser;

use super::expression::format_expression;

pub fn format_hubgs(contents: &str) -> String {
    let language = match parser::get_language("hubgs") {
        Some(lang) => lang,
        None => return contents.to_string(), // Graceful degradation
    };
    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        return contents.to_string();
    }
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return contents.to_string(),
    };
    let root = tree.root_node();

    if root.has_error() {
        return contents.to_string();
    }

    let mut sections = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "imports_section" => {
                sections.push(format_imports_section(child, contents));
            }
            "definitions_section" => {
                sections.push(format_definitions_section(child, contents));
            }
            "instances_section" => {
                sections.push(format_instances_section(child, contents));
            }
            _ => {}
        }
    }

    let non_empty_sections: Vec<_> = sections
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect();
    non_empty_sections.join(",\n\n") + "\n"
}

fn format_imports_section(node: tree_sitter::Node, contents: &str) -> String {
    let mut imports = Vec::new();
    let mut cursor = node.walk();
    for stmt in node.children(&mut cursor) {
        if stmt.kind() == "import_statement" {
            let (types, from_path) = parse_import_stmt(stmt, contents);
            imports.push(format!("    [ {} ] FROM {}", types.join(", "), from_path));
        }
    }
    format!("IMPORTS [\n{}\n]", imports.join(",\n"))
}

fn parse_import_stmt(stmt: tree_sitter::Node, contents: &str) -> (Vec<String>, String) {
    let mut types = Vec::new();
    let mut from_path = String::new();
    let mut cursor = stmt.walk();
    for child in stmt.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                types.push(contents[child.byte_range()].to_string());
            }
            "string" => {
                from_path = contents[child.byte_range()].to_string();
            }
            _ => {}
        }
    }
    (types, from_path)
}

fn format_definitions_section(node: tree_sitter::Node, contents: &str) -> String {
    let mut blocks = Vec::new();
    let mut cursor = node.walk();
    for block in node.children(&mut cursor) {
        match block.kind() {
            "fields_block" => {
                let fields = parse_fields(block, contents);
                if !fields.is_empty() {
                    blocks.push(format!("    FIELDS [\n{}\n    ]", fields.join(",\n")));
                }
            }
            "enums_block" => {
                let enums = parse_enums(block, contents);
                if !enums.is_empty() {
                    blocks.push(format!("    ENUMS [\n{}\n    ]", enums.join(",\n")));
                }
            }
            "structs_block" => {
                let structs = parse_structs(block, contents);
                if !structs.is_empty() {
                    blocks.push(format!("    STRUCTS [\n{}\n    ]", structs.join(",\n")));
                }
            }
            "hubs_block" => {
                let hubs = parse_hubs(block, contents);
                if !hubs.is_empty() {
                    blocks.push(format!("    HUBS [\n{}\n    ]", hubs.join(",\n")));
                }
            }
            _ => {}
        }
    }
    format!("DEFINITIONS [\n{}\n]", blocks.join(",\n"))
}

fn parse_fields(block: tree_sitter::Node, contents: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cursor = block.walk();
    for field in block.children(&mut cursor) {
        if field.kind() == "field_definition" {
            if let (Some(id_node), Some(type_node)) = (field.child(0), field.child(2)) {
                fields.push(format!(
                    "        {}: {}",
                    &contents[id_node.byte_range()],
                    &contents[type_node.byte_range()]
                ));
            }
        }
    }
    fields
}

/// Collect identifier names from a definition node, excluding the name node.
fn collect_identifiers_excluding(
    def: &tree_sitter::Node,
    exclude_id: usize,
    contents: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = def.walk();
    for child in def.children(&mut cursor) {
        if child.kind() == "identifier" && child.id() != exclude_id {
            names.push(contents[child.byte_range()].to_string());
        }
    }
    names
}

fn parse_enums(block: tree_sitter::Node, contents: &str) -> Vec<String> {
    let mut enums = Vec::new();
    let mut cursor = block.walk();
    for enum_def in block.children(&mut cursor) {
        if enum_def.kind() == "enum_definition" {
            if let Some(name_node) = enum_def.child(0) {
                let variants = collect_identifiers_excluding(&enum_def, name_node.id(), contents);
                enums.push(format!(
                    "        {} {{ {} }}",
                    &contents[name_node.byte_range()],
                    variants.join(", ")
                ));
            }
        }
    }
    enums
}

fn parse_structs(block: tree_sitter::Node, contents: &str) -> Vec<String> {
    let mut structs = Vec::new();
    let mut cursor = block.walk();
    for struct_def in block.children(&mut cursor) {
        if struct_def.kind() == "struct_definition" {
            if let Some(name_node) = struct_def.child(0) {
                let field_names =
                    collect_identifiers_excluding(&struct_def, name_node.id(), contents);
                structs.push(format!(
                    "        {} {{\n            {}\n        }}",
                    &contents[name_node.byte_range()],
                    field_names.join(",\n            ")
                ));
            }
        }
    }
    structs
}

fn parse_hubs(block: tree_sitter::Node, contents: &str) -> Vec<String> {
    let mut hubs = Vec::new();
    let mut cursor = block.walk();
    for hub_def in block.children(&mut cursor) {
        if hub_def.kind() == "hub_definition" {
            if let Some(name_node) = hub_def.child(0) {
                // ponytail: Extract parent types from optional EXTENDS clause
                let mut extends_parts: Vec<String> = Vec::new();
                let mut ext_cursor = hub_def.walk();
                for ext in hub_def.children(&mut ext_cursor) {
                    if ext.kind() == "extension_clause" {
                        let mut child_cursor = ext.walk();
                        for child in ext.children(&mut child_cursor) {
                            if child.kind() == "identifier" {
                                extends_parts.push(contents[child.byte_range()].to_string());
                            }
                        }
                    }
                }
                let extends_str = extends_parts.join(", ");
                let extends_part = if extends_str.is_empty() {
                    String::new()
                } else {
                    format!(" EXTENDS [{}]", extends_str)
                };

                let mut items = Vec::new();
                let mut item_cursor = hub_def.walk();
                for child in hub_def.children(&mut item_cursor) {
                    match child.kind() {
                        "hub_field" => {
                            items
                                .push(format!("            {}", format_hub_field(child, contents)));
                        }
                        "hub_role" => {
                            items.push(format!("            {}", format_hub_role(child, contents)));
                        }
                        "constraints_block" => {
                            items.push(format!(
                                "            {}",
                                format_constraints_block(child, contents)
                            ));
                        }
                        _ => {}
                    }
                }
                hubs.push(format!(
                    "        {}{} {{\n{}\n        }}",
                    &contents[name_node.byte_range()],
                    extends_part,
                    items.join(",\n")
                ));
            }
        }
    }
    hubs
}

fn format_hub_field(node: tree_sitter::Node, contents: &str) -> String {
    if let Some(id_node) = node.child(0) {
        let id_str = &contents[id_node.byte_range()];
        let mut parts = vec![id_str.to_string()];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                parts.push(format!("= {}", format_decorator(child, contents)));
            } else if child.kind() == "field_attribute" {
                parts.push(contents[child.byte_range()].trim().to_string());
            }
        }
        parts.join(" ")
    } else {
        contents[node.byte_range()].to_string()
    }
}

fn format_decorator(node: tree_sitter::Node, contents: &str) -> String {
    if let (Some(name_node), Some(expr_node)) = (node.child(0), node.child(2)) {
        let name_str = &contents[name_node.byte_range()];
        format!("{}({})", name_str, format_expression(expr_node, contents))
    } else {
        contents[node.byte_range()].to_string()
    }
}

fn format_hub_role(node: tree_sitter::Node, contents: &str) -> String {
    if let Some(id_node) = node.child(0) {
        let id_str = &contents[id_node.byte_range()];
        let dir_str = node
            .child(1)
            .map(|n| contents[n.byte_range()].trim())
            .unwrap_or("");
        let mult_str = node
            .child(3)
            .map(|n| contents[n.byte_range()].trim())
            .unwrap_or("");

        let mut allowed = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" && child.id() != id_node.id() {
                allowed.push(contents[child.byte_range()].trim().to_string());
            }
        }
        format!(
            "{} {} ({}) ALLOWS [ {} ]",
            id_str,
            dir_str,
            mult_str,
            allowed.join(", ")
        )
    } else {
        contents[node.byte_range()].to_string()
    }
}

fn format_constraints_block(node: tree_sitter::Node, contents: &str) -> String {
    let mut exprs = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        if kind != "@constraints"
            && kind != "["
            && kind != "]"
            && kind != ","
            && !child.is_missing()
        {
            exprs.push(format_expression(child, contents));
        }
    }
    format!("@constraints [ {} ]", exprs.join(", "))
}

fn format_instances_section(node: tree_sitter::Node, contents: &str) -> String {
    let mut instances = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "instance_block" {
            if let (Some(ref_node), Some(type_node)) = (
                child.child_by_field_name("ref"),
                child.child_by_field_name("type"),
            ) {
                let ref_name = &contents[ref_node.byte_range()];
                let type_name = &contents[type_node.byte_range()];

                let mut assignments = Vec::new();
                let mut max_ident_len = 0;
                let mut block_cursor = child.walk();
                for assign in child.children(&mut block_cursor) {
                    if assign.kind() == "instance_assignment" {
                        if let Some(id_node) = assign.child(0) {
                            let attr_name = contents[id_node.byte_range()].trim().to_string();
                            if let Some(expr_node) = assign.child(2) {
                                let val_str = format_expression(expr_node, contents);
                                max_ident_len = max_ident_len.max(attr_name.len());
                                assignments.push((attr_name, val_str));
                            }
                        }
                    }
                }

                let mut formatted_assigns = Vec::new();
                for (name, val) in assignments {
                    let padding = " ".repeat(max_ident_len - name.len());
                    formatted_assigns.push(format!("        {}{} = {}", name, padding, val));
                }

                let inner = if formatted_assigns.is_empty() {
                    String::new()
                } else {
                    format!("\n{}\n    ", formatted_assigns.join(",\n"))
                };

                instances.push(format!("    {}: {} {{{}}}", ref_name, type_name, inner));
            }
        }
    }
    format!("INSTANCES [\n{}\n]", instances.join(",\n"))
}
