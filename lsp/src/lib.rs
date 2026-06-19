pub mod db;
pub mod parser;
pub mod formatter;

use dashmap::DashMap;
use salsa::prelude::*;
use std::sync::{Arc, Mutex};
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
    pub open_files: Arc<DashMap<Url, String>>,
}

impl Backend {
    fn get_symbol_at_position(&self, uri: &Url, position: Position) -> Option<String> {
        if let Some(content) = self.open_files.get(uri) {
            let lines: Vec<&str> = content.lines().collect();
            let line = lines.get(position.line as usize)?;

            let start = position.character as usize;
            let mut end = start;
            let chars: Vec<char> = line.chars().collect();

            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            let mut start_idx = start;
            while start_idx > 0
                && (chars[start_idx - 1].is_alphanumeric() || chars[start_idx - 1] == '_')
            {
                start_idx -= 1;
            }

            if start_idx < end {
                return Some(chars[start_idx..end].iter().collect());
            }
        }
        None
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
            let ws = ws_mutex.lock().unwrap();
            ws.set_files(&mut *db).to(files);
        }
        client
            .log_message(MessageType::INFO, "Indexing complete.")
            .await;
    }

    async fn publish_diagnostics(&self, uri: Url) {
        if let Some(content) = self.open_files.get(&uri) {
            let path = uri.to_file_path().unwrap().to_string_lossy().to_string();

            let errors = {
                let mut db = self.db.lock().unwrap();
                let ws = *self.workspace_input.lock().unwrap();
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
            tokio::task::yield_now().await;
        }
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            // 1. Try resolve as Hub Instance
            if let Some(instance) = db::resolve_reference(db, ws, symbol.clone()) {
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
                    if let Some(hub_type) = db::resolve_type(db, ws, file, symbol) {
                        let target_uri = Url::from_file_path(hub_type.file(db).path(db)).unwrap();
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: target_uri,
                            range: hub_type.range(db).into(),
                        })));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn goto_type_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            // 1. Try resolve as Hub Instance -> return its Type
            if let Some(instance) = db::resolve_reference(db, ws, symbol.clone()) {
                let type_name = instance.type_name(db);
                if let Some(hub_type) = db::resolve_type(db, ws, instance.file(db), type_name) {
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
                    if let Some(hub_type) = db::resolve_type(db, ws, file, symbol) {
                        let target_uri = Url::from_file_path(hub_type.file(db).path(db)).unwrap();
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: target_uri,
                            range: hub_type.range(db).into(),
                        })));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn goto_declaration(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        self.goto_definition(params).await
    }

    async fn goto_implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            // 1. Try resolve as Hub Type -> return all its Instances
            if let Ok(path) = uri.to_file_path() {
                let path_str: String = path.to_string_lossy().to_string();
                let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);
                if let Some(file) = file {
                    if let Some(hub_type) = db::resolve_type(db, ws, file, symbol.clone()) {
                        let type_name = hub_type.name(db);
                        let instances = db::all_hub_instances(db, ws);
                        let locations: Vec<Location> = instances
                            .into_iter()
                            .filter(|i| i.type_name(db) == type_name)
                            .map(|i| {
                                let i_path = i.file(db).path(db);
                                Location {
                                    uri: Url::from_file_path(i_path).unwrap(),
                                    range: i.range(db).into(),
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

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            let refs = db::find_all_references(db, ws, symbol);
            let locations = refs
                .into_iter()
                .map(|r| {
                    let path = r.file(db).path(db);
                    Location {
                        uri: Url::from_file_path(path).unwrap(),
                        range: r.range(db).into(),
                    }
                })
                .collect();
            return Ok(Some(locations));
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let mut changes = std::collections::HashMap::new();

            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            if let Some(instance) = db::resolve_reference(db, ws, symbol.clone()) {
                let def_uri = Url::from_file_path(instance.file(db).path(db)).unwrap();
                let def_edit = TextEdit {
                    range: instance.range(db).into(),
                    new_text: new_name.clone(),
                };
                changes
                    .entry(def_uri)
                    .or_insert_with(Vec::new)
                    .push(def_edit);
            }

            let refs = db::find_all_references(db, ws, symbol);
            for r in refs {
                let ref_uri = Url::from_file_path(r.file(db).path(db)).unwrap();
                let ref_edit = TextEdit {
                    range: r.range(db).into(),
                    new_text: new_name.clone(),
                };
                changes
                    .entry(ref_uri)
                    .or_insert_with(Vec::new)
                    .push(ref_edit);
            }

            return Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let db_lock = self.db.lock().unwrap();
        let ws_lock = self.workspace_input.lock().unwrap();
        let db = &*db_lock;
        let ws = *ws_lock;

        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);

            if let Some(file) = file {
                let content = file.contents(db);
                if path_str.ends_with(".twxml") {
                    let ctx = parser::get_twxml_completion_context(&content, position.into());
                    match ctx {
                        parser::TwxmlCompletionContext::HubrefId => {
                            let instances = db::all_hub_instances(db, ws);
                            let items = instances
                                .into_iter()
                                .map(|i| CompletionItem {
                                    label: i.name(db),
                                    kind: Some(CompletionItemKind::REFERENCE),
                                    detail: Some("Hub Instance".to_string()),
                                    ..Default::default()
                                })
                                .collect();
                            return Ok(Some(CompletionResponse::Array(items)));
                        }
                        parser::TwxmlCompletionContext::HubrefField { id_val } => {
                            if let Some(instance) = db::resolve_reference(db, ws, id_val) {
                                let type_name = instance.type_name(db);
                                if let Some(hub_type) = db::resolve_type(db, ws, file, type_name) {
                                    let mut items = Vec::new();
                                    for field in hub_type.fields(db) {
                                        items.push(CompletionItem {
                                            label: field.name.clone(),
                                            kind: Some(CompletionItemKind::FIELD),
                                            detail: Some("Field".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                    for role in hub_type.roles(db) {
                                        items.push(CompletionItem {
                                            label: role.name.clone(),
                                            kind: Some(CompletionItemKind::INTERFACE),
                                            detail: Some("Role".to_string()),
                                            ..Default::default()
                                        });
                                    }
                                    return Ok(Some(CompletionResponse::Array(items)));
                                }
                            }
                        }
                        parser::TwxmlCompletionContext::None => {}
                    }
                } else if path_str.ends_with(".hubgs") {
                    let ctx = parser::get_hubgs_completion_context(&content, position.into());
                    match ctx {
                        parser::HubgsCompletionContext::AllowsList => {
                            let types = db::all_hub_types(db, ws);
                            let items = types
                                .into_iter()
                                .map(|t| CompletionItem {
                                    label: t.name(db),
                                    kind: Some(CompletionItemKind::CLASS),
                                    detail: Some("Hub Type".to_string()),
                                    ..Default::default()
                                })
                                .collect();
                            return Ok(Some(CompletionResponse::Array(items)));
                        }
                        parser::HubgsCompletionContext::InstanceAssignment { type_name, role_name } => {
                            if let Some(hub_type) = db::resolve_type(db, ws, file, type_name.clone()) {
                                if let Some(role) = hub_type.roles(db).iter().find(|r| r.name == role_name) {
                                    let instances = db::all_hub_instances(db, ws);
                                    let items = instances
                                        .into_iter()
                                        .filter(|i| role.allowed_types.contains(&i.type_name(db)))
                                        .map(|i| CompletionItem {
                                            label: i.name(db),
                                            kind: Some(CompletionItemKind::REFERENCE),
                                            detail: Some(format!("Hub Instance ({})", i.type_name(db))),
                                            ..Default::default()
                                        })
                                        .collect();
                                    return Ok(Some(CompletionResponse::Array(items)));
                                }
                            }
                        }
                        parser::HubgsCompletionContext::None => {}
                    }

                    if let Some(type_name) = db::get_hub_type_at_position(db, file, position.into())
                    {
                        if let Some(hub_type) = db::resolve_type(db, ws, file, type_name) {
                            let mut items = Vec::new();
                            for field in hub_type.fields(db) {
                                items.push(CompletionItem {
                                    label: field.name.clone(),
                                    kind: Some(CompletionItemKind::FIELD),
                                    detail: Some("Field".to_string()),
                                    ..Default::default()
                                });
                            }
                            for role in hub_type.roles(db) {
                                items.push(CompletionItem {
                                    label: role.name.clone(),
                                    kind: Some(CompletionItemKind::INTERFACE),
                                    detail: Some("Role".to_string()),
                                    ..Default::default()
                                });
                            }
                            return Ok(Some(CompletionResponse::Array(items)));
                        }
                    } else if db::is_in_hub_definition(db, file, position.into()) {
                        let global_fields = db::all_global_fields(db, ws);
                        let items = global_fields
                            .into_iter()
                            .map(|gf| CompletionItem {
                                label: gf.name(db),
                                kind: Some(CompletionItemKind::FIELD),
                                detail: Some(format!("Global Field ({})", gf.type_name(db))),
                                ..Default::default()
                            })
                            .collect();
                        return Ok(Some(CompletionResponse::Array(items)));
                    }
                }
            }
        }

        let instances = db::all_hub_instances(db, ws);
        let items = instances
            .into_iter()
            .map(|i| CompletionItem {
                label: i.name(db),
                kind: Some(CompletionItemKind::REFERENCE),
                detail: Some("Hub Instance".to_string()),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            // 1. Try resolve as Hub Instance
            if let Some(instance) = db::resolve_reference(db, ws, symbol.clone()) {
                let mut hover_text = format!(
                    "**Hub: {}** ({})",
                    instance.name(db),
                    instance.type_name(db)
                );
                if let Some(desc) = instance.description(db) {
                    hover_text.push_str("\n\n---\n\n");
                    hover_text.push_str(&desc);
                }

                hover_text.push_str("\n\n---\n\n**Fields:**\n");
                if let Some(hub_type) =
                    db::resolve_type(db, ws, instance.file(db), instance.type_name(db))
                {
                    for field in hub_type.fields(db) {
                        if let Some(val) =
                            db::compute_field_value(db, ws, instance, field.name.clone())
                        {
                            let val_str = match val {
                                db::HubValue::String(s) => format!("'{}'", s),
                                db::HubValue::Number(n) => n,
                                db::HubValue::Boolean(b) => b.to_string(),
                                db::HubValue::Identifier(i) => i,
                                db::HubValue::Array(_) => "[...]".to_string(),
                            };
                            hover_text.push_str(&format!("- **{}**: {}\n", field.name, val_str));
                        }
                    }
                }

                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                    range: Some(instance.range(db).into()),
                }));
            }

            // 2. Try resolve as Hub Type (scoped)
            if let Ok(path) = uri.to_file_path() {
                let path_str = path.to_string_lossy().to_string();
                let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);
                if let Some(file) = file {
                    if let Some(hub_type) = db::resolve_type(db, ws, file, symbol) {
                        let mut hover_text = format!("**Type: {}**", hub_type.name(db));
                        hover_text.push_str("\n\n---\n\n");

                        if !hub_type.fields(db).is_empty() {
                            hover_text.push_str("**Fields:**\n");
                            for f in hub_type.fields(db) {
                                hover_text.push_str(&format!("- {}\n", f.name));
                            }
                        }

                        if !hub_type.roles(db).is_empty() {
                            hover_text.push_str("\n**Roles:**\n");
                            for r in hub_type.roles(db) {
                                hover_text.push_str(&format!(
                                    "- {} {} ({}) ALLOWS [{}]\n",
                                    r.name,
                                    r.direction,
                                    r.multiplicity,
                                    r.allowed_types.join(", ")
                                ));
                            }
                        }

                        return Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                            range: Some(hub_type.range(db).into()),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let position = params.range.start;

        if let Some(content) = self.open_files.get(&uri) {
            if let Some((review_range, _hubref_range, id_val, field_val, current_text)) =
                crate::parser::find_review_at_position(&content, position.into())
            {
                let db_lock = self.db.lock().unwrap();
                let ws_lock = self.workspace_input.lock().unwrap();
                let db = &*db_lock;
                let ws = *ws_lock;

                if let Some(instance) = db::resolve_reference(db, ws, id_val.clone()) {
                    if let Some(eval_val) = db::compute_field_value(db, ws, instance, field_val.clone()) {
                        let canonical_str = match eval_val {
                            db::HubValue::String(s) => s,
                            db::HubValue::Number(n) => n,
                            db::HubValue::Boolean(b) => b.to_string(),
                            db::HubValue::Identifier(i) => i,
                            db::HubValue::Array(_) => "".to_string(),
                        };

                        let mut actions = Vec::new();

                        let sync_text = format!(
                            r#"<hubref id="{}" field="{}">{}</hubref>"#,
                            id_val, field_val, canonical_str
                        );
                        let sync_edit = TextEdit {
                            range: review_range.into(),
                            new_text: sync_text,
                        };
                        let mut changes_sync = std::collections::HashMap::new();
                        changes_sync.insert(uri.clone(), vec![sync_edit]);
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Sync and Resolve: change to '{}'", canonical_str),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes_sync),
                                ..Default::default()
                            }),
                            is_preferred: Some(true),
                            ..Default::default()
                        }));

                        let keep_text = format!(
                            r#"<hubref id="{}" field="{}">{}</hubref>"#,
                            id_val, field_val, current_text
                        );
                        let keep_edit = TextEdit {
                            range: review_range.into(),
                            new_text: keep_text,
                        };
                        let mut changes_keep = std::collections::HashMap::new();
                        changes_keep.insert(uri.clone(), vec![keep_edit]);
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Mark as Resolved: keep current text".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            edit: Some(WorkspaceEdit {
                                changes: Some(changes_keep),
                                ..Default::default()
                            }),
                            is_preferred: Some(false),
                            ..Default::default()
                        }));

                        return Ok(Some(actions));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        let db_lock = self.db.lock().unwrap();
        let ws_lock = self.workspace_input.lock().unwrap();
        let db = &*db_lock;
        let ws = *ws_lock;

        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);

            if let Some(file) = file {
                let tokens = db::get_semantic_tokens(db, file);
                let mut last_line = 0;
                let mut last_char = 0;

                let data: Vec<tower_lsp::lsp_types::SemanticToken> = tokens
                    .into_iter()
                    .map(|t| {
                        let delta_line = t.line - last_line;
                        let delta_start = if t.line == last_line {
                            t.character - last_char
                        } else {
                            t.character
                        };

                        last_line = t.line;
                        last_char = t.character;

                        tower_lsp::lsp_types::SemanticToken {
                            delta_line,
                            delta_start,
                            length: t.length,
                            token_type: t.token_type,
                            token_modifiers_bitset: t.token_modifiers,
                        }
                    })
                    .collect();

                return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                    result_id: None,
                    data,
                })));
            }
        }

        Ok(None)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;

        let db_lock = self.db.lock().unwrap();
        let ws_lock = self.workspace_input.lock().unwrap();
        let db = &*db_lock;
        let ws = *ws_lock;

        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);

            if let Some(file) = file {
                let ranges = db::get_folding_ranges(db, file);
                let folding_ranges = ranges
                    .into_iter()
                    .map(|r| FoldingRange {
                        start_line: r.start.line,
                        start_character: Some(r.start.character),
                        end_line: r.end.line,
                        end_character: Some(r.end.character),
                        kind: Some(FoldingRangeKind::Region),
                        ..Default::default()
                    })
                    .collect();

                return Ok(Some(folding_ranges));
            }
        }

        Ok(None)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(symbol) = self.get_symbol_at_position(&uri, position) {
            let db_lock = self.db.lock().unwrap();
            let ws_lock = self.workspace_input.lock().unwrap();
            let db = &*db_lock;
            let ws = *ws_lock;

            let mut highlights = Vec::new();

            // 1. If it's a definition in this file, highlight it
            if let Ok(path) = uri.to_file_path() {
                let path_str = path.to_string_lossy().to_string();
                if path_str.ends_with(".hubgs") {
                    let result = db::parse_hubgs(
                        db,
                        ws.files(db)
                            .into_iter()
                            .find(|f| f.path(db) == path_str)
                            .unwrap(),
                    );
                    if let Some(inst) = result.instances(db).iter().find(|i| i.name(db) == symbol) {
                        highlights.push(DocumentHighlight {
                            range: inst.range(db).into(),
                            kind: Some(DocumentHighlightKind::WRITE),
                        });
                    }
                    if let Some(t) = result.types(db).iter().find(|t| t.name(db) == symbol) {
                        highlights.push(DocumentHighlight {
                            range: t.range(db).into(),
                            kind: Some(DocumentHighlightKind::WRITE),
                        });
                    }
                }
            }

            // 2. Find all references and filter by this file
            let refs = db::find_all_references(db, ws, symbol);
            for r in refs {
                let r_path = r.file(db).path(db);
                if let Ok(uri_path) = uri.to_file_path() {
                    if r_path == uri_path.to_string_lossy().to_string() {
                        highlights.push(DocumentHighlight {
                            range: r.range(db).into(),
                            kind: Some(DocumentHighlightKind::READ),
                        });
                    }
                }
            }

            return Ok(Some(highlights));
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        if let Some(content) = self.open_files.get(&uri) {
            let file_type = if uri.as_str().ends_with(".twxml") {
                "twxml"
            } else if uri.as_str().ends_with(".hubgs") {
                "hubgs"
            } else {
                return Ok(None);
            };

            let new_text = formatter::format_source(&content, file_type);
            let last_line_len = content.lines().last().map(|l| l.len()).unwrap_or(0) as u32;
            let line_count = content.lines().count() as u32;
            let end_line = if line_count > 0 { line_count - 1 } else { 0 };

            let edit = TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: end_line,
                        character: last_line_len,
                    },
                },
                new_text,
            };

            return Ok(Some(vec![edit]));
        }

        Ok(None)
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let db_lock = self.db.lock().unwrap();
        let ws_lock = self.workspace_input.lock().unwrap();
        let db = &*db_lock;
        let ws = *ws_lock;

        let mut symbols = Vec::new();

        // 1. Hub Instances
        let instances = db::all_hub_instances(db, ws);
        for inst in instances {
            let name = inst.name(db);
            if name.to_lowercase().contains(&query) {
                let path = inst.file(db).path(db);
                symbols.push(SymbolInformation {
                    name,
                    kind: SymbolKind::VARIABLE,
                    #[allow(deprecated)]
                    deprecated: None,
                    tags: None,
                    location: Location {
                        uri: Url::from_file_path(path).unwrap(),
                        range: inst.range(db).into(),
                    },
                    container_name: Some("Instances".to_string()),
                });
            }
        }

        // 2. Hub Types
        let types = db::all_hub_types(db, ws);
        for t in types {
            let name = t.name(db);
            if name.to_lowercase().contains(&query) {
                let path = t.file(db).path(db);
                symbols.push(SymbolInformation {
                    name,
                    kind: SymbolKind::CLASS,
                    #[allow(deprecated)]
                    deprecated: None,
                    tags: None,
                    location: Location {
                        uri: Url::from_file_path(path).unwrap(),
                        range: t.range(db).into(),
                    },
                    container_name: Some("Types".to_string()),
                });
            }
        }

        Ok(Some(symbols))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let db_lock = self.db.lock().unwrap();
        let ws_lock = self.workspace_input.lock().unwrap();
        let db = &*db_lock;
        let ws = *ws_lock;

        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            let file = ws.files(db).into_iter().find(|f| f.path(db) == path_str);

            if let Some(file) = file {
                let mut symbols = Vec::new();

                #[allow(deprecated)]
                if path_str.ends_with(".hubgs") {
                    let result = db::parse_hubgs(db, file);

                    for inst in result.instances(db) {
                        symbols.push(SymbolInformation {
                            name: inst.name(db),
                            kind: SymbolKind::VARIABLE,
                            #[allow(deprecated)]
                            deprecated: None,
                            tags: None,

                            location: Location {
                                uri: uri.clone(),
                                range: inst.range(db).into(),
                            },
                            container_name: Some("Instances".to_string()),
                        });
                    }

                    for t in result.types(db) {
                        symbols.push(SymbolInformation {
                            name: t.name(db),
                            kind: SymbolKind::CLASS,
                            #[allow(deprecated)]
                            deprecated: None,
                            tags: None,

                            location: Location {
                                uri: uri.clone(),
                                range: t.range(db).into(),
                            },
                            container_name: Some("Types".to_string()),
                        });
                    }
                }

                return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
            }
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.open_files.insert(uri.clone(), text);
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.first() {
            self.open_files.insert(uri.clone(), change.text.clone());
            self.publish_diagnostics(uri.clone()).await;

            if uri.as_str().ends_with(".twxml") {
                let self_client = self.client.clone();
                let db_clone = self.db.clone();
                let ws_clone = self.workspace_input.clone();
                let uri_clone = uri.clone();

                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    let mut edits = Vec::new();
                    {
                        let db = db_clone.lock().unwrap();
                        let ws = *ws_clone.lock().unwrap();
                        let path = uri_clone.to_file_path().unwrap().to_string_lossy().to_string();
                        if let Some(file) = ws.files(&*db).into_iter().find(|f| f.path(&*db) == path) {
                            let refs = db::parse_twxml(&*db, file);
                            for r in refs {
                                if r.is_reviewed(&*db) {
                                    if let (Some(ref text_val), Some(ref field_name)) = (r.text(&*db), r.field(&*db)) {
                                        let name = r.name(&*db);
                                        if let Some(instance) = db::resolve_reference(&*db, ws, name.clone()) {
                                            if let Some(eval_val) = db::compute_field_value(&*db, ws, instance, field_name.clone()) {
                                                let canonical_str = match eval_val {
                                                    db::HubValue::String(s) => s,
                                                    db::HubValue::Number(n) => n,
                                                    db::HubValue::Boolean(b) => b.to_string(),
                                                    db::HubValue::Identifier(i) => i,
                                                    db::HubValue::Array(_) => "".to_string(),
                                                };
                                                if canonical_str == *text_val {
                                                    let review_range = r.tag_range(&*db);
                                                    let keep_text = format!(
                                                        r#"<hubref id="{}" field="{}">{}</hubref>"#,
                                                        name, field_name, text_val
                                                    );
                                                    edits.push(TextEdit {
                                                        range: review_range.into(),
                                                        new_text: keep_text,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !edits.is_empty() {
                        let mut changes = std::collections::HashMap::new();
                        changes.insert(uri_clone, edits);
                        let edit = WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        };
                        self_client.apply_edit(edit).await.ok();
                    }
                });
            } else if uri.as_str().ends_with(".hubgs") {
                let self_client = self.client.clone();
                let db_clone = self.db.clone();
                let ws_clone = self.workspace_input.clone();
                let open_files_clone = self.open_files.clone();
                
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    let mut changes = std::collections::HashMap::new();
                    {
                        let db = db_clone.lock().unwrap();
                        let ws = *ws_clone.lock().unwrap();
                        for file in ws.files(&*db) {
                            let path = file.path(&*db);
                            if path.ends_with(".twxml") {
                                let file_uri = Url::from_file_path(&path).unwrap();
                                let content = open_files_clone.get(&file_uri)
                                    .map(|x| x.clone())
                                    .unwrap_or_else(|| file.contents(&*db));
                                
                                let refs = db::parse_twxml(&*db, file);
                                let mut edits = Vec::new();
                                for r in refs {
                                    if r.is_reviewed(&*db) {
                                        continue;
                                    }
                                    if let (Some(ref text_val), Some(ref field_name)) = (r.text(&*db), r.field(&*db)) {
                                        let name = r.name(&*db);
                                        if let Some(instance) = db::resolve_reference(&*db, ws, name.clone()) {
                                            if let Some(eval_val) = db::compute_field_value(&*db, ws, instance, field_name.clone()) {
                                                let canonical_str = match eval_val {
                                                    db::HubValue::String(s) => s,
                                                    db::HubValue::Number(n) => n,
                                                    db::HubValue::Boolean(b) => b.to_string(),
                                                    db::HubValue::Identifier(i) => i,
                                                    db::HubValue::Array(_) => "".to_string(),
                                                };
                                                if canonical_str != *text_val {
                                                    let tag_range = r.tag_range(&*db);
                                                    let original_text = get_range_text(&content, tag_range);
                                                    if !original_text.is_empty() {
                                                        let new_text = format!("<review>{}</review>", original_text);
                                                        edits.push(TextEdit {
                                                            range: tag_range.into(),
                                                            new_text,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if !edits.is_empty() {
                                    changes.insert(file_uri, edits);
                                }
                            }
                        }
                    }
                    if !changes.is_empty() {
                        let edit = WorkspaceEdit {
                            changes: Some(changes),
                            ..Default::default()
                        };
                        self_client.apply_edit(edit).await.ok();
                    }
                });
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.open_files.remove(&uri);
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {}
}

fn get_range_text(contents: &str, range: db::LspRange) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;
    
    if start_line >= lines.len() || end_line >= lines.len() {
        return String::new();
    }
    
    if start_line == end_line {
        let line = lines[start_line];
        let start_char = range.start.character as usize;
        let end_char = range.end.character as usize;
        if start_char <= line.len() && end_char <= line.len() {
            return line[start_char..end_char].to_string();
        }
    } else {
        let mut result = Vec::new();
        let first_line = lines[start_line];
        let start_char = range.start.character as usize;
        if start_char <= first_line.len() {
            result.push(&first_line[start_char..]);
        }
        for i in (start_line + 1)..end_line {
            result.push(lines[i]);
        }
        let last_line = lines[end_line];
        let end_char = range.end.character as usize;
        if end_char <= last_line.len() {
            result.push(&last_line[..end_char]);
        }
        return result.join("\n");
    }
    String::new()
}
