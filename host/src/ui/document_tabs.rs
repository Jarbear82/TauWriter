//! Document tab bar rendering helpers.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate document tab UI logic.

use gpui::prelude::*;
use gpui::{div, px, Entity};
use gpui_component::{
    tab::{Tab, TabBar},
    IconName,
};

use super::OpenDocument;

/// Renders the document tab bar container driven by `gpui_component::tab::TabBar` and `Tab`.
pub(crate) fn render_doc_tab_bar(
    _theme_bg: gpui::Hsla,
    theme_sidebar: gpui::Hsla,
    theme_border: gpui::Hsla,
    _theme_fg: gpui::Hsla,
    theme_muted: gpui::Hsla,
    open_docs: &[OpenDocument],
    active_doc_idx: Option<usize>,
    view: Entity<super::MainView>,
) -> gpui::Div {
    let view_select = view.clone();

    let tab_items: Vec<Tab> = open_docs
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            let filename = doc
                .path
                .file_name()
                .map_or("No Name".to_string(), |n| n.to_string_lossy().to_string());
            let view_close = view.clone();

            let close_btn = div()
                .cursor_pointer()
                .px_1()
                .text_xs()
                .text_color(theme_muted)
                .hover(|s| s.text_color(gpui::rgb(0xff5f56)))
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                    view_close.update(cx, |this, cx| {
                        this.close_document_tab(i, cx);
                    });
                })
                .child("✕");

            Tab::new()
                .icon(IconName::File)
                .label(filename)
                .suffix(close_btn)
        })
        .collect();

    let mut tab_bar = TabBar::new("document-tab-bar")
        .children(tab_items)
        .on_click(move |index, _, cx| {
            let idx = *index;
            view_select.update(cx, |this, cx| {
                this.workspace.update(cx, |w, cx| {
                    w.active_doc_idx = Some(idx);
                    if let Some(doc) = w.open_docs.get(idx) {
                        w.selected_path = Some(doc.path.clone());
                    }
                    cx.notify();
                });
                cx.notify();
            });
        });

    if let Some(idx) = active_doc_idx {
        tab_bar = tab_bar.selected_index(idx);
    }

    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .bg(theme_sidebar)
        .border_b(px(1.))
        .border_color(theme_border)
        .child(div().flex().items_center().child(tab_bar))
}


