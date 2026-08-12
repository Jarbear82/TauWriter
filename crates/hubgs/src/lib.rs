//! `tauwriter-hubgs` — Standalone HubGS grammar parser and AST data structures.

use std::collections::{HashMap, HashSet};

unsafe extern "C" {
    fn tree_sitter_hubgs() -> *const std::ffi::c_void;
}

/// Load the HubGS tree-sitter language. Returns `None` if the symbol is missing or NULL.
pub fn load_hubgs_language() -> Option<tree_sitter::Language> {
    let ptr = unsafe { tree_sitter_hubgs() };
    if ptr.is_null() {
        None
    } else {
        unsafe { Some(tree_sitter::Language::from_raw(ptr.cast())) }
    }
}

pub mod ast;
pub use ast::*;


/// Extract text from a tree-sitter node given source bytes.
fn node_text(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(|s| s.to_string())
}

/// Strip surrounding quotes from a string literal (single or double),
/// decoding escapes via a single left-to-right scan.
pub fn unquote_string(s: &str) -> String {
    let s = s.trim();
    if s.len() < 2 {
        return s.to_string();
    }
    let first = s.chars().next().unwrap();
    let last = s.chars().last().unwrap();
    if !((first == '"' && last == '"') || (first == '\'' && last == '\'')) {
        return s.to_string();
    }

    let inner = &s[1..s.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

pub fn node_to_hub_value(node: tree_sitter::Node, contents: &str) -> Option<HubValue> {
    match node.kind() {
        "identifier" | "uuid" => Some(HubValue::Identifier(
            contents[node.byte_range()].to_string(),
        )),
        "number" => match contents[node.byte_range()].parse::<f64>() {
            Ok(n) => Some(HubValue::Number(n)),
            Err(_) => None,
        },
        "string" | "template_string" => Some(HubValue::Text(
            contents[node.byte_range()]
                .trim_matches('"')
                .trim_matches('\'')
                .trim_matches('`')
                .to_string(),
        )),
        "boolean" => Some(HubValue::Boolean(
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
            Some(HubValue::Array(values))
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

const MAX_RECURSION_DEPTH: usize = 200;

fn count_errors(node: tree_sitter::Node) -> usize {
    count_errors_impl(node, 0)
}

fn count_errors_impl(node: tree_sitter::Node, depth: usize) -> usize {
    if depth > MAX_RECURSION_DEPTH {
        return 0;
    }
    let mut count = if node.is_error() || node.is_missing() {
        1
    } else {
        0
    };
    for child in node.children(&mut node.walk()) {
        count += count_errors_impl(child, depth + 1);
    }
    count
}

fn parse_imports(root: &tree_sitter::Node, source: &[u8]) -> Vec<HubImport> {
    let mut imports = Vec::new();
    for child in root.named_children(&mut root.walk()) {
        if child.kind() != "imports_section" {
            continue;
        }
        let walker = &mut child.walk();
        for stmt in child.named_children(walker) {
            if stmt.kind() != "import_statement" {
                continue;
            }
            let mut identifiers = Vec::new();
            let mut source_str: Option<String> = None;
            let stmt_walker = &mut stmt.walk();
            for field_child in stmt.named_children(stmt_walker) {
                match field_child.kind() {
                    "identifier" => {
                        if let Some(text) = node_text(field_child, source) {
                            identifiers.push(text);
                        }
                    }
                    "string" => {
                        if let Some(text) = node_text(field_child, source) {
                            source_str = Some(unquote_string(&text));
                        }
                    }
                    _ => {}
                }
            }
            if !identifiers.is_empty() && source_str.is_some() {
                imports.push(HubImport {
                    types: identifiers,
                    from: source_str.unwrap(),
                });
            }
        }
    }
    imports
}

fn parse_hubs(root: &tree_sitter::Node, source: &[u8]) -> Vec<HubgsDefinition> {
    let mut definitions = Vec::new();
    for child in root.named_children(&mut root.walk()) {
        if child.kind() != "definitions_section" {
            continue;
        }
        let walker = &mut child.walk();
        for section_child in child.named_children(walker) {
            if section_child.kind() != "hubs_block" {
                continue;
            }
            let hb_walker = &mut section_child.walk();
            for hub_node in section_child.named_children(hb_walker) {
                if hub_node.kind() != "hub_definition" {
                    continue;
                }
                let mut name = String::new();
                let mut links = Vec::new();
                let mut parents = Vec::new();
                let mut identifier_count = 0usize;

                let field_walker = &mut hub_node.walk();
                for field_child in hub_node.named_children(field_walker) {
                    match field_child.kind() {
                        "identifier" => {
                            identifier_count += 1;
                            if identifier_count == 1 && name.is_empty() {
                                if let Some(text) = node_text(field_child, source) {
                                    name = text;
                                }
                            }
                        }
                        "extension_clause" => {
                            for parent_id in field_child.named_children(&mut field_child.walk()) {
                                if parent_id.kind() == "identifier" {
                                    if let Some(text) = node_text(parent_id, source) {
                                        parents.push(text);
                                    }
                                }
                            }
                        }
                        "hub_role" => {
                            if let Some(link) = parse_hub_role(&field_child, source) {
                                links.push(link);
                            }
                        }
                        _ => {}
                    }
                }

                if !name.is_empty() {
                    definitions.push(HubgsDefinition {
                        name,
                        links,
                        parents,
                    });
                }
            }
        }
    }

    let by_name: HashMap<String, usize> = definitions
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name.clone(), i))
        .collect();
    let mut inherited: Vec<(usize, Vec<HubgsLink>)> = Vec::new();
    for (i, def) in definitions.iter().enumerate() {
        let mut extra = Vec::new();
        for parent_name in &def.parents {
            if let Some(&pidx) = by_name.get(parent_name.as_str()) {
                extra.extend(definitions[pidx].links.iter().cloned());
            }
        }
        if !extra.is_empty() {
            inherited.push((i, extra));
        }
    }
    for (i, extra) in inherited {
        definitions[i].links.splice(0..0, extra);
    }

    definitions
}

fn parse_hub_role(node: &tree_sitter::Node, source: &[u8]) -> Option<HubgsLink> {
    let mut role_name = String::new();
    let mut arrow = String::new();
    let mut multiplicity = String::new();
    let mut identifier_count = 0usize;
    let mut targets: Vec<String> = Vec::new();

    for child in node.named_children(&mut node.walk()) {
        match child.kind() {
            "identifier" => {
                identifier_count += 1;
                if let Some(text) = node_text(child, source) {
                    if identifier_count == 1 {
                        role_name = text;
                    } else {
                        targets.push(text);
                    }
                }
            }
            "role_direction" => {
                if let Some(text) = node_text(child, source) {
                    arrow = text.trim().to_string();
                }
            }
            "multiplicity" => {
                if let Some(text) = node_text(child, source) {
                    multiplicity = text.trim().to_string();
                }
            }
            _ => {}
        }
    }

    if role_name.is_empty() || arrow.is_empty() || multiplicity.is_empty() {
        return None;
    }

    let target_str = if targets.len() > 1 {
        targets.join(",")
    } else if targets.len() == 1 {
        targets.remove(0)
    } else {
        String::new()
    };

    Some(HubgsLink {
        name: role_name,
        arrow,
        target: target_str,
        multiplicity,
    })
}

fn parse_instances(
    root: &tree_sitter::Node,
    source: &[u8],
    relations_by_type: &HashMap<&str, HashSet<&str>>,
) -> Vec<HubgsInstance> {
    let mut instances = Vec::new();

    for child in root.named_children(&mut root.walk()) {
        if child.kind() != "instances_section" {
            continue;
        }
        let mut blocks: Vec<tree_sitter::Node> = Vec::new();
        {
            let walker = &mut child.walk();
            for block in child.named_children(walker) {
                if block.kind() == "instance_block" {
                    blocks.push(block);
                }
            }
        }

        for block in blocks {
            let mut id = String::new();
            let mut type_name = String::new();
            let mut name = String::new();
            let mut theme_color: Option<u32> = None;
            let mut links = Vec::new();

            if let Some(ref_node) = block.child_by_field_name("ref") {
                if let Some(text) = node_text(ref_node, source) {
                    id = text;
                }
            }
            if let Some(type_node) = block.child_by_field_name("type") {
                if let Some(text) = node_text(type_node, source) {
                    type_name = text;
                }
            }

            let mut known_relations = relations_by_type
                .get(type_name.as_str())
                .cloned()
                .unwrap_or_default();

            {
                let walker = &mut block.walk();
                for assignment in block.named_children(walker) {
                    if assignment.kind() == "instance_assignment" {
                        parse_instance_assignment(
                            &assignment,
                            source,
                            &mut name,
                            &mut theme_color,
                            &mut links,
                            &mut known_relations,
                        );
                    }
                }
            }

            if !id.is_empty() && !type_name.is_empty() {
                let final_name = if name.is_empty() { id.clone() } else { name };
                instances.push(HubgsInstance {
                    id,
                    type_name,
                    name: final_name,
                    theme_color,
                    links,
                });
            }
        }
    }
    instances
}

fn parse_instance_assignment(
    node: &tree_sitter::Node,
    source: &[u8],
    name: &mut String,
    theme_color: &mut Option<u32>,
    links: &mut Vec<InstanceLink>,
    known_relations: &mut HashSet<&str>,
) {
    let mut key = String::new();

    let mut children: Vec<(String, String)> = Vec::new();
    for child in node.named_children(&mut node.walk()) {
        if let Some(text) = node_text(child, source) {
            children.push((child.kind().to_string(), text));
        }
    }

    for (kind, text) in &children {
        if kind == "identifier" && key.is_empty() {
            key = text.clone();
            break;
        }
    }

    if key.is_empty() {
        return;
    }

    let is_name_assignment = key == "name";

    if let Some(expr_node) = node.named_child(1) {
        if let Some(expr_text) = node_text(expr_node, source) {
            let expr_text = expr_text.trim().to_string();
            if is_name_assignment {
                *name = unquote_string(&expr_text);
            } else if expr_text.starts_with("0x") || expr_text.starts_with("0X") {
                let clean = expr_text.trim_start_matches("0x").trim_start_matches("0X");
                if let Ok(val) = u32::from_str_radix(clean, 16) {
                    *theme_color = Some(val);
                }
            } else if (expr_text.starts_with('[') && expr_text.ends_with(']'))
                && known_relations.contains(key.as_str())
            {
                let inner = &expr_text[1..expr_text.len() - 1];
                for t in inner.split(',') {
                    let target = t.trim().to_string();
                    if !target.is_empty() {
                        links.push(InstanceLink {
                            relation: key.clone(),
                            target,
                        });
                    }
                }
            }
        }
    }
}

fn parse_definitions_ast(
    root: &tree_sitter::Node,
    contents: &str,
    global_fields_ast: &mut Vec<GlobalFieldAst>,
    global_fields: &mut Vec<GlobalField>,
    enums_ast: &mut Vec<HubEnumAst>,
    enums: &mut Vec<HubEnum>,
    structs_ast: &mut Vec<HubStructAst>,
    structs: &mut Vec<HubStruct>,
    types_ast: &mut Vec<HubTypeAst>,
) {
    let mut def_cursor = root.walk();
    for section in root.children(&mut def_cursor) {
        if section.kind() == "definitions_section" {
            let mut sec_cursor = section.walk();
            for block in section.children(&mut sec_cursor) {
                match block.kind() {
                    "fields_block" => {
                        let mut cursor = block.walk();
                        for child in block.children(&mut cursor) {
                            if child.kind() == "field_definition" {
                                if let (Some(id_node), Some(type_node)) = (child.child(0), child.child(2)) {
                                    let id = contents[id_node.byte_range()].to_string();
                                    let type_str = contents[type_node.byte_range()].to_string();
                                    if !id.is_empty() && !type_str.is_empty() {
                                        global_fields.push(GlobalField {
                                            name: id.clone(),
                                            type_name: type_str.clone(),
                                        });
                                        global_fields_ast.push(GlobalFieldAst {
                                            name: id,
                                            type_name: type_str,
                                            range: ts_range_to_span(id_node.range()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    "enums_block" => {
                        let mut cursor = block.walk();
                        for child in block.children(&mut cursor) {
                            if child.kind() == "enum_definition" {
                                if let Some(name_node) = child.child(0) {
                                    let name = contents[name_node.byte_range()].to_string();
                                    if !name.is_empty() {
                                        let mut variants = Vec::new();
                                        let mut var_cursor = child.walk();
                                        for v_node in child.children(&mut var_cursor) {
                                            if v_node.kind() == "identifier" && v_node.id() != name_node.id() {
                                                let v_name = contents[v_node.byte_range()].to_string();
                                                if !v_name.is_empty() {
                                                    variants.push(v_name);
                                                }
                                            }
                                        }
                                        enums.push(HubEnum {
                                            name: name.clone(),
                                            variants: variants.clone(),
                                        });
                                        enums_ast.push(HubEnumAst {
                                            name,
                                            variants,
                                            range: ts_range_to_span(name_node.range()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    "structs_block" => {
                        let mut cursor = block.walk();
                        for child in block.children(&mut cursor) {
                            if child.kind() == "struct_definition" {
                                if let Some(name_node) = child.child(0) {
                                    let name = contents[name_node.byte_range()].to_string();
                                    if !name.is_empty() {
                                        let mut field_names = Vec::new();
                                        let mut f_cursor = child.walk();
                                        for f_node in child.children(&mut f_cursor) {
                                            if f_node.kind() == "identifier" && f_node.id() != name_node.id() {
                                                let f_name = contents[f_node.byte_range()].to_string();
                                                if !f_name.is_empty() {
                                                    field_names.push(f_name);
                                                }
                                            }
                                        }
                                        structs.push(HubStruct {
                                            name: name.clone(),
                                            field_names: field_names.clone(),
                                        });
                                        structs_ast.push(HubStructAst {
                                            name,
                                            field_names,
                                            range: ts_range_to_span(name_node.range()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    "hubs_block" => {
                        let mut cursor = block.walk();
                        for hub_def in block.children(&mut cursor) {
                            if hub_def.kind() == "hub_definition" {
                                let name_node = match hub_def.child(0) {
                                    Some(n) if !n.is_missing() => n,
                                    _ => continue,
                                };
                                let name = contents[name_node.byte_range()].to_string();
                                if name.is_empty() {
                                    continue;
                                }

                                let mut fields = Vec::new();
                                let mut roles = Vec::new();
                                let mut extends_parents = Vec::new();
                                let mut constraints = Vec::new();

                                let mut body_cursor = hub_def.walk();
                                for item in hub_def.children(&mut body_cursor) {
                                    match item.kind() {
                                        "extension_clause" => {
                                            let mut ext_cursor = item.walk();
                                            for p_node in item.children(&mut ext_cursor) {
                                                if p_node.kind() == "identifier" {
                                                    extends_parents.push(contents[p_node.byte_range()].to_string());
                                                }
                                            }
                                        }
                                        "hub_field" => {
                                            if let Some(id_node) = item.child(0) {
                                                let f_name = contents[id_node.byte_range()].to_string();
                                                if !f_name.is_empty() {
                                                    let mut is_disp = false;
                                                    let mut is_bg = false;
                                                    let mut attr_cursor = item.walk();
                                                    for child in item.children(&mut attr_cursor) {
                                                        if child.kind() == "field_attribute" {
                                                            let attr_str = &contents[child.byte_range()];
                                                            if attr_str.contains("display") {
                                                                is_disp = true;
                                                            }
                                                            if attr_str.contains("background") {
                                                                is_bg = true;
                                                            }
                                                        }
                                                    }
                                                    let (dec, expr) = parse_field_decorators(&item, contents);
                                                    fields.push(HubFieldDef {
                                                        name: f_name,
                                                        decorator: dec,
                                                        expression: expr,
                                                        is_display: is_disp,
                                                        is_background: is_bg,
                                                        range: ts_range_to_span(id_node.range()),
                                                    });
                                                }
                                            }
                                        }
                                        "hub_role" => {
                                            if let Some(role_def) = parse_hub_role_ast(&item, contents) {
                                                roles.push(role_def);
                                            }
                                        }
                                        "constraints_block" => {
                                            let mut c_cursor = item.walk();
                                            for c_child in item.children(&mut c_cursor) {
                                                if c_child.kind() != "@constraints" && c_child.kind() != "[" && c_child.kind() != "]" && c_child.kind() != "," {
                                                    let c_str = contents[c_child.byte_range()].to_string();
                                                    if !c_str.is_empty() {
                                                        constraints.push(c_str);
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                types_ast.push(HubTypeAst {
                                    name,
                                    range: ts_range_to_span(name_node.range()),
                                    block_range: ts_range_to_span(hub_def.range()),
                                    fields,
                                    roles,
                                    extends_parents,
                                    constraints,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn parse_field_decorators(item: &tree_sitter::Node, contents: &str) -> (Option<String>, Option<String>) {
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

fn parse_hub_role_ast(item: &tree_sitter::Node, contents: &str) -> Option<HubRoleDef> {
    let id_node = item.child(0)?;
    let role_name = contents[id_node.byte_range()].to_string();
    let direction = item.child(1).map(|n| contents[n.byte_range()].to_string()).unwrap_or_default();
    let multiplicity = item.child(3).map(|n| contents[n.byte_range()].to_string()).unwrap_or_default();

    let mut allowed_types = Vec::new();
    let mut list_cursor = item.walk();
    for child in item.children(&mut list_cursor) {
        if child.kind() == "identifier" && child.id() != id_node.id() {
            allowed_types.push(contents[child.byte_range()].to_string());
        }
    }

    Some(HubRoleDef {
        name: role_name,
        direction,
        multiplicity,
        allowed_types,
    })
}

fn parse_instances_ast(
    root: &tree_sitter::Node,
    contents: &str,
    instances_ast: &mut Vec<HubInstanceAst>,
) {
    let mut section_cursor = root.walk();
    for section in root.children(&mut section_cursor) {
        if section.kind() == "instances_section" {
            let mut block_cursor = section.walk();
            for child in section.children(&mut block_cursor) {
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
                    let mut b_cursor = child.walk();
                    for assignment in child.children(&mut b_cursor) {
                        if assignment.kind() == "instance_assignment" {
                            if let Some(id_node) = assignment.child(0) {
                                let attr_name = contents[id_node.byte_range()].to_string();
                                if !attr_name.is_empty() && !id_node.is_missing() {
                                    if let Some(expr_node) = assignment.child(2) {
                                        if let Some(val) = node_to_hub_value(expr_node, contents) {
                                            assignments.push(HubAssignmentAst {
                                                name: attr_name,
                                                range: ts_range_to_span(id_node.range()),
                                                value: val,
                                                value_range: ts_range_to_span(expr_node.range()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let description = assignments
                        .iter()
                        .find(|a| a.name == "description")
                        .and_then(|a| match &a.value {
                            HubValue::Text(s) => Some(s.clone()),
                            _ => None,
                        });

                    instances_ast.push(HubInstanceAst {
                        id: name,
                        type_name,
                        name_range: ts_range_to_span(ref_node.range()),
                        block_range: ts_range_to_span(child.range()),
                        description,
                        assignments,
                    });
                }
            }
        }
    }
}

/// Parse HubGS source text into definitions and instances.
pub fn parse_hubgs(
    content: &str,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let output = parse_hubgs_full(content)?;
    let language = load_hubgs_language()
        .ok_or_else(|| anyhow::anyhow!("HubGS tree-sitter grammar not linked"))?;
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_ok() {
        if let Some(tree) = parser.parse(content, None) {
            if tree.root_node().has_error() {
                anyhow::bail!("HubGS parse had error node(s)");
            }
        }
    }
    Ok((output.definitions, output.instances))
}

/// Parse HubGS source text into full AST parse output structure.
pub fn parse_hubgs_full(content: &str) -> anyhow::Result<HubgsParseOutput> {
    let source = content.as_bytes();

    let language = load_hubgs_language()
        .ok_or_else(|| anyhow::anyhow!("HubGS tree-sitter grammar not linked"))?;

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| anyhow::anyhow!("Failed to set HubGS language: {e}"))?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter returned an empty parse tree"))?;

    let root = tree.root_node();

    let imports = parse_imports(&root, source);
    let definitions = parse_hubs(&root, source);

    let relations_by_type: HashMap<&str, HashSet<&str>> = definitions
        .iter()
        .map(|d| (d.name.as_str(), d.links.iter().map(|l| l.name.as_str()).collect()))
        .collect();

    let instances = parse_instances(&root, source, &relations_by_type);

    let mut global_fields_ast = Vec::new();
    let mut global_fields = Vec::new();
    let mut enums_ast = Vec::new();
    let mut enums = Vec::new();
    let mut structs_ast = Vec::new();
    let mut structs = Vec::new();
    let mut types_ast = Vec::new();
    let mut instances_ast = Vec::new();

    parse_definitions_ast(
        &root,
        content,
        &mut global_fields_ast,
        &mut global_fields,
        &mut enums_ast,
        &mut enums,
        &mut structs_ast,
        &mut structs,
        &mut types_ast,
    );

    parse_instances_ast(&root, content, &mut instances_ast);

    Ok(HubgsParseOutput {
        definitions,
        instances,
        imports,
        enums,
        structs,
        global_fields,
        types_ast,
        instances_ast,
        enums_ast,
        structs_ast,
        global_fields_ast,
    })
}
