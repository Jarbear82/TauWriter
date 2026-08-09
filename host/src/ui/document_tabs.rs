//! Document tab bar rendering helpers.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate document tab UI logic.

use gpui::prelude::*;
use gpui::{div, px, Entity};

use super::OpenDocument;

/// Renders a single document tab item with filename, click handler (select), and close button.
fn render_doc_tab_item(
    theme_bg: gpui::Hsla,
    theme_sidebar: gpui::Hsla,
    theme_border: gpui::Hsla,
    theme_fg: gpui::Hsla,
    theme_muted: gpui::Hsla,
    index: usize,
    is_active: bool,
    filename: String,
    view: Entity<super::MainView>,
) -> gpui::Div {
    let bg = if is_active { theme_bg } else { theme_sidebar };
    let _border = if is_active {
        theme_border
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };
    let text_color = if is_active { theme_fg } else { theme_muted };

    let view_clone_select = view.clone();
    let view_clone_close = view;

    div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_2()
        .border_r(px(1.))
        .border_color(theme_border)
        .bg(bg)
        .child(
            div()
                .cursor_pointer()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(text_color)
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                    view_clone_select.update(cx, |this, cx| {
                        this.workspace.update(cx, |w, cx| {
                            w.active_doc_idx = Some(index);
                            if let Some(doc) = w.open_docs.get(index) {
                                w.selected_path = Some(doc.path.clone());
                            }
                            cx.notify();
                        });
                        cx.notify();
                    });
                })
                .child(filename),
        )
        .child(
            div()
                .cursor_pointer()
                .text_xs()
                .text_color(theme_muted)
                .hover(|s| s.text_color(theme_bg))
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                    view_clone_close.update(cx, |this, cx| {
                        this.close_document_tab(index, cx);
                    });
                })
                .child("✕"),
        )
}

/// Renders the document tab bar container with all tabs.
pub(crate) fn render_doc_tab_bar(
    theme_bg: gpui::Hsla,
    theme_sidebar: gpui::Hsla,
    theme_border: gpui::Hsla,
    theme_fg: gpui::Hsla,
    theme_muted: gpui::Hsla,
    open_docs: &[OpenDocument],
    active_doc_idx: Option<usize>,
    view: Entity<super::MainView>,
) -> gpui::Div {
    let mut doc_tabs = Vec::new();
    for (i, doc) in open_docs.iter().enumerate() {
        let filename = doc
            .path
            .file_name()
            .map_or("No Name".to_string(), |n| n.to_string_lossy().to_string());
        let is_active = Some(i) == active_doc_idx;

        doc_tabs.push(render_doc_tab_item(
            theme_bg,
            theme_sidebar,
            theme_border,
            theme_fg,
            theme_muted,
            i,
            is_active,
            filename,
            view.clone(),
        ));
    }

    div()
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .bg(theme_sidebar)
        .border_b(px(1.))
        .border_color(theme_border)
        .child(div().flex().items_center().children(doc_tabs))
}
