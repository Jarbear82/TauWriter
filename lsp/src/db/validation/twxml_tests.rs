use super::twxml::*;

fn parse_doc(content: &str) -> (tree_sitter::Tree, Vec<u8>) {
    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(content, None).unwrap();
    (tree, content.as_bytes().to_vec())
}

fn find_node<'a>(node: tree_sitter::Node<'a>, kind: &str) -> Option<tree_sitter::Node<'a>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(res) = find_node(child, kind) {
            return Some(res);
        }
    }
    None
}

fn find_node_with_name<'a>(node: tree_sitter::Node<'a>, kind: &str, name: &str, contents: &[u8]) -> Option<tree_sitter::Node<'a>> {
    if node.kind() == kind {
        let current_name = get_tag_name(&node, contents);
        if current_name == name {
            return Some(node);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(res) = find_node_with_name(child, kind, name, contents) {
            return Some(res);
        }
    }
    None
}

#[test]
fn test_get_tag_name_element() {
    let content = "<document><body><section></section></body></document>";
    let (tree, bytes) = parse_doc(content);
    let root = tree.root_node();
    
    // Find the element with tag name "section"
    let section = find_node_with_name(root, "element", "section", &bytes).unwrap();
    let name = get_tag_name(&section, &bytes);
    assert_eq!(name, "section");
}

#[test]
fn test_get_tag_name_self_closing() {
    let content = "<document><meta name=\"author\" /><body></body></document>";
    let (tree, bytes) = parse_doc(content);
    let root = tree.root_node();
    
    let meta = find_node(root, "meta_tag").unwrap();
    let name = get_tag_name(&meta, &bytes);
    assert_eq!(name, "meta_tag");
}

#[test]
fn test_has_attribute_check() {
    let content = "<document><body><include src=\"chapter1.twxml\" /></body></document>";
    let (tree, bytes) = parse_doc(content);
    let root = tree.root_node();
    
    let include_node = find_node(root, "self_closing_element").unwrap();
    assert!(has_attribute(&include_node, &bytes, "src"));
    assert!(!has_attribute(&include_node, &bytes, "href"));

    let name = get_tag_name(&include_node, &bytes);
    assert_eq!(name, "include");
}

#[test]
fn test_lsp_pos_to_byte_offset_mapping() {
    let content = "line0\nline1\nline2";
    let pos = crate::db::LspPosition { line: 1, character: 2 };
    let offset = lsp_pos_to_byte_offset(content.as_bytes(), pos);
    // "line0\n" is 6 bytes. "li" is 2 bytes. Total offset should be 8.
    assert_eq!(offset, 8);
}
