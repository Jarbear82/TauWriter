//! Window chrome helpers — TitleBar, status bar, and settings window logic.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate window chrome rendering.

use gpui::prelude::*;
use gpui::{div, Entity, SharedString};
use gpui_component::{Icon, IconName};

/// Creates the settings window content for a new window.
#[allow(dead_code)] // provided for future use; not called inline
pub(crate) fn create_settings_window_content(
    _window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> Entity<gpui_component::Root> {
    let view = cx.new(|cx| super::SettingsView::new(cx));
    cx.new(|cx| gpui_component::Root::new(view, _window, cx))
}

/// Renders the bottom status bar with file path, LSP indicator, and theme picker.
pub(crate) fn render_bottom_status_bar(
    active_file_str: String,
    lsp_connected: bool,
    theme_name: String,
    sidebar_bg: gpui::Hsla,
    border_color: gpui::Hsla,
    theme_muted_foreground: gpui::Hsla,
    success_color: gpui::Hsla,
    view: Entity<super::MainView>,
) -> impl IntoElement {
    let lsp_label: SharedString = if lsp_connected {
        "LSP Connected".into()
    } else {
        "LSP Offline".into()
    };

    div()
        .flex()
        .items_center()
        .justify_between()
        .h(gpui::px(26.))
        .bg(sidebar_bg)
        .border_t(gpui::px(1.))
        .border_color(border_color)
        .px_4()
        .text_xs()
        .text_color(theme_muted_foreground)
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::Folder).size(gpui::px(14.)))
                        .child(active_file_str),
                )
                .child(div().child(lsp_label)),
        )
        .child(
            div()
                .cursor_pointer()
                .hover(|s| s.underline())
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                    let was_open = view.read(cx).settings_window.is_some();
                    if was_open {
                        if let Some(handle) = view.update(cx, |this, _| this.settings_window.take())
                        {
                            let _ = handle.update(cx, |_, w, _| w.remove_window());
                        }
                        // Re-render MainView by updating it with a no-op
                        view.update(
                            cx,
                            |_: &mut super::MainView, cx: &mut Context<super::MainView>| {
                                cx.notify();
                            },
                        );
                    } else {
                        let bounds = gpui::Bounds::centered(
                            None,
                            gpui::size(gpui::px(350.), gpui::px(500.)),
                            cx,
                        );
                        if let Ok(handle) = cx.open_window(
                            gpui::WindowOptions {
                                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                                window_decorations: Some(gpui::WindowDecorations::Client),
                                ..Default::default()
                            },
                            move |window, cx| {
                                let view = cx.new(|cx| super::SettingsView::new(cx));
                                cx.new(|cx| gpui_component::Root::new(view, window, cx))
                            },
                        ) {
                            view.update(cx, |this, _| {
                                this.settings_window = Some(handle);
                            });
                            view.update(
                                cx,
                                |_: &mut super::MainView, cx: &mut Context<super::MainView>| {
                                    cx.notify();
                                },
                            );
                        }
                    }
                })
                .child(format!("Theme: {}", theme_name)),
        )
}
