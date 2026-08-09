//! Rendering logic for graph panes — GraphPaneView implementation using graphene_gpui::GraphCanvas.

use gpui::{div, prelude::*, SharedString, Window};
use graphene_core::{DataExpansionMode, NodeId, PropValue};
use graphene_gpui::{CanvasConfig, GraphCanvas};
use graphene_style::Theme;
use std::collections::HashMap;

use super::data::{GraphEvent, LayoutMode};
use super::state::GraphPaneView;

impl Render for GraphPaneView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if matches!(self.layout_mode, LayoutMode::RunLayout(_)) {
            self.layout_mode = LayoutMode::None;
            self.run_layout(cx);
        }

        let selected_hub_id = self.sync_active_tab(cx);
        let selected_node_id = self.resolve_selected_node(selected_hub_id.as_ref());
        let active_tab_idx = self.active_camera_idx;

        let tab_label = match active_tab_idx {
            0 => "Document Outline",
            1 => "Definitions Schema",
            _ => "Instances Relation",
        };

        let cam = self.camera_states[active_tab_idx];
        let viewport = self.active_viewport();
        let (active_state, active_view, _) = self.active_tab_context();

        let theme_ui = gpui_component::Theme::global(cx);
        let border_color = theme_ui.border;
        let sidebar_bg = theme_ui.sidebar;
        let fg_color = theme_ui.foreground;

        let canvas_config = CanvasConfig {
            edge_stroke_width: 2.0,
            arrow_length: 10.0,
            arrow_width: 8.0,
            node_border_width: 2.0,
            node_font_size: 11.0,
            color_config: graphene_style::ColorConfig {
                label_contrast_mode: if self.wcag_contrast_auto {
                    graphene_style::LabelContrastMode::WcagAuto
                } else {
                    graphene_style::LabelContrastMode::Fixed(graphene_style::Rgb::new(
                        200, 200, 200,
                    ))
                },
                auto_node_colors: self.auto_node_colors,
                auto_edge_colors: self.auto_edge_colors,
                canvas_background: graphene_style::Rgb::new(30, 30, 46),
            },
            ..CanvasConfig::default()
        };

        let canvas_element = GraphCanvas::new(
            active_view,
            &viewport,
            &self.interaction_state,
            &Theme::catppuccin_mocha(),
            selected_node_id,
            &HashMap::new(), // node_labels
            &HashMap::new(), // edge_labels
            30,
            &self.expansion_state.collapsed_parents,
        )
        .with_config(canvas_config)
        .into_element();

        let view_entity = cx.entity().clone();
        let bounds_reporter = gpui::canvas(
            move |_, _, _| {},
            move |bounds, _, _window, cx| {
                view_entity.update(cx, |this, cx| {
                    this.handle_bounds_changed(bounds, cx);
                });
            },
        )
        .absolute()
        .size_full();

        div()
            .flex_1()
            .h_full()
            .relative()
            .border_r(gpui::px(1.))
            .border_color(border_color)
            .flex()
            .flex_col()
            // GPUI Idiomatic listeners cleanly bind to our helper methods:
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_down),
            )
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel))
            .child(bounds_reporter)
            .child(canvas_element)
            // --- Tab Label Overlay ---
            .child(
                div()
                    .absolute()
                    .top(gpui::px(8.))
                    .left(gpui::px(8.))
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(4.))
                    .py_2()
                    .px_3()
                    .text_size(gpui::px(9.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(fg_color.opacity(0.8))
                    .child(tab_label),
            )
            // --- Zoom Controls ---
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .absolute()
                    .bottom(gpui::px(8.))
                    .right(gpui::px(8.))
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(6.))
                    .overflow_hidden()
                    .child(
                        gpui_component::button::Button::new("zoom_in")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, _, cx| this.apply_zoom(20.0, cx)),
                            )
                            .label("+")
                            .text_size(gpui::px(12.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .min_w(gpui::px(28.))
                            .h(gpui::px(28.))
                            .border_r(gpui::px(1.))
                            .border_color(border_color),
                    )
                    .child(
                        gpui_component::button::Button::new("zoom_out")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, _, cx| this.apply_zoom(-20.0, cx)),
                            )
                            .label("-")
                            .text_size(gpui::px(12.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .min_w(gpui::px(28.))
                            .h(gpui::px(28.))
                            .border_color(border_color),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .bottom(gpui::px(8.))
                    .left(gpui::px(8.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(6.))
                    .px_3()
                    .py_2()
                    .text_size(gpui::px(9.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(fg_color.opacity(0.8))
                    .child(format!("Zoom: {:.0}%", (cam.zoom * 100.0).round())),
            )
            // --- Node Properties Panel ---
            .children(selected_node_id.and_then(|sel_nid| {
                let &idx = active_state.node_keys.get(sel_nid)?;
                let node_data = active_state.nodes.get(idx);

                let display_name = active_state
                    .display_label(sel_nid)
                    .unwrap_or("Selected Node");
                let primary_label = node_data.primary_label().unwrap_or("Node");
                let cur_exp = node_data.expansion_mode;

                let prop_rows: Vec<_> = node_data
                    .props
                    .iter()
                    .filter(|(k, _)| !k.starts_with('@'))
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();

                Some(
                    div()
                        .absolute()
                        .bottom(gpui::px(42.))
                        .left(gpui::px(8.))
                        .w(gpui::px(230.))
                        .bg(sidebar_bg)
                        .border(gpui::px(1.))
                        .border_color(border_color)
                        .rounded(gpui::px(6.))
                        .shadow_md()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_size(gpui::px(12.))
                                .text_color(fg_color)
                                .child(display_name.to_string()),
                        )
                        .child(
                            div()
                                .text_size(gpui::px(10.))
                                .text_color(fg_color.opacity(0.7))
                                .child(format!("«{}»", primary_label)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .py_1()
                                .child(
                                    gpui_component::button::Button::new("exp_compact")
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.set_expansion_mode(
                                                    sel_nid,
                                                    DataExpansionMode::Compact,
                                                    cx,
                                                );
                                            }),
                                        )
                                        .label(if cur_exp == DataExpansionMode::Compact {
                                            "[Compact]"
                                        } else {
                                            "Compact"
                                        })
                                        .text_size(gpui::px(9.))
                                        .px_2()
                                        .py_1(),
                                )
                                .child(
                                    gpui_component::button::Button::new("exp_preview")
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.set_expansion_mode(
                                                    sel_nid,
                                                    DataExpansionMode::Preview,
                                                    cx,
                                                );
                                            }),
                                        )
                                        .label(if cur_exp == DataExpansionMode::Preview {
                                            "[Preview]"
                                        } else {
                                            "Preview"
                                        })
                                        .text_size(gpui::px(9.))
                                        .px_2()
                                        .py_1(),
                                )
                                .child(
                                    gpui_component::button::Button::new("exp_full")
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.set_expansion_mode(
                                                    sel_nid,
                                                    DataExpansionMode::Full,
                                                    cx,
                                                );
                                            }),
                                        )
                                        .label(if cur_exp == DataExpansionMode::Full {
                                            "[Full]"
                                        } else {
                                            "Full"
                                        })
                                        .text_size(gpui::px(9.))
                                        .px_2()
                                        .py_1(),
                                ),
                        )
                        .children(prop_rows.into_iter().map(|row| {
                            div()
                                .text_size(gpui::px(10.))
                                .font_family("monospace")
                                .text_color(fg_color.opacity(0.9))
                                .child(row)
                        })),
                )
            }))
    }
}
