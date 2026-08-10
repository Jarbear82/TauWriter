//! Sidebar rendering — file explorer with virtualized list and collapse state.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate navigation UI logic.

use gpui::prelude::*;
use gpui::{div, uniform_list, Context, Entity, EventEmitter};
use gpui_component::{
    list::ListItem,
    Icon, IconName,
};
use std::collections::HashSet;
use std::path::PathBuf;

use super::tree_view::flatten_file_tree_with_collapse;
use super::Workspace;

pub(crate) enum SidebarEvent {
    FileSelected(PathBuf),
}

pub(crate) struct SidebarView {
    workspace: Entity<Workspace>,
    collapsed_paths: HashSet<PathBuf>,
}

impl SidebarView {
    pub(crate) fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        Self {
            workspace,
            collapsed_paths: HashSet::new(),
        }
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

        let flat_nodes = flatten_file_tree_with_collapse(&file_tree, &self.collapsed_paths);

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
                    let flat = flatten_file_tree_with_collapse(file_tree, &this.collapsed_paths);

                    range
                        .map(|idx| {
                            let node = &flat[idx];
                            let padding_left = gpui::px((node.depth * 12 + 8) as f32);
                            let is_selected = selected_path.as_ref().map_or(false, |p| {
                                p.canonicalize().ok() == node.path.canonicalize().ok()
                            });
                            let is_collapsed = this.collapsed_paths.contains(&node.path);

                            let (chevron, icon) = if node.is_dir {
                                let chev = if is_collapsed {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronDown
                                };
                                (Some(chev), IconName::Folder)
                            } else {
                                (None, IconName::File)
                            };

                            let path = node.path.clone();
                            let is_dir = node.is_dir;

                            let mut row_content = div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .pl(padding_left)
                                .text_size(gpui::px(12.));

                            if let Some(chev) = chevron {
                                row_content = row_content.child(Icon::new(chev).size(gpui::px(12.)));
                            } else {
                                row_content = row_content.child(div().w(gpui::px(12.)));
                            }

                            row_content = row_content
                                .child(Icon::new(icon).size(gpui::px(14.)))
                                .child(node.name.clone());

                            let mut list_item = ListItem::new(("file", idx))
                                .selected(is_selected)
                                .child(row_content);

                            if is_dir {
                                list_item = list_item.on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        if !this.collapsed_paths.insert(path.clone()) {
                                            this.collapsed_paths.remove(&path);
                                        }
                                        cx.notify();
                                    }),
                                );
                            } else {
                                list_item = list_item.on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |_, _, _, cx| {
                                        cx.emit(SidebarEvent::FileSelected(path.clone()));
                                    }),
                                );
                            }

                            list_item
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



