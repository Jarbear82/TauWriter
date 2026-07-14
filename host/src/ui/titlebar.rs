//! Title bar and settings panel rendering.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate window chrome logic.
//! [user-review: split required] See task ticket for splitting rationale.

use gpui::prelude::*;
use gpui::{div, Context, Entity, IntoElement, Render};
use gpui_component::{Icon, IconName};

// ─── SettingsPanel (theme picker) ───────────────────────────────────────────

#[derive(IntoElement)]
pub(crate) struct SettingsPanel;

impl RenderOnce for SettingsPanel {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme_val = gpui_component::Theme::global(cx);
        let sidebar_bg = theme_val.sidebar;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_primary = theme_val.primary;
        let theme_accent = theme_val.accent;
        let theme_foreground = theme_val.foreground;

        let theme_name = theme_val.theme_name();
        let themes_list = gpui_component::ThemeRegistry::global(cx).sorted_themes();
        let mut theme_items = Vec::new();

        for (idx, theme_config) in themes_list.into_iter().enumerate() {
            let name = theme_config.name.clone();
            let is_current = theme_name == &name;
            let item_color = if is_current {
                theme_primary
            } else {
                theme_foreground
            };

            let mode_icon = gpui_component::Icon::new(if theme_config.mode.is_dark() {
                IconName::Moon
            } else {
                IconName::Sun
            });

            let item = gpui::div()
                .id(("theme", idx))
                .flex()
                .items_center()
                .justify_between()
                .h(gpui::px(32.))
                .px_3()
                .text_size(gpui::px(12.))
                .text_color(item_color)
                .hover(|s| s.bg(theme_accent))
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
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
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(mode_icon)
                        .child(theme_config.name.clone()),
                );

            theme_items.push(item);
        }

        gpui::div()
            .id("theme_settings_panel")
            .w_full()
            .h_full()
            .bg(sidebar_bg)
            .flex()
            .flex_col()
            .child(
                gpui::div()
                    .p_3()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme_muted_foreground)
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
}

// ─── SettingsView (separate window wrapper) ───────────────────────────────

pub(crate) struct SettingsView {
    focus_handle: gpui::FocusHandle,
}

impl SettingsView {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_val = gpui_component::Theme::global(cx);
        let bg_color = theme_val.background;
        let fg_color = theme_val.foreground;
        let border_color = theme_val.border;
        let sidebar_bg = theme_val.sidebar;

        gpui::div()
            .key_context("SettingsView")
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(bg_color)
            .text_color(fg_color)
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(gpui::px(32.))
                    .bg(sidebar_bg)
                    .border_b(gpui::px(1.))
                    .border_color(border_color)
                    .px_4()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Settings).size(gpui::px(13.)))
                            .child("Settings"),
                    ),
            )
            .child(gpui::div().flex_1().w_full().h_full().child(SettingsPanel))
    }
}

// ─── TitleBar component ─────────────────────────────────────────────────────

#[derive(IntoElement)]
pub(crate) struct TitleBar {
    pub(crate) settings_open: bool,
    pub(crate) title: gpui::SharedString,
    pub(crate) view: Entity<crate::ui::MainView>,
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let sidebar_bg = theme.sidebar;
        let border_color = theme.border;
        let theme_muted_foreground = theme.muted_foreground;
        let active_accent = theme.primary;
        let theme_button = theme.button;
        let theme_button_foreground = theme.button_foreground;
        let theme_primary_foreground = theme.primary_foreground;

        let settings_open = self.settings_open;
        let title = self.title.clone();
        let view = self.view.clone();

        gpui::div()
            .flex()
            .items_center()
            .justify_between()
            .h(gpui::px(40.))
            .bg(sidebar_bg)
            .border_b(gpui::px(1.))
            .border_color(border_color)
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
                            .text_color(theme_muted_foreground)
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
                                active_accent
                            } else {
                                theme_button
                            })
                            .text_color(if settings_open {
                                theme_primary_foreground
                            } else {
                                theme_button_foreground
                            })
                            .text_size(gpui::px(11.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .hover(|s| s.bg(theme.accent))
                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.toggle_settings(&crate::ui::ToggleSettings, window, cx);
                                });
                            })
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
}
