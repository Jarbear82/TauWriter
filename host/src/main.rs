//! TauWriter host — a GPUI desktop application for editing TWXML documents.
//!
//! Architecture:
//! - `ui::` — the DemoView component (window, tabs, panels)
//! - `parser::twxml` — TWXML → renderer_schema::Block conversion
//! - `graph_sim` — HubGS force-directed layout engine
//! - `lsp_client` — tauwriter-lsp subprocess management

use gpui::{prelude::*, px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use parser::{Block, TextRun};
use std::path::{Path, PathBuf};

mod graph_sim;
mod lsp_client;
mod parser;
mod ui;

#[cfg(test)]
mod lsp_client_tests;

#[cfg(test)]
mod graph_sim_tests;

use lsp_client::{Diagnostic, LspClient};
use parser::load_and_parse_twxml;
use ui::{DemoView, DocumentHome, ParseState, ToggleSettings, SelectDocumentTab, SelectGraphTab};

unsafe extern "C" {
    /// Safety: The function is safe to call as it returns a static, read-only
    /// TSLanguage pointer representing the TWXML grammar definition.
    fn tree_sitter_twxml() -> *const std::ffi::c_void;
}

fn main() {
    env_logger::init();

    // Parse the twxml path first
    let mut twxml_path = "examples/all_elements.twxml".to_string();
    for arg in std::env::args().skip(1) {
        let path = PathBuf::from(&arg);
        if path.exists() {
            if path.extension().map_or(false, |ext| ext == "twxml") {
                twxml_path = arg;
                break;
            }
        }
    }

    let platform = gpui_platform::current_platform(false);
    let twxml_path_clone = twxml_path.clone();
    Application::with_platform(platform).run(move |cx: &mut App| {
        // Initialize gpui_component library
        gpui_component::init(cx);
        open_window(twxml_path_clone, cx);
    });
}

fn open_window(twxml_path: String, cx: &mut App) {
    let workspace_root = match resolve_workspace_root() {
        Some(root) => root,
        None => {
            eprintln!("Error: Failed to resolve workspace root. Make sure the application is run from within the source tree.");
            cx.quit();
            return;
        }
    };

    let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
    let opened = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_decorations: Some(gpui::WindowDecorations::Client), // Disable native title bar for CSD
            ..Default::default()
        },
        move |window, cx| {
            // Bind global/view keys
            cx.bind_keys([
                gpui::KeyBinding::new("ctrl-s", ToggleSettings, None),
                gpui::KeyBinding::new("ctrl-1", SelectDocumentTab, None),
                gpui::KeyBinding::new("ctrl-2", SelectGraphTab, None),
            ]);

            // Load and parse twxml
            let path = PathBuf::from(&twxml_path);
            let (title, author, metadata, blocks) = match load_and_parse_twxml(&twxml_path) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Warning: Failed to load twxml: {err:#}. Using empty placeholder.");
                    (
                         "Error Loading Document".to_string(),
                         "System".to_string(),
                         Vec::new(),
                         vec![Block::Paragraph {
                             runs: vec![TextRun::new(format!("Could not load document: {err:#}"))],
                             id: None,
                             attributes: Vec::new(),
                             range: None,
                         }],
                    )
                }
            };

            let document_home = cx.new(|_| DocumentHome {
                title: title.into(),
                author: author.into(),
                metadata,
                blocks,
                parse_state: ParseState::Synced,
                hubgs_instances: std::collections::HashMap::new(),
            });

            // Build workspace model
            let workspace = cx.new(|_| ui::Workspace::new(workspace_root.clone()));

            // Load and watch themes from local themes directory
            let themes_dir = workspace_root.join("themes");
            let _ = gpui_component::ThemeRegistry::watch_dir(themes_dir, cx, |_| {});

            // Register custom tree-sitter language for twxml
            // Safety: The transmute from a raw pointer to `tree_sitter::Language` is safe because
            // `Language` is a transparent wrapper around a raw TSLanguage pointer (`*const std::ffi::c_void`),
            // and `tree_sitter_twxml()` is guaranteed to return a valid `*const std::ffi::c_void`.
            let language: tree_sitter::Language = unsafe {
                std::mem::transmute(tree_sitter_twxml())
            };
            let highlights = include_str!("../../extension/languages/twxml/highlights.scm");
            let config = gpui_component::highlighter::LanguageConfig::new(
                "twxml",
                language,
                vec![],
                highlights,
                "",
                "",
            );
            gpui_component::highlighter::LanguageRegistry::singleton().register("twxml", &config);

            // Initialize input state for XML Editor
            let input_state = cx.new(|cx| {
                gpui_component::input::InputState::new(window, cx)
                    .multi_line(true)
                    .code_editor("twxml")
                    .line_number(true)
            });

            let sidebar = cx.new(|cx| ui::SidebarView::new(workspace.clone(), cx));
            let document_view = cx.new(|cx| ui::DocumentView::new(workspace.clone(), document_home.clone(), input_state.clone(), cx));
            let graph_pane = cx.new(|cx| ui::GraphPaneView::new(workspace.clone(), cx));

            // Set initial XML Editor content
            let xml_content = std::fs::read_to_string(&path).unwrap_or_default();
            input_state.update(cx, |state, cx| {
                state.set_value(xml_content.clone(), window, cx);
            });

            let (diag_tx, mut diag_rx) =
                tokio::sync::mpsc::unbounded_channel::<(String, Vec<Diagnostic>)>();
            let lsp_client =
                LspClient::new(workspace_root.clone(), diag_tx).map(std::sync::Arc::new);

            workspace.update(cx, |w, _| {
                w.lsp_client = lsp_client.clone();
                w.selected_path = Some(path.clone());
            });

            let workspace_clone = workspace.clone();
            cx.spawn(|cx: &mut gpui::AsyncApp| {
                let cx = cx.clone();
                let workspace = workspace_clone;
                async move {
                    while let Some((_uri, diags)) = diag_rx.recv().await {
                        let _ = cx.update(|cx| {
                            workspace.update(cx, |this, cx| {
                                this.diagnostics = diags;
                                cx.notify();
                            });
                        });
                    }
                }
            })
            .detach();

            let demo_view = cx.new(|cx| {
                cx.observe(&document_home, |_, _, cx| cx.notify()).detach();
                cx.observe(&workspace, |_, _, cx| cx.notify()).detach();

                // Subscribe to SidebarView file selection event
                let sidebar_sub = cx.subscribe_in(&sidebar, window, {
                    move |this: &mut DemoView, _sidebar, ev: &ui::sidebar::SidebarEvent, window, cx| {
                        match ev {
                            ui::sidebar::SidebarEvent::FileSelected(path) => {
                                this.select_file(path.clone(), window, cx);
                            }
                        }
                    }
                });

                // Subscribe to InputEvent::Change to sync XML edits to the Preview
                let input_sub = cx.subscribe_in(&input_state, window, {
                    let input_state = input_state.clone();
                    let document_home = document_home.clone();
                    let workspace = workspace.clone();
                    move |_this: &mut DemoView,
                          _,
                          ev: &gpui_component::input::InputEvent,
                          _window,
                          cx| match ev {
                        gpui_component::input::InputEvent::Change => {
                            let text = input_state.read(cx).value().to_string();
                            let (selected_path, lsp_client) = workspace.update(cx, |w, _| {
                                (w.selected_path.clone(), w.lsp_client.clone())
                            });

                            if let Some(ref p) = selected_path {
                                let _ = std::fs::write(p, &text);
                                if let Some(ref client) = lsp_client {
                                    client.notify_change(p, &text);
                                }
                            }
                            match parser::parse_twxml(&text) {
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
                                        doc.parse_state = ParseState::OutOfSync { error: err.to_string() };
                                        cx.notify();
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                });

                DemoView {
                    focus_handle: cx.focus_handle(),
                    workspace,
                    sidebar,
                    document_view,
                    graph_pane,
                    active_tab: ui::ActiveTab::Document,
                    settings_open: false,
                    document_home,
                    input_state,
                    _subscriptions: vec![sidebar_sub, input_sub],
                }
            });

            if let Some(ref client) = lsp_client {
                client.notify_open(&path, &xml_content);
            }

            cx.new(|cx| gpui_component::Root::new(demo_view, window, cx))
        },
    );

    if let Err(error) = opened {
        eprintln!("Error: Failed to open window: {error:#}");
        cx.quit();
        return;
    }

    cx.activate(true);
}

/// Resolve the workspace root (parent of CARGO_MANIFEST_DIR). Returns `None` if
/// the path cannot be determined — this is a valid scenario when the binary runs
/// from an unusual location.
fn resolve_workspace_root() -> Option<PathBuf> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    Some(base.to_path_buf())
}
