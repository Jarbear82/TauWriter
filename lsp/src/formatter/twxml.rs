use tree_sitter::Parser;

const MAX_LINE_LEN: usize = 100;

// ponytail: TagBehavior replaces the old runtime content-sniffing (has_direct_significant_text)
// with a static schema lookup, making output independent of input content shape.
// Ceiling: DTD/schema validation would catch misplaced elements before formatting.
// Upgrade: load TagBehavior from a config file to reuse for other TWXML-dialect languages.
#[derive(PartialEq)]
enum TagBehavior {
    /// R1: Always multiline; children are block-level elements.
    ForcedExpandBlock,
    /// R1+R8: Always multiline; children are inline prose.
    ForcedExpandInline,
    /// R2/R9: Inline if content fits within MAX_LINE_LEN, else expands.
    LeafBlock,
    /// R4: Self-closing, own indented line in block context.
    SelfClosingBlock,
    /// R12: Verbatim content preserved.
    CodeBlock,
    /// R5: Inline line-break sentinel.
    Br,
    /// R14: Always inline, never on its own line.
    Fr,
}

fn leading_spaces(s: &str) -> usize {
    s.len() - s.trim_start_matches(' ').len()
}

fn tag_behavior(tag: &str) -> TagBehavior {
    match tag {
        "br" => TagBehavior::Br,
        "fr" => TagBehavior::Fr,
        "codeblock" => TagBehavior::CodeBlock,
        "hr" | "image" | "audio" | "video" | "meta" => TagBehavior::SelfClosingBlock,
        "paragraph" | "aside" | "blockquote" => TagBehavior::ForcedExpandInline,
        "document" | "body" | "section" | "ul" | "ol" | "dl" | "details" | "table" | "tr"
        | "footnote" | "review" => TagBehavior::ForcedExpandBlock,
        // ponytail: These tags were missing from the original tag_behavior map.
        // They were falling through to LeafBlock (correct behavior) but undocumented.
        "heading" | "li" | "dt" | "dd" | "summary" | "bold" | "italic" | "underline"
        | "strikethrough" | "super" | "sub" | "link" | "code" => TagBehavior::LeafBlock,
        _ => TagBehavior::LeafBlock,
    }
}

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

