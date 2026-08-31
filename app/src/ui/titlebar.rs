//! Title bar and settings panel rendering.
//!
//! Extracted from `ui/mod.rs` to reduce file length and isolate window chrome logic.

use gpui::prelude::*;
use gpui::{div, Context, Entity, IntoElement, Render};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::list::ListItem;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{Icon, IconName, WindowExt};

// ─── Settings Dialog ────────────────────────────────────────────────────────

pub(crate) fn open_settings_dialog(window: &mut gpui::Window, cx: &mut gpui::App) {
    window.open_dialog(cx, |dialog, _window, _cx| {
        dialog
            .w(gpui::px(480.))
            .title("Settings")
            .child(SettingsPanel)
    });
}

// ─── SettingsPanel (theme picker) ───────────────────────────────────────────

#[derive(IntoElement)]
pub(crate) struct SettingsPanel;

impl RenderOnce for SettingsPanel {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme_val = gpui_component::Theme::global(cx);
        let theme_muted_foreground = theme_val.muted_foreground;

        let theme_name = theme_val.theme_name();
        let themes_list = gpui_component::ThemeRegistry::global(cx).sorted_themes();
        let mut theme_items = Vec::new();

        for (idx, theme_config) in themes_list.into_iter().enumerate() {
            let name = theme_config.name.clone();
            let is_current = theme_name == &name;

            let mode_icon = gpui_component::Icon::new(if theme_config.mode.is_dark() {
                IconName::Moon
            } else {
                IconName::Sun
            });

            let item = ListItem::new(("theme", idx))
                .selected(is_current)
                .on_click(move |_, _, cx| {
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
                        .gap_2()
                        .child(mode_icon)
                        .child(theme_config.name.clone()),
                );

            theme_items.push(item);
        }

        gpui::div()
            .id("theme_settings_panel")
            .w_full()
            .flex()
            .flex_col()
            .child(
                gpui::div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme_muted_foreground)
                    .child("THEMES"),
            )
            .child(
                gpui::div()
                    .id("theme_list")
                    .max_h(gpui::px(350.))
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
                gpui_component::TitleBar::new()
                    .bg(sidebar_bg)
                    .border_color(border_color)
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
    pub(crate) title: gpui::SharedString,
    pub(crate) view: Entity<crate::ui::MainView>,
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let sidebar_bg = theme.sidebar;
        let border_color = theme.border;
        let theme_muted_foreground = theme.muted_foreground;

        let title = self.title.clone();
        let view_settings = self.view.clone();

        gpui_component::TitleBar::new()
            .bg(sidebar_bg)
            .border_color(border_color)
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        gpui::div()
                            .w(gpui::px(10.))
                            .h(gpui::px(10.))
                            .rounded_full()
                            .bg(theme.primary),
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
            .child(
                gpui::div().flex().items_center().gap_2().child(
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
