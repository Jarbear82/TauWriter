use tree_sitter::Parser;

const BLOCK_TAGS: &[&str] = &[
    "document",
    "metadata",
    "body",
    "meta",
    "section",
    "heading",
    "paragraph",
    "aside",
    "blockquote",
    "codeblock",
    "ul",
    "ol",
    "li",
    "dl",
    "dt",
    "dd",
    "details",
    "summary",
    "table",
    "tr",
    "th",
    "td",
    "footnote",
    "review",
];

pub fn format_twxml(contents: &str) -> String {
    let language = unsafe { super::tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(contents, None).unwrap();
    let root = tree.root_node();

    let mut result = String::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        result.push_str(&format_twxml_node(child, contents, 0, true));
    }
    result.trim().to_string() + "\n"
}

fn is_block_tag(name: &str) -> bool {
    BLOCK_TAGS.contains(&name)
}

fn contains_block_tag(node: tree_sitter::Node, contents: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "element" || child.kind() == "self_closing_element" {
            if let Some(nm) = child.child_by_field_name("name") {
                let name = &contents[nm.byte_range()];
                if is_block_tag(name) {
                    return true;
                }
            }
        }
        if contains_block_tag(child, contents) {
            return true;
        }
    }
    false
}

fn format_twxml_node(
    node: tree_sitter::Node,
    contents: &str,
    indent_level: usize,
    is_start_of_line: bool,
) -> String {
    let indent = "  ".repeat(indent_level);
    match node.kind() {
        "text" => format_text(contents, node, indent, is_start_of_line),
        "comment" => format_comment(contents, node, indent, is_start_of_line),
        "element" => format_element(node, contents, indent_level, is_start_of_line),
        "self_closing_element" => format_self_closing(node, contents, indent, is_start_of_line),
        "document_block" => format_document_block(node, contents, indent_level),
        "metadata_block" => format_metadata_block(node, contents, indent_level, &indent),
        "body_block" => format_body_block(node, contents, indent_level, &indent),
        _ => {
            let mut inner = String::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                inner.push_str(&format_twxml_node(
                    child,
                    contents,
                    indent_level,
                    is_start_of_line,
                ));
            }
            inner
        }
    }
}

fn format_text(contents: &str, node: tree_sitter::Node, indent: String, start: bool) -> String {
    let txt = contents[node.byte_range()].trim();
    if txt.is_empty() {
        String::new()
    } else if start {
        format!("{}{}", indent, txt)
    } else {
        txt.to_string()
    }
}

fn format_comment(contents: &str, node: tree_sitter::Node, indent: String, start: bool) -> String {
    let cmt = contents[node.byte_range()].trim();
    if start {
        format!("{}{}", indent, cmt)
    } else {
        cmt.to_string()
    }
}

fn extract_attrs(node: tree_sitter::Node, contents: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attribute" {
            attrs.push(contents[child.byte_range()].to_string());
        }
    }
    attrs
}

fn format_element(
    node: tree_sitter::Node,
    contents: &str,
    indent_level: usize,
    is_start_of_line: bool,
) -> String {
    let start_tag = node.child(0).unwrap();
    let name_node = start_tag.child_by_field_name("name").unwrap();
    let tag_name = &contents[name_node.byte_range()];
    let attrs = extract_attrs(start_tag, contents);
    let attrs_str = format_attrs(&attrs);

    let indent = "  ".repeat(indent_level);
    let is_block = is_block_tag(tag_name);
    let has_blocks = contains_block_tag(node, contents);

    if is_block && has_blocks {
        let mut inner = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "start_tag" && child.kind() != "end_tag" {
                let formatted = format_twxml_node(child, contents, indent_level + 1, true);
                if !formatted.is_empty() {
                    inner.push_str(&formatted);
                    inner.push('\n');
                }
            }
        }

        let start_part = if is_start_of_line {
            format!("{}<{}{}>\n", indent, tag_name, attrs_str)
        } else {
            format!("<{}{}>\n", tag_name, attrs_str)
        };
        let end_part = format!("{}</{}>", indent, tag_name);
        format!("{}{}{}", start_part, inner, end_part)
    } else {
        let mut inner = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "start_tag" && child.kind() != "end_tag" {
                inner.push_str(&format_twxml_node(child, contents, 0, false));
            }
        }

        if is_block && is_start_of_line {
            format!(
                "{}<{}{}>{}</{}>",
                indent, tag_name, attrs_str, inner, tag_name
            )
        } else {
            format!("<{}{}>{}</{}>", tag_name, attrs_str, inner, tag_name)
        }
    }
}

fn format_self_closing(
    node: tree_sitter::Node,
    contents: &str,
    indent: String,
    is_start_of_line: bool,
) -> String {
    let name_node = node.child_by_field_name("name").unwrap();
    let tag_name = &contents[name_node.byte_range()];
    let attrs = extract_attrs(node, contents);
    let attrs_str = format_attrs(&attrs);

    if is_block_tag(tag_name) && is_start_of_line {
        format!("{}<{}{}/>", indent, tag_name, attrs_str)
    } else {
        format!("<{}{}/>", tag_name, attrs_str)
    }
}

fn format_attrs(attrs: &[String]) -> String {
    if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    }
}

fn format_document_block(node: tree_sitter::Node, contents: &str, indent_level: usize) -> String {
    let mut inner = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "metadata_block" && child.kind() != "body_block" && !child.is_missing() {
            inner.push_str(&format_twxml_node(child, contents, indent_level + 1, true));
            inner.push('\n');
        }
    }

    let content = inner.trim();
    if content.is_empty() {
        format!("<document></document>")
    } else {
        format!("<document>\n{}\n</document>", content)
    }
}

fn format_metadata_block(
    node: tree_sitter::Node,
    contents: &str,
    indent_level: usize,
    indent: &str,
) -> String {
    let mut inner = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_missing() {
            let formatted = format_twxml_node(child, contents, indent_level + 1, true);
            if !formatted.is_empty() {
                inner.push_str(&formatted);
                inner.push('\n');
            }
        }
    }

    let content = inner.trim();
    if content.is_empty() {
        format!("{}<metadata></metadata>", indent)
    } else {
        format!("{}<metadata>\n{}\n{}</metadata>", indent, content, indent)
    }
}

fn format_body_block(
    node: tree_sitter::Node,
    contents: &str,
    indent_level: usize,
    indent: &str,
) -> String {
    let mut inner = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if !child.is_missing() {
            let formatted = format_twxml_node(child, contents, indent_level + 1, true);
            if !formatted.is_empty() {
                inner.push_str(&formatted);
                inner.push('\n');
            }
        }
    }

    let content = inner.trim();
    if content.is_empty() {
        format!("{}<body></body>", indent)
    } else {
        format!("{}<body>\n{}\n{}</body>", indent, content, indent)
    }
}
