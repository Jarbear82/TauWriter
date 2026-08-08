use crate::types::OutlineNode;
use streaming_iterator::StreamingIterator;

/// Parse the active TWXML text using Tree-sitter and the outlines.scm query
/// to produce nodes and parent-child edges.
pub fn parse_document_outline(text: &str) -> (Vec<OutlineNode>, Vec<(usize, usize)>) {
    let language = match crate::load_twxml_language() {
        Some(lang) => lang,
        None => return (Vec::new(), Vec::new()),
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return (Vec::new(), Vec::new());
    }
    let tree = match parser.parse(text, None) {
        Some(t) => t,
        None => return (Vec::new(), Vec::new()),
    };

    let query_str = include_str!("../../../extension/languages/twxml/outlines.scm");
    let query = match tree_sitter::Query::new(&language, query_str) {
        Ok(q) => q,
        Err(_) => return (Vec::new(), Vec::new()),
    };

    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = query_cursor.matches(&query, tree.root_node(), text.as_bytes());

    let mut nodes = Vec::new();
    let mut ts_id_to_idx = std::collections::HashMap::new();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            if node.kind() == "element" || node.kind() == "self_closing_element" {
                let node_id = node.id();
                if ts_id_to_idx.contains_key(&node_id) {
                    continue;
                }

                let mut tag_name = String::new();
                if let Some(start_tag) = node.child(0) {
                    if start_tag.kind() == "start_tag" {
                        if let Some(name_node) = start_tag.child_by_field_name("name") {
                            tag_name = text[name_node.byte_range()].to_string();
                        }
                    }
                }
                if tag_name.is_empty() && node.kind() == "self_closing_element" {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        tag_name = text[name_node.byte_range()].to_string();
                    }
                }
                if tag_name.is_empty() {
                    tag_name = "element".to_string();
                }

                let mut display_name = String::new();
                if tag_name == "section" {
                    if let Some(start_tag) = node.child(0) {
                        display_name = get_attribute_value_str(start_tag, text, "alias")
                            .unwrap_or_else(|| "Section".to_string());
                    }
                } else if tag_name == "heading" || tag_name == "paragraph" {
                    display_name = collect_node_text(node, text);
                    if display_name.len() > 15 {
                        display_name =
                            format!("{}...", display_name.chars().take(12).collect::<String>());
                    }
                } else if tag_name == "hubref" {
                    let start_tag = if node.kind() == "element" {
                        node.child(0).unwrap_or(node)
                    } else {
                        node
                    };
                    let id_val = get_attribute_value_str(start_tag, text, "id")
                        .unwrap_or_else(|| "hubref".to_string());
                    display_name = format!("Ref: {}", id_val);
                }

                if display_name.trim().is_empty() {
                    display_name = tag_name.clone();
                }

                let start_offset = node.start_byte();
                let idx = nodes.len();
                nodes.push((
                    node,
                    OutlineNode {
                        id: format!("{}_{}", tag_name, idx),
                        name: display_name,
                        kind: tag_name,
                        start_offset,
                    },
                ));
                ts_id_to_idx.insert(node_id, idx);
            }
        }
    }

    let mut edges = Vec::new();
    for (idx, (node, _)) in nodes.iter().enumerate() {
        let mut parent = node.parent();
        while let Some(p) = parent {
            if let Some(&parent_idx) = ts_id_to_idx.get(&p.id()) {
                edges.push((parent_idx, idx));
                break;
            }
            parent = p.parent();
        }
    }

    let final_nodes = nodes.into_iter().map(|(_, n)| n).collect();
    (final_nodes, edges)
}

fn get_attribute_value_str(
    tag_node: tree_sitter::Node,
    text: &str,
    attr_name: &str,
) -> Option<String> {
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let Some(name_node) = child.child(0) {
                let name = &text[name_node.byte_range()];
                if name == attr_name {
                    if let Some(val_node) = child.child(2) {
                        return Some(
                            text[val_node.byte_range()]
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string(),
                        );
                    }
                }
            }
        }
    }
    None
}

fn collect_node_text(node: tree_sitter::Node, text: &str) -> String {
    if node.kind() == "text" {
        return text[node.byte_range()].to_string();
    }
    let mut text_acc = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "start_tag" && child.kind() != "end_tag" {
            let t = collect_node_text(child, text);
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
