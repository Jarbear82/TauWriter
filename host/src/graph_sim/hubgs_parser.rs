use crate::graph_sim::{HubgsDefinition, HubgsInstance, HubgsLink, InstanceLink};
use gpui::SharedString;

/// Extract text from a tree-sitter node given its byte range and source bytes.
fn node_text(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(|s| s.to_string())
}

/// Strip surrounding quotes from a string literal (single or double),
/// decoding escapes via a single left-to-right scan so escape sequences
/// are never re-interpreted by a later replacement pass.
fn unquote_string(s: &str) -> String {
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
                // Unknown escape: preserve literally (backslash + char).
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'), // trailing backslash
        }
    }
    out
}

/// Recursively count error/missing nodes in a tree-sitter subtree.
const MAX_RECURSION_DEPTH: usize = 200;

fn count_errors(node: tree_sitter::Node) -> usize {
    count_errors_impl(node, 0)
}

fn count_errors_impl(node: tree_sitter::Node, depth: usize) -> usize {
    if depth > MAX_RECURSION_DEPTH {
        return 0; // bail out rather than overflow the stack on pathological input
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

/// Parse the imports_section if present. Returns list of (identifiers, source) tuples.
fn parse_imports(root: &tree_sitter::Node, source: &[u8]) -> Vec<(Vec<String>, String)> {
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
                imports.push((identifiers, source_str.unwrap()));
            }
        }
    }
    imports
}

/// Parse HUBS block and extract hub definitions with their roles/links.
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
                                        parents.push(SharedString::from(text));
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
                        name: SharedString::from(name),
                        links,
                        parents,
                    });
                }
            }
        }
    }

    // Flatten inherited links from EXTENDS parents (shallow, one pass;
    // assumes parents are declared before children, per grammar convention).
    let by_name: std::collections::HashMap<String, usize> = definitions
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name.to_string(), i))
        .collect();
    let mut inherited: Vec<(usize, Vec<HubgsLink>)> = Vec::new();
    for (i, def) in definitions.iter().enumerate() {
        let mut extra = Vec::new();
        for parent_name in &def.parents {
            if let Some(&pidx) = by_name.get(parent_name.as_ref()) {
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

/// Parse a single hub_role node into a HubgsLink.
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
        name: SharedString::from(role_name),
        arrow: SharedString::from(arrow),
        target: SharedString::from(target_str),
        multiplicity: SharedString::from(multiplicity),
    })
}

/// Parse instances_section and extract HubgsInstance values.
fn parse_instances(
    root: &tree_sitter::Node,
    source: &[u8],
    relations_by_type: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
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
                    id: SharedString::from(id),
                    type_name: SharedString::from(type_name),
                    name: SharedString::from(final_name),
                    theme_color,
                    links,
                });
            }
        }
    }
    instances
}

/// Parse an instance_assignment node (identifier = expression).
/// `known_relations` is the set of role names declared on this instance's
/// hub definition (including inherited ones) — used to decide whether an
/// assignment is a relation link rather than a scalar field.
fn parse_instance_assignment(
    node: &tree_sitter::Node,
    source: &[u8],
    name: &mut String,
    theme_color: &mut Option<u32>,
    links: &mut Vec<InstanceLink>,
    known_relations: &std::collections::HashSet<&str>,
) {
    let mut key = String::new();

    // Get all named children as (kind, text) pairs
    let mut children: Vec<(String, String)> = Vec::new();
    for child in node.named_children(&mut node.walk()) {
        if let Some(text) = node_text(child, source) {
            children.push((child.kind().to_string(), text));
        }
    }

    // First identifier is the key
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
                // Heuristic retained for color literals.
                let clean = expr_text.trim_start_matches("0x").trim_start_matches("0X");
                if let Ok(val) = u32::from_str_radix(clean, 16) {
                    *theme_color = Some(val);
                }
            } else if (expr_text.starts_with('[') && expr_text.ends_with(']'))
                && known_relations.contains(key.as_str())
            {
                let inner = &expr_text[1..expr_text.len() - 1];
                for t in inner.split(',') {
                    let cleaned = t.trim();
                    if !cleaned.is_empty() {
                        links.push(InstanceLink {
                            relation: SharedString::from(key.clone()),
                            target: SharedString::from(cleaned.to_string()),
                        });
                    }
                }
            }
        }
    }
}

pub(crate) fn parse_hubgs(
    content: &str,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let source = content.as_bytes();

    // Load the HubGS language from FFI via centralized ffi module
    let language = crate::ffi::load_hubgs_language()
        .ok_or_else(|| anyhow::anyhow!("HubGS tree-sitter grammar not linked (check build.rs)"))?;

    // Parse with tree-sitter
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| anyhow::anyhow!("Failed to set HubGS language: {e}"))?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter returned an empty parse tree"))?;

    let root = tree.root_node();

    if root.has_error() {
        let err_count = count_errors(root);
        anyhow::bail!(
            "HubGS parse had {err_count} error node(s) — the file may contain syntax errors"
        );
    }

    // Parse all sections from the AST
    let _imports = parse_imports(&root, source);
    let definitions = parse_hubs(&root, source);

    // Build type_name -> known relation names for schema-driven instance parsing.
    let relations_by_type: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
        definitions
            .iter()
            .map(|d| {
                (
                    d.name.as_ref(),
                    d.links.iter().map(|l| l.name.as_ref()).collect(),
                )
            })
            .collect();

    let instances = parse_instances(&root, source, &relations_by_type);

    Ok((definitions, instances))
}
