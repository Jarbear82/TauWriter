//! Demo view — the top-level GPUI component for TauWriter.
//!
//! Extracted rendering helpers into submodules ([graph_pane], [titlebar], [sidebar]) to
//! eliminate near-duplicate logic and reduce file length.
//! [user-review: split required] 1103-line monolith split per refactoring task ticket.

use gpui::{div, prelude::*, Entity, Hsla, Subscription};
use gpui_component::input::InputState;
use crate::parser::{Block, TextRun};
use std::path::Path;

mod document_view;
mod graph_pane;
mod sidebar;
mod titlebar;
mod tree_view;

pub(crate) use document_view::DocumentView;

pub(crate) use tree_view::{build_file_tree, FileNode};

/// Which tab is currently active in the main content area.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveTab {
    Document,
    Graph,
}

use super::graph_sim::GraphNode;

pub(crate) use super::lsp_client::Diagnostic;
pub(crate) use super::lsp_client::LspClient;

// ─── DemoView struct ────────────────────────────────────────────────────────

pub(crate) struct DemoView {
    pub(crate) document_home: Entity<DocumentHome>,
    pub(crate) view: Entity<DocumentView>,
    pub(crate) selected_path: Option<std::path::PathBuf>,
    pub(crate) file_tree: Vec<FileNode>,
    pub(crate) settings_open: bool,
    pub(crate) active_tab: ActiveTab,
    pub(crate) input_state: Entity<InputState>,
    pub(crate) _subscriptions: Vec<Subscription>,
    pub(crate) graph_nodes: Vec<GraphNode>,
    pub(crate) graph_edges: Vec<(usize, usize, String)>,
    pub(crate) def_nodes: Vec<GraphNode>,
    pub(crate) def_edges: Vec<(usize, usize, String)>,
    pub(crate) lsp_client: Option<std::sync::Arc<LspClient>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

// ─── DocumentHome & traits ──────────────────────────────────────────────────

pub(crate) struct DocumentHome {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) blocks: Vec<Block>,
}



// ─── DemoView methods ───────────────────────────────────────────────────────

impl DemoView {
    pub(crate) fn select_file(
        &mut self,
        path: std::path::PathBuf,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if let Ok(xml_content) = std::fs::read_to_string(&path) {
            // Update XML Editor
            self.input_state.update(cx, |state, cx| {
                state.set_value(xml_content.clone(), window, cx);
            });

            if let Some(ref client) = self.lsp_client {
                client.notify_open(&path, &xml_content);
            }
            self.diagnostics.clear();

            // Update Document Home
            let is_twxml = path.extension().map_or(false, |ext| ext == "twxml");
            if is_twxml {
                if let Ok((title, author, blocks)) =
                    super::parser::load_and_parse_twxml(&path.to_string_lossy())
                {
                    self.document_home.update(cx, |doc, cx| {
                        doc.title = title;
                        doc.author = author;
                        doc.blocks = blocks;
                        cx.notify();
                    });
                }
            } else {
                self.document_home.update(cx, |doc, cx| {
                    doc.title = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    doc.author = "System".to_string();
                    doc.blocks = vec![Block::Paragraph {
                        runs: vec![TextRun::new(
                            "Visual preview is only available for .twxml documents.",
                        )],
                    }];
                    cx.notify();
                });
            }

            self.selected_path = Some(path.clone());

            // Try to find and load matching hubgs
            let hubgs_path = path.with_extension("hubgs");
            let target_hubgs = if hubgs_path.exists() {
                Some(hubgs_path)
            } else {
                let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_path_buf();
                super::graph_sim::find_any_hubgs(&workspace_root)
            };

            if let Some(hp) = target_hubgs {
                if let Ok((defs, instances)) = super::graph_sim::parse_hubgs_file(&hp) {
                    let (nodes, edges) =
                        super::graph_sim::run_graph_simulation(&instances, 500.0, 500.0);
                    self.graph_nodes = nodes;
                    self.graph_edges = edges;

                    let (dnodes, dedges) =
                        super::graph_sim::run_def_simulation(&defs, 500.0, 500.0);
                    self.def_nodes = dnodes;
                    self.def_edges = dedges;
                }
            } else {
                self.graph_nodes.clear();
                self.graph_edges.clear();
                self.def_nodes.clear();
                self.def_edges.clear();
            }

            cx.notify();
        }
    }
}

// ─── Render implementation (delegated to submodules) ─────────────────────────

