//! Sidebar rendering — file explorer with Tree and TreeState.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate navigation UI logic.

use gpui::prelude::*;
use gpui::{div, Context, Entity, EventEmitter};
use gpui_component::{
    list::ListItem,
    tree::{tree, TreeState},
    Icon, IconName,
};
use std::path::PathBuf;

use super::tree_view::file_nodes_to_tree_items;
use super::Workspace;

pub(crate) enum SidebarEvent {
    FileSelected(PathBuf),
}

pub(crate) struct SidebarView {
    workspace: Entity<Workspace>,
    tree_state: Entity<TreeState>,
}

impl SidebarView {
    pub(crate) fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let items = {
            let ws = workspace.read(cx);
            file_nodes_to_tree_items(&ws.file_tree)
        };
        let tree_state = cx.new(|cx| TreeState::new(cx).items(items));

        cx.observe(&workspace, |this, ws, cx| {
            let items = file_nodes_to_tree_items(&ws.read(cx).file_tree);
            this.tree_state.update(cx, |state, cx| {
                state.set_items(items, cx);
            });
            cx.notify();
        })
        .detach();

        Self {
            workspace,
            tree_state,
        }
    }
}

impl EventEmitter<SidebarEvent> for SidebarView {}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let theme_muted_foreground = theme.muted_foreground;
        let sidebar_bg = theme.sidebar;
        let border_color = theme.border;

        let view = cx.entity();
        let tree_content = tree(&self.tree_state, {
            let workspace = self.workspace.clone();
            move |ix, entry, selected, _window, cx| {
                view.update(cx, |_, cx| {
                    let item = entry.item();
                    let path = PathBuf::from(item.id.as_str());
                    let is_dir = entry.is_folder();

                    let (chevron, icon) = if is_dir {
                        let chev = if entry.is_expanded() {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        };
                        (Some(chev), IconName::Folder)
                    } else {
                        (None, IconName::File)
                    };

                    let padding_left = gpui::px((entry.depth() * 12 + 8) as f32);

                    let is_active_selected = selected
                        || workspace.read(cx).selected_path.as_ref().map_or(false, |p| {
                            p.canonicalize().ok() == path.canonicalize().ok()
                        });

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
                        .child(item.label.clone());

                    let mut list_item = ListItem::new(ix)
                        .selected(is_active_selected)
                        .child(row_content);

                    if !is_dir {
                        list_item = list_item.on_click(cx.listener({
                            let path = path.clone();
                            move |_, _, _, cx| {
                                cx.emit(SidebarEvent::FileSelected(path.clone()));
                            }
                        }));
                    }

                    list_item
                })
            }
        });

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
                    .child(tree_content),
            )
    }
}




