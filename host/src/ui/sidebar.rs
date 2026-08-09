//! Sidebar rendering — file explorer and tab bar.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate navigation UI logic.
//! [user-review: split required] See task ticket for splitting rationale.

use gpui::prelude::*;
use gpui::{div, uniform_list, Context, Entity, EventEmitter};
use gpui_component::{
    tab::{Tab, TabBar},
    Icon, IconName,
};
use std::path::PathBuf;

use super::tree_view::flatten_file_tree;

use super::Workspace;

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

        let file_tree = {
            let workspace = self.workspace.read(cx);
            workspace.file_tree.clone()
        };

        // Flatten the tree for virtualization checks
        let flat_nodes = flatten_file_tree(&file_tree);

        let file_list_content = if flat_nodes.is_empty() {
            gpui::div()
                .p_3()
                .text_size(gpui::px(12.))
                .text_color(theme_muted_foreground)
                .child("No files in workspace.")
                .into_any_element()
        } else {
            uniform_list(
                "file_list_scroll",
                flat_nodes.len(),
                cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                    let workspace = this.workspace.read(cx);
                    let file_tree = &workspace.file_tree;
                    let selected_path = &workspace.selected_path;

                    let flat = flatten_file_tree(file_tree);

                    let theme = gpui_component::Theme::global(cx);
                    let theme_primary = theme.primary;
                    let theme_foreground = theme.foreground;
                    let theme_muted_foreground = theme.muted_foreground;
                    let theme_accent = theme.accent;

                    range
                        .map(|idx| {
                            let node = &flat[idx];
                            let padding_left = gpui::px((node.depth * 12 + 12) as f32);
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

                            let icon = if node.is_dir {
                                Some(IconName::Folder)
                            } else {
                                Some(IconName::File)
                            };
                            let path = node.path.clone();

                            let mut item = gpui::div()
                                .id(("file", idx))
                                .flex()
                                .items_center()
                                .h(gpui::px(26.))
                                .pl(padding_left)
                                .text_size(gpui::px(12.))
                                .text_color(color)
                                .hover(|s| s.bg(theme_accent));

                            if !node.is_dir {
                                item = item.on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |_, _, _, cx| {
                                        cx.emit(SidebarEvent::FileSelected(path.clone()));
                                    }),
                                );
                            }

                            item.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(icon.unwrap()).size(gpui::px(14.)))
                                    .child(node.name.clone()),
                            )
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .size_full()
            .into_any_element()
        };

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
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(file_list_content),
            )
    }
}


