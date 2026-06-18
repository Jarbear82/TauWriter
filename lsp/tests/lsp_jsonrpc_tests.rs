use dashmap::DashMap;
use futures::StreamExt;
use salsa::prelude::*;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauwriter_lsp::{Backend, RootDatabase};
use tower::Service;
use tower_lsp::jsonrpc::{Id, Request};
use tower_lsp::lsp_types::*;
use tower_lsp::LspService;

#[tokio::test]
async fn test_initialize_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let (mut service, _) = LspService::new(|client| Backend {
        client,
        db: Arc::new(Mutex::new(db)),
        workspace_input: Arc::new(Mutex::new(workspace_input)),
        open_files: Arc::new(DashMap::new()),
    });

    let initialize_params = InitializeParams::default();
    let request = Request::build("initialize")
        .id(1)
        .params(json!(initialize_params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");

    assert_eq!(response.id(), &Id::Number(1));
    let result = response.result().unwrap();
    assert!(result.get("capabilities").is_some());
}

#[tokio::test]
async fn test_did_open_did_close_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());
    let open_files = Arc::new(DashMap::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: Arc::new(Mutex::new(db)),
        workspace_input: Arc::new(Mutex::new(workspace_input)),
        open_files: open_files.clone(),
    });

    tokio::spawn(async move { while let Some(_) = socket.next().await {} });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let uri = Url::parse("file:///test.hubgs").unwrap();
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: "INSTANCES [ aragorn:Person {} ]".to_string(),
        },
    };

    let _ = service
        .call(
            Request::build("textDocument/didOpen")
                .params(json!(did_open_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(open_files.contains_key(&uri));

    let did_close_params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    };
    let _ = service
        .call(
            Request::build("textDocument/didClose")
                .params(json!(did_close_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!open_files.contains_key(&uri));
}

#[tokio::test]
async fn test_document_symbol_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));
    let open_files = Arc::new(DashMap::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: open_files.clone(),
    });

    tokio::spawn(async move { while let Some(_) = socket.next().await {} });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let path = std::env::current_dir().unwrap().join("test_symbols.hubgs");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "
DEFINITIONS [ HUBS [ Person { name } ] ],
INSTANCES [ aragorn:Person { name = 'Aragorn' } ]
";

    {
        let mut db_lock = db_arc.lock().unwrap();
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().unwrap();
        ws.set_files(&mut *db_lock).to(vec![source_file]);
    }

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: content.to_string(),
        },
    };
    let _ = service
        .call(
            Request::build("textDocument/didOpen")
                .params(json!(did_open_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/documentSymbol")
        .id(2)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: DocumentSymbolResponse =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    if let DocumentSymbolResponse::Flat(symbols) = result {
        let names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"Person".to_string()));
        assert!(names.contains(&"aragorn".to_string()));
    } else {
        panic!("Expected Flat DocumentSymbolResponse");
    }
}

#[tokio::test]
async fn test_document_highlight_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));
    let open_files = Arc::new(DashMap::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: open_files.clone(),
    });

    tokio::spawn(async move { while let Some(_) = socket.next().await {} });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let path = std::env::current_dir()
        .unwrap()
        .join("test_highlight.hubgs");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "
DEFINITIONS [ HUBS [ Person { name } ] ],
INSTANCES [ aragorn:Person { name = 'Aragorn' }, gandalf:Person { name = 'Gandalf', friend = aragorn } ]
";

    {
        let mut db_lock = db_arc.lock().unwrap();
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().unwrap();
        ws.set_files(&mut *db_lock).to(vec![source_file]);
    }

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: content.to_string(),
        },
    };
    let _ = service
        .call(
            Request::build("textDocument/didOpen")
                .params(json!(did_open_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let params = DocumentHighlightParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 2,
                character: 15,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/documentHighlight")
        .id(3)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Vec<DocumentHighlight> =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_type_definition_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));
    let open_files = Arc::new(DashMap::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: open_files.clone(),
    });

    tokio::spawn(async move { while let Some(_) = socket.next().await {} });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let path = std::env::current_dir().unwrap().join("test_type_def.hubgs");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "
DEFINITIONS [ HUBS [ Person { name } ] ],
INSTANCES [ aragorn:Person { name = 'Aragorn' } ]
";

    {
        let mut db_lock = db_arc.lock().unwrap();
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().unwrap();
        ws.set_files(&mut *db_lock).to(vec![source_file]);
    }

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: content.to_string(),
        },
    };
    let _ = service
        .call(
            Request::build("textDocument/didOpen")
                .params(json!(did_open_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 2,
                character: 15,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/typeDefinition")
        .id(4)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: GotoDefinitionResponse =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    if let GotoDefinitionResponse::Scalar(location) = result {
        assert_eq!(location.range.start.line, 1);
    } else {
        panic!("Expected Scalar GotoDefinitionResponse");
    }
}

#[tokio::test]
async fn test_implementation_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));
    let open_files = Arc::new(DashMap::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: open_files.clone(),
    });

    tokio::spawn(async move { while let Some(_) = socket.next().await {} });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let path = std::env::current_dir().unwrap().join("test_impl.hubgs");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "
DEFINITIONS [ HUBS [ Person { name } ] ],
INSTANCES [ aragorn:Person { name = 'Aragorn' }, gandalf:Person { name = 'Gandalf' } ]
";

    {
        let mut db_lock = db_arc.lock().unwrap();
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().unwrap();
        ws.set_files(&mut *db_lock).to(vec![source_file]);
    }

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: content.to_string(),
        },
    };
    let _ = service
        .call(
            Request::build("textDocument/didOpen")
                .params(json!(did_open_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Call implementation on 'Person' type on line 1
    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 1,
                character: 25, // 'Person'
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/implementation")
        .id(5)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: GotoDefinitionResponse =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    if let GotoDefinitionResponse::Array(locations) = result {
        assert_eq!(locations.len(), 2); // aragorn and gandalf
    } else {
        panic!("Expected Array GotoDefinitionResponse");
    }
}

#[tokio::test]
async fn test_formatting_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));
    let open_files = Arc::new(DashMap::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: open_files.clone(),
    });

    tokio::spawn(async move { while let Some(_) = socket.next().await {} });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let path = std::env::current_dir().unwrap().join("test_format.hubgs");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "INSTANCES [ aragorn:Person { name = 'Aragorn' } ]    "; // Trailing spaces

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: content.to_string(),
        },
    };
    let _ = service
        .call(
            Request::build("textDocument/didOpen")
                .params(json!(did_open_params))
                .finish(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let request = Request::build("textDocument/formatting")
        .id(6)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Vec<TextEdit> = serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].new_text,
        "INSTANCES [ aragorn:Person { name = 'Aragorn' } ]\n"
    );
}
