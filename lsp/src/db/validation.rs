use super::polymorphic::{hub_type_all_fields, hub_type_all_roles, hub_type_allows};
use super::resolution::*;
use super::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationError {
    pub range: super::LspRange,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Multiplicity {
    Exact(u32),
    Range(u32, Option<u32>), // min, max (None means *)
}

impl Multiplicity {
    pub fn parse(s: &str) -> Self {
        let s = s.trim_matches('(').trim_matches(')');
        if s == "*" {
            return Multiplicity::Range(1, None);
        }
        if let Ok(val) = s.parse::<u32>() {
            return Multiplicity::Exact(val);
        }
        if s.contains("..") {
            let parts: Vec<&str> = s.split("..").collect();
            let min = parts[0].parse::<u32>().unwrap_or(0);
            let max = if parts[1] == "*" {
                None
            } else {
                Some(parts[1].parse::<u32>().unwrap_or(0))
            };
            return Multiplicity::Range(min, max);
        }
        Multiplicity::Range(0, None)
    }

    pub fn validate(&self, count: usize) -> bool {
        let count = count as u32;
        match self {
            Multiplicity::Exact(val) => count == *val,
            Multiplicity::Range(min, max) => {
                if count < *min {
                    return false;
                }
                if let Some(max_val) = max {
                    if count > *max_val {
                        return false;
                    }
                }
                true
            }
        }
    }
}

const VALID_TWXML_TAGS: &[&str] = &[
    "document",
    "body",
    "meta",
    "section",
    "heading",
    "paragraph",
    "aside",
    "blockquote",
    "codeblock",
    "br",
    "hr",
    "ul",
    "ol",
    "li",
    "dl",
    "dt",
    "dd",
    "details",
    "summary",
    "hubref",
    "link",
    "image",
    "audio",
    "video",
    "code",
    "fr",
    "bold",
    "italic",
    "underline",
    "strikethrough",
    "super",
    "sub",
    "table",
    "tr",
    "th",
    "td",
    "footnote",
    "review",
    "include",
];

#[salsa::tracked]
pub fn validate_file(db: &dyn Db, workspace: Workspace, file: SourceFile) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if file.path(db).ends_with(".twxml") {
        validate_twxml(db, workspace, file, &mut errors);
    } else if file.path(db).ends_with(".hubgs") {
        validate_hubgs(db, workspace, file, &mut errors);
    }

    errors
}

