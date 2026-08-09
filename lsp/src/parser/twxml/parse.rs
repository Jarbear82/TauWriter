use crate::db::{Db, HubReference, LspPosition, LspRange, SourceFile};
use streaming_iterator::StreamingIterator;
use tree_sitter::Parser;

// user-review: Query string matches hubref elements in twxml grammar.
// Captures either <hubref> element or <hubref /> self-closing element.
const TWXML_HUBREF_QUERY: &str = r#"
    (
      [
        (element
          (start_tag
            (tag_name) @tag_name (#eq? @tag_name "hubref")
          )
        ) @element
        (self_closing_element
          (tag_name) @tag_name (#eq? @tag_name "hubref")
        ) @self_closing
      ]
    )
"#;

pub fn parse_twxml_ast(db: &dyn Db, file: SourceFile) -> Vec<HubReference<'_>> {
    let mut refs = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    let language = match super::super::get_twxml_language() {
        Some(lang) => lang,
        None => return refs,
    };

    let tree = {
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&language).is_err() {
            return refs;
        }
        match parser.parse(&contents, None) {
            Some(t) => t,
            None => return refs,
        }
    };

    let query = match tree_sitter::Query::new(&language, TWXML_HUBREF_QUERY) {
        Ok(q) => q,
        Err(_) => return refs, // Query syntax error — grammar mismatch
    };
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let node = capture.node;

            let (id_val_opt, field_opt, text_opt, tag_range) = match *capture_name {
                "element" => {
                    let start_tag = match node.child(0) {
                        Some(st) if !st.is_missing() => st,
                        _ => continue,
                    };
                    let (id_val, field) = get_attributes(start_tag, &contents);
                    let text = get_recursive_text(node, &contents);
                    let text_opt = if text.is_empty() {
                        None
                    } else {
                        Some(text.trim().to_string())
                    };
                    (
                        id_val,
                        field,
                        text_opt,
                        super::super::ts_range_to_lsp(node.range()),
                    )
                }
                "self_closing" => {
                    let (id_val, field) = get_attributes(node, &contents);
                    (
                        id_val,
                        field,
                        None,
                        super::super::ts_range_to_lsp(node.range()),
                    )
                }
                _ => continue,
            };

            if let Some((id_val, id_range)) = id_val_opt {
                let is_reviewed = is_parent_review(node, &contents);
                refs.push(HubReference::new(
                    db,
                    id_val,
                    file,
                    crate::db::LspRange::from(id_range),
                    field_opt,
                    text_opt,
                    tag_range,
                    is_reviewed,
                ));
            }
        }
    }

    refs
}

/// Public: find review context at a given cursor position.
pub fn find_review_at_position(
    contents: &str,
    pos: LspPosition,
) -> Option<(LspRange, LspRange, String, String, String)> {
    let language = match super::super::get_twxml_language() {
        Some(lang) => lang,
        None => return None,
    };

    let mut parser = Parser::new();
    if let Err(_) = parser.set_language(&language) {
        return None;
    }
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return None,
    };

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let mut node = tree
        .root_node()
        .descendant_for_point_range(ts_pos, ts_pos)?;

    while !matches!(node.kind(), "element" | "self_closing_element") {
        if let Some(parent) = node.parent() {
            node = parent;
        } else {
            break;
        }
    }

    let (review_range, hubref_node) = find_review_and_hubref(&node, contents)?;

    let start_tag = hubref_node.child(0).unwrap_or(hubref_node);
    let (id_opt, field_opt) = get_attributes(start_tag, contents);
    let (id_val, _) = id_opt?;
    let field_val = field_opt?;

    let text = get_recursive_text(hubref_node, contents);

    Some((
        super::super::ts_range_to_lsp(review_range),
        super::super::ts_range_to_lsp(hubref_node.range()),
        id_val,
        field_val,
        text,
    ))
}

// --- Private helpers ---

/// Resolve the enclosing parent tag name for a child element node.
fn resolve_parent_tag(element_node: &tree_sitter::Node, contents: &str) -> Option<String> {
    if let Some(parent_element_node) = element_node.parent() {
        match parent_element_node.kind() {
            "element" => {
                if let Some(p_start_tag) = parent_element_node.child(0) {
                    if p_start_tag.kind() == "start_tag" {
                        if let Some(p_tag_name_node) = p_start_tag.child_by_field_name("name") {
                            return Some(contents[p_tag_name_node.byte_range()].to_string());
                        }
                    }
                }
            }
            "body_block" => return Some("body".to_string()),
            "meta_tag" => return Some("meta".to_string()),
            _ => {}
        }
    }
    None
}

/// Extract id and field attributes from a tag node.
fn get_attributes(
    tag_node: tree_sitter::Node,
    contents: &str,
) -> (
    Option<(String, tower_lsp::lsp_types::Range)>,
    Option<String>,
) {
    let id_val = super::super::twxml::get_attribute(tag_node, contents, |name| name == "id");
    let field_val = super::super::twxml::get_attribute(tag_node, contents, |name| name == "field")
        .map(|(v, _)| v);
    (id_val, field_val)
}

