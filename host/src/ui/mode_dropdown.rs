//! Mode selector button for document mode switching.
//!
//! Uses gpui_component's DropdownButton — a button with an attached dropdown menu.
//! View is cloned inside the Fn callback body so each click handler owns its own
//! reference-counted handle, satisfying 'static bounds.

use gpui::Entity;
use gpui_component::button::{Button, DropdownButton};
use gpui_component::menu::PopupMenuItem;

use super::DocumentMode;

/// Renders a mode selector button with an attached dropdown menu.
pub(crate) fn render_mode_selector(
    current_mode: DocumentMode,
    doc_idx: usize,
    view: Entity<super::MainView>,
) -> impl gpui::IntoElement {
    let mode_label = match current_mode {
        DocumentMode::RawEditor => "Raw Editor",
        DocumentMode::WysiwygPreview => "WYSIWYG Preview",
        DocumentMode::MarkdownView => "Markdown View",
    };

    DropdownButton::new(format!("mode-selector-{doc_idx}"))
        .button(Button::new("mode-btn").label(mode_label))
        .dropdown_menu(move |menu, _, _| {
            // Clone the view handle here — cheap Arc increment.
            let view_for_raw = view.clone();
            let view_for_wysiwyg = view.clone();
            let view_for_md = view.clone();
            menu.item(PopupMenuItem::new("Raw Editor").on_click({
                move |_, _, cx| {
                    switch_view_mode(doc_idx, DocumentMode::RawEditor, view_for_raw.clone(), cx);
                }
            }))
            .item(PopupMenuItem::new("WYSIWYG Preview").on_click({
                move |_, _, cx| {
                    switch_view_mode(
                        doc_idx,
                        DocumentMode::WysiwygPreview,
                        view_for_wysiwyg.clone(),
                        cx,
                    );
                }
            }))
            .item(PopupMenuItem::new("Markdown View").on_click({
                move |_, _, cx| {
                    switch_view_mode(doc_idx, DocumentMode::MarkdownView, view_for_md.clone(), cx);
                }
            }))
        })
}

/// Updates the document mode on MainView's workspace.
fn switch_view_mode(
    doc_idx: usize,
    mode: DocumentMode,
    view: Entity<super::MainView>,
    cx: &mut gpui::App,
) {
    let _ = view.update(cx, |this, cx| {
        this.workspace.update(cx, |w, _| {
            if let Some(doc) = w.open_docs.get_mut(doc_idx) {
                doc.mode = mode;
            }
        });
    });
}