fn validate_twxml(
    db: &dyn Db,
    workspace: Workspace,
    file: SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let tags = all_twxml_tags(db, file);
    let contents = file.contents(db);

    // 0. Validate document skeleton: <meta /> tags are optional; exactly one <body> required
    let body_count = tags.iter().filter(|t| t.name(db) == "body").count();

    if body_count == 0 {
        errors.push(ValidationError {
            range: super::LspRange {
                start: super::LspPosition {
                    line: 0,
                    character: 0,
                },
                end: super::LspPosition {
                    line: 0,
                    character: 0,
                },
            },
            message: "Document missing required <body> block".to_string(),
        });
    } else if body_count > 1 {
        for tag in tags.iter().filter(|t| t.name(db) == "body") {
            errors.push(ValidationError {
                range: tag.range(db),
                message: "Duplicate <body> block — document must contain exactly one".to_string(),
            });
        }
    }
    // 1. Validate Hub References
    let refs = parse_twxml(db, file);
    for r in refs {
        let name = r.name(db);
        if let Some(instance) = resolve_reference(db, workspace, name.clone()) {
            if let Some(ref field_name) = r.field(db) {
                let type_name = instance.type_name(db);
                if let Some(hub_type) =
                    resolve_type(db, workspace, instance.file(db), type_name.clone())
                {
                    let is_field = hub_type.fields(db).iter().any(|f| &f.name == field_name);
                    let is_role = hub_type.roles(db).iter().any(|r| &r.name == field_name);
                    if !is_field && !is_role {
                        errors.push(ValidationError {
                            range: r.range(db),
                            message: format!(
                                "Unknown field '{}' for type '{}'",
                                field_name, type_name
                            ),
                        });
                    } else if let Some(ref text_val) = r.text(db) {
                        if let Some(eval_val) =
                            compute_field_value(db, workspace, instance, field_name.clone())
                        {
                            let canonical_str = value_to_canonical(eval_val);
                            if canonical_str != *text_val {
                                errors.push(ValidationError {
                                    range: r.range(db),
                                    message: format!(
                                        "Out of sync: expected '{}', found '{}'",
                                        canonical_str, text_val
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        } else {
            errors.push(ValidationError {
                range: r.range(db),
                message: format!("Hub reference '{}' not found", name),
            });
        }
    }

    // 2. Validate Tag Names
    for tag in tags.iter() {
        if !VALID_TWXML_TAGS.contains(&tag.name(db).as_str()) {
            let message = if tag.name(db) == "metadata" {
                "Unknown TWXML tag 'metadata'. Did you mean '<meta />' at the document root?".to_string()
            } else {
                format!("Unknown TWXML tag '{}'", tag.name(db))
            };
            errors.push(ValidationError {
                range: tag.range(db),
                message,
            });
        }

        // Validate nesting rules for 'heading'
        if tag.name(db) == "heading" {
            if let Some(parent_name) = tag.parent_name(db) {
                if parent_name != "section" && parent_name != "document" && parent_name != "body" {
                    errors.push(ValidationError {
                        range: tag.range(db),
                        message: format!(
                            "Invalid nesting: tag '{}' is not allowed as a child of '{}'",
                            tag.name(db),
                            parent_name
                        ),
                    });
                }
            }
        }
    }

    // 3. Validate matching start/end tags, '<include />' tags, and '<meta />' tags via AST
    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).ok();

    if let Some(tree) = parser.parse(&contents, None) {
        let mut stack = vec![tree.root_node()];
        let mut metas = Vec::new();

        while let Some(node) = stack.pop() {
            match node.kind() {
                "meta_tag" => {
                    metas.push(node);
                }
                "element" => {
                    // Child 0 is start_tag, the last child is end_tag
                    if let (Some(start_tag), Some(end_tag)) =
                        (node.child(0), node.child(node.child_count() - 1))
                    {
                        if start_tag.kind() == "start_tag" && end_tag.kind() == "end_tag" {
                            if let (Some(start_name_node), Some(end_name_node)) = (
                                start_tag.child_by_field_name("name"),
                                end_tag.child_by_field_name("name"),
                            ) {
                                let start_name = &contents[start_name_node.byte_range()];
                                let end_name = &contents[end_name_node.byte_range()];

                                if start_name != end_name {
                                    errors.push(ValidationError {
                                        range: crate::parser::ts_range_to_lsp(end_tag.range()),
                                        message: format!(
                                            "Mismatched closing tag. Expected `</{}>`",
                                            start_name
                                        ),
                                    });
                                }
                                if start_name == "include" {
                                    errors.push(ValidationError {
                                        range: crate::parser::ts_range_to_lsp(node.range()),
                                        message: "Invalid include: tag 'include' must be self-closing".to_string(),
                                    });
                                }
                                if start_name == "meta" {
                                    metas.push(node);
                                }
                            }
                        }
                    }
                }
                "self_closing_element" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &contents[name_node.byte_range()];
                        if name == "include" {
                            if !has_attribute(&node, &contents, "src") {
                                errors.push(ValidationError {
                                    range: crate::parser::ts_range_to_lsp(node.range()),
                                    message: "Invalid include: tag 'include' must have a non-empty 'src' attribute".to_string(),
                                });
                            }
                        }
                        if name == "meta" {
                            metas.push(node);
                        }
                    }
                }
                _ => {}
            }

            // Continue walking the tree
            let mut child_cursor = node.walk();
            for child in node.children(&mut child_cursor) {
                stack.push(child);
            }
        }

        // Validate `<meta />` nesting and positioning relative to `<body>`
        let body_offset = contents.find("<body>");
        for meta in &metas {
            if let Some(body_off) = body_offset {
                let meta_pos = crate::parser::ts_range_to_lsp(meta.range()).start;
                let meta_off = lsp_pos_to_byte_offset(&contents, meta_pos);
                if meta_off >= body_off {
                    errors.push(ValidationError {
                        range: crate::parser::ts_range_to_lsp(meta.range()),
                        message: "Invalid positioning: tag 'meta' must precede the <body> block".to_string(),
                    });
                }
            }
        }
        for meta in metas {
            if let Some(parent) = meta.parent() {
                if parent.kind() != "document_block" {
                    errors.push(ValidationError {
                        range: crate::parser::ts_range_to_lsp(meta.range()),
                        message: "Invalid nesting: tag 'meta' is only allowed as a direct child of 'document'".to_string(),
                    });
                }
            }
        }
    }
    validate_links(db, workspace, file, errors);
}

fn anchor_exists(contents: &str, anchor: &str) -> bool {
    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).ok();
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return false,
    };

    fn walk(node: tree_sitter::Node, contents: &str, anchor: &str) -> bool {
        if node.kind() == "start_tag" || node.kind() == "self_closing_element" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "attribute" {
                    if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                        let attr_name = &contents[name_node.byte_range()];
                        let attr_val = contents[val_node.byte_range()]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        if (attr_name == "id" || attr_name == "alias" || attr_name == "class") && attr_val == anchor {
                            return true;
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if walk(child, contents, anchor) {
                return true;
            }
        }
        false
    }

    walk(tree.root_node(), contents, anchor)
}

fn validate_links(
    db: &dyn Db,
    workspace: Workspace,
    file: SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let contents = file.contents(db);
    let current_path_str = file.path(db);
    let current_path = std::path::Path::new(&current_path_str);
    let parent_path = current_path.parent();

    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).ok();
    let tree = match parser.parse(&contents, None) {
        Some(t) => t,
        None => return,
    };

    fn walk_links(
        node: tree_sitter::Node,
        contents: &str,
        parent_path: Option<&std::path::Path>,
        db: &dyn Db,
        workspace: Workspace,
        current_file: SourceFile,
        errors: &mut Vec<ValidationError>,
    ) {
        if node.kind() == "start_tag" || node.kind() == "self_closing_element" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let tag_name = &contents[name_node.byte_range()];
                if tag_name == "link" {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "attribute" {
                            if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                                let attr_name = &contents[name_node.byte_range()];
                                if attr_name == "href" {
                                    let href = contents[val_node.byte_range()]
                                        .trim_matches('"')
                                        .trim_matches('\'')
                                        .to_string();

                                    if href.starts_with("http://") || href.starts_with("https://") {
                                        continue;
                                    }

                                    let parts: Vec<&str> = href.split('#').collect();
                                    let (target_file_path, anchor_id) = if parts.len() == 2 {
                                        (if parts[0].is_empty() { None } else { Some(parts[0]) }, Some(parts[1]))
                                    } else if href.starts_with('#') {
                                        (None, Some(&href[1..]))
                                    } else {
                                        (Some(href.as_str()), None)
                                    };

                                    let target_file = if let Some(path) = target_file_path {
                                        if let Some(parent) = parent_path {
                                            let full_path = parent.join(path);
                                            let full_path_str = full_path.to_string_lossy().to_string();
                                            workspace.files(db).into_iter().find(|f| f.path(db) == full_path_str)
                                        } else {
                                            None
                                        }
                                    } else {
                                        Some(current_file)
                                    };

                                    match target_file {
                                        None => {
                                            errors.push(ValidationError {
                                                range: crate::parser::ts_range_to_lsp(val_node.range()),
                                                message: format!("Target file '{}' not found in workspace", target_file_path.unwrap()),
                                            });
                                        }
                                        Some(tgt) => {
                                            if let Some(anchor) = anchor_id {
                                                let tgt_contents = tgt.contents(db);
                                                if !anchor_exists(&tgt_contents, anchor) {
                                                    errors.push(ValidationError {
                                                        range: crate::parser::ts_range_to_lsp(val_node.range()),
                                                        message: format!("Anchor '#{}' not found in target file", anchor),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk_links(child, contents, parent_path, db, workspace, current_file, errors);
        }
    }

    walk_links(tree.root_node(), &contents, parent_path, db, workspace, file, errors);
}

fn validate_hubgs(
    db: &dyn Db,
    workspace: Workspace,
    file: SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let result = parse_hubgs(db, file);

    // 0. Guard: orphaned instances (instances without definitions or imports)
    if !result.instances(db).is_empty()
        && result.types(db).is_empty()
        && result.imports(db).is_empty()
    {
        for instance in result.instances(db) {
            errors.push(ValidationError {
                range: instance.range(db),
                message: format!(
                    "Instance '{}' uses type '{}' but no definitions or imports are present",
                    instance.name(db),
                    instance.type_name(db)
                ),
            });
        }
    }

    let global_fields = all_global_fields(db, workspace);

    // 1. Validate Hub Type Definitions
    for hub_type in result.types(db) {
        for field in hub_type.fields(db) {
            if !global_fields.iter().any(|gf| gf.name(db) == field.name) {
                errors.push(ValidationError {
                    range: field.range,
                    message: format!(
                        "Field '{}' used in Hub '{}' must be defined in a FIELDS block",
                        field.name,
                        hub_type.name(db)
                    ),
                });
            }
        }
    }

    // 2. Validate Hub Instances
    for instance in result.instances(db) {
        let type_name = instance.type_name(db);
        if let Some(hub_type) = resolve_type(db, workspace, file, type_name.clone()) {
            validate_instance_assignments(
                db,
                workspace,
                &instance,
                hub_type,
                &global_fields,
                errors,
            );

            // 3. Missing required roles (minimum multiplicity > 0)
            check_missing_roles(
                db,
                workspace,
                &instance,
                hub_type,
                type_name.clone(),
                errors,
            );
        } else {
            errors.push(ValidationError {
                range: instance.range(db),
                message: format!("Unknown Hub type '{}'", type_name),
            });
        }
    }
}

fn validate_instance_assignments(
    db: &dyn Db,
    workspace: Workspace,
    instance: &HubInstance<'_>,
    hub_type: HubType<'_>,
    global_fields: &[GlobalField<'_>],
    errors: &mut Vec<ValidationError>,
) {
    for assignment in instance.assignments(db) {
        let name = &assignment.name;
        // ponytail: Use polymorphic field/role lookup to respect EXTENDS inheritance
        let all_fields = hub_type_all_fields(db, workspace, &hub_type);
        let all_roles = hub_type_all_roles(db, workspace, &hub_type);
        let is_field = all_fields.iter().any(|f| f.name.as_str() == name.as_str());
        let role_def = all_roles.iter().find(|r| r.name.as_str() == name.as_str());

        if !is_field && role_def.is_none() {
            errors.push(ValidationError {
                range: assignment.range,
                message: format!(
                    "Unknown field or role '{}' for type '{}'",
                    name,
                    instance.type_name(db)
                ),
            });
            continue;
        }

        if let Some(role_def) = role_def {
            validate_role_assignment(
                db,
                workspace,
                &assignment.value,
                role_def,
                assignment.range,
                errors,
            );
        } else if is_field {
            // Type checking for primitive fields
            if let Some(gf) = global_fields.iter().find(|gf| gf.name(db) == *name) {
                let expected_type = gf.type_name(db);
                if !validate_value_type(db, workspace, &assignment.value, &expected_type) {
                    errors.push(ValidationError {
                        range: assignment.range,
                        message: format!(
                            "Type mismatch for field '{}': expected '{}'",
                            name, expected_type
                        ),
                    });
                }
            }
        }
    }
}

fn validate_role_assignment(
    db: &dyn Db,
    workspace: Workspace,
    value: &HubValue,
    role_def: &HubRoleDef,
    assignment_range: super::LspRange,
    errors: &mut Vec<ValidationError>,
) {
    let refs = get_refs_from_value(value);

    // 1. Type mismatch validation (polymorphic: checks extends_parents chain)
    for ref_name in &refs {
        if let Some(target_inst) = resolve_reference(db, workspace, ref_name.clone()) {
            let target_type_name = target_inst.type_name(db);
            let hub_type = match resolve_type(
                db,
                workspace,
                target_inst.file(db),
                target_type_name.clone(),
            ) {
                Some(t) => t,
                None => continue,
            };

            if !hub_type_allows(db, workspace, &hub_type, &role_def.allowed_types) {
                errors.push(ValidationError {
                    range: assignment_range,
                    message: format!(
                        "Type mismatch: Role '{}' does not allow type '{}'",
                        role_def.name, target_type_name
                    ),
                });
            }
        } else {
            errors.push(ValidationError {
                range: assignment_range,
                message: format!("Hub reference '{}' not found", ref_name),
            });
        }
    }

    // 2. Multiplicity validation
    let mult = Multiplicity::parse(&role_def.multiplicity);
    if !mult.validate(refs.len()) {
        errors.push(ValidationError {
            range: assignment_range,
            message: format!(
                "Multiplicity violation for role '{}': expected {}, found {}",
                role_def.name,
                role_def.multiplicity,
                refs.len()
            ),
        });
    }
}

fn check_missing_roles(
    db: &dyn Db,
    workspace: Workspace,
    instance: &HubInstance<'_>,
    hub_type: HubType<'_>,
    type_name: String,
    errors: &mut Vec<ValidationError>,
) {
    // ponytail: Use polymorphic role lookup to respect EXTENDS inheritance
    let all_roles = hub_type_all_roles(db, workspace, &hub_type);
    for role_def in all_roles {
        let mult = Multiplicity::parse(&role_def.multiplicity);
        let min_required = match mult {
            Multiplicity::Range(min, _) => min > 0,
            Multiplicity::Exact(val) => val > 0,
        };

        if min_required {
            let is_assigned = instance
                .assignments(db)
                .iter()
                .any(|a| a.name == role_def.name);
            if !is_assigned {
                errors.push(ValidationError {
                    range: instance.range(db),
                    message: format!(
                        "Missing required role '{}' for type '{}'",
                        role_def.name, type_name
                    ),
                });
            }
        }
    }
}

fn get_refs_from_value(value: &HubValue) -> Vec<String> {
    match value {
        HubValue::Identifier(s) => vec![s.clone()],
        HubValue::Array(vals) => vals.iter().flat_map(get_refs_from_value).collect(),
        _ => Vec::new(),
    }
}

fn validate_value_type(
    db: &dyn Db,
    workspace: Workspace,
    value: &HubValue,
    type_name: &str,
) -> bool {
    match type_name {
        "Text" | "String" => matches!(value, HubValue::String(_)),
        "Number" => matches!(value, HubValue::Number(_)),
        "Boolean" => matches!(value, HubValue::Boolean(_)),
        "Array<Text>" | "Array<String>" => {
            if let HubValue::Array(vals) = value {
                vals.iter().all(|v| matches!(v, HubValue::String(_)))
            } else {
                false
            }
        }
        "Array<Number>" => {
            if let HubValue::Array(vals) = value {
                vals.iter().all(|v| matches!(v, HubValue::Number(_)))
            } else {
                false
            }
        }
        _ => {
            // check Enums
            if let Some(hub_enum) = all_enums(db, workspace)
                .into_iter()
                .find(|e| e.name(db) == type_name)
            {
                if let HubValue::Identifier(s) = value {
                    return hub_enum.variants(db).contains(s);
                }
                return false;
            }
            true // Default to true for now for complex types (Structs, etc)
        }
    }
}

fn value_to_canonical(val: HubValue) -> String {
    val.to_string()
}

fn has_attribute(node: &tree_sitter::Node, contents: &str, attr_name: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let Some(name_child) = child.child(0) {
                let name = &contents[name_child.byte_range()];
                if name == attr_name {
                    if let Some(val_child) = child.child(2) {
                        let val = contents[val_child.byte_range()]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .trim();
                        return !val.is_empty();
                    }
                }
            }
        }
    }
    false
}

fn lsp_pos_to_byte_offset(contents: &str, pos: super::LspPosition) -> usize {
    let mut offset = 0;
    for (i, line) in contents.lines().enumerate() {
        if i == pos.line as usize {
            offset += pos.character as usize;
            break;
        }
        offset += line.len() + 1;
    }
    offset
}
