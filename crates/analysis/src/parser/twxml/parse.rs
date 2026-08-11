use crate::db::{Db, HubReference, LspPosition, LspRange, SourceFile, TwxmlTag};

pub fn parse_twxml_ast(db: &dyn Db, file: SourceFile) -> Vec<HubReference<'_>> {
    let contents = file.contents(db);
    let refs_info = tauwriter_twxml::parse_hub_references(&contents);

    refs_info
        .into_iter()
        .map(|r| {
            let tag_start = byte_offset_to_lsp_pos(&contents, r.start_offset);
            let tag_end = byte_offset_to_lsp_pos(&contents, r.end_offset);
            let tag_range = LspRange { start: tag_start, end: tag_end };

            let id_start = byte_offset_to_lsp_pos(&contents, r.id_start_offset);
            let id_end = byte_offset_to_lsp_pos(&contents, r.id_end_offset);
            let name_range = LspRange { start: id_start, end: id_end };

            HubReference::new(
                db,
                r.name,
                file,
                name_range,
                r.field,
                r.text,
                tag_range,
                r.is_reviewed,
            )
        })
        .collect()
}

pub fn get_all_twxml_tags(db: &dyn Db, file: SourceFile) -> Vec<TwxmlTag<'_>> {
    let contents = file.contents(db);
    let tags_info = tauwriter_twxml::get_all_twxml_tags(&contents);

    tags_info
        .into_iter()
        .map(|t| {
            let start = byte_offset_to_lsp_pos(&contents, t.start_offset);
            let end = byte_offset_to_lsp_pos(&contents, t.end_offset);
            let range = LspRange { start, end };
            TwxmlTag::new(db, t.name, file, range, t.parent_name)
        })
        .collect()
}

pub fn find_review_at_position(
    contents: &str,
    pos: LspPosition,
) -> Option<(LspRange, LspRange, String, String, String)> {
    let language = super::super::get_twxml_language()?;
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return None;
    }
    let tree = parser.parse(contents, None)?;

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

fn byte_offset_to_lsp_pos(content: &str, offset: usize) -> LspPosition {
    let mut line = 0;
    let mut character = 0;
    for (i, c) in content.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }
    LspPosition { line, character }
}

fn get_attributes(
    tag_node: tree_sitter::Node,
    contents: &str,
) -> (
    Option<(String, lsp_types::Range)>,
    Option<String>,
) {
    let id_val = super::super::twxml::get_attribute(tag_node, contents, |name| name == "id");
    let field_val = super::super::twxml::get_attribute(tag_node, contents, |name| name == "field")
        .map(|(v, _)| v);
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
