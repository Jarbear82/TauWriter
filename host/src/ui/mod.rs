//! Demo view — the top-level GPUI component for TauWriter.
//!
//! Extracted rendering helpers into submodules ([graph_pane], [titlebar], [sidebar]) to
//! eliminate near-duplicate logic and reduce file length.
//! [user-review: split required] 1103-line monolith split per refactoring task ticket.

use gpui::{prelude::*, Entity, Hsla, Subscription};
use gpui_component::input::{InputState, Position};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParseState {
    Synced,
    OutOfSync { error: String },
}

// ─── DocumentHome & traits ──────────────────────────────────────────────────

pub(crate) struct DocumentHome {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) metadata: Vec<(String, String)>,
    pub(crate) blocks: Vec<Block>,
    pub(crate) parse_state: ParseState,
    pub(crate) hubgs_instances: std::collections::HashMap<String, (String, String, Vec<(String, String)>)>,
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

            let mut hubgs_map = std::collections::HashMap::new();
            if let Some(ref hp) = target_hubgs {
                if let Ok((defs, instances)) = super::graph_sim::parse_hubgs_file(hp) {
                    for inst in &instances {
                        hubgs_map.insert(
                            inst.id.clone(),
                            (inst.type_name.clone(), inst.name.clone(), inst.links.clone()),
                        );
                    }

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

            // Update Document Home
            let is_twxml = path.extension().map_or(false, |ext| ext == "twxml");
            if is_twxml {
                match super::parser::load_and_parse_twxml(&path.to_string_lossy()) {
                    Ok((title, author, metadata, blocks)) => {
                        self.document_home.update(cx, |doc, cx| {
                            doc.title = title;
                            doc.author = author;
                            doc.metadata = metadata;
                            doc.blocks = blocks;
                            doc.hubgs_instances = hubgs_map;
                            doc.parse_state = ParseState::Synced;
                            cx.notify();
                        });
                    }
                    Err(err) => {
                        self.document_home.update(cx, |doc, cx| {
                            doc.title = "Error Loading Document".to_string();
                            doc.author = "System".to_string();
                            doc.metadata = Vec::new();
                            doc.blocks = vec![Block::Paragraph {
                                runs: vec![TextRun::new(format!("Could not load document: {err:#}"))],
                                id: None,
                                attributes: Vec::new(),
                                range: None,
                            }];
                            doc.hubgs_instances = std::collections::HashMap::new();
                            doc.parse_state = ParseState::Synced;
                            cx.notify();
                        });
                    }
                }
            } else {
                self.document_home.update(cx, |doc, cx| {
                    doc.title = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    doc.author = "System".to_string();
                    doc.metadata = Vec::new();
                    doc.blocks = vec![Block::Paragraph {
                        runs: vec![TextRun::new(
                            "Visual preview is only available for .twxml documents.",
                        )],
                        id: None,
                        attributes: Vec::new(),
                        range: None,
                    }];
                    doc.hubgs_instances = std::collections::HashMap::new();
                    doc.parse_state = ParseState::Synced;
                    cx.notify();
                });
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
                _window,
                cx,
            ).into_any_element(),
            ActiveTab::Graph => self.render_graph_content(
                &bg_color,
                &fg_color,
                &border_color,
                &sidebar_bg,
                &active_accent,
                &theme_muted_foreground,
                &theme_foreground,
            ).into_any_element(),
        };

        // Workspace panel
        let workspace_panel = div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .child(tab_bar)
            .child(
                div()
                    .flex_1()
                    .h(gpui::px(0.))
                    .child(content_pane)
            );

        let viewport_width = _window.viewport_size().width;
        let explorer_min = viewport_width * 0.15;
        let explorer_max = viewport_width * 0.5;

        let mut workspace_group = gpui_component::resizable::h_resizable("explorer-workspace")
            .child(gpui_component::resizable::resizable_panel().size(gpui::px(250.)).size_range(explorer_min..explorer_max).child(file_explorer))
            .child(gpui_component::resizable::resizable_panel().child(workspace_panel));

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
            workspace_group = workspace_group.child(gpui_component::resizable::resizable_panel().size(gpui::px(300.)).child(settings_panel));
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
            .child(
                div()
                    .flex_1()
                    .h(gpui::px(0.))
                    .overflow_hidden()
                    .w_full()
                    .child(workspace_group)
            )
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
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
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
                    let input_state = self.input_state.clone();
                    let diag_line = diag.line;
                    gpui::div()
                        .id(("diag", idx))
                        .flex()
                        .gap_2()
                        .py_1()
                        .px_2()
                        .rounded(gpui::px(4.))
                        .text_size(gpui::px(11.))
                        .text_color(*theme_foreground)
                        .hover(|s| s.bg(gpui::rgb(0xe5e7eb)))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            let pos = Position::new(diag_line as u32, 0);
                            input_state.update(cx, |state, cx| {
                                state.set_cursor_position(pos, window, cx);
                            });
                        })
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

        let viewport_width = window.viewport_size().width;
        let preview_min = viewport_width * 0.2;

        let active_file = self.selected_path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "No File".to_string());
        let editor_header = format!("RAW EDITOR: {}", active_file);

        let metadata_doc = self.document_home.read(cx);
        let preview_header = if metadata_doc.title.is_empty() && metadata_doc.author.is_empty() {
            "RENDERED PREVIEW".to_string()
        } else {
            format!("PREVIEW: {} by {}", metadata_doc.title, metadata_doc.author)
        };

        // Frontmatter
        let mut frontmatter = String::new();
        if !metadata_doc.metadata.is_empty() {
            frontmatter.push_str("---\n");
            for (key, val) in &metadata_doc.metadata {
                frontmatter.push_str(&format!("{}: {}\n", key, val));
            }
            frontmatter.push_str("---");
        }
        let frontmatter_el = if !frontmatter.is_empty() {
            Some(
                gpui::div()
                    .mb_4()
                    .p_3()
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(4.))
                    .font_family("Courier New")
                    .text_xs()
                    .text_color(*theme_foreground)
                    .child(frontmatter)
            )
        } else {
            None
        };

        let left_pane = gpui_component::resizable::v_resizable("editor-diagnostics")
            .child(
                gpui_component::resizable::resizable_panel()
                    .child(
                        gpui::div()
                            .size_full()
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
                                    .child(editor_header),
                            )
                            .child(
                                gpui::div()
                                    .id("source_editor_container")
                                    .flex_1()
                                    .p_4()
                                    .bg(*theme_group_box)
                                    .child(
                                        gpui::div()
                                            .size_full()
                                            .children(frontmatter_el)
                                            .child(gpui_component::input::Input::new(&self.input_state).size_full()),
                                    ),
                            )
                    )
            )
            .child(
                gpui_component::resizable::resizable_panel()
                    .size(gpui::px(180.))
                    .size_range(gpui::px(80.)..gpui::px(400.))
                    .child(
                        gpui::div()
                            .size_full()
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
                            )
                    )
            );

        let right_pane = gpui::div()
            .size_full()
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
                    .child(preview_header),
            )
            .child(
                gpui::div()
                    .id("preview_container")
                    .flex_1()
                    .child(self.view.clone()),
            );

        gpui::div()
            .flex_1()
            .size_full()
            .child(
                gpui_component::resizable::h_resizable("editor-preview")
                    .child(gpui_component::resizable::resizable_panel().child(left_pane))
                    .child(gpui_component::resizable::resizable_panel().size_range(preview_min..gpui::Pixels::MAX).child(right_pane))
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
    ) -> impl IntoElement {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_document_home_state_transitions_correctly() {
        // Setup initial DocumentHome state (Synced)
        let mut doc = DocumentHome {
            title: "Test".to_string(),
            author: "Author".to_string(),
            metadata: Vec::new(),
            blocks: vec![],
            parse_state: ParseState::Synced,
            hubgs_instances: std::collections::HashMap::new(),
        };
        assert_eq!(doc.parse_state, ParseState::Synced);

        // Exercise: Transition to OutOfSync due to a parse error
        doc.parse_state = ParseState::OutOfSync {
            error: "Unclosed tag <bold>".to_string(),
        };

        // Verify: Ensure state is OutOfSync with the correct error payload
        match &doc.parse_state {
            ParseState::OutOfSync { error } => {
                assert_eq!(error, "Unclosed tag <bold>");
            }
            _ => panic!("Expected OutOfSync state"),
        }

        // Exercise: Transition back to Synced
        doc.blocks = vec![Block::Heading {
            level: 1,
            text: "Hello".to_string(),
            id: None,
            attributes: Vec::new(),
            range: None,
        }];
        doc.parse_state = ParseState::Synced;

        // Verify: Ensure state is Synced and blocks updated
        assert_eq!(doc.parse_state, ParseState::Synced);
        assert_eq!(doc.blocks.len(), 1);
    }
}
