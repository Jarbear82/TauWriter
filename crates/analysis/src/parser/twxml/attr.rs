use crate::parser::ts_range_to_lsp;
use lsp_types::Range;

pub fn parse_tag(tree: &tree_sitter::Tree) -> Option<&str> {
    let mut cursor = tree.root_node().walk();
    for child in tree.root_node().children(&mut cursor) {
        if child.kind() == "start_tag" || child.kind() == "self_closing_element" {
            return Some(child.kind());
        }
        let mut inner = child.walk();
        for inner_child in child.children(&mut inner) {
            if inner_child.kind() == "start_tag" || inner_child.kind() == "self_closing_element" {
                return Some(inner_child.kind());
            }
        }
    }
    None
}

pub fn get_attribute<F>(
    tag_node: tree_sitter::Node,
    contents: &str,
    predicate: F,
) -> Option<(String, Range)>
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
                    return Some((attr_val, ts_range_to_lsp(val_node.range()).into()));
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
) -> Vec<(String, Range)>
where
    F: Fn(&str) -> bool,
{
    tauwriter_twxml::get_all_attributes(tag_node, contents, predicate)
        .into_iter()
        .map(|(val, _range)| (val, Range::default()))
        .collect()
}
