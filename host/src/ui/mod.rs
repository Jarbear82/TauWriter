//! Demo view — the top-level GPUI component for TauWriter.
//!
//! Extracted rendering helpers into submodules ([graph_pane], [titlebar], [sidebar]) to
//! eliminate near-duplicate logic and reduce file length.
//! [user-review: split required] 1103-line monolith split per refactoring task ticket.

use crate::graph_sim::InstanceLink;
use crate::parser::{Block, TextRun};
use gpui::{div, prelude::*, px, Entity, SharedString, Subscription};
use gpui_component::input::InputState;
use gpui_component::{Icon, IconName};
use std::path::PathBuf;

gpui::actions!(
    tauwriter,
    [ToggleSettings, SelectDocumentTab, SelectGraphTab]
);

mod document_view;
pub(crate) mod graph_pane;
pub(crate) mod sidebar;
pub(crate) mod titlebar;
mod tree_view;
#[cfg(test)]
mod ui_tests;

pub(crate) use super::lsp_client::Diagnostic;
pub(crate) use super::lsp_client::LspClient;
pub(crate) use document_view::DocumentView;
pub(crate) use tree_view::{build_file_tree, FileNode};

pub(crate) use graph_pane::GraphPaneView;
pub(crate) use sidebar::SidebarView;
pub(crate) use titlebar::{SettingsView, TitleBar};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocumentMode {
    RawEditor,
    WysiwygPreview,
    MarkdownView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphTab {
    DocumentGraph,
    DefinitionsSchema,
    InstancesRelation,
}

pub(crate) struct OpenDocument {
    pub(crate) path: PathBuf,
    pub(crate) mode: DocumentMode,
    pub(crate) document_home: Entity<DocumentHome>,
    pub(crate) input_state: Entity<gpui_component::input::InputState>,
    pub(crate) show_mode_dropdown: bool,
}

// ─── Workspace Model ────────────────────────────────────────────────────────

pub(crate) struct Workspace {
    pub(crate) file_tree: Vec<FileNode>,
    pub(crate) open_docs: Vec<OpenDocument>,
    pub(crate) active_doc_idx: Option<usize>,
    pub(crate) active_graph_tab: GraphTab,
    pub(crate) selected_path: Option<PathBuf>,
    pub(crate) lsp_client: Option<std::sync::Arc<LspClient>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) selected_hub_id: Option<String>,
}

impl Workspace {
    pub(crate) fn new(workspace_root: PathBuf) -> Self {
        let file_tree = build_file_tree(&workspace_root);
        Self {
            file_tree,
            open_docs: Vec::new(),
            active_doc_idx: None,
            active_graph_tab: GraphTab::InstancesRelation,
            selected_path: None,
            lsp_client: None,
            diagnostics: Vec::new(),
            selected_hub_id: None,
        }
    }
}

// ─── MainView struct ────────────────────────────────────────────────────────

pub(crate) struct MainView {
    pub(crate) focus_handle: gpui::FocusHandle,
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) sidebar: Entity<sidebar::SidebarView>,
    pub(crate) document_view: Entity<DocumentView>,
    pub(crate) graph_pane: Entity<graph_pane::GraphPaneView>,
    pub(crate) settings_window: Option<gpui::WindowHandle<gpui_component::Root>>,
    pub(crate) document_home: Entity<DocumentHome>,
    pub(crate) input_state: Entity<InputState>,
    pub(crate) _subscriptions: Vec<Subscription>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseState {
    Synced,
    OutOfSync { error: String },
}

// ─── DocumentHome & traits ──────────────────────────────────────────────────

pub(crate) struct DocumentHome {
    pub(crate) title: gpui::SharedString,
    pub(crate) author: gpui::SharedString,
    pub(crate) metadata: Vec<(String, String)>,
    pub(crate) blocks: Vec<Block>,
    pub(crate) parse_state: ParseState,
    pub(crate) hubgs_instances:
        std::collections::HashMap<String, (String, String, Vec<InstanceLink>)>,
}

