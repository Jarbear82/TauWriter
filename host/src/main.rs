//! TauWriter host — a GPUI desktop application for editing TWXML documents.
//!
//! Architecture:
//! - `ui::` — the DemoView component (window, tabs, panels)
//! - `parser::twxml` — TWXML → renderer_schema::Block conversion
//! - `graph_sim` — HubGS force-directed layout engine
//! - `lsp_client` — tauwriter-lsp subprocess management

use gpui::{prelude::*, px, size, App, Application, Bounds, Entity, WindowBounds, WindowOptions};
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
use ui::{build_file_tree, ActiveTab, DemoView, DocumentHome};

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
    let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
    let opened = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_decorations: Some(gpui::WindowDecorations::Client), // Disable native title bar for CSD
            ..Default::default()
        },
        move |window, cx| {
            // Load and parse twxml
            let path = PathBuf::from(&twxml_path);
            let (title, author, blocks) = match load_and_parse_twxml(&twxml_path) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Warning: Failed to load twxml: {err:#}. Using empty placeholder.");
                    (
                        "Error Loading Document".to_string(),
                        "System".to_string(),
                        vec![Block::Paragraph {
                            runs: vec![TextRun::new(format!("Could not load document: {err:#}"))],
                        }],
                    )
                }
            };

            let document_home = cx.new(|_| DocumentHome {
                title,
                author,
                blocks,
            });

            let view = cx.new(|cx| ui::DocumentView::new(document_home.clone(), cx));

            // Build file tree — use fallible path resolution
            let workspace_root = resolve_workspace_root().unwrap_or_else(|| PathBuf::from("."));
            let file_tree = build_file_tree(&workspace_root);

            // Load and watch themes from local themes directory
            let themes_dir = workspace_root.join("themes");
            let _ = gpui_component::ThemeRegistry::watch_dir(themes_dir, cx, |_| {});

            // Initialize input state for XML Editor
            let input_state = cx.new(|cx| {
                gpui_component::input::InputState::new(window, cx)
                    .multi_line(true)
                    .code_editor("xml")
                    .line_number(true)
            });

            // Set initial XML Editor content
            let xml_content = std::fs::read_to_string(&path).unwrap_or_default();
            input_state.update(cx, |state, cx| {
                state.set_value(xml_content.clone(), window, cx);
            });

            // Initialize Graph data
            let mut graph_nodes = Vec::new();
            let mut graph_edges = Vec::new();
            let mut def_nodes = Vec::new();
            let mut def_edges = Vec::new();
            let hubgs_path = path.with_extension("hubgs");
            let target_hubgs = if hubgs_path.exists() {
                Some(hubgs_path)
            } else {
                graph_sim::find_any_hubgs(&workspace_root)
            };

            if let Some(hp) = target_hubgs {
                if let Ok((defs, instances)) = graph_sim::parse_hubgs_file(&hp) {
                    let (n, e) = graph_sim::run_graph_simulation(&instances, 500.0, 500.0);
                    graph_nodes = n;
                    graph_edges = e;

                    let (dn, de) = graph_sim::run_def_simulation(&defs, 500.0, 500.0);
                    def_nodes = dn;
                    def_edges = de;
                }
            }

            let (diag_tx, mut diag_rx) =
                tokio::sync::mpsc::unbounded_channel::<(String, Vec<Diagnostic>)>();
            let demo_view_handle: std::sync::Arc<std::sync::Mutex<Option<Entity<DemoView>>>> =
                std::sync::Arc::new(std::sync::Mutex::new(None));
            let lsp_client =
                LspClient::new(workspace_root.clone(), diag_tx).map(std::sync::Arc::new);

            let demo_view_handle_clone = demo_view_handle.clone();
            cx.spawn(|cx: &mut gpui::AsyncApp| {
                let cx = (*cx).clone();
                async move {
                    while let Some((_uri, diags)) = diag_rx.recv().await {
                        if let Some(view) = &*demo_view_handle_clone.lock().unwrap() {
                            let _ = cx.update(|cx| {
                                let _ = view.update(cx, |this, cx| {
                                    this.diagnostics = diags;
                                    cx.notify();
                                });
                            });
                        }
                    }
                }
            })
            .detach();

            let demo_view = cx.new(|cx| {
                cx.observe(&document_home, |_, _, cx| cx.notify()).detach();

                // Subscribe to InputEvent::Change to sync XML edits to the Preview
                let subscriptions = vec![cx.subscribe_in(&input_state, window, {
                    let input_state = input_state.clone();
                    let document_home = document_home.clone();
                    let lsp_client = lsp_client.clone();
                    move |this: &mut DemoView,
                          _,
                          ev: &gpui_component::input::InputEvent,
                          _window,
                          cx| match ev {
                        gpui_component::input::InputEvent::Change => {
                            let text = input_state.read(cx).value().to_string();
                            if let Some(ref p) = this.selected_path {
                                let _ = std::fs::write(p, &text);
                                if let Some(ref client) = lsp_client {
                                    client.notify_change(p, &text);
                                }
                            }
                            if let Ok((title, author, blocks)) = parser::load_and_parse_twxml(&text)
                            {
                                document_home.update(cx, |doc, cx| {
                                    doc.title = title;
                                    doc.author = author;
                                    doc.blocks = blocks;
                                    cx.notify();
                                });
                            }
                        }
                        _ => {}
                    }
                })];

                DemoView {
                    document_home,
                    view,
                    selected_path: Some(path.clone()),
                    file_tree,
                    settings_open: false,
                    active_tab: ActiveTab::Document,
                    input_state,
                    _subscriptions: subscriptions,
                    graph_nodes,
                    graph_edges,
                    def_nodes,
                    def_edges,
                    lsp_client: lsp_client.clone(),
                    diagnostics: Vec::new(),
                }
            });

            *demo_view_handle.lock().unwrap() = Some(demo_view.clone());

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
