//! Sidebar rendering — file explorer and tab bar.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate navigation UI logic.
//! [user-review: split required] See task ticket for splitting rationale.

use gpui::prelude::*;
use gpui::{Entity, EventEmitter};

use std::path::PathBuf;

use super::{FileNode, Workspace};

pub(crate) enum SidebarEvent {
    FileSelected(PathBuf),
}

pub(crate) struct SidebarView {
    workspace: Entity<Workspace>,
}

impl SidebarView {
    pub(crate) fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        Self { workspace }
    }
}

impl EventEmitter<SidebarEvent> for SidebarView {}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let theme_muted_foreground = theme.muted_foreground;
        let border_color = theme.border;
        let sidebar_bg = theme.sidebar;

        let (file_tree, selected_path) = {
            let workspace = self.workspace.read(cx);
            (workspace.file_tree.clone(), workspace.selected_path.clone())
        };

        // Recursive helper to render the file tree nodes into a list
        fn render_nodes(
            nodes: &[FileNode],
            depth: usize,
            selected_path: &Option<PathBuf>,
            cx: &mut Context<SidebarView>,
            items: &mut Vec<gpui::Div>,
        ) {
            let theme = gpui_component::Theme::global(cx);
            let theme_primary = theme.primary;
            let theme_foreground = theme.foreground;
            let theme_muted_foreground = theme.muted_foreground;
            let theme_accent = theme.accent;

            for node in nodes {
                let padding_left = gpui::px((depth * 12 + 12) as f32);
                let is_selected = selected_path.as_ref().map_or(false, |p| {
                    p.canonicalize().ok() == node.path.canonicalize().ok()
                });
                let color = if is_selected {
                    theme_primary
                } else if !node.is_dir {
                    theme_foreground
                } else {
                    theme_muted_foreground
                };

                let icon = if node.is_dir { "📁 " } else { "📄 " };
                let path = node.path.clone();

                let mut item = gpui::div()
                    .flex()
                    .items_center()
                    .h(gpui::px(26.))
                    .pl(padding_left)
                    .text_size(gpui::px(12.))
                    .text_color(color)
                    .hover(|s| s.bg(theme_accent));

                if !node.is_dir {
                    // path is cloned to be moved into the listener closure, and cloned
                    // again inside the closure because the listener can be invoked multiple times.
                    item = item.on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |_, _, _, cx| {
                            cx.emit(SidebarEvent::FileSelected(path.clone()));
                        }),
                    );
                }

                item = item.child(format!("{}{}", icon, node.name));
                items.push(item);

                if !node.children.is_empty() {
                    render_nodes(&node.children, depth + 1, selected_path, cx, items);
                }
            }
        }

        let mut file_items = Vec::new();
        render_nodes(&file_tree, 0, &selected_path, cx, &mut file_items);

        gpui::div()
            .w_full()
            .h_full()
            .bg(sidebar_bg)
            .border_r(gpui::px(1.))
            .border_color(border_color)
            .flex()
            .flex_col()
            .child(
                gpui::div()
                    .p_3()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme_muted_foreground)
                    .child("WORKSPACE FILES"),
            )
            .child(
                gpui::div()
                    .id("file_list")
                    .flex_1()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .children(file_items),
            )
    }
}

// ─── TabBar component ────────────────────────────────────────────────────────

#[derive(IntoElement)]
pub(crate) struct TabBar {
    pub(crate) active_tab: crate::ui::ActiveTab,
    pub(crate) view: Entity<crate::ui::DemoView>,
}

impl RenderOnce for TabBar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let bg_color = theme.background;
        let sidebar_bg = theme.sidebar;
        let border_color = theme.border;
        let theme_muted_foreground = theme.muted_foreground;
        let theme_accent = theme.accent;
        let theme_primary = theme.primary;

        let doc_tab_active = self.active_tab == crate::ui::ActiveTab::Document;
        let graph_tab_active = self.active_tab == crate::ui::ActiveTab::Graph;
        let view_doc = self.view.clone();
        let view_graph = self.view.clone();

        gpui::div()
            .flex()
            .h(gpui::px(38.))
            .bg(sidebar_bg)
            .border_b(gpui::px(1.))
            .border_color(border_color)
            .child(
                // Document Tab
                gpui::div()
                    .id("tab_document")
                    .px_4()
                    .py_2()
                    .border_r(gpui::px(1.))
                    .border_color(border_color)
                    .bg(if doc_tab_active { bg_color } else { sidebar_bg })
                    .text_color(if doc_tab_active { theme_primary } else { theme_muted_foreground })
                    .font_weight(if doc_tab_active { gpui::FontWeight::BOLD } else { gpui::FontWeight::NORMAL })
                    .text_size(gpui::px(12.))
                    .hover(|s| s.bg(theme_accent))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        move |_, _, cx| {
                            view_doc.update(cx, |this, cx| {
                                this.active_tab = crate::ui::ActiveTab::Document;
                                cx.notify();
                            });
                        },
                    )
                    .child("📝 Document Editor & Preview"),
            )
            .child(
                // Graph Tab
                gpui::div()
                    .id("tab_graph")
                    .px_4()
                    .py_2()
                    .border_r(gpui::px(1.))
                    .border_color(border_color)
                    .bg(if graph_tab_active { bg_color } else { sidebar_bg })
                    .text_color(if graph_tab_active { theme_primary } else { theme_muted_foreground })
                    .font_weight(if graph_tab_active { gpui::FontWeight::BOLD } else { gpui::FontWeight::NORMAL })
                    .text_size(gpui::px(12.))
                    .hover(|s| s.bg(theme_accent))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        move |_, _, cx| {
                            view_graph.update(cx, |this, cx| {
                                this.active_tab = crate::ui::ActiveTab::Graph;
                                cx.notify();
                            });
                        },
                    )
                    .child("🕸️ Knowledge Graph (HubGS)"),
            )
    }
}
