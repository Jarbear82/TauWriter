use tree_sitter::Parser;

const MAX_LINE_LEN: usize = 80;

pub fn format_twxml(contents: &str) -> String {
    let language = unsafe { super::tree_sitter_twxml() };
    let mut parser = Parser::new();
    if parser.set_language(language).is_err() {
        return contents.to_string();
    }

    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return contents.to_string(),
    };

    // R13: Parse errors: return original text unchanged.
    if tree.root_node().has_error() {
        return contents.to_string();
    }

    let mut result = format_node(tree.root_node(), contents, 0, None);
    result = result.trim().to_string();
    if !result.is_empty() {
        result.push('\n');
    }
    result
}

// ponytail: deleted hardcoded TagCategory enum and aggressive hard-wrapping.
// We dynamically infer inline vs block context.
// Ceiling: Relies on direct-text heuristics. Edge cases in malformed XML might inline blocks.
// Upgrade: Full schema-aware formatting phase with DTD.
fn format_node(
    node: tree_sitter::Node,
    contents: &str,
    indent: usize,
    block_indent: Option<usize>,
) -> String {
    let ind_str = "  ".repeat(indent);

    match node.kind() {
        "text" => {
            if block_indent.is_some() {
                collapse_whitespace(&contents[node.byte_range()])
            } else {
                String::new() // Drop text nodes caught in pure block context (whitespace)
            }
        }
        "comment" => {
            let cmt = contents[node.byte_range()].trim();
            if block_indent.is_some() {
                cmt.to_string()
            } else {
                format!("{}{}\n", ind_str, cmt)
            }
        }
        "element" | "document_block" | "metadata_block" | "body_block" | "self_closing_element" => {
            format_element(node, contents, indent, block_indent)
        }
        _ => {
            let mut out = String::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                out.push_str(&format_node(child, contents, indent, block_indent));
            }
            out
        }
    }
}

