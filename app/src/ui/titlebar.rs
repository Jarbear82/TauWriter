//! Title bar and settings panel rendering.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate window chrome logic.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::list::ListItem;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::{Icon, IconName, WindowExt};
use gpui_kit::prelude::*;
use gpui_kit::{div, Context, Entity, IntoElement, Render};

// ─── Settings Dialog ────────────────────────────────────────────────────────

pub(crate) fn open_settings_dialog(window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) {
    window.open_dialog(cx, |dialog, _window, _cx| {
        dialog
            .w(gpui_kit::px(480.))
            .title("Settings")
            .child(SettingsPanel)
    });
}

// ─── SettingsPanel (theme picker) ───────────────────────────────────────────

#[derive(IntoElement)]
pub(crate) struct SettingsPanel;

impl RenderOnce for SettingsPanel {
    fn render(self, _window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) -> impl IntoElement {
        let theme_val = gpui_kit::component::Theme::global(cx);
        let theme_muted_foreground = theme_val.muted_foreground;

        let theme_name = theme_val.theme_name();
        let themes_list = gpui_kit::component::ThemeRegistry::global(cx).sorted_themes();
        let mut theme_items = Vec::new();

        for (idx, theme_config) in themes_list.into_iter().enumerate() {
            let name = theme_config.name.clone();
            let is_current = theme_name == &name;

            let mode_icon = gpui_kit::component::Icon::new(if theme_config.mode.is_dark() {
                IconName::Moon
            } else {
                IconName::Sun
            });

            let item = ListItem::new(("theme", idx))
                .selected(is_current)
                .on_click(move |_, _, cx| {
                    let theme_registry = gpui_kit::component::ThemeRegistry::global(cx);
                    if let Some(config) = theme_registry.themes().get(&name).cloned() {
                        let mode = config.mode;
                        let theme = gpui_kit::component::Theme::global_mut(cx);
                        if mode.is_dark() {
                            theme.dark_theme = config.clone();
                        } else {
                            theme.light_theme = config.clone();
                        }
                        gpui_kit::component::Theme::change(mode, None, cx);
                        cx.refresh_windows();
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(mode_icon)
                        .child(theme_config.name.clone()),
                );

            theme_items.push(item);
        }

        gpui_kit::div()
            .id("theme_settings_panel")
            .w_full()
            .flex()
            .flex_col()
            .child(
                gpui_kit::div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .font_weight(gpui_kit::FontWeight::BOLD)
                    .text_color(theme_muted_foreground)
                    .child("THEMES"),
            )
            .child(
                gpui_kit::div()
                    .id("theme_list")
                    .max_h(gpui_kit::px(350.))
                    .overflow_y_scrollbar()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_1()
                    .children(theme_items),
            )
    }
}

// ─── SettingsView (separate window wrapper) ───────────────────────────────

pub(crate) struct SettingsView {
    focus_handle: gpui_kit::FocusHandle,
}

impl SettingsView {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for SettingsView {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme_val = gpui_kit::component::Theme::global(cx);
        let bg_color = theme_val.background;
        let fg_color = theme_val.foreground;
        let border_color = theme_val.border;
        let sidebar_bg = theme_val.sidebar;

        gpui_kit::div()
            .key_context("SettingsView")
            .track_focus(&self.focus_handle)
            .size_full()
            .flex()
            .flex_col()
            .bg(bg_color)
            .text_color(fg_color)
            .child(
                gpui_kit::component::TitleBar::new()
                    .bg(sidebar_bg)
                    .border_color(border_color)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(IconName::Settings).size(gpui_kit::px(13.)))
                            .child("Settings"),
                    ),
            )
            .child(
                gpui_kit::div()
                    .flex_1()
                    .w_full()
                    .h_full()
                    .child(SettingsPanel),
            )
    }
}

// ─── TitleBar component ─────────────────────────────────────────────────────

#[derive(IntoElement)]
pub(crate) struct TitleBar {
    pub(crate) title: gpui_kit::SharedString,
    pub(crate) view: Entity<crate::ui::MainView>,
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) -> impl IntoElement {
        let theme = gpui_kit::component::Theme::global(cx);
        let sidebar_bg = theme.sidebar;
        let border_color = theme.border;
        let theme_muted_foreground = theme.muted_foreground;

        let title = self.title.clone();
        let view_settings = self.view.clone();

        gpui_kit::component::TitleBar::new()
            .bg(sidebar_bg)
            .border_color(border_color)
            .child(
                gpui_kit::div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        gpui_kit::div()
                            .w(gpui_kit::px(10.))
                            .h(gpui_kit::px(10.))
                            .rounded_full()
                            .bg(theme.primary),
                    )
                    .child(
                        gpui_kit::div()
                            .font_weight(gpui_kit::FontWeight::BOLD)
                            .text_size(gpui_kit::px(13.))
                            .child("TauWriter Editor"),
                    )
                    .child(
                        gpui_kit::div()
                            .text_xs()
                            .text_color(theme_muted_foreground)
                            .child(format!("— {title}")),
                    ),
            )
            .child(
                gpui_kit::div().flex().items_center().gap_2().child(
                    Button::new("settings_btn")
                        .label("Settings")
                        .icon(IconName::Settings)
                        .ghost()
                        .on_click(move |_, window, cx| {
                            view_settings.update(cx, |this, cx| {
                                this.toggle_settings(&crate::ui::ToggleSettings, window, cx);
                            });
                        }),
                ),
            )
    }
}
