// Color and link related features

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser;
use crate::Backend;

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
                    if let (Some(bg), Some(range)) = (
                        crate::db::resolution::hub_instance_metadata_background(&db, ws, inst),
                        crate::db::resolution::hub_instance_metadata_background_range(
                            &db, ws, inst,
                        ),
                    ) {
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
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: 1.0,
            })
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? as f32 / 15.0;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? as f32 / 15.0;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? as f32 / 15.0;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? as f32 / 15.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: a,
            })
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: 1.0,
            })
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
            let a = u8::from_str_radix(&s[6..8], 16).ok()? as f32 / 255.0;
            Some(Color {
                red: r,
                green: g,
                blue: b,
                alpha: a,
            })
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
        let language = match parser::get_language("twxml") {
            Some(lang) => lang,
            None => return Ok(None),
        };
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).ok();
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
                if let Some((href, range)) =
                    crate::parser::get_attribute(node, content, |name| name == "href")
                {
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
                if let Some((id_val, id_range)) =
                    crate::parser::get_attribute(node, content, |name| name == "id")
                {
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

fn resolve_instance_link_target(
    id: &str,
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
) -> Option<Url> {
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
        (
            if parts[0].is_empty() {
                None
            } else {
                Some(parts[0])
            },
            Some(parts[1]),
        )
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
    let language = match parser::get_language("twxml") {
        Some(lang) => lang,
        None => return None,
    };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok();
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return None,
    };

    fn walk(node: tree_sitter::Node, contents: &str, anchor: &str) -> Option<u32> {
        if node.kind() == "start_tag" || node.kind() == "self_closing_element" {
            // Use shared utility to find matching attribute, then check its value against the anchor.
            let attrs = crate::parser::get_all_attributes(node, contents, |name| {
                name == "id" || name == "alias" || name == "class"
            });
            if attrs.iter().any(|(val, _)| val == anchor) {
                return Some(node.start_position().row as u32 + 1);
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
    let language = match parser::get_language("hubgs") {
        Some(lang) => lang,
        None => return Vec::new(), // Graceful degradation
    };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok();
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
