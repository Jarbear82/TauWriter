use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn semantic_tokens_full(
    server: &Backend,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    let uri = params.text_document.uri;

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws
            .files(db)
            .into_iter()
            .find(|f| f.path(db) == path_str);

        if let Some(file) = file {
            return semantic_tokens_impl(db, file);
        }
    }

    Ok(None)
}

fn semantic_tokens_impl(
    db: &dyn crate::db::Db,
    file: crate::db::SourceFile,
) -> Result<Option<SemanticTokensResult>> {
    let mut tokens = crate::db::get_semantic_tokens(db, file);
    tokens.sort_by_key(|t| (t.line, t.character));
    let mut last_line: u32 = 0;
    let mut last_char: u32 = 0;

    let data: Vec<tower_lsp::lsp_types::SemanticToken> = tokens
        .into_iter()
        .map(|t| {
            let delta_line = t.line - last_line;
            let delta_start = if t.line == last_line {
                t.character - last_char
            } else {
                t.character
            };

            last_line = t.line;
            last_char = t.character;

            tower_lsp::lsp_types::SemanticToken {
                delta_line,
                delta_start,
                length: t.length,
                token_type: t.token_type,
                token_modifiers_bitset: t.token_modifiers,
            }
        })
        .collect();

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data,
    })))
}

pub async fn folding_range(
    server: &Backend,
    params: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>> {
    let uri = params.text_document.uri;

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws
            .files(db)
            .into_iter()
            .find(|f| f.path(db) == path_str);

        if let Some(file) = file {
            return folding_range_impl(db, file);
        }
    }

    Ok(None)
}

fn folding_range_impl(
    db: &dyn crate::db::Db,
    file: crate::db::SourceFile,
) -> Result<Option<Vec<FoldingRange>>> {
    let ranges = crate::db::get_folding_ranges(db, file);
    let folding_ranges: Vec<FoldingRange> = ranges
        .into_iter()
        .map(|r| FoldingRange {
            start_line: r.start.line,
            start_character: Some(r.start.character),
            end_line: r.end.line,
            end_character: Some(r.end.character),
            kind: Some(FoldingRangeKind::Region),
            ..Default::default()
        })
        .collect();

    Ok(Some(folding_ranges))
}

pub async fn formatting(
    server: &Backend,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;

    if let Some(content) = server.open_files.get(&uri).map(|r| r.to_string()) {
        let file_type = if uri.as_str().ends_with(".twxml") {
            "twxml"
        } else if uri.as_str().ends_with(".hubgs") {
            "hubgs"
        } else {
            return Ok(None);
        };

        let new_text = crate::formatter::format_source(&content, file_type);
        let last_line_len = content.lines().last().map(|l| l.len()).unwrap_or(0) as u32;
        let line_count = content.lines().count() as u32;
        let end_line = if line_count > 0 { line_count - 1 } else { 0 };

        let edit = TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: end_line,
                    character: last_line_len,
                },
            },
            new_text,
        };

        return Ok(Some(vec![edit]));
    }

    Ok(None)
}

/// Auto-close TWXML tags when the user types `>`.
pub async fn on_type_formatting(
    server: &Backend,
    params: DocumentOnTypeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    if params.ch != ">" {
        return Ok(None);
    }

    let uri = params.text_document_position.text_document.uri;
    if !uri.as_str().ends_with(".twxml") {
        return Ok(None);
    }

    let position = params.text_document_position.position;

    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => return Ok(None),
    };

    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return Ok(None);
    }

    let line = lines[line_idx];

    let Some(tag_name) = extract_opening_tag_name(&line, position.character as usize) else {
        return Ok(None);
    };

    if tag_name == "metadata" {
        return Ok(None);
    }

    let closing = format!("</{}>", tag_name);

    let edit = TextEdit {
        range: Range {
            start: position,
            end: position,
        },
        new_text: closing,
    };

    Ok(Some(vec![edit]))
}

/// Extract the tag name from an opening tag at the end of `text`.
/// Returns `None` for closing tags, self-closing tags, comments,
/// or already-balanced tags on the same line.
fn extract_opening_tag_name(text: &str, cursor: usize) -> Option<String> {
    let text = if cursor <= text.len() {
        &text[..cursor]
    } else {
        text
    };

    let trimmed = text.trim_end();
    if !trimmed.ends_with('>') {
        return None;
    }

    let after_last_lt = trimmed.rfind('<')?;
    let between = &trimmed[after_last_lt..];

    // Skip comments: <!--
    if between.starts_with("<!--") {
        return None;
    }

    // Skip closing tags: </
    if between.starts_with("</") {
        return None;
    }

    // Skip self-closing: <.../>
    if between.ends_with("/>") {
        return None;
    }

    // Skip if line already has a matching closing tag after this opening tag
    let rest = &text[(after_last_lt + between.len())..];
    let tag_name_candidate = extract_name_from_tag(between)?;
    let closing_pattern = format!("</{}>", tag_name_candidate);
    if rest.starts_with(&closing_pattern) {
        return None;
    }

    Some(tag_name_candidate)
}