// ─── MainView methods ───────────────────────────────────────────────────────

impl MainView {
    pub(crate) fn toggle_settings(
        &mut self,
        _: &ToggleSettings,
        _: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(handle) = self.settings_window.take() {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        } else {
            let bounds =
                gpui::Bounds::centered(None, gpui::size(gpui::px(350.), gpui::px(500.)), cx);
            if let Ok(handle) = cx.open_window(
                gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                    window_decorations: Some(gpui::WindowDecorations::Client),
                    ..Default::default()
                },
                move |window, cx| {
                    let view = cx.new(|cx| SettingsView::new(cx));
                    cx.new(|cx| gpui_component::Root::new(view, window, cx))
                },
            ) {
                self.settings_window = Some(handle);
            }
        }
        cx.notify();
    }

    pub(crate) fn select_document_tab(
        &mut self,
        _: &SelectDocumentTab,
        _: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.workspace.update(cx, |w, cx| {
            if !w.open_docs.is_empty() {
                w.active_doc_idx = Some(0);
            }
            cx.notify();
        });
        cx.notify();
    }

    pub(crate) fn select_graph_tab(
        &mut self,
        _: &SelectGraphTab,
        _: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.workspace.update(cx, |w, cx| {
            w.active_graph_tab = GraphTab::DefinitionsSchema;
            cx.notify();
        });
        cx.notify();
    }

    pub(crate) fn select_file(
        &mut self,
        path: std::path::PathBuf,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let mut found_idx = None;
        self.workspace.update(cx, |w, _| {
            for (idx, doc) in w.open_docs.iter().enumerate() {
                if doc.path == path {
                    found_idx = Some(idx);
                    break;
                }
            }
        });

        if let Some(idx) = found_idx {
            self.workspace.update(cx, |w, cx| {
                w.active_doc_idx = Some(idx);
                w.selected_path = Some(path.clone());
                w.diagnostics.clear();
                cx.notify();
            });
            cx.notify();
            return;
        }

        // Initialize entities for the new tab
        let title = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let document_home = cx.new(|_| DocumentHome {
            title: title.clone().into(),
            author: "System".into(),
            metadata: Vec::new(),
            blocks: Vec::new(),
            parse_state: ParseState::Synced,
            hubgs_instances: std::collections::HashMap::new(),
        });
        let input_state = cx.new(|cx| {
            gpui_component::input::InputState::new(window, cx)
                .multi_line(true)
                .code_editor("twxml")
                .line_number(true)
        });

        // Push new document tab
        let _new_idx = self.workspace.update(cx, |w, cx| {
            w.open_docs.push(OpenDocument {
                path: path.clone(),
                mode: DocumentMode::RawEditor,
                document_home: document_home.clone(),
                input_state: input_state.clone(),
                show_mode_dropdown: false,
            });
            let idx = w.open_docs.len() - 1;
            w.active_doc_idx = Some(idx);
            w.selected_path = Some(path.clone());
            w.diagnostics.clear();
            cx.notify();
            idx
        });

        // Sync input state text edits
        let main_view_weak = cx.entity().downgrade();
        let input_sub = cx.subscribe_in(&input_state, window, move |_this: &mut MainView, _, ev, _, cx| {
            match ev {
                gpui_component::input::InputEvent::Change => {
                    if let Some(this) = main_view_weak.upgrade() {
                        this.update(cx, |this, cx| {
                            this.handle_document_change(cx);
                        });
                    }
                }
                _ => {}
            }
        });
        self._subscriptions.push(input_sub);

        // Async read and parse the file
        let workspace = self.workspace.clone();
        let window_handle = window.window_handle();
        cx.spawn(
            move |this: gpui::WeakEntity<MainView>, cx: &mut gpui::AsyncApp| {
                let cx = cx.clone();
                async move {
                    let path_clone = path.clone();

                    let task_twxml = cx.background_executor().spawn(async move {
                        let content = std::fs::read_to_string(&path_clone).unwrap_or_default();
                        let parsed =
                            crate::parser::load_and_parse_twxml(&path_clone.to_string_lossy()).ok();
                        (content, parsed)
                    });

                    let hubgs_path = path.with_extension("hubgs");
                    let workspace_root = crate::utils::resolve_workspace_root()
                        .expect("workspace root resolves");

                    let task_hubgs = cx.background_executor().spawn(async move {
                        let target_hubgs = if hubgs_path.exists() {
                            Some(hubgs_path)
                        } else {
                            crate::graph_sim::find_any_hubgs(&workspace_root)
                        };

                        let mut hubgs_map = std::collections::HashMap::new();
                        if let Some(ref hp) = target_hubgs {
                            if let Ok((_defs, instances)) = crate::graph_sim::parse_hubgs_file(hp) {
                                for inst in &instances {
                                    hubgs_map.insert(
                                        inst.id.clone(),
                                        (
                                            inst.type_name.clone(),
                                            inst.name.clone(),
                                            inst.links.clone(),
                                        ),
                                    );
                                }
                            }
                        }
                        hubgs_map
                    });

                    let (xml_content, parsed_twxml) = task_twxml.await;
                    let hubgs_data = task_hubgs.await;

                    let _ = cx.update(|cx| {
                        let _ = window_handle.update(cx, |_, window, cx| {
                            input_state.update(cx, |state, cx| {
                                state.set_value(xml_content.clone(), window, cx);
                            });
                        });

                        let lsp_client = workspace.read(cx).lsp_client.clone();
                        if let Some(ref client) = lsp_client {
                            client.notify_open(&path, &xml_content);
                        }

                        document_home.update(cx, |doc, cx| {
                            doc.hubgs_instances = hubgs_data;
                            let is_twxml = path.extension().map_or(false, |ext| ext == "twxml");
                            if is_twxml {
                                if let Some((title, author, metadata, blocks)) = parsed_twxml {
                                    doc.title = title.into();
                                    doc.author = author.into();
                                    doc.metadata = metadata;
                                    doc.blocks = blocks;
                                    doc.parse_state = ParseState::Synced;
                                } else {
                                    doc.title = "Error Loading Document".into();
                                    doc.author = "System".into();
                                    doc.metadata = Vec::new();
                                    doc.blocks = vec![Block::Paragraph {
                                        runs: vec![TextRun::new("Could not parse TWXML document.")],
                                        id: None,
                                        attributes: Vec::new(),
                                        range: None,
                                    }];
                                    doc.parse_state = ParseState::Synced;
                                }
                            } else {
                                doc.title = path.file_name().unwrap_or_default().to_string_lossy().to_string().into();
                                doc.author = "System".into();
                                doc.metadata = Vec::new();
                                doc.blocks = vec![Block::Paragraph {
                                    runs: vec![TextRun::new("Visual preview is only available for .twxml documents.")],
                                    id: None,
                                    attributes: Vec::new(),
                                    range: None,
                                }];
                                doc.parse_state = ParseState::Synced;
                            }
                            cx.notify();
                        });

                        if let Some(this) = this.upgrade() {
                            this.update(cx, |_, cx| {
                                cx.notify();
                            });
                        }
                    });
                }
            }
        ).detach();
    }

    pub(crate) fn handle_document_change(&mut self, cx: &mut Context<Self>) {
        let (active_doc_path, input_state, document_home) = {
            let w = self.workspace.read(cx);
            if let Some(idx) = w.active_doc_idx {
                if let Some(doc) = w.open_docs.get(idx) {
                    (Some(doc.path.clone()), doc.input_state.clone(), doc.document_home.clone())
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let text = input_state.read(cx).value().to_string();
        let lsp_client = self.workspace.update(cx, |w, _| w.lsp_client.clone());

        if let Some(ref p) = active_doc_path {
            let _ = std::fs::write(p, &text);
            if let Some(ref client) = lsp_client {
                client.notify_change(p, &text);
            }
            let base_dir = p.parent();
            let mut visited = std::collections::HashSet::new();
            if let Ok(abs) = p.canonicalize() {
                visited.insert(abs);
            }
            match crate::parser::parse_twxml_internal(&text, base_dir, &mut visited) {
                Ok((title, author, metadata, blocks)) => {
                    document_home.update(cx, |doc, cx| {
                        doc.title = title.into();
                        doc.author = author.into();
                        doc.metadata = metadata;
                        doc.blocks = blocks;
                        doc.parse_state = ParseState::Synced;
                        cx.notify();
                    });
                }
                Err(err) => {
                    document_home.update(cx, |doc, cx| {
                        doc.parse_state = ParseState::OutOfSync {
                            error: err.to_string(),
                        };
                        cx.notify();
                    });
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn close_document_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, cx| {
            if idx < w.open_docs.len() {
                w.open_docs.remove(idx);
                if w.open_docs.is_empty() {
                    w.active_doc_idx = None;
                    w.selected_path = None;
                } else {
                    let active_idx = w.active_doc_idx.unwrap_or(0);
                    if active_idx >= w.open_docs.len() {
                        w.active_doc_idx = Some(w.open_docs.len() - 1);
                    } else if active_idx == idx {
                        w.active_doc_idx = Some(active_idx.saturating_sub(1));
                    }
                    if let Some(i) = w.active_doc_idx {
                        w.selected_path = Some(w.open_docs[i].path.clone());
                    }
                }
                cx.notify();
            }
        });
        cx.notify();
    }

    pub(crate) fn handle_node_click(&mut self, node_id: String, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if node_id.contains('_') {
            let parts: Vec<&str> = node_id.split('_').collect();
            if let Some(last) = parts.last() {
                if let Ok(idx) = last.parse::<usize>() {
                    let offset_opt = self.workspace.update(cx, |w, cx| {
                        if let Some(doc_idx) = w.active_doc_idx {
                            if let Some(doc) = w.open_docs.get(doc_idx) {
                                let text = doc.input_state.read(cx).value().to_string();
                                let (nodes, _) = crate::parser::parse_document_outline(&text);
                                if idx < nodes.len() {
                                    return Some((nodes[idx].start_offset, doc.input_state.clone()));
                                }
                            }
                        }
                        None
                    });

                    if let Some((offset, input_state)) = offset_opt {
                        input_state.update(cx, |state, cx| {
                            let text = state.value().to_string();
                            if let Some(pos) = crate::ui::document_view::jump_links::offset_to_position(&text, offset) {
                                state.set_cursor_position(pos, window, cx);
                            }
                        });
                    }
                }
            }
        } else {
            let workspace_root = crate::utils::resolve_workspace_root().unwrap();
            let ref_path = find_file_referencing_hub(&workspace_root, &node_id);
            if let Some(path) = ref_path {
                self.select_file(path, window, cx);
            }
            self.workspace.update(cx, |w, cx| {
                w.selected_hub_id = Some(node_id);
                cx.notify();
            });
        }
        cx.notify();
    }
}

fn find_file_referencing_hub(dir: &std::path::Path, hub_id: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str == "target" || name_str == ".git" || name_str == ".gemini" {
                        continue;
                    }
                }
                if let Some(p) = find_file_referencing_hub(&path, hub_id) {
                    return Some(p);
                }
            } else if path.extension().map_or(false, |ext| ext == "twxml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains(&format!("id=\"{}\"", hub_id))
                        || content.contains(&format!("id='{}'", hub_id))
                    {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

// ─── Render implementation ───────────────────────────────────────────────────

impl gpui::Render for MainView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.document_home.read(cx).title.clone();

        let theme_val = gpui_component::Theme::global(cx);
        let bg_color = theme_val.background;
        let fg_color = theme_val.foreground;
        let border_color = theme_val.border;
        let sidebar_bg = theme_val.sidebar;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_name = theme_val.theme_name().to_string();
        let view = cx.entity().clone();

        // Left sidebar file explorer
        let file_explorer = self.sidebar.clone();

        let workspace = self.workspace.read(cx);
        let active_doc_idx = workspace.active_doc_idx;

        // 1. Render Document Tabs (Left Pane)
        let mut doc_tabs = Vec::new();
        for (i, doc) in workspace.open_docs.iter().enumerate() {
            let filename = doc.path.file_name()
                .map_or("No Name".to_string(), |n| n.to_string_lossy().to_string());
            let is_active = Some(i) == active_doc_idx;
            
            let bg = if is_active { theme_val.background } else { theme_val.sidebar };
            let border = if is_active { border_color } else { gpui::hsla(0.0, 0.0, 0.0, 0.0) };
            let text_color = if is_active { theme_val.foreground } else { theme_muted_foreground };

            let view_clone = cx.entity().clone();
            let view_clone_close = cx.entity().clone();
            
            doc_tabs.push(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .border_r(px(1.))
                    .border_color(border_color)
                    .bg(bg)
                    .child(
                        div()
                            .cursor_pointer()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(text_color)
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                view_clone.update(cx, |this, cx| {
                                    this.workspace.update(cx, |w, cx| {
                                        w.active_doc_idx = Some(i);
                                        w.selected_path = Some(w.open_docs[i].path.clone());
                                        cx.notify();
                                    });
                                    cx.notify();
                                });
                            })
                            .child(filename)
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(theme_muted_foreground)
                            .hover(|s| s.text_color(theme_val.danger))
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                view_clone_close.update(cx, |this, cx| {
                                    this.close_document_tab(i, cx);
                                });
                            })
                            .child("✕")
                    )
            );
        }

        let mut mode_selector = None;
        let mut dropdown_el = None;
        if let Some(idx) = active_doc_idx {
            if let Some(doc) = workspace.open_docs.get(idx) {
                let current_mode_str = match doc.mode {
                    DocumentMode::RawEditor => "Raw Editor",
                    DocumentMode::WysiwygPreview => "WYSIWYG Preview",
                    DocumentMode::MarkdownView => "Markdown View",
                };
                let view_clone = cx.entity().clone();
                mode_selector = Some(
                    div()
                        .relative()
                        .flex()
                        .items_center()
                        .px_2()
                        .py_1()
                        .bg(theme_val.sidebar)
                        .border(px(1.))
                        .border_color(border_color)
                        .rounded(px(4.))
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                            view_clone.update(cx, |this, cx| {
                                this.workspace.update(cx, |w, cx| {
                                    if let Some(i) = w.active_doc_idx {
                                        if let Some(d) = w.open_docs.get_mut(i) {
                                            d.show_mode_dropdown = !d.show_mode_dropdown;
                                        }
                                    }
                                    cx.notify();
                                });
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme_val.foreground)
                                .child(format!("{} ▾", current_mode_str))
                        )
                );

                if doc.show_mode_dropdown {
                    let view_clone = cx.entity().clone();
                    dropdown_el = Some(
                        div()
                            .absolute()
                            .top(px(32.))
                            .left(px(150.)) // Positioned below mode selector
                            .bg(theme_val.background)
                            .border(px(1.))
                            .border_color(border_color)
                            .rounded(px(4.))
                            .shadow_md()
                            .flex()
                            .flex_col()
                            .p_1()
                            .w(px(150.))
                            .child(
                                div()
                                    .p_2()
                                    .text_xs()
                                    .text_color(fg_color)
                                    .hover(|s| s.bg(theme_val.accent.opacity(0.3)))
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view_clone = view_clone.clone();
                                        move |_, _, cx| {
                                            view_clone.update(cx, |this, cx| {
                                                this.workspace.update(cx, |w, cx| {
                                                    if let Some(i) = w.active_doc_idx {
                                                        if let Some(d) = w.open_docs.get_mut(i) {
                                                            d.mode = DocumentMode::RawEditor;
                                                            d.show_mode_dropdown = false;
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child("Raw Editor"),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .text_xs()
                                    .text_color(fg_color)
                                    .hover(|s| s.bg(theme_val.accent.opacity(0.3)))
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view_clone = view_clone.clone();
                                        move |_, _, cx| {
                                            view_clone.update(cx, |this, cx| {
                                                this.workspace.update(cx, |w, cx| {
                                                    if let Some(i) = w.active_doc_idx {
                                                        if let Some(d) = w.open_docs.get_mut(i) {
                                                            d.mode = DocumentMode::WysiwygPreview;
                                                            d.show_mode_dropdown = false;
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child("WYSIWYG Preview"),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .text_xs()
                                    .text_color(fg_color)
                                    .hover(|s| s.bg(theme_val.accent.opacity(0.3)))
                                    .on_mouse_down(gpui::MouseButton::Left, {
                                        let view_clone = view_clone.clone();
                                        move |_, _, cx| {
                                            view_clone.update(cx, |this, cx| {
                                                this.workspace.update(cx, |w, cx| {
                                                    if let Some(i) = w.active_doc_idx {
                                                        if let Some(d) = w.open_docs.get_mut(i) {
                                                            d.mode = DocumentMode::MarkdownView;
                                                            d.show_mode_dropdown = false;
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .child("Markdown View"),
                            )
                    );
                }
            }
        }

        let mut doc_tab_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .bg(theme_val.sidebar)
            .border_b(px(1.))
            .border_color(border_color)
            .child(
                div()
                    .flex()
                    .items_center()
                    .children(doc_tabs)
            );

        if let Some(ms) = mode_selector {
            doc_tab_bar = doc_tab_bar.child(
                div()
                    .px_3()
                    .flex()
                    .items_center()
                    .child(ms)
            );
        }

        // 2. Render Graph Tabs (Right Pane)
        let active_graph_tab = workspace.active_graph_tab;
        let selected_graph_index = match active_graph_tab {
            GraphTab::DocumentGraph => 0,
            GraphTab::DefinitionsSchema => 1,
            GraphTab::InstancesRelation => 2,
        };

        let view_clone_graph = cx.entity().clone();
        let graph_tab_configs = vec![
            ("Document Graph", IconName::File),
            ("Definitions Schema", IconName::LayoutDashboard),
            ("Instances Relation", IconName::Network),
        ];

        let graph_tab_bar = gpui_component::tab::TabBar::new("graph-tab-bar")
            .selected_index(selected_graph_index)
            .on_click(move |index, _, cx| {
                let tab = match index {
                    0 => GraphTab::DocumentGraph,
                    1 => GraphTab::DefinitionsSchema,
                    2 => GraphTab::InstancesRelation,
                    _ => return,
                };
                view_clone_graph.update(cx, |this, cx| {
                    this.workspace.update(cx, |w, cx| {
                        w.active_graph_tab = tab;
                        cx.notify();
                    });
                    cx.notify();
                });
            })
            .children(
                graph_tab_configs
                    .into_iter()
                    .map(|(label, icon)| gpui_component::tab::Tab::new().icon(icon).label(label)),
            );

        // Workspace Column containing the split panel
        let workspace_column = div()
            .flex_1()
            .h_full()
            .relative()
            .child(
                gpui_component::resizable::h_resizable("document-graph-split")
                    .child(
                        gpui_component::resizable::resizable_panel()
                            .child(
                                div()
                                    .size_full()
                                    .flex()
                                    .flex_col()
                                    .child(doc_tab_bar)
                                    .child(div().flex_1().h(gpui::px(0.)).child(self.document_view.clone()))
                            )
                    )
                    .child(
                        gpui_component::resizable::resizable_panel()
                            .child(
                                div()
                                    .size_full()
                                    .flex()
                                    .flex_col()
                                    .child(graph_tab_bar)
                                    .child(div().flex_1().h(gpui::px(0.)).child(self.graph_pane.clone()))
                            )
                    )
            )
            .children(dropdown_el);

        let viewport_width = _window.viewport_size().width;
        let explorer_min = viewport_width * 0.15;
        let explorer_max = viewport_width * 0.5;

        // Main splitter (horizontal resizable)
        let main_splitter = gpui_component::resizable::h_resizable("explorer-workspace")
            .child(
                gpui_component::resizable::resizable_panel()
                    .size(gpui::px(250.))
                    .size_range(explorer_min..explorer_max)
                    .child(file_explorer),
            )
            .child(gpui_component::resizable::resizable_panel().child(workspace_column));

        let title_bar = TitleBar {
            settings_open: self.settings_window.is_some(),
            title: title.clone(),
            view: cx.entity().clone(),
        };

        // Bottom status bar
        let active_file_str =
            workspace
                .selected_path
                .as_ref()
                .map_or("No file selected".to_string(), |p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });

        let lsp_indicator = if workspace.lsp_client.is_some() {
            gpui::div()
                .w(px(8.))
                .h(px(8.))
                .rounded_full()
                .bg(theme_val.success)
        } else {
            gpui::div()
                .w(px(8.))
                .h(px(8.))
                .rounded_full()
                .bg(theme_val.danger)
        };

        let lsp_label: SharedString = if workspace.lsp_client.is_some() {
            "LSP Connected".into()
        } else {
            "LSP Offline".into()
        };

        let bottom_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .h(gpui::px(26.))
            .bg(sidebar_bg)
            .border_t(gpui::px(1.))
            .border_color(border_color)
            .px_4()
            .text_xs()
            .text_color(theme_muted_foreground)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Folder).size(gpui::px(14.)))
                            .child(active_file_str),
                    )
                    .child(div().child(lsp_label)),
            )
            .child(
                div()
                    .cursor_pointer()
                    .hover(|s| s.underline())
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                        let was_open = view.read(cx).settings_window.is_some();
                        if was_open {
                            if let Some(handle) =
                                view.update(cx, |this, _| this.settings_window.take())
                            {
                                let _ = handle.update(cx, |_, w, _| w.remove_window());
                            }
                            // Re-render MainView by updating it with a no-op
                            view.update(cx, |_: &mut MainView, cx: &mut Context<MainView>| {
                                cx.notify();
                            });
                        } else {
                            let bounds = gpui::Bounds::centered(
                                None,
                                gpui::size(gpui::px(350.), gpui::px(500.)),
                                cx,
                            );
                            if let Ok(handle) = cx.open_window(
                                gpui::WindowOptions {
                                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                    window_decorations: Some(gpui::WindowDecorations::Client),
                                    ..Default::default()
                                },
                                move |window, cx| {
                                    let view = cx.new(|cx| SettingsView::new(cx));
                                    cx.new(|cx| gpui_component::Root::new(view, window, cx))
                                },
                            ) {
                                view.update(cx, |this, _| {
                                    this.settings_window = Some(handle);
                                });
                                view.update(cx, |_: &mut MainView, cx: &mut Context<MainView>| {
                                    cx.notify();
                                });
                            }
                        }
                    })
                    .child(format!("Theme: {}", theme_name)),
            );

        div()
            .key_context("MainView")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_settings))
            .on_action(cx.listener(Self::select_document_tab))
            .on_action(cx.listener(Self::select_graph_tab))
            .size_full()
            .flex()
            .flex_col()
            .bg(bg_color)
            .text_color(fg_color)
            .child(title_bar)
            .child(
                div()
                    .id("main_content")
                    .flex_1()
                    .h(gpui::px(0.))
                    .overflow_hidden()
                    .w_full()
                    .child(main_splitter),
            )
            .child(bottom_bar)
    }
}