impl gpui::Render for DemoView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui::*;

        let title = self.document_home.read(cx).title.clone();

        // Grab current theme colors from gpui-component and copy them to release the borrow on cx
        let (
            bg_color,
            fg_color,
            border_color,
            sidebar_bg,
            active_accent,
            theme_accent,
            theme_button,
            theme_button_foreground,
            theme_group_box,
            theme_muted_foreground,
            theme_primary_foreground,
            theme_primary,
            theme_foreground,
            theme_name,
        ) = {
            let theme_val = gpui_component::Theme::global(cx);
            (
                theme_val.background,
                theme_val.foreground,
                theme_val.border,
                theme_val.sidebar,
                theme_val.primary,
                theme_val.accent,
                theme_val.button,
                theme_val.button_foreground,
                theme_val.group_box,
                theme_val.muted_foreground,
                theme_val.primary_foreground,
                theme_val.primary,
                theme_val.foreground,
                theme_val.theme_name().to_string(),
            )
        };

        // File explorer (left sidebar)
        let file_explorer = sidebar::render_file_explorer(
            cx,
            &theme_muted_foreground,
            &border_color,
            &sidebar_bg,
            &self.file_tree,
            &self.selected_path,
        );

        // Tab header buttons
        let tab_bar = sidebar::render_tab_bar(
            &bg_color,
            &sidebar_bg,
            &border_color,
            &theme_muted_foreground,
            &active_accent,
            &theme_primary,
            self.active_tab,
            cx,
        );

        // Content pane (tab-selected)
        let content_pane = match self.active_tab {
            ActiveTab::Document => self.render_document_content(
                &bg_color,
                &fg_color,
                &border_color,
                &sidebar_bg,
                &theme_group_box,
                &theme_muted_foreground,
                &theme_primary,
                &theme_foreground,
            ),
            ActiveTab::Graph => self.render_graph_content(
                &bg_color,
                &fg_color,
                &border_color,
                &sidebar_bg,
                &active_accent,
                &theme_muted_foreground,
                &theme_foreground,
            ),
        };

        // Workspace panel
        let workspace_panel = div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .child(tab_bar)
            .child(content_pane);

        let mut workspace_children = div()
            .flex_1()
            .h(gpui::px(0.)) // Force height constraint in flex layout
            .flex()
            .child(file_explorer)
            .child(workspace_panel);

        // Settings panel (optional right sidebar)
        if self.settings_open {
            let settings_panel = titlebar::render_settings(
                &sidebar_bg,
                &border_color,
                &theme_muted_foreground,
                &theme_primary,
                &theme_accent,
                &theme_foreground,
                cx,
            );
            workspace_children = workspace_children.child(settings_panel);
        }

        // Title bar (CSD)
        let title_bar = titlebar::render_titlebar(
            &bg_color,
            &sidebar_bg,
            &border_color,
            &theme_muted_foreground,
            &active_accent,
            &theme_button,
            &theme_button_foreground,
            &theme_primary_foreground,
            self.settings_open,
            &title,
            cx,
        );

        // Bottom status bar
        let active_file_str = self.selected_path.as_ref().map_or("No file selected".to_string(), |p| {
            p.file_name().unwrap_or_default().to_string_lossy().to_string()
        });
        
        let lsp_status = if self.lsp_client.is_some() {
            "🟢 LSP: Connected"
        } else {
            "🔴 LSP: Offline"
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
                    .gap_4()
                    .child(div().child(format!("📁 {}", active_file_str)))
                    .child(div().child(lsp_status))
            )
            .child(
                div()
                    .child(format!("Theme: {}", theme_name))
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(bg_color)
            .text_color(fg_color)
            .child(title_bar)
            .child(workspace_children)
            .child(bottom_bar)
    }
}

// ─── Document content pane (still in mod.rs — core DemoView logic) ────────────

