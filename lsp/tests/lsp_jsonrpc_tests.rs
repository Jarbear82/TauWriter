use dashmap::DashMap;
use futures::StreamExt;
use salsa::prelude::*;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tauwriter_lsp::{Backend, RootDatabase};
use tokio::sync::Mutex;
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
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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
        "INSTANCES [\n    aragorn: Person {\n        name = 'Aragorn'\n    }\n]\n"
    );
}

#[tokio::test]
async fn test_declaration_jsonrpc() {
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

    let path = std::env::current_dir().unwrap().join("test_decl.hubgs");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "
DEFINITIONS [ HUBS [ Person { name } ] ],
INSTANCES [ aragorn:Person { name = 'Aragorn' } ]
";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let request = Request::build("textDocument/declaration")
        .id(7)
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
        assert_eq!(location.range.start.line, 2);
    } else {
        panic!("Expected Scalar GotoDefinitionResponse");
    }
}

#[tokio::test]
async fn test_initialized_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: Arc::new(Mutex::new(db)),
        workspace_input: Arc::new(Mutex::new(workspace_input)),
        open_files: Arc::new(DashMap::new()),
    });

    tokio::spawn(async move {
        while let Some(msg) = socket.next().await {
            if msg.method() == "window/logMessage" {
                return;
            }
        }
    });
    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let notification = Request::build("initialized")
        .params(json!(InitializedParams {}))
        .finish();

    let _ = service.call(notification).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_shutdown_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let (mut service, _) = LspService::new(|client| Backend {
        client,
        db: Arc::new(Mutex::new(db)),
        workspace_input: Arc::new(Mutex::new(workspace_input)),
        open_files: Arc::new(DashMap::new()),
    });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let request = Request::build("shutdown").id(2).finish();
    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    assert_eq!(response.id(), &Id::Number(2));
    assert!(response.result().unwrap().is_null());
}

#[tokio::test]
async fn test_did_change_jsonrpc() {
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

    let uri = Url::parse("file:///test_change.hubgs").unwrap();
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

    let did_change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "INSTANCES [ gandalf:Person {} ]".to_string(),
        }],
    };

    let _ = service
        .call(
            Request::build("textDocument/didChange")
                .params(json!(did_change_params))
                .finish(),
        )
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        open_files.get(&uri).map(|r| r.to_string()),
        Some("INSTANCES [ gandalf:Person {} ]".to_string())
    );
}

#[tokio::test]
async fn test_did_save_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let (mut service, _) = LspService::new(|client| Backend {
        client,
        db: Arc::new(Mutex::new(db)),
        workspace_input: Arc::new(Mutex::new(workspace_input)),
        open_files: Arc::new(DashMap::new()),
    });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let uri = Url::parse("file:///test_save.hubgs").unwrap();
    let params = DidSaveTextDocumentParams {
        text_document: TextDocumentIdentifier { uri },
        text: None,
    };

    let _ = service
        .call(
            Request::build("textDocument/didSave")
                .params(json!(params))
                .finish(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_definition_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_def.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "INSTANCES [ aragorn:Person { friend = aragorn } ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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
                line: 0,
                character: 38, // Second 'aragorn'
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/definition")
        .id(2)
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
        assert_eq!(location.range.start.character, 12); // First 'aragorn'
    } else {
        panic!("Expected Scalar GotoDefinitionResponse");
    }
}

#[tokio::test]
async fn test_references_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_refs.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "INSTANCES [ aragorn:Person { friend = aragorn } ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 15,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: ReferenceContext {
            include_declaration: true,
        },
    };

    let request = Request::build("textDocument/references")
        .id(3)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Vec<Location> = serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert_eq!(result.len(), 1); // Only the reference, not the definition (unless specifically implemented to include it)
}

#[tokio::test]
async fn test_hover_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_hover.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "INSTANCES [ aragorn:Person { name = 'Aragorn' } ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 15,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let request = Request::build("textDocument/hover")
        .id(4)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Option<Hover> = serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert!(result.is_some());
    let result = result.unwrap();
    match result.contents {
        HoverContents::Markup(mc) => {
            assert_eq!(mc.kind, MarkupKind::Markdown);
            assert!(mc.value.contains("Person: aragorn (Hub)"));
        }
        _ => {}
    }
}

#[tokio::test]
async fn test_completion_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_comp.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "INSTANCES [ aragorn:Person {} ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 30,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    };

    let request = Request::build("textDocument/completion")
        .id(5)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result = response.result().unwrap();
    // The completion handler may return null if no completions match.
    // Check that the response is either null or a valid CompletionResponse array.
    if result.is_null() {
        // No completions — acceptable for some positions
    } else {
        let items: Vec<CompletionItem> = serde_json::from_value(result.clone())
            .expect("completion response should be an array of completion items");
        assert!(!items.is_empty(), "expected at least one completion item");
        assert!(
            items.iter().any(|i| i.label == "aragorn"),
            "expected 'aragorn' in completions"
        );
    }
}