/// Extract just the tag name from a start tag string like `<section id="1">`.
fn extract_name_from_tag(tag: &str) -> Option<String> {
    let inner = tag.strip_prefix('<')?.strip_suffix('>')?;
    let name = inner
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()?
        .to_string();

    if name.is_empty() {
        return None;
    }

    Some(name)
}

pub async fn document_color(
    server: &Backend,
    params: DocumentColorParams,
) -> Result<Option<Vec<ColorInformation>>> {
    let uri = params.text_document.uri;
    let (db, ws) = server.read_db();

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws.files(&db).into_iter().find(|f| f.path(&db) == path_str);
        if let Some(file) = file {
            if path_str.ends_with(".hubgs") {
                let parse_res = crate::db::parse_hubgs(&db, file);
                let mut colors = Vec::new();
                for inst in parse_res.instances(&db) {
                    if let (Some(bg), Some(range)) = (inst.metadata_background(&db), inst.metadata_background_range(&db)) {
                        if let Some(color) = parse_hex_color(&bg) {
                            colors.push(ColorInformation {
                                range: range.into(),
                                color,
                            });
                        }
                    }
                }
                return Ok(Some(colors));
            }
        }
    }
    Ok(None)
}

pub async fn color_presentation(
    _server: &Backend,
    params: ColorPresentationParams,
) -> Result<Option<Vec<ColorPresentation>>> {
    let new_text = format_hex_color(params.color);
    let presentation = ColorPresentation {
        label: new_text.clone(),
        text_edit: Some(TextEdit {
            range: params.range,
            new_text,
        }),
        additional_text_edits: None,
    };
    Ok(Some(vec![presentation]))
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim().strip_prefix('#')?;
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? as f32 / 15.0;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? as f32 / 15.0;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? as f32 / 15.0;
            Some(Color { red: r, green: g, blue: b, alpha: 1.0 })
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? as f32 / 15.0;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? as f32 / 15.0;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? as f32 / 15.0;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? as f32 / 15.0;
            Some(Color { red: r, green: g, blue: b, alpha: a })
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
            Some(Color { red: r, green: g, blue: b, alpha: 1.0 })
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
            let a = u8::from_str_radix(&s[6..8], 16).ok()? as f32 / 255.0;
            Some(Color { red: r, green: g, blue: b, alpha: a })
        }
        _ => None,
    }
}

fn format_hex_color(color: Color) -> String {
    let r = (color.red * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (color.green * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (color.blue * 255.0).round().clamp(0.0, 255.0) as u8;
    let a = (color.alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    if a == 255 {
        format!("\"#{:02x}{:02x}{:02x}\"", r, g, b)
    } else {
        format!("\"#{:02x}{:02x}{:02x}{:02x}\"", r, g, b, a)
    }
}

pub async fn document_link(
    server: &Backend,
    params: DocumentLinkParams,
) -> Result<Option<Vec<DocumentLink>>> {
    let uri = params.text_document.uri;
    let (db, ws) = server.read_db();

    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => return Ok(None),
    };

    let mut links = Vec::new();

    if uri.as_str().ends_with(".twxml") {
        let language = unsafe { crate::parser::tree_sitter_twxml() };
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language).ok();
        let tree = match parser.parse(&content, None) {
            Some(t) => t,
            None => return Ok(None),
        };

        find_document_links(tree.root_node(), &content, &uri, &db, ws, &mut links);
    } else if uri.as_str().ends_with(".hubgs") {
        let imports = get_hubgs_imports(&content);
        for (path_str, range) in imports {
            if let Ok(current_path) = uri.to_file_path() {
                if let Some(parent) = current_path.parent() {
                    let target_path = parent.join(&path_str);
                    if let Ok(target) = Url::from_file_path(target_path) {
                        links.push(DocumentLink {
                            range,
                            target: Some(target),
                            tooltip: Some(format!("Go to {}", path_str)),
                            data: None,
                        });
                    }
                }
            }
        }
    }

    Ok(Some(links))
}

fn find_document_links(
    node: tree_sitter::Node,
    content: &str,
    uri: &Url,
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    links: &mut Vec<DocumentLink>,
) {
    if node.kind() == "start_tag" || node.kind() == "self_closing_element" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let tag_name = &content[name_node.byte_range()];
            if tag_name == "link" {
                if let Some((href, range)) = get_href_attribute(node, content) {
                    if let Some(target) = resolve_link_target(uri, &href, db, ws) {
                        links.push(DocumentLink {
                            range,
                            target: Some(target),
                            tooltip: Some(format!("Go to {}", href)),
                            data: None,
                        });
                    }
                }
            } else if tag_name == "hubref" {
                if let Some((id_val, id_range)) = get_id_attribute(node, content) {
                    if let Some(target_uri) = resolve_instance_link_target(&id_val, db, ws) {
                        links.push(DocumentLink {
                            range: id_range,
                            target: Some(target_uri),
                            tooltip: Some(format!("Go to instance {}", id_val)),
                            data: None,
                        });
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_document_links(child, content, uri, db, ws, links);
    }
}


fn get_href_attribute(tag_node: tree_sitter::Node, contents: &str) -> Option<(String, Range)> {
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                let attr_name = &contents[name_node.byte_range()];
                let attr_val = contents[val_node.byte_range()]
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if attr_name == "href" {
                    return Some((attr_val, crate::parser::ts_range_to_lsp(val_node.range()).into()));
                }
            }
        }
    }
    None
}