fn format_node(
    node: tree_sitter::Node,
    contents: &str,
    indent: usize,
    block_indent: Option<usize>,
) -> String {
    let ind_str = "  ".repeat(indent);
    match node.kind() {
        "text" => {
            // In block context, whitespace-only text is dropped.
            if block_indent.is_some() {
                collapse_whitespace(&contents[node.byte_range()])
            } else {
                String::new()
            }
        }
        "comment" => {
            // R11: always own indented line, regardless of context.
            format!("{}{}\n", ind_str, contents[node.byte_range()].trim())
        }
        "element" | "document_block" | "body_block" | "self_closing_element" => {
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
    let ind_str = "  ".repeat(indent);
    let attr_str = format_attrs(&attrs, &tag_name, indent);

    match tag_behavior(&tag_name) {
        // ── R12: verbatim codeblock ──────────────────────────────────────────
        // ── R12: verbatim codeblock ──────────────────────────────────────────
        TagBehavior::CodeBlock => {
            let mut inner = String::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "text" {
                    inner.push_str(&contents[child.byte_range()]);
                }
            }
            if inner.trim().is_empty() {
                return format!("{}<{}{}></{}>", ind_str, tag_name, attr_str, tag_name);
            }

            // Strip the surrounding blank lines produced by the format string
            // ("\n" after the opening tag, and "\n{ind_str}" before the closing tag).
            // Whatever non-empty lines remain are the raw code content.
            let all_lines: Vec<&str> = inner.split('\n').collect();
            let start = all_lines
                .iter()
                .position(|l| !l.trim().is_empty())
                .unwrap_or(0);
            let end = all_lines
                .iter()
                .rposition(|l| !l.trim().is_empty())
                .unwrap_or(start);
            let content_lines = &all_lines[start..=end];

            // Compute signed shift: how many spaces to add (positive) or remove (negative)
            // from every line so that line 0 lands exactly at indent+1.
            let target_indent = "  ".repeat(indent + 1);
            let first_leading = leading_spaces(content_lines[0]);
            let shift: i64 = target_indent.len() as i64 - first_leading as i64;

            let mut padded = String::new();
            for (i, line) in content_lines.iter().enumerate() {
                if i > 0 {
                    padded.push('\n');
                }
                if !line.trim().is_empty() {
                    let new_spaces = (leading_spaces(line) as i64 + shift).max(0) as usize;
                    padded.push_str(&" ".repeat(new_spaces));
                    padded.push_str(line.trim_start_matches(' '));
                }
                // Blank interior lines: emit nothing — the '\n' above is enough.
            }

            format!(
                "{}<{}{}>\n{}\n{}</{}>",
                ind_str, tag_name, attr_str, padded, ind_str, tag_name
            )
        }

        // ── R5: <br/> inline line-break ─────────────────────────────────────
        // In inline context, br is handled as a None sentinel by build_inline_atoms.
        // This arm is only reached when br appears as a direct block-level child.
        TagBehavior::Br => {
            if let Some(lvl) = block_indent {
                format!("<br/>\n{}", "  ".repeat(lvl + 1))
            } else {
                format!("{}<br/>", ind_str)
            }
        }

        // ── R14: <fr/> stays inline in all contexts ──────────────────────────
        TagBehavior::Fr => {
            let fr_attrs = attrs_str_simple(&attrs);
            if block_indent.is_some() {
                format!("<fr{}/>", fr_attrs)
            } else {
                format!("{}<fr{}/>", ind_str, fr_attrs)
            }
        }

        // ── R4: self-closing block tags (hr, image, audio, video) ────────────
        TagBehavior::SelfClosingBlock => {
            if block_indent.is_some() {
                format!("<{}{}/>", tag_name, attr_str)
            } else {
                format!("{}<{}{}/>", ind_str, tag_name, attr_str)
            }
        }

        behavior => {
            // Self-closing grammar nodes not in the explicit SelfClosingBlock list
            // (e.g. <meta/>) still need self-closing treatment.
            if node.kind() == "self_closing_element" {
                return if block_indent.is_some() {
                    format!("<{}{}/>", tag_name, attr_str)
                } else {
                    format!("{}<{}{}/>", ind_str, tag_name, attr_str)
                };
            }

            match behavior {
                // ── R1: always-expand block container ─────────────────────
                TagBehavior::ForcedExpandBlock => {
                    if is_node_empty_of_content(&node, contents) {
                        return format!("{}<{}{}></{}>", ind_str, tag_name, attr_str, tag_name);
                    }
                    let mut children_out: Vec<String> = Vec::new();
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if matches!(child.kind(), "start_tag" | "end_tag" | "tag_name" | "text") {
                            continue;
                        }
                        let c_str = format_node(child, contents, indent + 1, None);
                        if !c_str.trim().is_empty() {
                            children_out.push(c_str);
                        }
                    }
                    if children_out.is_empty() {
                        return format!("{}<{}{}></{}>", ind_str, tag_name, attr_str, tag_name);
                    }
                    let mut out = format!("{}<{}{}>\n", ind_str, tag_name, attr_str);
                    for c in &children_out {
                        out.push_str(c);
                        if !c.ends_with('\n') {
                            out.push('\n');
                        }
                    }
                    out.push_str(&format!("{}</{}>", ind_str, tag_name));
                    out
                }

                // ── R1+R8: always-expand prose container ──────────────────
                TagBehavior::ForcedExpandInline => {
                    if is_node_empty_of_content(&node, contents) {
                        return format!("{}<{}{}></{}>", ind_str, tag_name, attr_str, tag_name);
                    }
                    let atoms = build_inline_atoms(node, contents, indent + 1);
                    let content = join_atoms_with_wrap(&atoms, MAX_LINE_LEN, indent + 1);
                    if content.is_empty() {
                        return format!("{}<{}{}></{}>", ind_str, tag_name, attr_str, tag_name);
                    }
                    format!(
                        "{}<{}{}>\n{}\n{}</{}>",
                        ind_str, tag_name, attr_str, content, ind_str, tag_name
                    )
                }

                // ── R2/R9: inline-if-fits, else expand ────────────────────
                TagBehavior::LeafBlock => {
                    if is_node_empty_of_content(&node, contents) {
                        let compact = format!("<{}{}></{}>", tag_name, attr_str, tag_name);
                        return if block_indent.is_some() {
                            compact
                        } else {
                            format!("{}{}", ind_str, compact)
                        };
                    }
                    let child_indent = block_indent.unwrap_or(indent);
                    let atoms = build_inline_atoms(node, contents, child_indent);
                    let flat = join_atoms_flat(&atoms);

                    if block_indent.is_some() {
                        // Inside inline context — render as a flat inline tag.
                        format!("<{}{}>{}</{}>", tag_name, attr_str, flat, tag_name)
                    } else {
                        // Block context: R2 measure excludes indentation prefix.
                        let single = format!("<{}{}>{}</{}>", tag_name, attr_str, flat, tag_name);
                        if single.len() <= MAX_LINE_LEN && !flat.contains('\n') {
                            format!("{}{}", ind_str, single)
                        } else {
                            let wrapped = join_atoms_with_wrap(&atoms, MAX_LINE_LEN, indent + 1);
                            format!(
                                "{}<{}{}>\n{}\n{}</{}>",
                                ind_str, tag_name, attr_str, wrapped, ind_str, tag_name
                            )
                        }
                    }
                }

                _ => unreachable!(),
            }
        }
    }
}

