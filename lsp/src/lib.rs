pub mod db;
pub mod formatter;
pub mod handlers;
pub mod parser;

use dashmap::DashMap;
use ropey::Rope;
use salsa::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::CLASS,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::ENUM,
];

const LEGEND_MODIFIER: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
];

#[salsa::db]
#[derive(Default, Clone)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for RootDatabase {}

#[salsa::db]
impl db::Db for RootDatabase {
    fn find_file(&self, _path: &str) -> Option<db::SourceFile> {
        None
    }
}

pub struct Backend {
    pub client: Client,
    pub db: Arc<Mutex<RootDatabase>>,
    pub workspace_input: Arc<Mutex<db::Workspace>>,
    pub open_files: Arc<DashMap<Url, Rope>>,
}

impl Backend {
    fn get_symbol_at_position(&self, uri: &Url, position: Position) -> Option<String> {
        let content = self.open_files.get(uri).map(|r| r.to_string())?;

        // Pick grammar based on file extension
        let language = if uri.as_str().ends_with(".hubgs") {
            unsafe { parser::tree_sitter_hubgs() }
        } else if uri.as_str().ends_with(".twxml") {
            unsafe { parser::tree_sitter_twxml() }
        } else {
            return None;
        };

        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser.set_language(language).ok()?;
        let tree = ts_parser.parse(&content, None)?;

        let ts_pos = tree_sitter::Point {
            row: position.line as usize,
            column: position.character as usize,
        };

        // Walk up to find a symbol node at the cursor.
        // HubGS uses "identifier" nodes; TWXML uses different node kinds per grammar.
        let mut node = tree
            .root_node()
            .descendant_for_point_range(ts_pos, ts_pos)?;

        loop {
            match node.kind() {
                "identifier" => break,
                "attribute_value" => {
                    // TWXML: attribute value content is an anonymous regex inside this node.
                    // Strip quotes and return the raw value. Only useful inside <hubref> tags.
                    let attr = node.parent()?;
                    if attr.kind() != "attribute" {
                        return None;
                    }
                    // Make sure parent tag is a hubref, otherwise this isn't a symbol we track
                    let parent_tag = attr.parent()?;
                    if parent_tag.kind() != "start_tag"
                        && parent_tag.kind() != "self_closing_element"
                    {
                        return None;
                    }
                    if let Some(name_node) = parent_tag.child_by_field_name("name") {
                        let tag_name = &content[name_node.byte_range()];
                        if tag_name != "hubref" {
                            return None;
                        }
                    } else {
                        return None;
                    }
                    // Extract the attribute name to know which symbol we're on
                    if let Some(name_child) = attr.child(0) {
                        let attr_name = &content[name_child.byte_range()];
                        if attr_name == "id" {
                            break; // Fall through to extract text below
                        }
                    }
                    return None;
                }
                _ => {}
            }
            if let Some(parent) = node.parent() {
                node = parent;
            } else {
                return None;
            }
        }

        // Make sure the cursor is actually inside the node's range, not just anywhere in its subtree
        let range = node.range();
        if ts_pos.row >= range.start_point.row && ts_pos.row <= range.end_point.row {
            // For attribute_value, strip surrounding quotes
            let raw = content[node.byte_range()].to_string();
            if node.kind() == "attribute_value" {
                Some(raw.trim_matches('"').trim_matches('\'').to_string())
            } else {
                Some(raw)
            }
        } else {
            None
        }
    }

    async fn lock_db(
        &self,
    ) -> (
        tokio::sync::MutexGuard<'_, RootDatabase>,
        tokio::sync::MutexGuard<'_, db::Workspace>,
    ) {
        let db = self.db.lock().await;
        let ws = self.workspace_input.lock().await;
        (db, ws)
    }

    async fn index_directory(
        client: Client,
        db_mutex: Arc<Mutex<RootDatabase>>,
        ws_mutex: Arc<Mutex<db::Workspace>>,
        root: std::path::PathBuf,
    ) {
        use walkdir::WalkDir;

        let mut files = Vec::new();
        {
            let mut db = db_mutex.lock().await;
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|s| s.to_str());
                    if matches!(ext, Some("hubgs") | Some("twxml")) {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let path_str = path.to_string_lossy().to_string();
                            let source = db::SourceFile::new(&mut *db, path_str, content);
                            files.push(source);
                        }
                    }
                }
            }
            let ws = ws_mutex.lock().await;
            ws.set_files(&mut *db).to(files);
        }
        client
            .log_message(MessageType::INFO, "Indexing complete.")
            .await;
    }

    async fn publish_diagnostics(&self, uri: Url) {
        let content = if let Some(rope) = self.open_files.get(&uri) {
            rope.to_string()
        } else {
            return;
        };
        let path = uri.to_file_path().unwrap().to_string_lossy().to_string();

        let errors = {
            let mut db = self.db.lock().await;
            let ws = *self.workspace_input.lock().await;
            let source_file = db::SourceFile::new(&mut *db, path, content.clone());
            db::validate_file(&*db, ws, source_file)
        };

        let diagnostics = errors
            .into_iter()
            .map(|err| Diagnostic {
                range: err.range.into(),
                severity: Some(DiagnosticSeverity::ERROR),
                message: err.message,
                ..Default::default()
            })
            .collect();

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(folders) = params.workspace_folders {
            for folder in folders {
                if let Ok(path) = folder.uri.to_file_path() {
                    let client = self.client.clone();
                    let db = self.db.clone();
                    let ws = self.workspace_input.clone();
                    tokio::spawn(async move {
                        Self::index_directory(client, db, ws, path).await;
                    });
                }
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "\"".to_string(),
                        "'".to_string(),
                        ":".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: TextDocumentRegistrationOptions {
                                document_selector: Some(vec![
                                    DocumentFilter {
                                        language: Some("hubgs".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    },
                                    DocumentFilter {
                                        language: Some("twxml".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    },
                                ]),
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions {
                                    work_done_progress: Some(false),
                                },
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.to_vec(),
                                    token_modifiers: LEGEND_MODIFIER.to_vec(),
                                },
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions { id: None },
                        },
                    ),
                ),
                inlay_hint_provider: Some(OneOf::Left(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "TauWriter LSP initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // --- Navigation ---

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        handlers::goto_definition(self, params).await
    }

    async fn goto_type_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        handlers::goto_type_definition(self, params).await
    }

    async fn goto_declaration(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        handlers::goto_declaration(self, params).await
    }

    async fn goto_implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        handlers::goto_implementation(self, params).await
    }

    // --- Symbols / References ---

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        handlers::references(self, params).await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        handlers::rename(self, params).await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        handlers::document_highlight(self, params).await
    }

    // --- Completion ---

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        handlers::completion(self, params).await
    }

    // --- Information ---

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        handlers::hover(self, params).await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        handlers::code_action(self, params).await
    }

    // --- Features ---

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        handlers::semantic_tokens_full(self, params).await
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        handlers::inlay_hints(self, params).await
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        handlers::folding_range(self, params).await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        handlers::formatting(self, params).await
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        handlers::symbol(self, params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        handlers::document_symbol(self, params).await
    }

    // --- Document lifecycle ---

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        handlers::did_open(self, params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        handlers::did_change(self, params).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        handlers::did_close(self, params).await;
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {}
}