fn get_id_attribute(tag_node: tree_sitter::Node, contents: &str) -> Option<(String, Range)> {
    let mut cursor = tag_node.walk();
    for child in tag_node.children(&mut cursor) {
        if child.kind() == "attribute" {
            if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                let attr_name = &contents[name_node.byte_range()];
                let attr_val = contents[val_node.byte_range()]
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if attr_name == "id" {
                    return Some((attr_val, crate::parser::ts_range_to_lsp(val_node.range()).into()));
                }
            }
        }
    }
    None
}

fn resolve_instance_link_target(id: &str, db: &dyn crate::db::Db, ws: crate::db::Workspace) -> Option<Url> {
    if let Some(instance) = crate::db::resolve_reference(db, ws, id.to_string()) {
        let path = instance.file(db).path(db);
        if let Ok(mut target_uri) = Url::from_file_path(path) {
            let start_line = instance.range(db).start.line + 1; // 1-indexed for fragment
            target_uri.set_fragment(Some(&format!("L{}", start_line)));
            return Some(target_uri);
        }
    }
    None
}

fn resolve_link_target(
    current_uri: &Url,
    href: &str,
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
) -> Option<Url> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return Url::parse(href).ok();
    }

    let parts: Vec<&str> = href.split('#').collect();
    let (target_file_path, anchor_id) = if parts.len() == 2 {
        (if parts[0].is_empty() { None } else { Some(parts[0]) }, Some(parts[1]))
    } else if href.starts_with('#') {
        (None, Some(&href[1..]))
    } else {
        (Some(href), None)
    };

    let mut target_uri = if let Some(path) = target_file_path {
        let current_path = current_uri.to_file_path().ok()?;
        let parent = current_path.parent()?;
        let target_path = parent.join(path);
        Url::from_file_path(target_path).ok()?
    } else {
        current_uri.clone()
    };

    if let Some(anchor) = anchor_id {
        if let Ok(path) = target_uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            if let Some(file) = ws.files(db).into_iter().find(|f| f.path(db) == path_str) {
                let file_contents = file.contents(db);
                if let Some(line) = find_anchor_line(&file_contents, anchor) {
                    target_uri.set_fragment(Some(&format!("L{}", line)));
                }
            }
        }
    }

    Some(target_uri)
}

