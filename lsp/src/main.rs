use dashmap::DashMap;
use std::sync::Arc;
use tauwriter_lsp::{Backend, RootDatabase};
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut db = RootDatabase::default();
    let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());
    let db_handle = tauwriter_lsp::SalsaThreadHandle::new(db);

    let (service, socket) = LspService::new(|client| Backend {
        client,
        db: db_handle.clone(),
        workspace_input,
        open_files: Arc::new(DashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
