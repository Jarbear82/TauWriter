use crate::types::{HubReferenceInfo, TwxmlTagInfo};
use streaming_iterator::StreamingIterator;

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

pub fn get_attribute<F>(
    tag_node: tree_sitter::Node,
    contents: &str,
    predicate: F,
) -> Option<(String, std::ops::Range<usize>)>
where
    F: Fn(&str) -> bool,
{
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                let attr_name = &contents[name_node.byte_range()];
                let attr_val = contents[val_node.byte_range()]
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if predicate(attr_name) {
                    return Some((attr_val, val_node.byte_range()));
                }
            }
        }
    }
    None
}

pub fn get_all_attributes<F>(
    tag_node: tree_sitter::Node,
    contents: &str,
    predicate: F,
) -> Vec<(String, std::ops::Range<usize>)>
where
    F: Fn(&str) -> bool,
{
    let mut result = Vec::new();
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                let attr_name = &contents[name_node.byte_range()];
                let attr_val = contents[val_node.byte_range()]
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if predicate(attr_name) {
                    result.push((attr_val, val_node.byte_range()));
                }
            }
        }
    }
    result
}

pub fn parse_hub_references(contents: &str) -> Vec<HubReferenceInfo> {
    let mut refs = Vec::new();

    let language = match crate::load_twxml_language() {
        Some(lang) => lang,
        None => return refs,
    };

    let tree = {
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&language).is_err() {
            return refs;
        }
        match parser.parse(contents, None) {
            Some(t) => t,
            None => return refs,
        }
    };

    let query = match tree_sitter::Query::new(&language, TWXML_HUBREF_QUERY) {
        Ok(q) => q,
        Err(_) => return refs,
    };
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let node = capture.node;

            let (id_val_opt, field_opt, text_opt) = match *capture_name {
                "element" => {
                    let start_tag = match node.child(0) {
                        Some(st) if !st.is_missing() => st,
                        _ => continue,
                    };
                    let (id_val, field) = get_hubref_attributes(start_tag, contents);
                    let text = get_recursive_text(node, contents);
                    let text_opt = if text.is_empty() {
                        None
                    } else {
                        Some(text.trim().to_string())
                    };
                    (id_val, field, text_opt)
                }
                "self_closing" => {
                    let (id_val, field) = get_hubref_attributes(node, contents);
                    (id_val, field, None)
                }
                _ => continue,
            };

            if let Some((id_val, id_range)) = id_val_opt {
                let is_reviewed = is_parent_review(node, contents);
                refs.push(HubReferenceInfo {
                    name: id_val,
                    field: field_opt,
                    text: text_opt,
                    start_offset: node.start_byte(),
                    end_offset: node.end_byte(),
                    id_start_offset: id_range.start,
                    id_end_offset: id_range.end,
                    is_reviewed,
                });
            }
        }
    }

    refs
}

pub fn get_all_twxml_tags(contents: &str) -> Vec<TwxmlTagInfo> {
    let mut tags = Vec::new();
    let language = match crate::load_twxml_language() {
        Some(lang) => lang,
        None => return tags,
    };

    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return tags;
    }
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return tags,
    };

    let root = tree.root_node();
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
                tags.push(TwxmlTagInfo {
                    name: "meta".to_string(),
                    start_offset: child.start_byte(),
                    end_offset: child.end_byte(),
                    parent_name: Some("document".to_string()),
                });
            }
            "body_block" => {
                tags.push(TwxmlTagInfo {
                    name: "body".to_string(),
                    start_offset: child.start_byte(),
                    end_offset: child.end_byte(),
                    parent_name: Some("document".to_string()),
                });
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
                            let parent_name = resolve_parent_tag(&element_node, contents);
                            tags.push(TwxmlTagInfo {
                                name: tag_name.clone(),
                                start_offset: node.start_byte(),
                                end_offset: node.end_byte(),
                                parent_name,
                            });
                        }
                    }
                } else if parent_node.kind() == "self_closing_element" {
                    let parent_name = resolve_parent_tag(&parent_node, contents);
                    tags.push(TwxmlTagInfo {
                        name: tag_name.clone(),
                        start_offset: node.start_byte(),
                        end_offset: node.end_byte(),
                        parent_name,
                    });
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
            "meta_tag" => return Some("meta".to_string()),
            _ => {}
        }
    }
    None
}

fn get_hubref_attributes(
    tag_node: tree_sitter::Node,
    contents: &str,
) -> (
    Option<(String, std::ops::Range<usize>)>,
    Option<String>,
) {
    let id_val = get_attribute(tag_node, contents, |name| name == "id");
    let field_val = get_attribute(tag_node, contents, |name| name == "field").map(|(v, _)| v);
    (id_val, field_val)
}

fn is_parent_review(node: tree_sitter::Node, contents: &str) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
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
        current = parent.parent();
    }
    false
}

fn get_recursive_text(node: tree_sitter::Node, contents: &str) -> String {
    if node.kind() == "text" {
        return contents[node.byte_range()].to_string();
    }
    let mut text_acc = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "start_tag" && child.kind() != "end_tag" {
            let t = get_recursive_text(child, contents);
            if !t.is_empty() {
                if !text_acc.is_empty() && !text_acc.ends_with(' ') {
                    text_acc.push(' ');
                }
                text_acc.push_str(&t);
            }
        }
    }
    text_acc
}
