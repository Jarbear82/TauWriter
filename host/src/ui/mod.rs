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
mod graph_pane;
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
pub(crate) use sidebar::{SidebarView, TabBar};
pub(crate) use titlebar::{SettingsView, TitleBar};

/// Which tab is currently active in the main content area.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveTab {
    RawEditor,
    RenderedPreview,
    DefinitionsGraph,
    InstancesGraph,
}

// ─── Workspace Model ────────────────────────────────────────────────────────

pub(crate) struct Workspace {
    pub(crate) file_tree: Vec<FileNode>,
    pub(crate) selected_path: Option<PathBuf>,
    pub(crate) lsp_client: Option<std::sync::Arc<LspClient>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) active_tab: ActiveTab,
}

impl Workspace {
    pub(crate) fn new(workspace_root: PathBuf) -> Self {
        let file_tree = build_file_tree(&workspace_root);
        Self {
            file_tree,
            selected_path: None,
            lsp_client: None,
            diagnostics: Vec::new(),
            active_tab: ActiveTab::RawEditor,
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
            w.active_tab = ActiveTab::RawEditor;
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
            w.active_tab = ActiveTab::DefinitionsGraph;
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
        // Set selected path immediately to update UI
        self.workspace.update(cx, |w, cx| {
            w.selected_path = Some(path.clone());
            w.diagnostics.clear();
            cx.notify();
        });

        let document_home = self.document_home.clone();
        let input_state = self.input_state.clone();
        let workspace = self.workspace.clone();
        let window_handle = window.window_handle();

        cx.spawn(
            move |this: gpui::WeakEntity<MainView>, cx: &mut gpui::AsyncApp| {
                let cx = cx.clone();
                async move {
                    let path_clone = path.clone();

                    // 1. Spawn file reading and parsing task in background thread
                    let task_twxml = cx.background_executor().spawn(async move {
                        let content = std::fs::read_to_string(&path_clone).unwrap_or_default();
                        let parsed =
                            crate::parser::load_and_parse_twxml(&path_clone.to_string_lossy()).ok();
                        (content, parsed)
                    });

                    // 2. Spawn HubGS definitions & instances loading task in background thread
                    let hubgs_path = path.with_extension("hubgs");
                    let workspace_root = crate::utils::resolve_workspace_root()
                        .expect("CARGO_MANIFEST_DIR must resolve to a parent directory");

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

                    // Await both tasks in parallel
                    let (xml_content, parsed_twxml) = task_twxml.await;
                    let hubgs_data = task_hubgs.await;

                    // 3. Update main UI state on the GUI thread
                    let _ = cx.update(|cx| {
                        // Update input state value
                        let _ = window_handle.update(cx, |_, window, cx| {
                            input_state.update(cx, |state, cx| {
                                state.set_value(xml_content.clone(), window, cx);
                            });
                        });

                        // Notify LSP
                        let lsp_client = workspace.read(cx).lsp_client.clone();
                        if let Some(ref client) = lsp_client {
                            client.notify_open(&path, &xml_content);
                        }

                        // Update DocumentHome with parsed data
                        let is_twxml = path.extension().map_or(false, |ext| ext == "twxml");
                        document_home.update(cx, |doc, cx| {
                            doc.hubgs_instances = hubgs_data;
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

                        let _ = this.update(cx, |_, cx| {
                            cx.notify();
                        });
                    });
                }
            },
        )
        .detach();
    }
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

        // Workspace column (tabs + content)
        let active_tab = self.workspace.read(cx).active_tab;
        let tab_bar = TabBar {
            active_tab,
            view: cx.entity().clone(),
        };

        let content_pane = match active_tab {
            ActiveTab::RawEditor | ActiveTab::RenderedPreview => {
                self.document_view.clone().into_any_element()
            }
            ActiveTab::DefinitionsGraph | ActiveTab::InstancesGraph => {
                self.graph_pane.clone().into_any_element()
            }
        };

        let workspace_column = div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .child(tab_bar)
            .child(div().flex_1().h(gpui::px(0.)).child(content_pane));

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
        let workspace = self.workspace.read(cx);
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