fn find_anchor_line(contents: &str, anchor: &str) -> Option<u32> {
    let language = unsafe { crate::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(contents, None)?;

    fn walk(node: tree_sitter::Node, contents: &str, anchor: &str) -> Option<u32> {
        if node.kind() == "start_tag" || node.kind() == "self_closing_element" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "attribute" {
                    if let (Some(name_node), Some(val_node)) = (child.child(0), child.child(2)) {
                        let attr_name = &contents[name_node.byte_range()];
                        let attr_val = contents[val_node.byte_range()]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();
                        if (attr_name == "id" || attr_name == "alias" || attr_name == "class") && attr_val == anchor {
                            return Some(node.start_position().row as u32 + 1);
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(line) = walk(child, contents, anchor) {
                return Some(line);
            }
        }
        None
    }

    walk(tree.root_node(), contents, anchor)
}

fn get_hubgs_imports(contents: &str) -> Vec<(String, Range)> {
    let language = unsafe { crate::parser::tree_sitter_hubgs() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).ok();
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut imports = Vec::new();
    fn walk(node: tree_sitter::Node, contents: &str, imports: &mut Vec<(String, Range)>) {
        if node.kind() == "import_statement" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "string" {
                    let path = contents[child.byte_range()]
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                    imports.push((path, crate::parser::ts_range_to_lsp(child.range()).into()));
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            walk(child, contents, imports);
        }
    }
    walk(tree.root_node(), contents, &mut imports);
    imports
}

pub async fn range_formatting(
    server: &Backend,
    params: DocumentRangeFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = params.text_document.uri;
    let range = params.range;

    if let Some(content) = server.open_files.get(&uri).map(|r| r.to_string()) {
        let file_type = if uri.as_str().ends_with(".twxml") {
            "twxml"
        } else if uri.as_str().ends_with(".hubgs") {
            "hubgs"
        } else {
            return Ok(None);
        };

        let formatted = crate::formatter::format_source(&content, file_type);
        if formatted == content {
            return Ok(Some(vec![]));
        }

        let orig_lines: Vec<&str> = content.lines().collect();
        let new_lines: Vec<String> = formatted.lines().map(|s| s.to_string()).collect();

        let mut prefix = 0;
        while prefix < orig_lines.len() && prefix < new_lines.len() && orig_lines[prefix] == new_lines[prefix] {
            prefix += 1;
        }

        let mut suffix = 0;
        while suffix < orig_lines.len() - prefix
            && suffix < new_lines.len() - prefix
            && orig_lines[orig_lines.len() - 1 - suffix] == new_lines[new_lines.len() - 1 - suffix]
        {
            suffix += 1;
        }

        let orig_start_line = prefix as u32;
        let orig_end_line = (orig_lines.len() - suffix) as u32;

        let range_start_line = range.start.line;
        let range_end_line = range.end.line;

        if orig_start_line <= range_end_line && orig_end_line >= range_start_line {
            let start_pos = Position {
                line: orig_start_line,
                character: 0,
            };
            let end_pos = if orig_end_line == 0 {
                Position { line: 0, character: 0 }
            } else if orig_end_line as usize >= orig_lines.len() {
                let last_line = orig_lines.last().copied().unwrap_or("");
                Position {
                    line: (orig_lines.len() - 1) as u32,
                    character: last_line.len() as u32,
                }
            } else {
                Position {
                    line: orig_end_line,
                    character: 0,
                }
            };

            let replacement_lines = &new_lines[prefix..(new_lines.len() - suffix)];
            let mut new_text = replacement_lines.join("\n");
            if (orig_end_line as usize) < orig_lines.len() || (content.ends_with('\n') && orig_end_line as usize == orig_lines.len()) {
                new_text.push('\n');
            }

            let edit = TextEdit {
                range: Range {
                    start: start_pos,
                    end: end_pos,
                },
                new_text,
            };
            return Ok(Some(vec![edit]));
        }

        return Ok(Some(vec![]));
    }

    Ok(None)
}

pub async fn signature_help(
    server: &Backend,
    params: SignatureHelpParams,
) -> Result<Option<SignatureHelp>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => return Ok(None),
    };

    let (db, ws) = server.read_db();

    if uri.as_str().ends_with(".hubgs") {
        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            let file = ws.files(&db).into_iter().find(|f| f.path(&db) == path_str);
            if let Some(file) = file {
                let parse_res = crate::db::parse_hubgs(&db, file);
            let instances = parse_res.instances(&db);

            let cursor_line = position.line;
            let cursor_char = position.character;

            let current_inst = instances.iter().find(|inst| {
                let b_range = inst.block_range(&db);
                let after_start = cursor_line > b_range.start.line
                    || (cursor_line == b_range.start.line && cursor_char >= b_range.start.character);
                let before_end = cursor_line < b_range.end.line
                    || (cursor_line == b_range.end.line && cursor_char <= b_range.end.character);
                after_start && before_end
            });

            if let Some(inst) = current_inst {
                let type_name = inst.type_name(&db);
                if let Some(hub_type) = crate::db::resolve_type(&db, ws, file, type_name) {
                    let (label, parameters) = format_hub_type_signature(&db, &hub_type);

                    let lines: Vec<&str> = content.lines().collect();
                    let mut cursor_idx = 0;
                    for i in 0..(position.line as usize) {
                        if i < lines.len() {
                            cursor_idx += lines[i].len() + 1;
                        }
                    }
                    if (position.line as usize) < lines.len() {
                        cursor_idx += crate::utf16_idx_to_byte_idx(lines[position.line as usize], position.character as usize);
                    }

                    let active_parameter = find_active_parameter(&content, cursor_idx, &parameters);

                    let signature = SignatureInformation {
                        label,
                        documentation: Some(Documentation::String(format!("Hub Type definition for {}", hub_type.name(&db)))),
                        parameters: Some(parameters),
                        active_parameter,
                    };

                    return Ok(Some(SignatureHelp {
                        signatures: vec![signature],
                        active_signature: Some(0),
                        active_parameter,
                    }));
                }
            }
        }
    }
}

    Ok(None)
}

