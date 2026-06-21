use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::Backend;

pub async fn goto_definition(
    server: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;
        return resolve_definition_impl(&*db, *ws, &symbol, &uri);
    }

    Ok(None)
}

fn resolve_definition_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    symbol: &str,
    uri: &Url,
) -> Result<Option<GotoDefinitionResponse>> {
    // 1. Try resolve as Hub Instance
    if let Some(instance) = crate::db::resolve_reference(db, ws, symbol.to_string()) {
        let target_uri = Url::from_file_path(instance.file(db).path(db)).unwrap();
        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: target_uri,
            range: instance.range(db).into(),
        })));
    }

    // 2. Try resolve as Hub Type (scoped)
    if let Ok(path) = uri.to_file_path() {
        let path_str = path.to_string_lossy().to_string();
        let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);
        if let Some(file) = file {
            if let Some(hub_type) = crate::db::resolve_type(db, ws, file, symbol.to_string()) {
                let target_uri = Url::from_file_path(hub_type.file(db).path(db)).unwrap();
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: target_uri,
                    range: hub_type.range(db).into(),
                })));
            }
        }
    }

    Ok(None)
}

pub async fn goto_type_definition(
    server: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;
        return resolve_type_definition_impl(&*db, *ws, &symbol, &uri);
    }

    Ok(None)
}

fn resolve_type_definition_impl(
    db: &dyn crate::db::Db,
    ws: crate::db::Workspace,
    symbol: &str,
    uri: &Url,
) -> Result<Option<GotoDefinitionResponse>> {
    // 1. Try resolve as Hub Instance -> return its Type
    if let Some(instance) = crate::db::resolve_reference(db, ws, symbol.to_string()) {
        let type_name = instance.type_name(db);
        if let Some(hub_type) = crate::db::resolve_type(db, ws, instance.file(db), type_name) {
            let target_uri = Url::from_file_path(hub_type.file(db).path(db)).unwrap();
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: target_uri,
                range: hub_type.range(db).into(),
            })));
        }
    }

    // 2. Try resolve as Hub Type -> return itself
    if let Ok(path) = uri.to_file_path() {
        let path_str: String = path.to_string_lossy().to_string();
        let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);
        if let Some(file) = file {
            if let Some(hub_type) = crate::db::resolve_type(db, ws, file, symbol.to_string()) {
                let target_uri = Url::from_file_path(hub_type.file(db).path(db)).unwrap();
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: target_uri,
                    range: hub_type.range(db).into(),
                })));
            }
        }
    }

    Ok(None)
}

pub async fn goto_declaration(
    server: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    goto_definition(server, params).await
}

pub async fn goto_implementation(
    server: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(symbol) = server.get_symbol_at_position(&uri, position) {
        let (db, ws) = server.lock_db().await;

        // 1. Try resolve as Hub Type -> return all its Instances
        if let Ok(path) = uri.to_file_path() {
            let path_str: String = path.to_string_lossy().to_string();
            let file = (*ws)
                .files(&*db)
                .into_iter()
                .find(|f| f.path(&*db) == path_str);
            if let Some(file) = file {
                if let Some(hub_type) = crate::db::resolve_type(&*db, *ws, file, symbol.clone()) {
                    let type_name = hub_type.name(&*db);
                    let instances = crate::db::all_hub_instances(&*db, *ws);
                    let locations: Vec<Location> = instances
                        .into_iter()
                        .filter(|i| i.type_name(&*db) == type_name)
                        .map(|i| {
                            let i_path = i.file(&*db).path(&*db);
                            Location {
                                uri: Url::from_file_path(i_path).unwrap(),
                                range: i.range(&*db).into(),
                            }
                        })
                        .collect();

                    if !locations.is_empty() {
                        return Ok(Some(GotoDefinitionResponse::Array(locations)));
                    }
                }
            }
        }
    }

    Ok(None)
}