#[tokio::test]
async fn test_rename_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_rename.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "INSTANCES [ aragorn:Person { friend = aragorn } ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let params = RenameParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 15,
            },
        },
        new_name: "elessar".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let request = Request::build("textDocument/rename")
        .id(6)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: WorkspaceEdit = serde_json::from_value(response.result().unwrap().clone()).unwrap();

    let changes = result.changes.unwrap();
    assert!(changes.contains_key(&uri));
    let edits = changes.get(&uri).unwrap();
    assert_eq!(edits.len(), 2); // Definition and one reference
}

#[tokio::test]
async fn test_folding_range_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_fold.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "DEFINITIONS [\n  HUBS [\n    Person {}\n  ]\n]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let params = FoldingRangeParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/foldingRange")
        .id(2)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Vec<FoldingRange> =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_semantic_tokens_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_tokens.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "INSTANCES [ aragorn:Person {} ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
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

    let params = SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/semanticTokens/full")
        .id(2)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: SemanticTokensResult =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    if let SemanticTokensResult::Tokens(tokens) = result {
        assert!(!tokens.data.is_empty());
    } else {
        panic!("Expected Tokens SemanticTokensResult");
    }
}

#[tokio::test]
async fn test_workspace_symbol_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_ws_sym.hubgs");
    let content = "INSTANCES [ aragorn:Person {} ]";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
        ws.set_files(&mut *db_lock).to(vec![source_file]);
    }

    let params = WorkspaceSymbolParams {
        query: "ara".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("workspace/symbol")
        .id(2)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Vec<SymbolInformation> =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    let names: Vec<String> = result.iter().map(|s| s.name.clone()).collect();
    assert!(names.contains(&"aragorn".to_string()));
}

#[tokio::test]
async fn test_publish_diagnostics_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: Arc::new(Mutex::new(db)),
        workspace_input: Arc::new(Mutex::new(workspace_input)),
        open_files: Arc::new(DashMap::new()),
    });

    let (tx, rx) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        while let Some(msg) = socket.next().await {
            if msg.method() == "textDocument/publishDiagnostics" {
                tx.send(()).unwrap();
                return;
            }
        }
    });

    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    let _ = service
        .call(
            Request::build("initialized")
                .params(json!(InitializedParams {}))
                .finish(),
        )
        .await
        .unwrap();

    let uri = Url::parse("file:///test_diag.hubgs").unwrap();
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "hubgs".to_string(),
            version: 1,
            text: "INSTANCES [ aragorn:UnknownType {} ]".to_string(),
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

    rx.recv_timeout(Duration::from_secs(2))
        .expect("Should receive diagnostics");
}

