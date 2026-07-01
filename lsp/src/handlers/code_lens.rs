use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn code_lens(server: &Backend, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
    let uri = params.text_document.uri;

    let (db_val, ws_val) = server.read_db();
    let db = &db_val;
    let ws = &ws_val;

    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();

        // HubGS: CodeLens on instance definitions and type definitions
        if path_str.ends_with(".hubgs") {
            let file = ws
                .files(db)
                .into_iter()
                .find(|f| f.path(db) == path_str);
            if let Some(file) = file {
                return code_lens_hubgs_impl(db, *ws, file);
            }
        }

        // TWXML: CodeLens on hubref tags
        if path_str.ends_with(".twxml") {
            let file = ws
                .files(db)
                .into_iter()
                .find(|f| f.path(db) == path_str);
            if let Some(file) = file {
                return code_lens_twxml_impl(db, file);
            }
        }
    }

    Ok(None)
}

fn code_lens_hubgs_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    file: crate::db::SourceFile,
) -> Result<Option<Vec<CodeLens>>> {
    let parse_result = crate::db::parse_hubgs(db, file);
    let mut lenses = Vec::new();

    // CodeLens on Hub Instance definitions: "N references"
    for instance in parse_result.instances(db) {
        let name = instance.name(db);
        let refs = crate::db::find_all_references(db, ws, name.clone());
        let count = refs.len();

        let lens = CodeLens {
            range: instance.range(db).into(),
            command: Some(Command {
                title: format!("{} reference{}", count, if count == 1 { "" } else { "s" }),
                command: "editor.action.findReferences".to_string(),
                arguments: None,
            }),
            data: None,
        };
        lenses.push(lens);
    }

    // CodeLens on Hub Type definitions: "N instances"
    for hub_type in parse_result.types(db) {
        let type_name = hub_type.name(db);
        let all_instances = crate::db::all_hub_instances(db, ws);
        let instance_count = all_instances
            .iter()
            .filter(|i| i.type_name(db) == type_name)
            .count();

        let lens = CodeLens {
            range: hub_type.range(db).into(),
            command: Some(Command {
                title: format!(
                    "{} instance{}",
                    instance_count,
                    if instance_count == 1 { "" } else { "s" }
                ),
                command: "editor.action.goToImplementation".to_string(),
                arguments: None,
            }),
            data: None,
        };
        lenses.push(lens);
    }

    Ok(Some(lenses))
}

fn code_lens_twxml_impl(
    db: &dyn crate::db::Db,
    file: crate::db::SourceFile,
) -> Result<Option<Vec<CodeLens>>> {
    let refs = crate::db::parse_twxml(db, file);
    let mut lenses = Vec::new();

    for ref_item in refs {
        let lens = CodeLens {
            range: ref_item.range(db).into(),
            command: Some(Command {
                title: "Go to definition".to_string(),
                command: "editor.action.goToDeclaration".to_string(),
                arguments: None,
            }),
            data: None,
        };
        lenses.push(lens);
    }

    Ok(Some(lenses))
}
