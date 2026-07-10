use lsp_types::CompletionItemKind;

use super::links;
use super::ValidationError;
/// TWXML structural validation rules.
use crate::db::{resolution, HubValue};

/// Metadata for every valid TWXML tag — the single source of truth used by
/// both validation and completion. Encoded as (name, CompletionItemKind, description).
pub const TWXML_TAG_INFO: &[(&str, CompletionItemKind, &str)] = &[
    // Structural
    ("document", CompletionItemKind::CLASS, "TWXML Document"),
    ("body", CompletionItemKind::CLASS, "Body Block"),
    ("meta", CompletionItemKind::CLASS, "Meta Tag"),
    // Content blocks
    ("section", CompletionItemKind::CLASS, "Section"),
    ("heading", CompletionItemKind::CLASS, "Heading"),
    ("paragraph", CompletionItemKind::CLASS, "Paragraph"),
    ("aside", CompletionItemKind::CLASS, "Aside"),
    ("blockquote", CompletionItemKind::CLASS, "Blockquote"),
    ("codeblock", CompletionItemKind::CLASS, "Code Block"),
    // Lists
    ("ul", CompletionItemKind::CLASS, "Unordered List"),
    ("ol", CompletionItemKind::CLASS, "Ordered List"),
    ("li", CompletionItemKind::CLASS, "List Item"),
    ("dl", CompletionItemKind::CLASS, "Definition List"),
    ("dt", CompletionItemKind::CLASS, "Definition Term"),
    ("dd", CompletionItemKind::CLASS, "Definition Description"),
    // Interactive
    ("details", CompletionItemKind::CLASS, "Details"),
    ("summary", CompletionItemKind::CLASS, "Summary"),
    // Tables
    ("table", CompletionItemKind::CLASS, "Table"),
    ("tr", CompletionItemKind::CLASS, "Table Row"),
    ("th", CompletionItemKind::CLASS, "Table Header"),
    ("td", CompletionItemKind::CLASS, "Table Cell"),
    // Inline
    ("hubref", CompletionItemKind::REFERENCE, "Hub Reference"),
    ("link", CompletionItemKind::REFERENCE, "Link"),
    ("image", CompletionItemKind::VALUE, "Image"),
    ("audio", CompletionItemKind::VALUE, "Audio"),
    ("video", CompletionItemKind::VALUE, "Video"),
    ("code", CompletionItemKind::VALUE, "Inline Code"),
    ("bold", CompletionItemKind::VALUE, "Bold"),
    ("italic", CompletionItemKind::VALUE, "Italic"),
    ("underline", CompletionItemKind::VALUE, "Underline"),
    ("strikethrough", CompletionItemKind::VALUE, "Strikethrough"),
    ("super", CompletionItemKind::VALUE, "Superscript"),
    ("sub", CompletionItemKind::VALUE, "Subscript"),
    // Special
    ("br", CompletionItemKind::VALUE, "Line Break"),
    ("hr", CompletionItemKind::VALUE, "Horizontal Rule"),
    ("fr", CompletionItemKind::REFERENCE, "Footnote Reference"),
    ("footnote", CompletionItemKind::CLASS, "Footnote"),
    ("review", CompletionItemKind::CLASS, "Review"),
    ("include", CompletionItemKind::VALUE, "Include"),
];

/// Returns true if *name* is a known TWXML tag.
pub fn is_valid_twxml_tag(name: &str) -> bool {
    TWXML_TAG_INFO.iter().any(|(n, _, _)| *n == name)
}

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
        if !is_valid_twxml_tag(tag.name(db).as_str()) {
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
pub(crate) fn get_tag_name(node: &tree_sitter::Node, contents: &[u8]) -> String {
    if node.kind() == "element" || node.kind() == "self_closing_element" {
        let target = if node.kind() == "element" {
            node.child(0)
        } else {
            Some(*node)
        };
        if let Some(t) = target {
            if let Some(name_node) = t.child_by_field_name("name") {
                return String::from_utf8_lossy(&contents[name_node.byte_range()]).to_string();
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
                let tag_name = get_tag_name(&node, &contents);

                if node.kind() == "element" {
                    if let (Some(start_tag), Some(end_tag)) =
                        (node.child(0), node.child((node.child_count() - 1) as u32))
                    {
                        if start_tag.kind() == "start_tag" && end_tag.kind() == "end_tag" {
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
                        }
                    }

                    // Block include check
                    if tag_name == "include" {
                        errors.push(ValidationError {
                            range: crate::parser::ts_range_to_lsp(node.range()),
                            message: "Invalid include: tag 'include' must be self-closing"
                                .to_string(),
                        });
                    }

                    // Block meta tracking for nesting/positioning rules
                    if tag_name == "meta" {
                        metas.push(node);
                    }
                } else if node.kind() == "self_closing_element" {
                    // Self-closing include check
                    if tag_name == "include" {
                        if !has_attribute(&node, &contents, "src") {
                            errors.push(ValidationError {
                                range: crate::parser::ts_range_to_lsp(node.range()),
                                message: "Invalid include: tag 'include' must have a non-empty 'src' attribute".to_string(),
                            });
                        }
                    }

                    // Self-closing meta tracking for nesting/positioning rules
                    if tag_name == "meta" {
                        metas.push(node);
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

    links::validate_links(db, workspace, file, errors);
}

fn value_to_canonical(val: HubValue) -> String {
    val.to_string()
}

pub(crate) fn has_attribute(node: &tree_sitter::Node, contents: &[u8], attr_name: &str) -> bool {
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

pub(crate) fn lsp_pos_to_byte_offset(contents: &[u8], pos: crate::db::LspPosition) -> usize {
    let mut current_line = 0;
    let mut current_offset = 0;
    
    while current_line < pos.line as usize && current_offset < contents.len() {
        if contents[current_offset] == b'\n' {
            current_line += 1;
        }
        current_offset += 1;
    }
    
    if current_line == pos.line as usize {
        let remaining = &contents[current_offset..];
        if let Ok(s) = std::str::from_utf8(remaining) {
            let mut char_count = 0;
            let mut byte_count = 0;
            for c in s.chars() {
                if char_count >= pos.character as usize || c == '\n' {
                    break;
                }
                char_count += 1;
                byte_count += c.len_utf8();
            }
            return current_offset + byte_count;
        }
    }
    
    contents.len()
}
