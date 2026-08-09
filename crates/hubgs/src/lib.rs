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
        const _: () = assert!(
            std::mem::size_of::<*const std::ffi::c_void>()
                == std::mem::size_of::<tree_sitter::Language>(),
            "tree_sitter::Language layout changed — transmute is no longer valid"
        );
        let lang = unsafe { std::mem::transmute::<*const std::ffi::c_void, tree_sitter::Language>(ptr) };
        Some(lang)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsLink {
    pub name: String,
    pub arrow: String,
    pub target: String,
    pub multiplicity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsDefinition {
    pub name: String,
    pub links: Vec<HubgsLink>,
    pub parents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InstanceLink {
    pub relation: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsInstance {
    pub id: String,
    pub type_name: String,
    pub name: String,
    pub theme_color: Option<u32>,
    pub links: Vec<InstanceLink>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubImport {
    pub types: Vec<String>,
    pub from: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubEnum {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubStruct {
    pub name: String,
    pub field_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GlobalField {
    pub name: String,
    pub type_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsParseOutput {
    pub definitions: Vec<HubgsDefinition>,
    pub instances: Vec<HubgsInstance>,
    pub imports: Vec<HubImport>,
    pub enums: Vec<HubEnum>,
    pub structs: Vec<HubStruct>,
    pub global_fields: Vec<GlobalField>,
}

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
                    let cleaned = t.trim();
                    if !cleaned.is_empty() {
                        links.push(InstanceLink {
                            relation: key.clone(),
                            target: cleaned.to_string(),
                        });
                    }
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

    if root.has_error() {
        let err_count = count_errors(root);
        anyhow::bail!(
            "HubGS parse had {err_count} error node(s) — the file may contain syntax errors"
        );
    }

    let imports = parse_imports(&root, source);
    let definitions = parse_hubs(&root, source);

    let relations_by_type: HashMap<&str, HashSet<&str>> = definitions
        .iter()
        .map(|d| (d.name.as_str(), d.links.iter().map(|l| l.name.as_str()).collect()))
        .collect();

    let instances = parse_instances(&root, source, &relations_by_type);

    Ok(HubgsParseOutput {
        definitions,
        instances,
        imports,
        enums: Vec::new(),
        structs: Vec::new(),
        global_fields: Vec::new(),
    })
}
