//! Demo view — the top-level GPUI component for TauWriter.
//!
//! Extracted rendering helpers into submodules to eliminate near-duplicate logic and reduce file length.
//! [user-review: split required] 1103-line monolith split per refactoring task ticket.

use crate::graph_sim::InstanceLink;
use crate::parser::{Block, TextRun};
use gpui::{div, prelude::*, Entity, SharedString};
use gpui_component::button::ButtonGroup;
use gpui_component::input::InputState;
use gpui_component::IconName;
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

// Split-out modules
mod document_tabs;
mod mode_dropdown;
mod window_chrome;

pub(crate) use super::lsp_client::Diagnostic;
pub(crate) use super::lsp_client::LspClient;
pub(crate) use document_view::DocumentView;
pub(crate) use tree_view::{build_file_tree, FileNode};

pub(crate) use graph_pane::GraphPaneView;
pub(crate) use sidebar::SidebarView;
pub(crate) use titlebar::{SettingsView, TitleBar};

// ─── Enums ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocumentMode {
    RawEditor,
    BlockEditor,
    MarkdownView,
    FlowTextEditor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GraphTab {
    DocumentGraph,
    DefinitionsSchema,
    InstancesRelation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LayoutType {
    ForceDirected,
    Sugiyama,
    Cose,
    Circular,
    Grid,
}

// ─── Data Structures ────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct OpenDocument {
    pub(crate) path: PathBuf,
    pub(crate) mode: DocumentMode,
    pub(crate) document_home: Entity<DocumentHome>,
    #[allow(dead_code)]
    pub(crate) input_state: Entity<gpui_component::input::InputState>,
    /// Subscriptions owned by this document; dropped when the doc is closed.
    pub(crate) doc_subscriptions: Vec<gpui::Subscription>,
}

// ─── Workspace Model ────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct Workspace {
    pub(crate) file_tree: Vec<FileNode>,
    pub(crate) open_docs: Vec<OpenDocument>,
    pub(crate) active_doc_idx: Option<usize>,
    pub(crate) active_graph_tab: GraphTab,
    pub(crate) selected_path: Option<PathBuf>,
    pub(crate) lsp_client: Option<std::sync::Arc<LspClient>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) selected_hub_id: Option<SharedString>,
    pub(crate) layout_type: LayoutType,
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
            layout_type: LayoutType::ForceDirected,
        }
    }
}

// ─── MainView struct ────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct MainView {
    pub(crate) focus_handle: gpui::FocusHandle,
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) sidebar: Entity<sidebar::SidebarView>,
    pub(crate) document_view: Entity<DocumentView>,
    pub(crate) graph_pane: Entity<graph_pane::GraphPaneView>,
    pub(crate) settings_window: Option<gpui::WindowHandle<gpui_component::Root>>,
    #[allow(dead_code)]
    pub(crate) document_home: Entity<DocumentHome>,
    #[allow(dead_code)]
    pub(crate) input_state: Entity<InputState>,
    /// App-level subscriptions kept alive for the lifetime of MainView.
    #[allow(dead_code)]
    pub(crate) _sidebar_sub: gpui::Subscription,
    #[allow(dead_code)]
    pub(crate) _graph_sub: gpui::Subscription,
    #[allow(dead_code)]
    pub(crate) _input_sub: gpui::Subscription,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseState {
    Synced,
    OutOfSync { error: SharedString },
}

// ─── DocumentHome & traits ──────────────────────────────────────────────────