impl DemoView {
    fn render_document_content(
        &self,
        _bg_color: &Hsla,
        _fg_color: &Hsla,
        border_color: &Hsla,
        sidebar_bg: &Hsla,
        theme_group_box: &Hsla,
        theme_muted_foreground: &Hsla,
        _theme_primary: &Hsla,
        theme_foreground: &Hsla,
    ) -> gpui::Div {
        let border_color = *border_color;
        let sidebar_bg = *sidebar_bg;

        // LSP Diagnostics content
        let diagnostics_content: Vec<gpui::AnyElement> = if self.diagnostics.is_empty() {
            vec![gpui::div()
                .text_color(gpui::rgb(0x2ECC71))
                .text_size(gpui::px(12.))
                .child("✓ No diagnostic issues found.")
                .into_any_element()]
        } else {
            self.diagnostics
                .iter()
                .enumerate()
                .map(|(idx, diag)| {
                    let is_error = diag.severity == 1;
                    let severity_icon = if is_error { "🔴" } else { "🟡" };
                    let color = if is_error {
                        gpui::rgb(0xE74C3C)
                    } else {
                        gpui::rgb(0xF39C12)
                    };
                    let line_val = diag.line + 1;
                    let message = diag.message.clone();
                    gpui::div()
                        .id(("diag", idx))
                        .flex()
                        .gap_2()
                        .py_1()
                        .text_size(gpui::px(11.))
                        .text_color(*theme_foreground)
                        .child(gpui::div().text_color(color).child(severity_icon))
                        .child(
                            gpui::div()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(format!("Line {}:", line_val)),
                        )
                        .child(gpui::div().child(message))
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        };

        // Document tab: split between XML Editor (left) and WASM Preview (right)
        div()
            .flex_1()
            .flex()
            .size_full()
            .child(
                // Left Pane: XML Editor & Diagnostics
                gpui::div()
                    .flex_1()
                    .h_full()
                    .border_r(gpui::px(1.))
                    .border_color(border_color)
                    .flex()
                    .flex_col()
                    .child(
                        gpui::div()
                            .p_2()
                            .bg(sidebar_bg)
                            .border_b(gpui::px(1.))
                            .border_color(border_color)
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(*theme_muted_foreground)
                            .child("XML SOURCE EDITOR"),
                    )
                    .child(
                        gpui::div()
                            .id("source_editor_container")
                            .flex_1()
                            .h(gpui::px(0.)) // Force height constraint for editor container
                            .overflow_y_scroll()
                            .p_4()
                            .bg(*theme_group_box)
                            .child(
                                gpui::div()
                                    .size_full()
                                    .child(gpui_component::input::Input::new(&self.input_state).size_full()),
                            ),
                    )
                    .child(
                        // LSP Diagnostics Pane at the bottom of the editor
                        gpui::div()
                            .h(gpui::px(180.))
                            .border_t(gpui::px(1.))
                            .border_color(border_color)
                            .bg(sidebar_bg)
                            .flex()
                            .flex_col()
                            .child(
                                gpui::div()
                                    .p_2()
                                    .bg(sidebar_bg)
                                    .border_b(gpui::px(1.))
                                    .border_color(border_color)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(*theme_muted_foreground)
                                    .child("LSP DIAGNOSTICS"),
                            )
                            .child(
                                gpui::div()
                                    .id("diagnostics_list")
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .p_2()
                                    .children(diagnostics_content),
                            ),
                    ),
            )
            .child(
                // Right Pane: WASM Preview
                gpui::div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(
                        gpui::div()
                            .p_2()
                            .bg(sidebar_bg)
                            .border_b(gpui::px(1.))
                            .border_color(border_color)
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(*theme_muted_foreground)
                            .child("RENDERED PREVIEW"),
                    )
                    .child(
                        gpui::div()
                            .id("preview_container")
                            .flex_1()
                            .h(gpui::px(0.)) // Force height constraint for preview container
                            .overflow_y_scroll()
                            .child(self.view.clone()),
                    ),
            )
    }

    fn render_graph_content(
        &self,
        bg_color: &Hsla,
        fg_color: &Hsla,
        border_color: &Hsla,
        sidebar_bg: &Hsla,
        active_accent: &Hsla,
        theme_muted_foreground: &Hsla,
        _theme_foreground: &Hsla,
    ) -> gpui::Div {
        let left_panel = graph_pane::GraphPanel {
            nodes: self.def_nodes.clone(),
            edges: self.def_edges.clone(),
            label: "DEFINITIONS SCHEMA GRAPH",
        };
        let right_panel = graph_pane::GraphPanel {
            nodes: self.graph_nodes.clone(),
            edges: self.graph_edges.clone(),
            label: "INSTANCES RELATION GRAPH",
        };

        graph_pane::render_graph_panels(
            left_panel,
            right_panel,
            bg_color,
            fg_color,
            border_color,
            sidebar_bg,
            active_accent,
            theme_muted_foreground,
        )
    }
}
