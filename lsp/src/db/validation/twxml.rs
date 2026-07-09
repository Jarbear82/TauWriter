use super::ValidationError;
/// TWXML structural validation rules.
use crate::db::{resolution, HubValue};

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

pub(crate) fn validate_twxml(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    file: crate::db::SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let tags = resolution::all_twxml_tags(db, file.clone());
    let contents = file.contents(db).to_string();

    // 0. Validate document skeleton: <meta /> tags are optional; exactly one <body> required
    let body_count = tags.iter().filter(|t| t.name(db) == "body").count();

    if body_count == 0 {
        errors.push(ValidationError {
            range: crate::db::LspRange {
                start: crate::db::LspPosition {
                    line: 0,
                    character: 0,
                },
                end: crate::db::LspPosition {
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
    let refs = resolution::parse_twxml(db, file.clone());
    for r in refs {
        let name = r.name(db);
        if let Some(instance) = resolution::resolve_reference(db, workspace.clone(), name.clone()) {
            if let Some(ref field_name) = r.field(db) {
                let type_name = instance.type_name(db);
                if let Some(hub_type) = resolution::resolve_type(
                    db,
                    workspace.clone(),
                    instance.file(db),
                    type_name.clone(),
                ) {
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
                        if let Ok(Some(eval_val)) = resolution::compute_field_value(
                            db,
                            workspace.clone(),
                            instance,
                            field_name.clone(),
                        ) {
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
                "Unknown TWXML tag 'metadata'. Did you mean '<meta />' at the document root?"
                    .to_string()
            } else {
                format!("Unknown TWXML tag '{}'", tag.name(db))
            };
            errors.push(ValidationError {
                range: tag.range(db),
                message,
            });
        }

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
    parser.set_language(&language).ok();

    if let Some(tree) = parser.parse(&contents, None) {
        validate_tag_structure(
            db,
            workspace.clone(),
            file.clone(),
            &tree,
            contents.into_bytes(),
            errors,
        );
    }
}

/// Extract a tag name from an element or self_closing_element node.
fn get_tag_name(node: &tree_sitter::Node, contents: &[u8]) -> String {
    if node.kind() == "element" || node.kind() == "self_closing_element" {
        let first = node.child(0);
        if let Some(start_tag) = first {
            if let Some(tag_name_node) = start_tag.child(1) {
                return String::from_utf8_lossy(&contents[tag_name_node.byte_range()]).to_string();
            }
        }
    }
    node.kind().to_string()
}

fn validate_tag_structure(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    file: crate::db::SourceFile,
    tree: &tree_sitter::Tree,
    contents: Vec<u8>,
    errors: &mut Vec<ValidationError>,
) {
    let mut stack = vec![tree.root_node()];
    let mut metas = Vec::new();

    while let Some(node) = stack.pop() {
        match node.kind() {
            "meta_tag" => {
                metas.push(node);
            }
            "element" | "self_closing_element" => {
                if let (Some(start_tag), Some(end_tag)) =
                    (node.child(0), node.child((node.child_count() - 1) as u32))
                {
                    if start_tag.kind() == "start_tag" && end_tag.kind() == "end_tag" {
                        // Get tag name based on node type
                        let tag_name = get_tag_name(&node, &contents);

                        // Match end tag name
                        let end_name = end_tag
                            .child_by_field_name("name")
                            .map(|n| String::from_utf8_lossy(&contents[n.byte_range()]).to_string())
                            .unwrap_or_default();

                        if tag_name != end_name {
                            errors.push(ValidationError {
                                range: crate::parser::ts_range_to_lsp(end_tag.range()),
                                message: format!(
                                    "Mismatched closing tag. Expected `</{}>`",
                                    tag_name
                                ),
                            });
                        }

                        // Self-closing include check
                        if node.kind() == "self_closing_element" && tag_name == "include" {
                            if !has_attribute(&node, &contents, "src") {
                                errors.push(ValidationError {
                                    range: crate::parser::ts_range_to_lsp(node.range()),
                                    message: "Invalid include: tag 'include' must have a non-empty 'src' attribute".to_string(),
                                });
                            }
                            if tag_name == "meta" {
                                metas.push(node);
                            }
                        }

                        // Block include check
                        if node.kind() == "element" && tag_name == "include" {
                            errors.push(ValidationError {
                                range: crate::parser::ts_range_to_lsp(node.range()),
                                message: "Invalid include: tag 'include' must be self-closing"
                                    .to_string(),
                            });
                        }

                        // Block meta tracking for nesting/positioning rules
                        if node.kind() == "element" && tag_name == "meta" {
                            metas.push(node);
                        }
                    }
                }
            }
            _ => {}
        }

        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            stack.push(child);
        }
    }

    // Validate `<meta />` nesting and positioning relative to `<body>`
    let body_offset = String::from_utf8_lossy(&contents).find("<body>");
    for meta in &metas {
        if let Some(body_off) = body_offset {
            let meta_pos = crate::parser::ts_range_to_lsp(meta.range()).start;
            let meta_off =
                lsp_pos_to_byte_offset(String::from_utf8_lossy(&contents).as_bytes(), meta_pos);
            if meta_off >= body_off {
                errors.push(ValidationError {
                    range: crate::parser::ts_range_to_lsp(meta.range()),
                    message: "Invalid positioning: tag 'meta' must precede the <body> block"
                        .to_string(),
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

    validate_links(db, workspace, file, errors);
}

fn anchor_exists(contents: &str, anchor: &str) -> bool {
    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok();
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return false,
    };

    fn walk(node: tree_sitter::Node, contents: &str, anchor: &str) -> bool {
        if node.kind() == "anchor" {
            let name_node = node.child_by_field_name("name");
            if let Some(name_node) = name_node {
                let name = &contents[name_node.byte_range()];
                if name == anchor {
                    return true;
                }
            }
        }
        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            if walk(child, contents, anchor) {
                return true;
            }
        }
        false
    }

    walk(tree.root_node(), contents, anchor)
}

fn validate_links(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    file: crate::db::SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let contents = file.contents(db);
    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok();

    if let Some(tree) = parser.parse(&contents, None) {
        fn get_tag_name_for_href(node: &tree_sitter::Node, contents: &[u8]) -> String {
            if node.kind() == "element" || node.kind() == "self_closing_element" {
                let first = node.child(0);
                if let Some(start_tag) = first {
                    if let Some(tag_name_node) = start_tag.child(1) {
                        return String::from_utf8_lossy(&contents[tag_name_node.byte_range()])
                            .to_string();
                    }
                }
            }
            node.kind().to_string()
        }

        fn collect_link_hrefs<'a>(
            node: tree_sitter::Node,
            contents: &[u8],
            found: &mut Vec<(String, tree_sitter::Range)>,
        ) {
            let tag_name = get_tag_name_for_href(&node, contents);

            if (node.kind() == "element" || node.kind() == "self_closing_element")
                && (tag_name == "link" || tag_name == "hubref")
            {
                let mut href = String::new();
                for child in node.named_children(&mut node.walk()) {
                    if child.kind() == "attribute" {
                        if let (Some(name_nn), Some(val_nn)) = (child.child(0), child.child(2)) {
                            let attr_name = &contents[name_nn.byte_range()];
                            if attr_name == b"href" {
                                let raw = String::from_utf8_lossy(&contents[val_nn.byte_range()]);
                                let val = raw.trim_matches('"').trim_matches('\'');
                                href.push_str(val);
                            }
                        }
                    }
                }
                if !href.is_empty() {
                    found.push((href, node.range()));
                }
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_link_hrefs(child, contents, found);
            }
        }

        let mut links: Vec<(String, tree_sitter::Range)> = Vec::new();
        collect_link_hrefs(tree.root_node(), contents.as_bytes(), &mut links);

        for (href, range) in links {
            let (file_part, anchor) = href.split_once('#').unwrap_or((href.as_str(), ""));

            if !file_part.is_empty() && !file.path(db).ends_with(file_part) {
                let target_exists = workspace
                    .files(db)
                    .into_iter()
                    .any(|f| f.path(db).ends_with(file_part));

                if !target_exists {
                    errors.push(ValidationError {
                        range: crate::parser::ts_range_to_lsp(range),
                        message: format!("Target file '{}' not found", file_part),
                    });
                }
            }

            if !anchor.is_empty() && !anchor_exists(&contents, anchor) {
                errors.push(ValidationError {
                    range: crate::parser::ts_range_to_lsp(range),
                    message: format!("Anchor '{}' not found", anchor),
                });
            }
        }
    }
}

fn value_to_canonical(val: HubValue) -> String {
    val.to_string()
}

fn has_attribute(node: &tree_sitter::Node, contents: &[u8], attr_name: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let Some(name_child) = child.child(0) {
                let name = &contents[name_child.byte_range()];
                if name == attr_name.as_bytes() {
                    if let Some(val_child) = child.child(2) {
                        let raw_val = String::from_utf8_lossy(&contents[val_child.byte_range()]);
                        let val = raw_val.trim_matches('"').trim_matches('\'');
                        return !val.trim().is_empty();
                    }
                }
            }
        }
    }
    false
}

fn lsp_pos_to_byte_offset(contents: &[u8], pos: crate::db::LspPosition) -> usize {
    let mut offset = 0;
    for (i, line) in contents.split(|&b| b == b'\n').enumerate() {
        if i == pos.line as usize {
            offset += pos.character as usize;
            break;
        }
        offset += line.len(); // +1 for the newline will be added when we continue to next line
    }
    offset
}
