pub mod db;
pub mod formatter;
pub mod handlers;
pub mod parser;

use dashmap::DashMap;
use ropey::Rope;
use salsa::prelude::*;
use std::sync::Arc;
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
    pub db: Arc<std::sync::Mutex<RootDatabase>>,
    pub workspace_input: db::Workspace,
    pub open_files: Arc<DashMap<Url, Rope>>,
}

use std::sync::OnceLock;

#[derive(Clone)]
pub struct CachedTree {
    pub tree: tree_sitter::Tree,
    pub content_len: usize,
    pub content_hash: u64,
    pub needs_reparse: bool,
}

pub fn calculate_hash(s: &str) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(s.as_bytes());
    hasher.finish()
}

pub static TREE_CACHE: OnceLock<DashMap<String, CachedTree>> = OnceLock::new();

pub fn get_tree_cache() -> &'static DashMap<String, CachedTree> {
    TREE_CACHE.get_or_init(DashMap::new)
}

pub fn utf16_idx_to_byte_idx(s: &str, utf16_idx: usize) -> usize {
    let mut utf16_current = 0;
    let mut byte_current = 0;
    for c in s.chars() {
        if utf16_current >= utf16_idx {
            break;
        }
        utf16_current += c.len_utf16();
        byte_current += c.len_utf8();
    }
    if utf16_current < utf16_idx {
        s.len()
    } else {
        byte_current
    }
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

        let lines: Vec<&str> = content.lines().collect();
        let line_idx = position.line as usize;
        let column_byte = if line_idx < lines.len() {
            utf16_idx_to_byte_idx(lines[line_idx], position.character as usize)
        } else {
            position.character as usize
        };

        let ts_pos = tree_sitter::Point {
            row: position.line as usize,
            column: column_byte,
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

    pub fn read_db(&self) -> (RootDatabase, db::Workspace) {
        let db = self.db.lock().unwrap();
        (db.clone(), self.workspace_input)
    }

    async fn index_directory(
        client: Client,
        db_mutex: Arc<std::sync::Mutex<RootDatabase>>,
        ws: db::Workspace,
        root: std::path::PathBuf,
    ) {
        use walkdir::WalkDir;
        use tower_lsp::lsp_types::notification::Progress;

        let token = NumberOrString::String("indexing-progress".to_string());
        use tower_lsp::lsp_types::request::WorkDoneProgressCreate;
        let _ = client.send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
            token: token.clone(),
        }).await;

        let _ = client.send_notification::<Progress>(ProgressParams {
            token: token.clone(),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title: "Indexing Workspace".to_string(),
                cancellable: Some(false),
                message: Some("Scanning directory...".to_string()),
                percentage: Some(10),
            })),
        }).await;

        let mut files = Vec::new();
        {
            let mut db = db_mutex.lock().unwrap();
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
            ws.set_files(&mut *db).to(files);
        }

        let _ = client.send_notification::<Progress>(ProgressParams {
            token: token.clone(),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Report(WorkDoneProgressReport {
                cancellable: Some(false),
                message: Some("Populating Salsa database...".to_string()),
                percentage: Some(80),
            })),
        }).await;

        let _ = client.send_notification::<Progress>(ProgressParams {
            token: token.clone(),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
                message: Some("Indexing complete.".to_string()),
            })),
        }).await;

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
            let mut db = self.db.lock().unwrap();
            let source_file = db::SourceFile::new(&mut *db, path, content.clone());
            db::validate_file(&*db, self.workspace_input, source_file)
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
                        "<".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
                    first_trigger_character: ">".to_string(),
                    more_trigger_character: None,
                }),
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
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: None,
                }),
                inlay_hint_provider: Some(OneOf::Left(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["{".to_string(), ",".to_string(), "=".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
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

        let registration = Registration {
            id: "workspace/didChangeWatchedFiles".to_string(),
            method: "workspace/didChangeWatchedFiles".to_string(),
            register_options: Some(serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                watchers: vec![
                    FileSystemWatcher {
                        glob_pattern: GlobPattern::String("**/*.hubgs".to_string()),
                        kind: None,
                    },
                    FileSystemWatcher {
                        glob_pattern: GlobPattern::String("**/*.twxml".to_string()),
                        kind: None,
                    },
                ],
            }).unwrap()),
        };
        let client = self.client.clone();
        tokio::spawn(async move {
            client.register_capability(vec![registration]).await.ok();
        });
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

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        handlers::document_color(self, params).await.map(|opt| opt.unwrap_or_default())
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        handlers::color_presentation(self, params).await.map(|opt| opt.unwrap_or_default())
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        handlers::document_link(self, params).await
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        handlers::prepare_call_hierarchy(self, params).await
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        handlers::incoming_calls(self, params).await
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        handlers::outgoing_calls(self, params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        handlers::code_lens(self, params).await
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        handlers::folding_range(self, params).await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        handlers::formatting(self, params).await
    }

    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        handlers::signature_help(self, params).await
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        handlers::selection_range(self, params).await
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        handlers::range_formatting(self, params).await
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        handlers::on_type_formatting(self, params).await
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

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        handlers::did_change_watched_files(self, params).await;
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {}

    async fn execute_command(&self, _params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        handlers::did_create_files(self, params).await;
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        handlers::did_rename_files(self, params).await;
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        handlers::did_delete_files(self, params).await;
    }

    async fn will_save(&self, _params: WillSaveTextDocumentParams) {}

    async fn will_save_wait_until(&self, _params: WillSaveTextDocumentParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(None)
    }

    async fn moniker(&self, _params: MonikerParams) -> Result<Option<Vec<Moniker>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf16_idx_to_byte_idx() {
        // ASCII
        assert_eq!(utf16_idx_to_byte_idx("hello", 0), 0);
        assert_eq!(utf16_idx_to_byte_idx("hello", 2), 2);
        assert_eq!(utf16_idx_to_byte_idx("hello", 10), 5);

        // Multi-byte Unicode (curly quotes: 3 bytes each, 1 UTF-16 code unit each)
        let s = "“hello”";
        assert_eq!(utf16_idx_to_byte_idx(s, 0), 0);
        assert_eq!(utf16_idx_to_byte_idx(s, 1), 3); // after “
        assert_eq!(utf16_idx_to_byte_idx(s, 6), 8); // after o

        // Surrogate pairs (smiley face: 4 bytes, 2 UTF-16 code units)
        let smiley = "a😊b";
        assert_eq!(utf16_idx_to_byte_idx(smiley, 0), 0);
        assert_eq!(utf16_idx_to_byte_idx(smiley, 1), 1); // before 😊
        assert_eq!(utf16_idx_to_byte_idx(smiley, 2), 5); // middle of 😊, snaps to after 😊
        assert_eq!(utf16_idx_to_byte_idx(smiley, 3), 5); // after 😊
        assert_eq!(utf16_idx_to_byte_idx(smiley, 4), 6); // after b
    }

    #[test]
    fn test_incremental_parse_cache() {
        let mut db = RootDatabase::default();
        let path = "/test/file.hubgs".to_string();
        get_tree_cache().remove(&path);

        let source_file = db::SourceFile::new(&mut db, path.clone(), "INSTANCES [ x:Y {} ]".to_string());
        
        let _result = crate::db::parse_hubgs(&db, source_file);
        
        assert!(get_tree_cache().contains_key(&path));
        
        {
            let entry = get_tree_cache().get(&path).unwrap();
            assert!(!entry.needs_reparse);
            assert_eq!(entry.content_len, 20);
        }

        let range = lsp_types::Range {
            start: lsp_types::Position { line: 0, character: 14 },
            end: lsp_types::Position { line: 0, character: 15 },
        };
        let rope = ropey::Rope::from_str("INSTANCES [ x:Y {} ]");
        {
            let mut entry = get_tree_cache().get_mut(&path).unwrap();
            crate::handlers::documents::edit_tree(
                &mut entry.value_mut().tree,
                &rope,
                range,
                "Z",
                14,
                15,
                0,
                0,
            );
            
            let new_contents = "INSTANCES [ x:Z {} ]";
            entry.value_mut().content_len = new_contents.len();
            entry.value_mut().content_hash = calculate_hash(new_contents);
            entry.value_mut().needs_reparse = true;
        }

        {
            let entry = get_tree_cache().get(&path).unwrap();
            assert!(entry.needs_reparse);
            assert_eq!(entry.content_len, 20);
        }

        let source_file2 = db::SourceFile::new(&mut db, path.clone(), "INSTANCES [ x:Z {} ]".to_string());
        let _result2 = crate::db::parse_hubgs(&db, source_file2);

        {
            let entry = get_tree_cache().get(&path).unwrap();
            assert!(!entry.needs_reparse);
            assert_eq!(entry.content_len, 20);
        }
    }
}