/// Collect inline content as a flat list of atomic units for word-wrapping.
/// `None` entries mark R5 `<br/>` line-break positions.
/// Text nodes are split into individual words; child elements become single atoms.
/// Comments are skipped — they are structurally meaningless inside inline content.
fn build_inline_atoms(
    node: tree_sitter::Node,
    contents: &str,
    child_indent: usize,
) -> Vec<Option<String>> {
    let mut atoms: Vec<Option<String>> = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "start_tag" | "end_tag" | "tag_name" | "comment" => continue,
            "text" => {
                for word in collapse_whitespace(&contents[child.byte_range()])
                    .split_whitespace()
                    .map(str::to_owned)
                {
                    atoms.push(Some(word));
                }
            }
            "element"
            | "self_closing_element"
            | "document_block"
            | "body_block" => {
                let tag = get_tag_name(&child, contents).unwrap_or_default();
                match tag.as_str() {
                    "br" => atoms.push(None),
                    "fr" => {
                        let fr_attrs = get_attributes(&child, contents);
                        atoms.push(Some(format!("<fr{}/>", attrs_str_simple(&fr_attrs))));
                    }
                    _ => {
                        let t = format_node(child, contents, 0, Some(child_indent))
                            .trim()
                            .to_string();
                        if !t.is_empty() {
                            atoms.push(Some(t));
                        }
                    }
                }
            }
            _ => {
                let t = format_node(child, contents, 0, Some(child_indent))
                    .trim()
                    .to_string();
                if !t.is_empty() {
                    atoms.push(Some(t));
                }
            }
        }
    }
    atoms
}

/// Join atoms into a single flat string without line-breaking.
/// Used when rendering an inline tag nested inside another inline context.
/// `None` atoms (`<br/>`) are ignored — the outer prose container handles them.
fn join_atoms_flat(atoms: &[Option<String>]) -> String {
    let mut out = String::new();
    for atom in atoms.iter().flatten() {
        if out.is_empty() || atom.starts_with(['.', ',', ':', ';', '!', '?']) {
            out.push_str(atom);
        } else {
            out.push(' ');
            out.push_str(atom);
        }
    }
    out
}

/// Join atoms with greedy word-wrapping at `max_len` characters (R8).
/// `None` atoms (from `<br/>`) force a hard line break (R5).
/// Each output line is prefixed with `content_indent` levels of indentation.
fn join_atoms_with_wrap(atoms: &[Option<String>], max_len: usize, content_indent: usize) -> String {
    let ind = "  ".repeat(content_indent);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for atom in atoms {
        match atom {
            None => {
                // R5: hard line break at <br/>.
                if !current.is_empty() {
                    lines.push(format!("{}{}", ind, current));
                    current.clear();
                }
            }
            Some(s) => {
                let needs_space =
                    !current.is_empty() && !s.starts_with(['.', ',', ':', ';', '!', '?']);
                let extra = usize::from(needs_space);
                if current.is_empty() {
                    current.push_str(s);
                } else if current.len() + extra + s.len() <= max_len {
                    if needs_space {
                        current.push(' ');
                    }
                    current.push_str(s);
                } else {
                    lines.push(format!("{}{}", ind, current));
                    current = s.clone();
                }
            }
        }
    }
    if !current.is_empty() {
        lines.push(format!("{}{}", ind, current));
    }
    lines.join("\n")
}

/// Format element attributes per R7.
/// Returns the string to insert between the tag name and its closing bracket.
///
/// Inline:   `" attr1="v" attr2="v"`  — caller appends `>` or `/>` directly.
/// Expanded: `"\n  attr1="v"\n  attr2="v"\n{ind}"` — caller appends `>` on its own line.
fn format_attrs(attrs: &[String], tag_name: &str, indent: usize) -> String {
    if attrs.is_empty() {
        return String::new();
    }
    let inline = format!(" {}", attrs.join(" "));
    let ind_str = "  ".repeat(indent);
    // Full line: indentation + "<" + tag_name + attrs + ">"
    if ind_str.len() + 1 + tag_name.len() + inline.len() + 1 <= MAX_LINE_LEN {
        return inline;
    }
    // R7: one attr per line at indent+1; closing ">" on its own line at indent.
    let mut out = String::new();
    for attr in attrs {
        out.push_str(&format!("\n{}{}", "  ".repeat(indent + 1), attr));
    }
    out.push('\n');
    out.push_str(&ind_str);
    out
}

/// Simple inline attribute string with no length check.
/// Used for `<fr/>` (always inline) and other unconditional self-closing tags.
fn attrs_str_simple(attrs: &[String]) -> String {
    if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
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
            "meta_tag" => Some("meta".to_string()),
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
        if matches!(child.kind(), "start_tag" | "end_tag" | "tag_name") {
            continue;
        }
        if child.kind() == "text" {
            if !contents[child.byte_range()].trim().is_empty() {
                return false;
            }
        } else {
            return false;
        }
    }
    true
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
