use super::lsp_client::*;
use std::path::PathBuf;

#[test]
fn test_locate_lsp_binary() {
    let binary = find_lsp_binary();
    assert!(binary.is_some(), "Expected to find compiled lsp binary");
    let path = binary.unwrap();
    assert!(path.exists(), "Found lsp binary path does not exist");
}

#[tokio::test]
async fn test_lsp_client_lifecycle() {
    let (diag_tx, _diag_rx) = tokio::sync::mpsc::unbounded_channel();
    
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let client_opt = LspClient::new(workspace_root, diag_tx);
    assert!(client_opt.is_some(), "Failed to spawn LspClient");
    let client = client_opt.unwrap();
    
    let temp_file = PathBuf::from("temp.twxml");
    client.notify_open(&temp_file, "<document><body>Hello</body></document>");
    client.notify_change(&temp_file, "<document><body>Hello World</body></document>");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}
