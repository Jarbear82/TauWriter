// Signature help for Hub Types

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

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
                        || (cursor_line == b_range.start.line
                            && cursor_char >= b_range.start.character);
                    let before_end = cursor_line < b_range.end.line
                        || (cursor_line == b_range.end.line
                            && cursor_char <= b_range.end.character);
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
                            cursor_idx += crate::utf16_idx_to_byte_idx(
                                lines[position.line as usize],
                                position.character as usize,
                            );
                        }

                        let active_parameter =
                            find_active_parameter(&content, cursor_idx, &parameters);

                        let signature = SignatureInformation {
                            label,
                            documentation: Some(Documentation::String(format!(
                                "Hub Type definition for {}",
                                hub_type.name(&db)
                            ))),
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

fn format_hub_type_signature(
    db: &dyn crate::db::Db,
    hub_type: &crate::db::HubType,
) -> (String, Vec<ParameterInformation>) {
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