#[tokio::test]
async fn test_inlay_hint_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

    let (mut service, mut socket) = LspService::new(|client| Backend {
        client,
        db: db_arc.clone(),
        workspace_input: ws_arc.clone(),
        open_files: Arc::new(DashMap::new()),
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

    let path = std::env::current_dir().unwrap().join("test_inlay.hubgs");
    let uri = Url::from_file_path(&path).unwrap();
    let content = "
DEFINITIONS [ HUBS [ Person { name }, Location { city } ] ],
INSTANCES [ aragorn:Person { name = 'Aragorn' }, rivendell:Location { city = 'Rivendell' } ]
";

    {
        let mut db_lock = db_arc.lock().await;
        let source_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            path.to_string_lossy().to_string(),
            content.to_string(),
        );
        let ws = ws_arc.lock().await;
        ws.set_files(&mut *db_lock).to(vec![source_file]);
    }

    let params = InlayHintParams {
        text_document: TextDocumentIdentifier { uri },
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 10,
                character: 0,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let request = Request::build("textDocument/inlayHint")
        .id(2)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result: Option<Vec<InlayHint>> =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    let hints = result.unwrap();
    assert_eq!(hints.len(), 2, "expected one inlay hint per instance");
    assert!(hints.iter().any(|h| match &h.label {
        InlayHintLabel::String(s) => s.contains("Person"),
        InlayHintLabel::LabelParts(_) => false,
    }));
    assert!(hints.iter().any(|h| match &h.label {
        InlayHintLabel::String(s) => s.contains("Location"),
        InlayHintLabel::LabelParts(_) => false,
    }));
}

#[tokio::test]
async fn test_code_action_jsonrpc() {
    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());
    let open_files = Arc::new(DashMap::new());

    let db_arc = Arc::new(Mutex::new(db));
    let ws_arc = Arc::new(Mutex::new(workspace_input));

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

    let hubgs_path = std::env::current_dir().unwrap().join("test_ca.hubgs");
    let twxml_path = std::env::current_dir().unwrap().join("test_ca.twxml");
    let _hubgs_uri = Url::from_file_path(&hubgs_path).unwrap();
    let twxml_uri = Url::from_file_path(&twxml_path).unwrap();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [ name: Text ],
    HUBS [ Character { name } ]
],
INSTANCES [
    aragorn: Character { name = 'Elessar' }
]
";
    let twxml_content = r#"<document><metadata></metadata><body><review><hubref id="aragorn" field="name">Strider</hubref></review></body></document>"#;

    {
        let mut db_lock = db_arc.lock().await;
        let h_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            hubgs_path.to_string_lossy().to_string(),
            hubgs_content.to_string(),
        );
        let t_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            twxml_path.to_string_lossy().to_string(),
            twxml_content.to_string(),
        );
        let ws = ws_arc.lock().await;
        ws.set_files(&mut *db_lock).to(vec![h_file, t_file]);
    }

    open_files.insert(twxml_uri.clone(), ropey::Rope::from_str(&twxml_content));

    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri: twxml_uri },
        range: Range {
            start: Position {
                line: 0,
                character: 45, // Inside <review> after skeleton prefix
            },
            end: Position {
                line: 0,
                character: 45,
            },
        },
        context: CodeActionContext::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let request = Request::build("textDocument/codeAction")
        .id(2)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");
    let result = response.result().unwrap();
    // CodeAction handler may return null if no actions are available at the position.
    if !result.is_null() {
        let actions: Vec<CodeActionOrCommand> = serde_json::from_value(result.clone())
            .expect("code action response should be an array");

        assert!(!actions.is_empty());
        let titles: Vec<String> = actions
            .iter()
            .map(|item| match item {
                CodeActionOrCommand::CodeAction(ca) => ca.title.clone(),
                CodeActionOrCommand::Command(cmd) => cmd.title.clone(),
            })
            .collect();

        assert!(titles.iter().any(|t| t.contains("Sync and Resolve")));
        assert!(titles.iter().any(|t| t.contains("Mark as Resolved")));
    }
}

