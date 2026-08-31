use gpui::prelude::*;
use gpui::{Entity, IntoElement};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::status_bar::StatusBar;
use gpui_component::{h_flex, Icon, IconName, Sizable};

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
    _theme_muted_foreground: gpui::Hsla,
    _success_color: gpui::Hsla,
    view: Entity<super::MainView>,
) -> impl IntoElement {
    let (lsp_icon, lsp_label, dot_color) = if lsp_connected {
        (
            IconName::CircleCheck,
            "LSP Connected",
            gpui::hsla(142.0 / 360.0, 0.71, 0.45, 1.0),
        )
    } else {
        (
            IconName::CircleX,
            "LSP Offline",
            gpui::hsla(0.0, 0.75, 0.55, 1.0),
        )
    };

    StatusBar::new()
        .bg(sidebar_bg)
        .border_color(border_color)
        .left(
            h_flex()
                .items_center()
                .gap_2()
                .child(Icon::new(IconName::Folder).size(gpui::px(14.)))
                .child(active_file_str),
        )
        .left(
            h_flex()
                .items_center()
                .gap_1p5()
                .child(
                    gpui_component::badge::Badge::new()
                        .dot()
                        .color(dot_color)
                        .child(Icon::new(lsp_icon).size(gpui::px(13.))),
                )
                .child(lsp_label),
        )
        .right(
            Button::new("status_theme_btn")
                .ghost()
                .xsmall()
                .label(format!("Theme: {}", theme_name))
                .on_click(move |_, window, cx| {
                    view.update(cx, |this, cx| {
                        this.toggle_settings(&super::ToggleSettings, window, cx);
                    });
                }),
        )
}
