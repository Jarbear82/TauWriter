//! Demo view — the top-level GPUI component for TauWriter.
//!
//! Extracted rendering helpers into submodules ([graph_pane], [titlebar], [sidebar]) to
//! eliminate near-duplicate logic and reduce file length.
//! [user-review: split required] 1103-line monolith split per refactoring task ticket.

use gpui::{prelude::*, Entity, Subscription, div};
use gpui_component::input::InputState;
use crate::parser::{Block, TextRun};
use std::path::{Path, PathBuf};

mod document_view;
mod graph_pane;
pub(crate) mod sidebar;
pub(crate) mod titlebar;
mod tree_view;

pub(crate) use document_view::DocumentView;
pub(crate) use tree_view::{build_file_tree, FileNode};
pub(crate) use super::lsp_client::Diagnostic;
pub(crate) use super::lsp_client::LspClient;

pub(crate) use sidebar::{TabBar, SidebarView};
pub(crate) use titlebar::{SettingsPanel, TitleBar};
pub(crate) use graph_pane::GraphPaneView;

/// Which tab is currently active in the main content area.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveTab {
    Document,
    Graph,
}

// ─── Workspace Model ────────────────────────────────────────────────────────

pub(crate) struct Workspace {
    pub(crate) file_tree: Vec<FileNode>,
    pub(crate) selected_path: Option<PathBuf>,
    pub(crate) lsp_client: Option<std::sync::Arc<LspClient>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl Workspace {
    pub(crate) fn new(workspace_root: PathBuf) -> Self {
        let file_tree = build_file_tree(&workspace_root);
        Self {
            file_tree,
            selected_path: None,
            lsp_client: None,
            diagnostics: Vec::new(),
        }
    }
}

// ─── DemoView struct ────────────────────────────────────────────────────────

pub(crate) struct DemoView {
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) sidebar: Entity<sidebar::SidebarView>,
    pub(crate) document_view: Entity<DocumentView>,
    pub(crate) graph_pane: Entity<graph_pane::GraphPaneView>,
    pub(crate) active_tab: ActiveTab,
    pub(crate) settings_open: bool,
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
        
        cx.spawn(move |this: gpui::WeakEntity<DemoView>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let path_clone = path.clone();
                
                // 1. Read file and parse TWXML in background thread
                let (xml_content, parsed_twxml) = cx.background_executor().spawn(async move {
                    let content = std::fs::read_to_string(&path_clone).unwrap_or_default();
                    let parsed = crate::parser::load_and_parse_twxml(&path_clone.to_string_lossy()).ok();
                    (content, parsed)
                }).await;

                // 2. Load HubGS definitions & instances in background thread
                let hubgs_path = path.with_extension("hubgs");
                let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .to_path_buf();
                    
                let hubgs_data = cx.background_executor().spawn(async move {
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
                                    (inst.type_name.clone(), inst.name.clone(), inst.links.clone()),
                                );
                            }
                        }
                    }
                    hubgs_map
                }).await;

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
                                doc.title = title;
                                doc.author = author;
                                doc.metadata = metadata;
                                doc.blocks = blocks;
                                doc.parse_state = ParseState::Synced;
                            } else {
                                doc.title = "Error Loading Document".to_string();
                                doc.author = "System".to_string();
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
                            doc.title = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            doc.author = "System".to_string();
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

                    let _ = this.update(cx, |_, cx| {
                        cx.notify();
                    });
                });
            }
        }).detach();
    }
}

// ─── Render implementation ───────────────────────────────────────────────────

impl gpui::Render for DemoView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.document_home.read(cx).title.clone();
        
        let theme_val = gpui_component::Theme::global(cx);
        let bg_color = theme_val.background;
        let fg_color = theme_val.foreground;
        let border_color = theme_val.border;
        let sidebar_bg = theme_val.sidebar;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_name = theme_val.theme_name().to_string();

        // Left sidebar file explorer
        let file_explorer = self.sidebar.clone();

        // Workspace panel (tabs + main content pane)
        let tab_bar = TabBar {
            active_tab: self.active_tab,
            view: cx.entity().clone(),
        };

        let content_pane = match self.active_tab {
            ActiveTab::Document => self.document_view.clone().into_any_element(),
            ActiveTab::Graph => self.graph_pane.clone().into_any_element(),
        };

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

        if self.settings_open {
            let settings_panel = SettingsPanel;
            workspace_group = workspace_group.child(gpui_component::resizable::resizable_panel().size(gpui::px(300.)).child(settings_panel));
        }

        let title_bar = TitleBar {
            settings_open: self.settings_open,
            title: title.clone(),
            view: cx.entity().clone(),
        };

        // Bottom status bar
        let workspace = self.workspace.read(cx);
        let active_file_str = workspace.selected_path.as_ref().map_or("No file selected".to_string(), |p| {
            p.file_name().unwrap_or_default().to_string_lossy().to_string()
        });
        
        let lsp_status = if workspace.lsp_client.is_some() {
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
