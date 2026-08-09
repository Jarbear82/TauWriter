use lsp_types::DocumentDiagnosticReportResult;
use tower_lsp::jsonrpc::Result;

use crate::Backend;

/// Handler for textDocument/diagnostic (LSP 3.17 pull diagnostics).
pub async fn diagnostic_pull_handler(server: &Backend) -> Result<DocumentDiagnosticReportResult> {
    let _ = server; // Server reference available for future validation logic
                    // TODO: Implement actual pull diagnostics when lsp-types supports full LSP 3.17 API
    panic!("Pull diagnostics not yet implemented")
}
