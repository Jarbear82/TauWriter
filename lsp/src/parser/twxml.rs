use crate::db::{Db, HubReference, LspPosition, SourceFile};
use tree_sitter::Parser;

pub fn parse_twxml_ast(db: &dyn Db, file: SourceFile) -> Vec<HubReference<'_>> {
    let mut refs = Vec::new();
    let contents = file.contents(db);

    let language = unsafe { super::tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let query_str = r#"
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
    let query = tree_sitter::Query::new(language, query_str).unwrap();
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    for m in matches {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let node = capture.node;

            let (id_val_opt, field_opt, text_opt, tag_range) = match capture_name.as_str() {
                "element" => {
                    let start_tag = node.child(0).unwrap();
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
                        super::ts_range_to_lsp(node.range()),
                    )
                }
                "self_closing" => {
                    let (id_val, field) = get_attributes(node, &contents);
                    (id_val, field, None, super::ts_range_to_lsp(node.range()))
                }
                _ => continue,
            };

            if let Some((id_val, id_range)) = id_val_opt {
                let is_reviewed = is_parent_review(node, &contents);
                refs.push(HubReference::new(
                    db,
                    id_val,
                    file,
                    id_range,
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

pub fn get_all_twxml_tags(db: &dyn Db, file: SourceFile) -> Vec<crate::db::TwxmlTag<'_>> {
    let mut tags = Vec::new();
    let contents = file.contents(db);

    let language = unsafe { super::tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let root = tree.root_node();
    for child in root.children(&mut root.walk()) {
        match child.kind() {
            "metadata_block" => {
                tags.push(crate::db::TwxmlTag::new(
                    db,
                    "metadata".to_string(),
                    file,
                    super::ts_range_to_lsp(child.range()),
                    Some("document".to_string()),
                ));
            }
            "body_block" => {
                tags.push(crate::db::TwxmlTag::new(
                    db,
                    "body".to_string(),
                    file,
                    super::ts_range_to_lsp(child.range()),
                    Some("document".to_string()),
                ));
            }
            _ => {}
        }
    }

    let query_str = "(tag_name) @tag";
    let query = tree_sitter::Query::new(language, query_str).unwrap();
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    for m in matches {
        for capture in m.captures {
            let node = capture.node;
            let tag_name = contents[node.byte_range()].to_string();

            if let Some(start_tag_node) = node.parent() {
                if start_tag_node.kind() == "start_tag" {
                    if let Some(element_node) = start_tag_node.parent() {
                        if element_node.kind() == "element" {
                            let parent_name = resolve_parent_tag(&element_node, &contents);

                            tags.push(crate::db::TwxmlTag::new(
                                db,
                                tag_name.clone(),
                                file,
                                super::ts_range_to_lsp(node.range()),
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
            "metadata_block" => return Some("metadata".to_string()),
            _ => {}
        }
    }
    None
}

fn get_attributes(
    tag_node: tree_sitter::Node,
    contents: &str,
) -> (Option<(String, crate::db::LspRange)>, Option<String>) {
    let mut id_val = None;
    let mut field_val = None;
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                let attr_name = &contents[name_node.byte_range()];
                let attr_val = contents[val_node.byte_range()]
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if attr_name == "id" {
                    id_val = Some((attr_val, super::ts_range_to_lsp(val_node.range())));
                } else if attr_name == "field" {
                    field_val = Some(attr_val);
                }
            }
        }
    }
    (id_val, field_val)
}

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

fn is_parent_review(node: tree_sitter::Node, contents: &str) -> bool {
    if let Some(parent) = node.parent() {
        if parent.kind() == "element" {
            if let Some(start_tag) = parent.child(0) {
                if start_tag.kind() == "start_tag" {
                    if let Some(name_node) = start_tag.child_by_field_name("name") {
                        if &contents[name_node.byte_range()] == "review" {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn find_review_at_position(
    contents: &str,
    pos: LspPosition,
) -> Option<(
    crate::db::LspRange,
    crate::db::LspRange,
    String,
    String,
    String,
)> {
    let language = unsafe { super::tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(contents, None)?;

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let mut node = tree
        .root_node()
        .descendant_for_point_range(ts_pos, ts_pos)?;

    while node.kind() != "element" && node.kind() != "self_closing_element" {
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
        super::ts_range_to_lsp(review_range),
        super::ts_range_to_lsp(hubref_node.range()),
        id_val,
        field_val,
        text,
    ))
}

fn find_review_and_hubref<'a>(
    node: &'a tree_sitter::Node,
    contents: &str,
) -> Option<(tree_sitter::Range, tree_sitter::Node<'a>)> {
    if node.kind() != "element" {
        return None;
    }

    let start_tag = node.child(0)?;
    let name_node = start_tag.child_by_field_name("name")?;
    let name = &contents[name_node.byte_range()];

    if name == "review" {
        // Look for hubref child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "element" {
                if let Some(st) = child.child(0) {
                    if let Some(nm) = st.child_by_field_name("name") {
                        if &contents[nm.byte_range()] == "hubref" {
                            return Some((node.range(), st));
                        }
                    }
                }
            }
        }
    } else if name == "hubref" {
        // Look for review parent
        if let Some(parent) = node.parent() {
            if parent.kind() == "element" {
                if let Some(st) = parent.child(0) {
                    if let Some(nm) = st.child_by_field_name("name") {
                        if &contents[nm.byte_range()] == "review" {
                            return Some((parent.range(), start_tag));
                        }
                    }
                }
            }
        }
    }

    None
}

pub enum TwxmlCompletionContext {
    HubrefId,
    HubrefField { id_val: String },
    None,
}

pub fn get_twxml_completion_context(contents: &str, pos: LspPosition) -> TwxmlCompletionContext {
    let language = unsafe { super::tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return TwxmlCompletionContext::None,
    };

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let node = match tree.root_node().descendant_for_point_range(ts_pos, ts_pos) {
        Some(n) => n,
        None => return TwxmlCompletionContext::None,
    };

    let mut current = node;
    let mut attribute_node = None;
    while current.kind() != "document" && current.kind() != "source_file" {
        if current.kind() == "attribute" {
            attribute_node = Some(current);
            break;
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    let attr = match attribute_node {
        Some(a) => a,
        None => return TwxmlCompletionContext::None,
    };

    if let (Some(name_node), Some(_val_node)) = (attr.child(0), attr.child(2)) {
        let attr_name = &contents[name_node.byte_range()];
        let parent = attr.parent().unwrap();

        let is_hubref = if parent.kind() == "start_tag" || parent.kind() == "self_closing_element" {
            if let Some(nm) = parent.child_by_field_name("name") {
                &contents[nm.byte_range()] == "hubref"
            } else {
                false
            }
        } else {
            false
        };

        if is_hubref {
            if attr_name == "id" {
                return TwxmlCompletionContext::HubrefId;
            } else if attr_name == "field" {
                let (id_opt, _) = get_attributes(parent, contents);
                if let Some((id_val, _)) = id_opt {
                    return TwxmlCompletionContext::HubrefField { id_val };
                }
            }
        }
    }

    TwxmlCompletionContext::None
}