fn format_hub_type_signature(db: &dyn crate::db::Db, hub_type: &crate::db::HubType) -> (String, Vec<ParameterInformation>) {
    let mut params = Vec::new();
    let mut label_parts = Vec::new();

    for field in hub_type.fields(db) {
        let f_label = format!("{}: Value", field.name);
        params.push(ParameterInformation {
            label: ParameterLabel::Simple(f_label.clone()),
            documentation: Some(Documentation::String(format!("Field: {}", field.name))),
        });
        label_parts.push(f_label);
    }

    for role in hub_type.roles(db) {
        let types = role.allowed_types.join(" | ");
        let r_label = format!("{}: {}", role.name, types);
        params.push(ParameterInformation {
            label: ParameterLabel::Simple(r_label.clone()),
            documentation: Some(Documentation::String(format!(
                "Role: {} (Multiplicity: {}, Direction: {})",
                role.name, role.multiplicity, role.direction
            ))),
        });
        label_parts.push(r_label);
    }

    let label = format!("{} {{ {} }}", hub_type.name(db), label_parts.join(", "));
    (label, params)
}

fn find_active_parameter(
    contents: &str,
    cursor_idx: usize,
    params: &[ParameterInformation],
) -> Option<u32> {
    if cursor_idx > contents.len() {
        return None;
    }
    let prefix = &contents[..cursor_idx];
    let mut idx = prefix.len();
    while idx > 0 {
        idx -= 1;
        let c = prefix.as_bytes()[idx];
        if c == b'{' || c == b',' || c == b'\n' {
            let segment = &prefix[idx + 1..];
            if let Some(eq_idx) = segment.find('=') {
                let name = segment[..eq_idx].trim();
                for (i, p) in params.iter().enumerate() {
                    if let ParameterLabel::Simple(ref label) = p.label {
                        if label.starts_with(name) {
                            return Some(i as u32);
                        }
                    }
                }
            }
            break;
        }
    }
    None
}

pub async fn selection_range(
    server: &Backend,
    params: SelectionRangeParams,
) -> Result<Option<Vec<SelectionRange>>> {
    let uri = params.text_document.uri;
    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => return Ok(None),
    };

    let language = if uri.as_str().ends_with(".twxml") {
        unsafe { crate::parser::tree_sitter_twxml() }
    } else if uri.as_str().ends_with(".hubgs") {
        unsafe { crate::parser::tree_sitter_hubgs() }
    } else {
        return Ok(None);
    };

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).ok();
    let tree = match parser.parse(&content, None) {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut selection_ranges = Vec::new();

    for pos in params.positions {
        let lines: Vec<&str> = content.lines().collect();
        let mut byte_idx = 0;
        for i in 0..(pos.line as usize) {
            if i < lines.len() {
                byte_idx += lines[i].len() + 1;
            }
        }
        if (pos.line as usize) < lines.len() {
            byte_idx += crate::utf16_idx_to_byte_idx(lines[pos.line as usize], pos.character as usize);
        }

        let mut node = tree.root_node().descendant_for_byte_range(byte_idx, byte_idx);

        let mut path = Vec::new();
        while let Some(n) = node {
            path.push(n);
            node = n.parent();
        }

        let mut current_range: Option<SelectionRange> = None;
        for n in path.iter().rev() {
            let range = crate::parser::ts_range_to_lsp(n.range()).into();
            current_range = Some(SelectionRange {
                range,
                parent: current_range.map(Box::new),
            });
        }

        if let Some(r) = current_range {
            selection_ranges.push(r);
        }
    }

    Ok(Some(selection_ranges))
}





