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

    let (service, socket) = LspService::new(|client| Backend {
        client,
        db: Arc::new(std::sync::Mutex::new(db)),
        workspace_input,
        open_files: Arc::new(DashMap::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
