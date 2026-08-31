//! Mode selector button for document mode switching.
//!
//! Uses gpui_component's DropdownButton — a button with an attached dropdown menu.
//! View is cloned inside the Fn callback body so each click handler owns its own
//! reference-counted handle, satisfying 'static bounds.

use gpui::Entity;
use gpui_component::button::{Button, ButtonVariants, DropdownButton};
use gpui_component::menu::PopupMenuItem;
use gpui_component::{IconName, Sizable};

use super::DocumentMode;

/// Renders a mode selector button with an attached dropdown menu.
pub(crate) fn render_mode_selector(
    current_mode: DocumentMode,
    doc_idx: usize,
    view: Entity<super::MainView>,
) -> impl gpui::IntoElement {
    let (mode_label, mode_icon) = match current_mode {
        DocumentMode::RawEditor => ("Raw Editor", IconName::File),
        DocumentMode::BlockEditor => ("Block Editor", IconName::Menu),
        DocumentMode::MarkdownView => ("Markdown View", IconName::Eye),
        DocumentMode::FlowTextEditor => ("FlowText Editor (Stub)", IconName::LayoutDashboard),
    };

    DropdownButton::new(format!("mode-selector-{doc_idx}"))
        .button(
            Button::new("mode-btn")
                .ghost()
                .small()
                .icon(mode_icon)
                .label(mode_label),
        )
        .dropdown_menu(move |menu, _, _| {
            // Clone the view handle here — cheap Arc increment.
            let view_for_raw = view.clone();
            let view_for_block = view.clone();
            let view_for_md = view.clone();
            let view_for_flow = view.clone();
            menu.item(
                PopupMenuItem::new("Raw Editor")
                    .icon(IconName::File)
                    .on_click({
                        move |_, _, cx| {
                            switch_view_mode(
                                doc_idx,
                                DocumentMode::RawEditor,
                                view_for_raw.clone(),
                                cx,
                            );
                        }
                    }),
            )
            .item(
                PopupMenuItem::new("Block Editor")
                    .icon(IconName::Menu)
                    .on_click({
                        move |_, _, cx| {
                            switch_view_mode(
                                doc_idx,
                                DocumentMode::BlockEditor,
                                view_for_block.clone(),
                                cx,
                            );
                        }
                    }),
            )
            .item(
                PopupMenuItem::new("Markdown View")
                    .icon(IconName::Eye)
                    .on_click({
                        move |_, _, cx| {
                            switch_view_mode(
                                doc_idx,
                                DocumentMode::MarkdownView,
                                view_for_md.clone(),
                                cx,
                            );
                        }
                    }),
            )
            .item(
                PopupMenuItem::new("FlowText Editor (Stub)")
                    .icon(IconName::LayoutDashboard)
                    .on_click({
                        move |_, _, cx| {
                            switch_view_mode(
                                doc_idx,
                                DocumentMode::FlowTextEditor,
                                view_for_flow.clone(),
                                cx,
                            );
                        }
                    }),
            )
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