fn format_element(
    node: tree_sitter::Node,
    contents: &str,
    indent: usize,
    block_indent: Option<usize>,
) -> String {
    let tag_name = get_tag_name(&node, contents).unwrap_or_default();
    if tag_name.is_empty() {
        return String::new();
    }

    let attrs = get_attributes(&node, contents);
    let attr_str = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };
    let ind_str = "  ".repeat(indent);

    if tag_name == "codeblock" {
        let mut inner = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "text" {
                inner.push_str(&contents[child.byte_range()]);
            }
        }
        if inner.is_empty() {
            return format!("{}<{}{}></{}>", ind_str, tag_name, attr_str, tag_name);
        }

        let trimmed = inner.trim_matches(|c| c == '\r' || c == '\n');
        let mut padded = String::new();
        for (i, line) in trimmed.split('\n').enumerate() {
            if i == 0 {
                padded.push_str(&format!("{}{}", "  ".repeat(indent + 1), line));
            } else {
                padded.push('\n');
                padded.push_str(line);
            }
        }
        return format!(
            "{}<{}{}>\n{}\n{}</{}>",
            ind_str, tag_name, attr_str, padded, ind_str, tag_name
        );
    }

    if is_node_empty_of_content(&node, contents) && tag_name != "br" && tag_name != "fr" {
        if block_indent.is_some() {
            return format!("<{}{}/>", tag_name, attr_str);
        } else {
            return format!("{}<{}{}/>", ind_str, tag_name, attr_str);
        }
    }

    if tag_name == "br" {
        if let Some(lvl) = block_indent {
            return format!("<br/>\n{}", "  ".repeat(lvl + 1));
        } else {
            return format!("{}<br/>", ind_str);
        }
    }

    if tag_name == "fr" {
        if block_indent.is_some() {
            return format!("<fr{attr_str}/>");
        } else {
            return format!("{}<fr{attr_str}/>", ind_str);
        }
    }

    // It's an inline container IF we're already nested in one, OR it holds direct text
    let has_text = has_direct_significant_text(&node, contents);
    let is_inline = block_indent.is_some() || has_text;

    if is_inline {
        let mut inner = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let next_lvl = block_indent.or(Some(indent));

            if child.kind() == "text" {
                let mut cw = collapse_whitespace(&contents[child.byte_range()]);
                // Clean up spacing artifacts from older hard-wraps before punctuation
                if cw.starts_with(" .")
                    || cw.starts_with(" ,")
                    || cw.starts_with(" !")
                    || cw.starts_with(" ?")
                    || cw.starts_with(" ;")
                    || cw.starts_with(" :")
                {
                    cw.remove(0);
                }

                if inner.ends_with(|c: char| c.is_whitespace()) {
                    inner.push_str(cw.trim_start());
                } else {
                    inner.push_str(&cw);
                }
            } else if child.kind() != "start_tag"
                && child.kind() != "end_tag"
                && child.kind() != "tag_name"
            {
                let c_str = format_node(child, contents, 0, next_lvl);
                if inner.ends_with(|c: char| c.is_whitespace()) {
                    inner.push_str(c_str.trim_start());
                } else {
                    inner.push_str(&c_str);
                }
            }
        }

        if block_indent.is_some() {
            // Nested inline element - wrap tightly
            format!("<{}{}>{}</{}>", tag_name, attr_str, inner, tag_name)
        } else {
            // Root inline container (e.g., paragraph)
            let mock = format!("<{}{}>{}</{}>", tag_name, attr_str, inner.trim(), tag_name);
            if mock.len() <= MAX_LINE_LEN && !inner.contains('\n') {
                format!("{}{}", ind_str, mock)
            } else {
                format!(
                    "{}<{}{}>\n  {}{}\n{}</{}>",
                    ind_str,
                    tag_name,
                    attr_str,
                    ind_str,
                    inner.trim(),
                    ind_str,
                    tag_name
                )
            }
        }
    } else {
        // BLOCK CONTAINER
        let mut children_out = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "start_tag"
                && child.kind() != "end_tag"
                && child.kind() != "tag_name"
                && child.kind() != "text"
            {
                let c_str = format_node(child, contents, indent + 1, None);
                if !c_str.trim().is_empty() {
                    children_out.push(c_str);
                }
            }
        }

        // Try placing single-child blocks inline (e.g. <td><hubref>...</hubref></td>)
        if children_out.len() == 1 {
            let joined = children_out[0].trim();
            let mock = format!("<{}{}>{}</{}>", tag_name, attr_str, joined, tag_name);
            if mock.len() <= MAX_LINE_LEN && !joined.contains('\n') {
                return format!("{}{}", ind_str, mock);
            }
        }

        let mut out = format!("{}<{}{}>\n", ind_str, tag_name, attr_str);
        for c in children_out {
            out.push_str(&c);
            if !c.ends_with('\n') {
                out.push('\n');
            }
        }
        out.push_str(&format!("{}</{}>", ind_str, tag_name));
        out
    }
}

fn get_tag_name(node: &tree_sitter::Node, contents: &str) -> Option<String> {
    if node.kind() == "element" || node.kind() == "self_closing_element" {
        let target = if node.kind() == "element" {
            node.child(0)?
        } else {
            *node
        };
        let name_node = target.child_by_field_name("name")?;
        Some(contents[name_node.byte_range()].to_string())
    } else {
        match node.kind() {
            "document_block" => Some("document".to_string()),
            "metadata_block" => Some("metadata".to_string()),
            "body_block" => Some("body".to_string()),
            _ => None,
        }
    }
}

fn get_attributes(node: &tree_sitter::Node, contents: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let target = if node.kind() == "element" {
        node.child(0).unwrap_or(*node)
    } else {
        *node
    };

    let mut cursor = target.walk();
    for child in target.children(&mut cursor) {
        if child.kind() == "attribute" {
            attrs.push(contents[child.byte_range()].to_string());
        }
    }
    attrs
}

fn is_node_empty_of_content(node: &tree_sitter::Node, contents: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "start_tag" && child.kind() != "end_tag" && child.kind() != "tag_name" {
            if child.kind() == "text" {
                if !contents[child.byte_range()].trim().is_empty() {
                    return false;
                }
            } else {
                return false;
            }
        }
    }
    true
}

fn has_direct_significant_text(node: &tree_sitter::Node, contents: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "text" && !contents[child.byte_range()].trim().is_empty() {
            return true;
        }
    }
    false
}

fn collapse_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            if !in_space {
                result.push(' ');
                in_space = true;
            }
        } else {
            result.push(c);
            in_space = false;
        }
    }
    result
}
