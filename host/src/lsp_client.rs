//! LSP client — manages the tauwriter-lsp language server process.
//! Extracted from `main.rs` to isolate I/O-heavy subprocess management.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clone)]
pub(crate) struct Diagnostic {
    pub(crate) line: usize,
    pub(crate) severity: usize,
    pub(crate) message: String,
}

pub(crate) struct LspClient {
    tx: std::sync::mpsc::Sender<String>,
}

impl LspClient {
    /// Spawn the tauwriter-lsp binary and return a client handle, or `None`
    /// if the binary cannot be found.  All path resolution is fallible.
    pub(crate) fn new(
        workspace_root: PathBuf,
        diag_tx: tokio::sync::mpsc::UnboundedSender<(String, Vec<Diagnostic>)>,
    ) -> Option<Self> {
        let lsp_path = find_lsp_binary()?;

        let mut child = Command::new(&lsp_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let mut stdin = match child.stdin.take() {
            Some(s) => s,
            None => {
                eprintln!("Warning: Failed to take stdin of LSP child process");
                return None;
            }
        };
        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                eprintln!("Warning: Failed to take stdout of LSP child process");
                return None;
            }
        };

        let (tx, rx) = std::sync::mpsc::channel::<String>();

        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                let payload = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);
                if stdin.write_all(payload.as_bytes()).is_err() {
                    break;
                }
                let _ = stdin.flush();
            }
        });

        let mut reader = BufReader::new(stdout);
        std::thread::spawn(move || {
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).is_err() || line.is_empty() {
                    break;
                }
                if line.starts_with("Content-Length:") {
                    let len_str = line.trim_start_matches("Content-Length:").trim();
                    if let Ok(len) = len_str.parse::<usize>() {
                        let mut empty = String::new();
                        let _ = reader.read_line(&mut empty);

                        let mut body_buf = vec![0u8; len];
                        if reader.read_exact(&mut body_buf).is_ok() {
                            if let Ok(json_str) = String::from_utf8(body_buf) {
                                if let Ok(val) =
                                    serde_json::from_str::<serde_json::Value>(&json_str)
                                {
                                    if val["method"] == "textDocument/publishDiagnostics" {
                                        let mut diags = Vec::new();
                                        if let Some(arr) = val["params"]["diagnostics"].as_array() {
                                            for item in arr {
                                                let msg = item["message"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string();
                                                let line_num = item["range"]["start"]["line"]
                                                    .as_u64()
                                                    .unwrap_or(0)
                                                    as usize;
                                                let severity =
                                                    item["severity"].as_u64().unwrap_or(1) as usize;
                                                diags.push(Diagnostic {
                                                    line: line_num,
                                                    severity,
                                                    message: msg,
                                                });
                                            }
                                        }

                                        let uri =
                                            val["params"]["uri"].as_str().unwrap_or("").to_string();
                                        let _ = diag_tx.send((uri, diags));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let root_uri = format!("file://{}", workspace_root.display());
        let init_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {},
                "workspaceFolders": [
                    {
                        "uri": root_uri,
                        "name": "workspace"
                    }
                ]
            }
        });
        let _ = tx.send(init_msg.to_string());

        let initialized_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        let _ = tx.send(initialized_msg.to_string());

        Some(LspClient { tx })
    }

    pub(crate) fn notify_open(&self, path: &std::path::Path, content: &str) {
        let uri = format!("file://{}", path.display());
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": if path.extension().map_or(false, |ext| ext == "hubgs") { "hubgs" } else { "xml" },
                    "version": 1,
                    "text": content
                }
            }
        });
        let _ = self.tx.send(msg.to_string());
    }

    pub(crate) fn notify_change(&self, path: &std::path::Path, content: &str) {
        let uri = format!("file://{}", path.display());
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "version": 2
                },
                "contentChanges": [
                    {
                        "text": content
                    }
                ]
            }
        });
        let _ = self.tx.send(msg.to_string());
    }
}

pub(crate) fn find_lsp_binary() -> Option<std::path::PathBuf> {
    let direct = std::path::PathBuf::from("target/debug/tauwriter-lsp");
    if direct.exists() {
        return Some(direct);
    }

    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    for candidate in [
        base.join("target/debug/tauwriter-lsp"),
        base.join("target/debug/tauwriter_lsp"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