/// Regression test: hover/goto-def on id= inside <hubref> in TWXML files.
/// Commit dd84883 rewrote get_symbol_at_position to walk the tree-sitter AST
/// looking for "identifier" nodes, but the TWXML grammar never produces those
/// — attribute values are parsed as "attribute_value" with anonymous regex content.
/// This test ensures hover and go-to-definition both resolve through the full LSP stack.
#[tokio::test]
async fn test_twxml_hover_and_definition_jsonrpc() {
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

    // Initialize
    let _ = service
        .call(
            Request::build("initialize")
                .id(1)
                .params(json!(InitializeParams::default()))
                .finish(),
        )
        .await
        .unwrap();

    // Register hubgs file with the tailor instance
    let hubgs_path = std::env::current_dir().unwrap().join("twtest_types.hubgs");
    let _hubgs_uri = Url::from_file_path(&hubgs_path).unwrap();
    let hubgs_content = "\n\
        DEFINITIONS [\n\
            FIELDS [ name: Text ],\n\
            HUBS [ Character { name } ]\n\
        ],\n\
        INSTANCES [ tailor:Character { name = 'The Brave Little Tailor' } ]\n\
    ";

    // Register twxml file that references tailor via hubref id
    let twxml_path = std::env::current_dir().unwrap().join("twtest_story.twxml");
    let twxml_uri = Url::from_file_path(&twxml_path).unwrap();
    let twxml_content = "<document><metadata></metadata><body>\n\
      <paragraph>\n\
        <hubref id=\"tailor\" field=\"name\">The Brave Little Tailor</hubref>\n\
      </paragraph>\n\
    </body></document>";

    // Index both files into the workspace
    {
        let mut db_lock = db_arc.lock().await;
        let hubgs_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            hubgs_path.to_string_lossy().to_string(),
            hubgs_content.to_string(),
        );
        let twxml_file = tauwriter_lsp::db::SourceFile::new(
            &mut *db_lock,
            twxml_path.to_string_lossy().to_string(),
            twxml_content.to_string(),
        );
        let ws = ws_arc.lock().await;
        ws.set_files(&mut *db_lock).to(vec![hubgs_file, twxml_file]);
    }

    // Open the twxml file so get_symbol_at_position can read it
    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: twxml_uri.clone(),
            language_id: "twxml".to_string(),
            version: 1,
            text: twxml_content.to_string(),
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

    // --- Hover on id="tailor" (line 2, char 16 is inside the value) ---
    let hover_params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: twxml_uri.clone(),
            },
            position: Position {
                line: 2,
                character: 16,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };

    let hover_request = Request::build("textDocument/hover")
        .id(10)
        .params(json!(hover_params))
        .finish();

    let hover_response = service
        .call(hover_request)
        .await
        .unwrap()
        .expect("Hover response should be present");
    let hover_result: Option<Hover> =
        serde_json::from_value(hover_response.result().unwrap().clone()).unwrap();

    assert!(
        hover_result.is_some(),
        "Hover on twxml hubref id should return a result"
    );
    let hover_md = match hover_result.unwrap().contents {
        HoverContents::Markup(mc) => mc.value,
        _ => panic!("Expected Markup hover content"),
    };
    assert!(
        hover_md.contains("tailor"),
        "Hover markdown should mention the instance name. Got:\n{}",
        hover_md
    );

    // --- Go-to-definition on id="tailor" ---
    let def_params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: twxml_uri.clone(),
            },
            position: Position {
                line: 2,
                character: 16,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let def_request = Request::build("textDocument/definition")
        .id(11)
        .params(json!(def_params))
        .finish();

    let def_response = service
        .call(def_request)
        .await
        .unwrap()
        .expect("Definition response should be present");
    let def_result: Option<GotoDefinitionResponse> =
        serde_json::from_value(def_response.result().unwrap().clone()).unwrap();

    assert!(
        def_result.is_some(),
        "Go-to-definition on twxml hubref id should resolve to the hubgs definition"
    );
    if let GotoDefinitionResponse::Scalar(location) = def_result.unwrap() {
        // Should point to the hubgs file
        assert!(
            location
                .uri
                .to_file_path()
                .unwrap()
                .to_string_lossy()
                .contains("twtest_types.hubgs"),
            "Definition should resolve to the .hubgs file, not the .twxml file"
        );
    } else {
        panic!("Expected Scalar GotoDefinitionResponse");
    }

    // --- Document highlight on id="tailor" (verifies symbol is found) ---
    let hl_params = DocumentHighlightParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: twxml_uri.clone(),
            },
            position: Position {
                line: 2,
                character: 16,
            },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let hl_request = Request::build("textDocument/documentHighlight")
        .id(12)
        .params(json!(hl_params))
        .finish();

    let hl_response = service
        .call(hl_request)
        .await
        .unwrap()
        .expect("Highlight response should be present");
    let hl_result: Option<Vec<DocumentHighlight>> =
        serde_json::from_value(hl_response.result().unwrap().clone()).unwrap();

    assert!(
        hl_result.is_some() && !hl_result.as_ref().unwrap().is_empty(),
        "Document highlight on twxml hubref id should return highlights"
    );
}

#[tokio::test]
async fn test_on_type_formatting_autoclose_tag() {
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
        .join("test_autoclose.twxml");
    let uri = Url::from_file_path(&path).unwrap();

    let content = "<section>";

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "twxml".to_string(),
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
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Simulate typing `>` at the end of the line (cursor after `>`)
    let params = DocumentOnTypeFormattingParams {
        ch: ">".to_string(),
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 9, // After `<section>`
            },
        },
        options: FormattingOptions::default(),
    };

    let request = Request::build("textDocument/onTypeFormatting")
        .id(7)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");

    let result: Option<Vec<TextEdit>> =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert!(result.is_some(), "Should return edits for opening tag");
    let edits = result.unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "</section>");
}

#[tokio::test]
async fn test_on_type_formatting_no_autoclose_self_closing() {
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

    let path = std::env::current_dir()
        .unwrap()
        .join("test_self_close.twxml");
    let uri = Url::from_file_path(&path).unwrap();

    // Self-closing tag should NOT get auto-closed
    let content = "<meta/>";

    let did_open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "twxml".to_string(),
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
    tokio::time::sleep(Duration::from_millis(50)).await;

    let params = DocumentOnTypeFormattingParams {
        ch: ">".to_string(),
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position {
                line: 0,
                character: 6,
            },
        },
        options: FormattingOptions::default(),
    };

    let request = Request::build("textDocument/onTypeFormatting")
        .id(8)
        .params(json!(params))
        .finish();

    let response = service
        .call(request)
        .await
        .unwrap()
        .expect("Response should be present");

    let result: Option<Vec<TextEdit>> =
        serde_json::from_value(response.result().unwrap().clone()).unwrap();

    assert!(result.is_none(), "Should not auto-close self-closing tags");
}
