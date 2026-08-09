use lsp_types::Position;

#[derive(Debug)]
pub enum TwxmlCompletionContext {
    HubrefId,
    HubrefField {
        id_val: String,
    },
    /// User just typed `<` and is about to type a tag name.
    /// `parent` is the current parent tag name, if any.
    Tag {
        parent: Option<String>,
    },
    None,
}

pub fn get_twxml_completion_context(contents: &str, pos: Position) -> TwxmlCompletionContext {
    let language = match super::super::get_twxml_language() {
        Some(lang) => lang,
        None => return TwxmlCompletionContext::None,
    };
    let mut parser = tree_sitter::Parser::new();
    if let Err(_) = parser.set_language(&language) {
        return TwxmlCompletionContext::None;
    }
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return TwxmlCompletionContext::None,
    };

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    // Check for tag name completion — cursor just after `<` (but not `</` or `<!--`)
    if let Some(line) = contents.lines().nth(pos.line as usize) {
        let col = pos.character as usize;
        if col <= line.len() {
            let before = &line[..col];
            if let Some(lt_pos) = before.rfind('<') {
                let after_lt = &before[lt_pos..];
                // Match `<` followed by zero or more alphanumeric/underscore chars (partial tag name)
                // Exclude closing tags and comments
                if !after_lt.ends_with("/") && !after_lt.ends_with("!--") {
                    let partial = after_lt.strip_prefix("<").unwrap_or("");
                    let is_tag_name = partial.is_empty()
                        || partial.chars().all(|c| c.is_alphanumeric() || c == '_');
                    if is_tag_name {
                        // Find current parent tag by walking the AST
                        let parent = if let Some(node) =
                            tree.root_node().descendant_for_point_range(ts_pos, ts_pos)
                        {
                            find_parent_tag_name(&node, contents)
                        } else {
                            None
                        };
                        return TwxmlCompletionContext::Tag { parent };
                    }
                }
            }
        }
    }

    let node = match tree.root_node().descendant_for_point_range(ts_pos, ts_pos) {
        Some(n) => n,
        None => return TwxmlCompletionContext::None,
    };

    // Walk up to find attribute node (attribute is a child of start_tag/self_closing_element)
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

    // Check if it's an id/field attribute on a hubref tag
    if let (Some(name_node), Some(_val_node)) = (attr.child(0), attr.child(2)) {
        let attr_name = contents.get(name_node.byte_range()).unwrap_or("");
        let parent = match attr.parent() {
            Some(p) => p,
            None => return TwxmlCompletionContext::None,
        };

        let is_hubref = if matches!(parent.kind(), "start_tag" | "self_closing_element") {
            if let Some(nm) = parent.child_by_field_name("name") {
                contents.get(nm.byte_range()) == Some("hubref")
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
                // Extract id value from the parent's attributes
                let mut id_val = None;
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "attribute" {
                        if let (Some(n), Some(v)) = (child.child(0), child.child(2)) {
                            let n_str = contents.get(n.byte_range()).unwrap_or("");
                            let v_str = contents
                                .get(v.byte_range())
                                .map(|s| s.trim_matches('"').to_string())
                                .unwrap_or_default();
                            if n_str == "id" {
                                id_val = Some(v_str);
                            }
                        }
                    }
                }
                if let Some(id_value) = id_val {
                    return TwxmlCompletionContext::HubrefField { id_val: id_value };
                }
            }
        }
    }

    TwxmlCompletionContext::None
}

/// Walk up from `node` to find the nearest enclosing element's tag name.
fn find_parent_tag_name(node: &tree_sitter::Node, contents: &str) -> Option<String> {
    let mut current = *node;
    loop {
        if let Some(parent) = current.parent() {
            match parent.kind() {
                "element" => {
                    if let Some(start_tag) = parent.child(0) {
                        if let Some(name_node) = start_tag.child_by_field_name("name") {
                            return Some(contents[name_node.byte_range()].to_string());
                        }
                    }
                }
                "body_block" => return Some("body".to_string()),
                "meta_tag" => return Some("meta".to_string()),
                "document_block" => return Some("document".to_string()),
                _ => {}
            }
            current = parent;
        } else {
            break;
        }
    }
    None
}