/// Recursively extract text content from a tree-sitter node, excluding tag nodes.
fn get_recursive_text(node: tree_sitter::Node, contents: &str) -> String {
    if node.kind() == "text" {
        return contents[node.byte_range()].to_string();
    }
    let mut text = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "start_tag" && child.kind() != "end_tag" {
            text.push_str(&get_recursive_text(child, contents));
        }
    }
    text
}

/// Check if the parent element is named "review".
fn is_parent_review(node: tree_sitter::Node, contents: &str) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind() != "element" {
        return false;
    }
    let Some(start_tag) = parent.child(0) else {
        return false;
    };
    if start_tag.kind() != "start_tag" {
        return false;
    }
    let Some(name_node) = start_tag.child_by_field_name("name") else {
        return false;
    };
    contents.get(name_node.byte_range()) == Some("review")
}

/// Find a review element and its hubref child, or a hubref element with review parent.
fn find_review_and_hubref<'a>(
    node: &'a tree_sitter::Node,
    contents: &str,
) -> Option<(tree_sitter::Range, tree_sitter::Node<'a>)> {
    if node.kind() != "element" {
        return None;
    }

    let start_tag = node.child(0)?;
    let name_node = start_tag.child_by_field_name("name")?;

    if contents.get(name_node.byte_range()) == Some("review") {
        // Look for hubref child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "element" {
                if let Some(st) = child.child(0) {
                    if let Some(nm) = st.child_by_field_name("name") {
                        if contents.get(nm.byte_range()) == Some("hubref") {
                            return Some((node.range(), st));
                        }
                    }
                }
            }
        }
    } else if contents.get(name_node.byte_range()) == Some("hubref") {
        // Look for review parent
        if let Some(parent) = node.parent() {
            if parent.kind() == "element" {
                if let Some(st) = parent.child(0) {
                    if let Some(nm) = st.child_by_field_name("name") {
                        if contents.get(nm.byte_range()) == Some("review") {
                            return Some((parent.range(), start_tag));
                        }
                    }
                }
            }
        }
    }

    None
}

// --- Tag enumeration (separate from parse to keep scope tight) ---

pub fn get_all_twxml_tags(db: &dyn Db, file: SourceFile) -> Vec<crate::db::TwxmlTag<'_>> {
    let mut tags = Vec::new();
    let contents = file.contents(db);

    let language = match super::super::get_twxml_language() {
        Some(lang) => lang,
        None => return tags,
    };

    let mut parser = Parser::new();
    if let Err(_) = parser.set_language(&language) {
        return tags;
    }
    let tree = match parser.parse(&contents, None) {
        Some(t) => t,
        None => return tags,
    };

    let root = tree.root_node();
    // Root is source_file → document_block; body/meta live under document_block
    let container = if root.kind() == "source_file" {
        root.child(0)
    } else {
        Some(root)
    };
    let children: Vec<_> = match container {
        Some(node) => node.children(&mut node.walk()).collect(),
        None => vec![],
    };
    for child in children {
        match child.kind() {
            "meta_tag" => {
                tags.push(crate::db::TwxmlTag::new(
                    db,
                    "meta".to_string(),
                    file,
                    super::super::ts_range_to_lsp(child.range()),
                    Some("document".to_string()),
                ));
            }
            "body_block" => {
                tags.push(crate::db::TwxmlTag::new(
                    db,
                    "body".to_string(),
                    file,
                    super::super::ts_range_to_lsp(child.range()),
                    Some("document".to_string()),
                ));
            }
            _ => {}
        }
    }

    let query = match tree_sitter::Query::new(&language, "(tag_name) @tag") {
        Ok(q) => q,
        Err(_) => return tags,
    };
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches_result = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    while let Some(m) = matches_result.next() {
        for capture in m.captures {
            let node = capture.node;
            let tag_name = contents.get(node.byte_range()).unwrap_or("").to_string();

            if let Some(parent_node) = node.parent() {
                if parent_node.kind() == "start_tag" {
                    if let Some(element_node) = parent_node.parent() {
                        if element_node.kind() == "element" {
                            let parent_name = resolve_parent_tag(&element_node, &contents);
                            tags.push(crate::db::TwxmlTag::new(
                                db,
                                tag_name.clone(),
                                file,
                                super::super::ts_range_to_lsp(node.range()),
                                parent_name,
                            ));
                        }
                    }
                } else if parent_node.kind() == "self_closing_element" {
                    let parent_name = resolve_parent_tag(&parent_node, &contents);
                    tags.push(crate::db::TwxmlTag::new(
                        db,
                        tag_name.clone(),
                        file,
                        super::super::ts_range_to_lsp(node.range()),
                        parent_name,
                    ));
                }
            }
        }
    }

    tags
}