pub(crate) struct DocumentHome {
    pub(crate) title: gpui::SharedString,
    pub(crate) author: gpui::SharedString,
    pub(crate) metadata: Vec<(SharedString, SharedString)>,
    pub(crate) blocks: Vec<Block>,
    pub(crate) parse_state: ParseState,
    pub(crate) hubgs_instances:
        std::collections::HashMap<SharedString, (SharedString, SharedString, Vec<InstanceLink>)>,
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
        let title = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
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
                doc_subscriptions: Vec::new(),
            });
            w.active_doc_idx = Some(0);
            w.selected_path = Some(path.clone());
            w.diagnostics.clear();
            cx.notify();
            w.open_docs.len() - 1
        });

        // Sync input state text edits
        let main_view_weak = cx.entity().downgrade();
        let input_sub = cx.subscribe_in(
            &input_state,
            window,
            move |_this: &mut MainView, _, ev, _, cx| match ev {
                gpui_component::input::InputEvent::Change => {
                    if let Some(this) = main_view_weak.upgrade() {
                        this.update(cx, |this, cx| {
                            this.handle_document_change(cx);
                        });
                    }
                }
                _ => {}
            },
        );
        self.workspace.update(cx, |w, _| {
            w.open_docs
                .last_mut()
                .unwrap()
                .doc_subscriptions
                .push(input_sub);
        });

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
                    let workspace_root =
                        crate::utils::resolve_workspace_root().expect("workspace root resolves");

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
                                doc.title = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string()
                                    .into();
                                doc.author = "System".into();
                                doc.metadata = Vec::new();
                                doc.blocks = vec![Block::Paragraph {
                                    runs: vec![TextRun::new(
                                        "Visual preview is only available for .twxml documents.",
                                    )],
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
            },
        )
        .detach();
    }

    pub(crate) fn handle_document_change(&mut self, cx: &mut Context<Self>) {
        let (active_doc_path, input_state) = {
            let w = self.workspace.read(cx);
            if let Some(idx) = w.active_doc_idx {
                if let Some(doc) = w.open_docs.get(idx) {
                    (
                        Some(doc.path.clone()),
                        doc.input_state.clone(),
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };

        let text = input_state.read(cx).value().to_string();
        let lsp_client_opt = self.workspace.update(cx, |w, _| w.lsp_client.clone());

        if let Some(p) = active_doc_path {
            let base_dir = p.parent().map(|d| d.to_path_buf());
            let abs_path = p.canonicalize().ok();
            let mut visited = std::collections::HashSet::new();
            if let Some(abs) = abs_path {
                visited.insert(abs);
            }

            // Debounce: wait 300ms after the last keystroke before writing/parsing.
            let client = lsp_client_opt.clone();
            let text_clone = text.clone();
            let p_clone = p;
            let base_dir_clone = base_dir;
            let visited_inner = std::sync::Arc::new(std::sync::Mutex::new(visited));

            cx.spawn(
                async move |_this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(300))
                        .await;

                    let write_result = std::fs::write(&p_clone, &text_clone);

                    if let Err(e) = write_result {
                        log::warn!("Failed to write {}: {e}", p_clone.display());
                        _this
                            .update(cx, |_this, cx| {
                                let doc_home = _this
                                    .workspace
                                    .read(cx)
                                    .open_docs
                                    .first()
                                    .map(|d| d.document_home.clone());
                                if let Some(home) = doc_home {
                                    home.update(cx, |doc, cx| {
                                        doc.parse_state = ParseState::OutOfSync {
                                            error: format!("Failed to save file: {e}").into(),
                                        };
                                        cx.notify();
                                    });
                                }
                            })
                            .ok();
                        return;
                    }

                    if let Some(ref client) = client {
                        client.notify_change(&p_clone, &text_clone);
                    }

                    let mut v_guard = visited_inner.lock().unwrap();
                    let parse_result = crate::parser::parse_twxml_internal(
                        &text_clone,
                        base_dir_clone.as_deref(),
                        &mut *v_guard,
                    );
                    drop(v_guard);

                    let _ = _this.update(cx, |_this, cx| {
                        let doc_home = _this
                            .workspace
                            .read(cx)
                            .open_docs
                            .first()
                            .map(|d| d.document_home.clone());
                        if let Some(home) = doc_home {
                            match parse_result {
                                Ok((title, author, metadata, blocks)) => {
                                    home.update(cx, |doc, cx| {
                                        doc.title = title.into();
                                        doc.author = author.into();
                                        doc.metadata = metadata;
                                        doc.blocks = blocks;
                                        doc.parse_state = ParseState::Synced;
                                        cx.notify();
                                    });
                                }
                                Err(err) => {
                                    home.update(cx, |doc, cx| {
                                        doc.parse_state = ParseState::OutOfSync {
                                            error: err.to_string().into(),
                                        };
                                        cx.notify();
                                    });
                                }
                            }
                        }
                    });

                    let _ = _this.update(cx, |_, cx| cx.notify());
                },
            )
            .detach();
        }
        cx.notify();
    }

    pub(crate) fn close_document_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.workspace.update(cx, |w, cx| {
            if idx < w.open_docs.len() {
                let _removed = w.open_docs.remove(idx);
                if w.open_docs.is_empty() {
                    w.active_doc_idx = None;
                    w.selected_path = None;
                } else {
                    let active_idx = w.active_doc_idx.unwrap_or(0);
                    let new_active = if active_idx == idx {
                        // The active tab was closed — select the previous one (or 0).
                        active_idx.saturating_sub(1)
                    } else if active_idx > idx {
                        // An earlier tab closed — every index after `idx` shifted down by one.
                        active_idx - 1
                    } else {
                        active_idx
                    };
                    // Clamp in case active_idx was already out of range.
                    w.active_doc_idx = Some(new_active.min(w.open_docs.len() - 1));
                    if let Some(i) = w.active_doc_idx {
                        w.selected_path = Some(w.open_docs[i].path.clone());
                    }
                }
                cx.notify();
            }
        });
        cx.notify();
    }

    pub(crate) fn handle_node_click(
        &mut self,
        node_id: SharedString,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
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
                                    return Some((
                                        nodes[idx].start_offset,
                                        doc.input_state.clone(),
                                    ));
                                }
                            }
                        }
                        None
                    });

                    if let Some((offset, input_state)) = offset_opt {
                        input_state.update(cx, |state, cx| {
                            let text = state.value().to_string();
                            if let Some(pos) =
                                crate::ui::document_view::jump_links::offset_to_position(
                                    &text, offset,
                                )
                            {
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
    find_file_referencing_hub_impl(dir, hub_id, 0)
}

fn find_file_referencing_hub_impl(
    dir: &std::path::Path,
    hub_id: &str,
    depth: usize,
) -> Option<std::path::PathBuf> {
    const MAX_DEPTH: usize = 64;
    if depth > MAX_DEPTH {
        return None;
    }
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
                if let Some(p) = find_file_referencing_hub_impl(&path, hub_id, depth + 1) {
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

// ─── Render implementation ──────────────────────────────────────────────────

impl gpui::Render for MainView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Theme setup
        let theme_val = gpui_component::Theme::global(cx);
        let bg_color = theme_val.background;
        let fg_color = theme_val.foreground;
        let border_color = theme_val.border;
        let sidebar_bg = theme_val.sidebar;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_name = theme_val.theme_name().to_string();

        // Workspace state
        let workspace = self.workspace.read(cx);
        let active_doc_idx = workspace.active_doc_idx;
        let file_explorer = self.sidebar.clone();

        // ── Doc tabs + mode selector ──
        let mut doc_tab_bar = document_tabs::render_doc_tab_bar(
            theme_val.background,
            sidebar_bg,
            border_color,
            fg_color,
            theme_muted_foreground,
            &workspace.open_docs,
            active_doc_idx,
            cx.entity().clone(),
        );

        // Assemble tab bar + mode selector area
        if let Some(idx) = active_doc_idx {
            if workspace.open_docs.get(idx).is_some() {
                doc_tab_bar = doc_tab_bar.child(div().px_3().flex().items_center().child(
                    mode_dropdown::render_mode_selector(
                        workspace.open_docs[idx].mode,
                        idx,
                        cx.entity().clone(),
                    ),
                ));
            }
        }

        // ── Graph tabs bar ──
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

        // ── Layout type selector + run layout button ──
        let current_layout_type = workspace.layout_type;

        let graph_pane_for_layout = self.graph_pane.clone();

        use gpui_component::button::{Button, DropdownButton as GpuiDropdownButton};
        use gpui_component::menu::PopupMenuItem;

        let layout_label = match current_layout_type {
            LayoutType::ForceDirected => "Force",
            LayoutType::Sugiyama => "Tree",
            LayoutType::Cose => "Compound",
            LayoutType::Circular => "Circle",
            LayoutType::Grid => "Grid",
        };

        let graph_pane_for_dropdown = graph_pane_for_layout.clone();

        let layout_selector_bar = gpui::div()
            .flex()
            .items_center()
            .gap_1p5()
            .px_2()
            .py_1()
            .border_b(gpui::px(1.))
            .border_color(border_color)
            .child(
                GpuiDropdownButton::new("layout-mode-selector")
                    .button(Button::new("layout-btn").label(layout_label))
                    .dropdown_menu(move |menu, _event, _cx| {
                        let graph_pane = graph_pane_for_dropdown.clone();
                        [
                            ("Force (ForceAtlas2)", LayoutType::ForceDirected),
                            ("Tree (Sugiyama)", LayoutType::Sugiyama),
                            ("Compound (CoSE)", LayoutType::Cose),
                            ("Circle", LayoutType::Circular),
                            ("Grid", LayoutType::Grid),
                        ]
                        .into_iter()
                        .fold(menu, |menu, (label, layout_type)| {
                            let graph_pane = graph_pane.clone();
                            menu.item(PopupMenuItem::new(label).on_click(
                                move |_event, _window, cx| {
                                    let _ = graph_pane.update(cx, |pane, cx| {
                                        pane.workspace.update(cx, |w, _| {
                                            w.layout_type = layout_type;
                                        });
                                    });
                                },
                            ))
                        })
                    }),
            )
            .child(
                ButtonGroup::new("button-group")
                    .child(Button::new("run-layout").label("Run Layout").on_click(
                        move |_, _, cx| {
                            let _ =
                                graph_pane_for_layout.update(cx, |pane, cx| pane.run_layout(cx));
                        },
                    ))
                    .child({
                        let pane_entity = self.graph_pane.clone();
                        let auto_colors = self.graph_pane.read(cx).auto_node_colors;
                        gpui_component::button::Button::new("toggle_auto_colors")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                let _ = pane_entity.update(cx, |this, cx| {
                                    this.auto_node_colors = !this.auto_node_colors;
                                    this.auto_edge_colors = this.auto_node_colors;
                                    cx.notify();
                                });
                            })
                            .label(if auto_colors {
                                "Auto Color: ON"
                            } else {
                                "Auto Color: OFF"
                            })
                    })
                    .child({
                        let pane_entity = self.graph_pane.clone();
                        let is_ticking = self.graph_pane.read(cx).is_ticking;
                        gpui_component::button::Button::new("toggle_physics")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                let _ = pane_entity.update(cx, |this, cx| {
                                    if this.is_ticking {
                                        this.is_ticking = false;
                                    } else {
                                        this.run_layout(cx);
                                    }
                                });
                            })
                            .label(if is_ticking {
                                "Pause Physics"
                            } else {
                                "Play Physics"
                            })
                    })
                    .child({
                        let pane_entity = self.graph_pane.clone();
                        gpui_component::button::Button::new("fit_view")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                let _ = pane_entity.update(cx, |this, cx| {
                                    this.fit_view(cx);
                                });
                            })
                            .label("Fit View")
                    }),
            );

        // ── Resizable layout assembly ──
        let workspace_column = div().flex_1().h_full().relative().child(
            gpui_component::resizable::h_resizable("document-graph-split")
                .child(
                    gpui_component::resizable::resizable_panel().child(
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .child(doc_tab_bar)
                            .child(
                                div()
                                    .flex_1()
                                    .h(gpui::px(0.))
                                    .w_full()
                                    .overflow_hidden()
                                    .child(self.document_view.clone()),
                            ),
                    ),
                )
                .child(
                    gpui_component::resizable::resizable_panel().child(
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .child(graph_tab_bar)
                            .child(layout_selector_bar)
                            .child(
                                div()
                                    .flex_1()
                                    .h(gpui::px(0.))
                                    .child(self.graph_pane.clone()),
                            ),
                    ),
                ),
        );

        let viewport_width = _window.viewport_size().width;
        let explorer_min = viewport_width * 0.15;
        let explorer_max = viewport_width * 0.5;

        let main_splitter = gpui_component::resizable::h_resizable("explorer-workspace")
            .child(
                gpui_component::resizable::resizable_panel()
                    .size(gpui::px(250.))
                    .size_range(explorer_min..explorer_max)
                    .child(file_explorer),
            )
            .child(gpui_component::resizable::resizable_panel().child(workspace_column));

        // ── Title bar ──
        let title = self.document_home.read(cx).title.clone();
        let title_bar = TitleBar {
            settings_open: self.settings_window.is_some(),
            title,
            view: cx.entity().clone(),
        };

        // ── Bottom status bar ──
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

        let bottom_bar = window_chrome::render_bottom_status_bar(
            active_file_str,
            workspace.lsp_client.is_some(),
            theme_name,
            sidebar_bg,
            border_color,
            theme_muted_foreground,
            theme_val.success,
            cx.entity().clone(),
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
