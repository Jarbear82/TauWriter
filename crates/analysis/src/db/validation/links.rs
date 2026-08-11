/// Anchor and link validation for TWXML documents.
use super::ValidationError;
use crate::db::{SourceFile, Workspace};

/// Check whether an anchor exists in the given TWXML contents by parsing it with tree-sitter.
pub fn anchor_exists(contents: &str, anchor: &str) -> bool {
    let language = crate::parser::tree_sitter_twxml();
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

/// Validate `link` and `hubref` hrefs in the given file (check file targets and anchors).
pub fn validate_links(
    db: &dyn crate::db::Db,
    workspace: Workspace,
    file: SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let contents = file.contents(db);
    let language = crate::parser::tree_sitter_twxml();
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

        let bytes = contents.as_bytes();
        let mut links: Vec<(String, tree_sitter::Range)> = Vec::new();
        collect_link_hrefs(tree.root_node(), bytes, &mut links);

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
