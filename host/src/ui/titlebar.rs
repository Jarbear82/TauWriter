//! Title bar and settings panel rendering.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate window chrome logic.
//! [user-review: split required] See task ticket for splitting rationale.

use gpui::prelude::*;
use gpui::{Hsla, IntoElement};

/// Render the settings (theme picker) panel shown in the right sidebar.
pub(crate) fn render_settings(
    sidebar_bg: &Hsla,
    border_color: &Hsla,
    theme_muted_foreground: &Hsla,
    theme_primary: &Hsla,
    theme_accent: &Hsla,
    theme_foreground: &Hsla,
    cx: &mut Context<super::DemoView>,
) -> impl IntoElement {
    let border_color = *border_color;
    let sidebar_bg = *sidebar_bg;

    let theme_val = gpui_component::Theme::global(cx);
    let theme_name = theme_val.theme_name();
    let themes_list = gpui_component::ThemeRegistry::global(cx).sorted_themes();
    let mut theme_items = Vec::new();

    for (idx, theme_config) in themes_list.into_iter().enumerate() {
        let name = theme_config.name.clone();
        let is_current = theme_name == &name;
        let item_color = if is_current {
            *theme_primary
        } else {
            *theme_foreground
        };

        let mode_icon = if theme_config.mode.is_dark() {
            "🌙"
        } else {
            "☀️"
        };

        let item = gpui::div()
            .id(("theme", idx))
            .flex()
            .items_center()
            .justify_between()
            .h(gpui::px(32.))
            .px_3()
            .text_size(gpui::px(12.))
            .text_color(item_color)
            .hover(|s| s.bg(*theme_accent))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |_this, _, _, cx| {
                    let theme_registry = gpui_component::ThemeRegistry::global(cx);
                    if let Some(config) = theme_registry.themes().get(&name).cloned() {
                        let mode = config.mode;
                        let theme = gpui_component::Theme::global_mut(cx);
                        if mode.is_dark() {
                            theme.dark_theme = config.clone();
                        } else {
                            theme.light_theme = config.clone();
                        }
                        gpui_component::Theme::change(mode, None, cx);
                        cx.refresh_windows();
                    }
                }),
            )
            .child(format!("{} {}", mode_icon, theme_config.name));

        theme_items.push(item);
    }

    gpui::div()
        .id("theme_list")
        .w(gpui::px(250.))
        .h_full()
        .bg(sidebar_bg)
        .border_l(gpui::px(1.))
        .border_color(border_color)
        .flex()
        .flex_col()
        .child(
            gpui::div()
                .p_3()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(*theme_muted_foreground)
                .child("THEME SETTINGS"),
        )
        .child(
            gpui::div()
                .id("theme_list")
                .flex_1()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(theme_items),
        )
}

/// Render the custom title bar (CSD) with window controls and settings toggle.
pub(crate) fn render_titlebar(
    _bg_color: &Hsla,
    sidebar_bg: &Hsla,
    border_color: &Hsla,
    theme_muted_foreground: &Hsla,
    active_accent: &Hsla,
    theme_button: &Hsla,
    theme_button_foreground: &Hsla,
    theme_primary_foreground: &Hsla,
    settings_open: bool,
    title: &str,
    cx: &mut Context<super::DemoView>,
) -> gpui::Div {
    gpui::div()
        .flex()
        .items_center()
        .justify_between()
        .h(gpui::px(40.))
        .bg(*sidebar_bg)
        .border_b(gpui::px(1.))
        .border_color(*border_color)
        // Drag and Move Area (spans left and middle)
        .child(
            gpui::div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .gap_3()
                .pl_4()
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                    window.start_window_move();
                })
                .child(
                    gpui::div()
                        .w(gpui::px(10.))
                        .h(gpui::px(10.))
                        .rounded_full()
                        .bg(gpui::rgb(0x00cc66)),
                )
                .child(
                    gpui::div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_size(gpui::px(13.))
                        .child("TauWriter Editor"),
                )
                .child(
                    gpui::div()
                        .text_xs()
                        .text_color(*theme_muted_foreground)
                        .child(format!("— {title}")),
                ),
        )
        // Window Controls & Settings Button
        .child(
            gpui::div()
                .flex()
                .items_center()
                .gap_2()
                .pr_4()
                .child(
                    // Settings Button
                    gpui::div()
                        .id("settings_btn")
                        .px_2()
                        .py_1()
                        .rounded(gpui::px(4.))
                        .bg(if settings_open {
                            *active_accent
                        } else {
                            *theme_button
                        })
                        .text_color(if settings_open {
                            *theme_primary_foreground
                        } else {
                            *theme_button_foreground
                        })
                        .text_size(gpui::px(11.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .hover(|s| s.bg(gpui_component::Theme::global(cx).accent))
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.settings_open = !this.settings_open;
                                cx.notify();
                            }),
                        )
                        .child("Settings"),
                )
                .child(
                    // Minimize Button
                    gpui::div()
                        .w(gpui::px(12.))
                        .h(gpui::px(12.))
                        .rounded_full()
                        .bg(gpui::rgb(0xffbd2e))
                        .hover(|s| s.bg(gpui::rgb(0xe0a92a)))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                            window.minimize_window();
                        }),
                )
                .child(
                    // Zoom/Maximize Button
                    gpui::div()
                        .w(gpui::px(12.))
                        .h(gpui::px(12.))
                        .rounded_full()
                        .bg(gpui::rgb(0x27c93f))
                        .hover(|s| s.bg(gpui::rgb(0x20a834)))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                            window.zoom_window();
                        }),
                )
                .child(
                    // Close Button
                    gpui::div()
                        .w(gpui::px(12.))
                        .h(gpui::px(12.))
                        .rounded_full()
                        .bg(gpui::rgb(0xff5f56))
                        .hover(|s| s.bg(gpui::rgb(0xe04f46)))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                            window.remove_window();
                        }),
                ),
        )
}
