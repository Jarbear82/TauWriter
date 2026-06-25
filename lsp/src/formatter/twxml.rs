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
        // Check the child's tag name if it's an element or self_closing_element.
        // element nodes carry their name inside start_tag,
        // self_closing_element carries it directly.
        let name = match child.kind() {
            "element" => child
                .child(0)
                .and_then(|st| st.child_by_field_name("name"))
                .map(|nm| &contents[nm.byte_range()]),
            "self_closing_element" => child
                .child_by_field_name("name")
                .map(|nm| &contents[nm.byte_range()]),
            _ => None,
        };

        if let Some(n) = name {
            if is_block_tag(n) {
                return true;
            }
        }

        // Recurse into children to find deeply nested block tags.
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
    let raw = &contents[node.byte_range()];
    if start {
        let txt = raw.trim();
        if txt.is_empty() {
            String::new()
        } else {
            format!("{}{}", indent, txt)
        }
    } else {
        // Inline text: preserve spaces around inline tags, just collapse newlines
        let collapsed = raw.replace('\n', " ");
        if collapsed.trim().is_empty() {
            String::new()
        } else {
            collapsed
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_elements_with_nested_blocks_expand_multiline() {
        let input = r#"<document>
  <metadata></metadata>
  <body><section><paragraph>Hello</paragraph></section></body>
</document>"#;
        let output = format_twxml(input);
        // <section> contains <paragraph>, so it should expand to multiline
        assert!(
            output.contains("<section>\n"),
            "section should have newline after opening tag, got:\n{}",
            output
        );
        assert!(
            output.contains("</section>"),
            "closing tag should exist, got:\n{}",
            output
        );
    }

    #[test]
    fn block_with_only_inline_content_stays_single_line() {
        let input = r#"<document>
  <metadata></metadata>
  <body><heading>Just text</heading></body>
</document>"#;
        let output = format_twxml(input);
        // <heading> has no nested block children, so stays on one line
        assert!(
            output.contains("<heading>Just text</heading>"),
            "heading with only text should stay inline, got:\n{}",
            output
        );
    }

    #[test]
    fn inline_text_preserves_spaces_around_tags() {
        let input = r#"<document>
  <metadata></metadata>
  <body><paragraph>Hello <b>world</b>!</paragraph></body>
</document>"#;
        let output = format_twxml(input);
        // Spaces around <b> should be preserved
        assert!(
            output.contains("Hello ") || output.contains("Hello\n"),
            "space before inline tag should be preserved, got:\n{}",
            output
        );
    }
}
