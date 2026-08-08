//! Rendering logic for graph panes — GraphPaneView implementation using graphene_gpui::GraphCanvas.

use std::collections::HashMap;
use gpui::{div, prelude::*, SharedString, Window};
use graphene_core::{DataExpansionMode, NodeId, PropValue};
use graphene_gpui::{CanvasConfig, GraphCanvas};
use graphene_style::Theme;

use super::data::{GraphEvent, LayoutMode};
use super::state::GraphPaneView;

impl Render for GraphPaneView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check if a layout run is pending and execute it
        if matches!(self.layout_mode, LayoutMode::RunLayout(_)) {
            self.layout_mode = LayoutMode::None;
            self.run_layout(cx);
        }

        let (active_tab, selected_hub_id) = {
            let w = self.workspace.read(cx);
            (w.active_graph_tab, w.selected_hub_id.clone())
        };

        // Align active camera index to visible tab
        let old_tab_idx = self.active_camera_idx;
        match active_tab {
            crate::ui::GraphTab::DocumentGraph => self.select_tab_camera(0),
            crate::ui::GraphTab::DefinitionsSchema => self.select_tab_camera(1),
            crate::ui::GraphTab::InstancesRelation => self.select_tab_camera(2),
        }
        if old_tab_idx != self.active_camera_idx {
            self.rebuild_interaction_grid();
        }

        let active_tab_idx = self.active_camera_idx;
        let tab_label = match active_tab_idx {
            0 => "Document Outline",
            1 => "Definitions Schema",
            _ => "Instances Relation",
        };

        let cx_entity = cx.entity().clone();

        let (active_state, active_view, id_map) = match active_tab_idx {
            0 => (&self.outline_state, &self.outline_view, &self.outline_id_map),
            1 => (&self.def_state, &self.def_view, &self.def_id_map),
            _ => (&self.instances_state, &self.instances_view, &self.inst_id_map),
        };

        // Determine selected graphene NodeId from workspace selected_hub_id or self.selected_node
        let selected_node_id: Option<NodeId> = self.selected_node.or_else(|| {
            selected_hub_id
                .as_ref()
                .and_then(|id_str| id_map.get(id_str.as_str()).copied())
                .or_else(|| {
                    selected_hub_id.as_ref().and_then(|id_str| {
                        active_state.node_index_to_id.iter().copied().find(|&nid| {
                            active_state
                                .display_label(nid)
                                .map_or(false, |lbl| lbl == id_str.as_str())
                        })
                    })
                })
        });

        let cam = self.camera_states[active_tab_idx];
        let viewport = self.active_viewport();

        let theme = Theme::catppuccin_mocha();
        let node_labels = HashMap::new();
        let edge_labels = HashMap::new();
        let collapsed_parents = &self.expansion_state.collapsed_parents;

        let canvas_element = GraphCanvas::new(
            active_view,
            &viewport,
            &self.interaction_state,
            &theme,
            selected_node_id,
            &node_labels,
            &edge_labels,
            30,
            collapsed_parents,
        )
        .with_config(CanvasConfig {
            edge_stroke_width: 2.0,
            arrow_length: 10.0,
            arrow_width: 8.0,
            node_border_width: 2.0,
            node_font_size: 11.0,
            color_config: graphene_style::ColorConfig {
                label_contrast_mode: if self.wcag_contrast_auto {
                    graphene_style::LabelContrastMode::WcagAuto
                } else {
                    graphene_style::LabelContrastMode::Fixed(graphene_style::Rgb::new(200, 200, 200))
                },
                auto_node_colors: self.auto_node_colors,
                auto_edge_colors: self.auto_edge_colors,
                canvas_background: graphene_style::Rgb::new(30, 30, 46),
            },
            ..CanvasConfig::default()
        })
        .into_element();

        // Mouse handlers driven by GraphCanvasController
        let on_mouse_down = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();

            move |ev: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut gpui::App| {
                let click_pos = gpui::point(f32::from(ev.position.x), f32::from(ev.position.y));
                let is_shift = ev.modifiers.shift;

                let _ = pane_entity.update(cx, |this, cx| {
                    let viewport = this.active_viewport();
                    let view_ref = match this.active_camera_idx {
                        0 => &this.outline_view,
                        1 => &this.def_view,
                        _ => &this.instances_view,
                    };

                    let mut interaction = this.interaction_state.clone();
                    let mut expansion = this.expansion_state.clone();
                    let mut controller = this.controller.clone();

                    let res = controller.handle_mouse_down(
                        click_pos,
                        is_shift,
                        this.selected_node,
                        &viewport,
                        view_ref,
                        &mut interaction,
                        &mut expansion,
                        this.is_ticking,
                    );

                    this.controller = controller;
                    this.interaction_state = interaction;
                    this.expansion_state = expansion;

                    if let Some(sel) = res.selected_node {
                        this.selected_node = sel;
                        if sel.is_none() {
                            this.workspace.update(cx, |w, cx| {
                                w.selected_hub_id = None;
                                cx.notify();
                            });
                        }
                    }

                    // Drag phase -> update graph state & view positions if needed
                    if let Some((node_id, target_pos, _phase)) = res.drag_update {
                        let (state, view) = this.active_state_and_view();
                        state.set_node_position(node_id, target_pos);
                        if let Some(vn) = view.nodes.get_mut(&node_id) {
                            vn.pos = target_pos;
                        }
                    }

                    // Emit Hub click for TauWriter
                    let clicked_node_id = match res.selected_node {
                        Some(Some(nid)) => Some(nid),
                        _ => None,
                    };

                    if let Some(nid) = clicked_node_id {
                        let (state, _, _) = match this.active_camera_idx {
                            0 => (&this.outline_state, &this.outline_view, &this.outline_id_map),
                            1 => (&this.def_state, &this.def_view, &this.def_id_map),
                            _ => (&this.instances_state, &this.instances_view, &this.inst_id_map),
                        };

                        let id_str = if let Some(PropValue::Text(id_val)) =
                            state.get_node_prop(nid, "id")
                        {
                            Some(SharedString::from(id_val.as_str().to_string()))
                        } else if let Some(lbl) = state.display_label(nid) {
                            Some(SharedString::from(lbl.to_string()))
                        } else {
                            None
                        };

                        if let Some(node_str) = id_str {
                            this.workspace.update(cx, |w, cx| {
                                w.selected_hub_id = Some(node_str.clone());
                                cx.notify();
                            });
                            cx.emit(GraphEvent::NodeClicked(node_str));
                        }
                    }

                    // Handle CanvasAction if any
                    if let Some(action) = res.action {
                        match action {
                            graphene_gpui::CanvasAction::ToggleParentCollapse { .. } => {
                                this.rebuild_interaction_grid();
                            }
                            graphene_gpui::CanvasAction::CreateEdge { source, target } => {
                                let _ = (source, target);
                            }
                            graphene_gpui::CanvasAction::AddNewNode { .. } => {}
                        }
                    }

                    cx.notify();
                });
            }
        });

        let on_mouse_move = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |ev: &gpui::MouseMoveEvent, _window: &mut Window, cx: &mut gpui::App| {
                let mouse_pos = gpui::point(f32::from(ev.position.x), f32::from(ev.position.y));

                let _ = pane_entity.update(cx, |this, cx| {
                    let mut viewport = this.active_viewport();
                    let view_ref = match this.active_camera_idx {
                        0 => &this.outline_view,
                        1 => &this.def_view,
                        _ => &this.instances_view,
                    };
                    let mut interaction = this.interaction_state.clone();

                    if let Some((node_id, target_pos, _phase)) =
                        this.controller.handle_mouse_move(
                            mouse_pos,
                            &mut viewport,
                            view_ref,
                            &mut interaction,
                        )
                    {
                        let (state, view) = this.active_state_and_view();
                        state.set_node_position(node_id, target_pos);
                        if let Some(vn) = view.nodes.get_mut(&node_id) {
                            vn.pos = target_pos;
                        }
                    }

                    // Controller / interaction state may also pan; sync camera back
                    this.write_viewport_to_camera(&viewport);
                    this.interaction_state = interaction;
                    cx.notify();
                });
            }
        });

        let on_mouse_up = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, cx| {
                    let view_ref = match this.active_camera_idx {
                        0 => &this.outline_view,
                        1 => &this.def_view,
                        _ => &this.instances_view,
                    };
                    let mut interaction = this.interaction_state.clone();

                    if let Some((node_id, target_pos, _phase)) =
                        this.controller.handle_mouse_up(&mut interaction, view_ref)
                    {
                        let (state, view) = this.active_state_and_view();
                        state.set_node_position(node_id, target_pos);
                        if let Some(vn) = view.nodes.get_mut(&node_id) {
                            vn.pos = target_pos;
                        }
                    }

                    this.interaction_state = interaction;
                    this.rebuild_interaction_grid();
                    cx.notify();
                });
            }
        });

        let on_scroll_wheel = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |ev: &gpui::ScrollWheelEvent, _window: &mut Window, cx: &mut gpui::App| {
                let amount = match ev.delta {
                    gpui::ScrollDelta::Pixels(p) => f32::from(p.y),
                    gpui::ScrollDelta::Lines(p) => p.y * 20.0,
                };

                let _ = pane_entity.update(cx, |this, cx| {
                    let mut viewport = this.active_viewport();
                    this.controller.handle_scroll(amount, &mut viewport);
                    this.write_viewport_to_camera(&viewport);
                    cx.notify();
                });
            }
        });

        let on_zoom_in = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, cx| {
                    let mut viewport = this.active_viewport();
                    this.controller.handle_scroll(20.0, &mut viewport);
                    this.write_viewport_to_camera(&viewport);
                    cx.notify();
                });
            }
        });

        let on_zoom_out = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, cx| {
                    let mut viewport = this.active_viewport();
                    this.controller.handle_scroll(-20.0, &mut viewport);
                    this.write_viewport_to_camera(&viewport);
                    cx.notify();
                });
            }
        });

        let on_fit_view = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, cx| {
                    let view = match this.active_camera_idx {
                        0 => &this.outline_view,
                        1 => &this.def_view,
                        _ => &this.instances_view,
                    };
                    let mut vp = this.active_viewport();
                    vp.fit_to_graph(view);
                    this.write_viewport_to_camera(&vp);
                    cx.notify();
                });
            }
        });

        let on_bounds_changed = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |bounds: gpui::Bounds<f32>, _window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    if this.pane_bounds != bounds {
                        this.pane_bounds = bounds;
                        this.rebuild_interaction_grid();
                    }
                });
            }
        });

        let bounds_reporter = gpui::canvas(
            move |_, _, _| {},
            move |bounds, _, window, cx| {
                let bounds_f32 = gpui::Bounds {
                    origin: gpui::point(bounds.origin.x.as_f32(), bounds.origin.y.as_f32()),
                    size: gpui::size(bounds.size.width.as_f32(), bounds.size.height.as_f32()),
                };
                on_bounds_changed(
                    bounds_f32,
                    window,
                    cx,
                );
            },
        )
        .absolute()
        .size_full();

        let theme_ui = gpui_component::Theme::global(cx);
        let border_color = theme_ui.border;
        let sidebar_bg = theme_ui.sidebar;
        let fg_color = theme_ui.foreground;

        div()
            .flex_1()
            .h_full()
            .relative()
            .border_r(gpui::px(1.))
            .border_color(border_color)
            .flex()
            .flex_col()
            .on_mouse_down(gpui::MouseButton::Left, move |ev, window, cx| {
                on_mouse_down(ev, window, cx);
            })
            .on_mouse_move(move |ev, window, cx| {
                on_mouse_move(ev, window, cx);
            })
            .on_mouse_up(gpui::MouseButton::Left, move |_ev, window, cx| {
                on_mouse_up(window, cx);
            })
            .on_scroll_wheel(move |ev, window, cx| {
                on_scroll_wheel(ev, window, cx);
            })
            .child(bounds_reporter)
            .child(canvas_element)
            .child(
                gpui::div()
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
                    .child(format!("{}", tab_label)),
            )
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
                    .child({
                        let on_zoom_in = on_zoom_in.clone();
                        gpui_component::button::Button::new("zoom_in")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                                on_zoom_in(window, cx);
                            })
                            .label("+")
                            .text_size(gpui::px(12.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .min_w(gpui::px(28.))
                            .h(gpui::px(28.))
                            .border_r(gpui::px(1.))
                            .border_color(border_color)
                    })
                    .child({
                        let on_zoom_out = on_zoom_out.clone();
                        gpui_component::button::Button::new("zoom_out")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                                on_zoom_out(window, cx);
                            })
                            .label("-")
                            .text_size(gpui::px(12.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .min_w(gpui::px(28.))
                            .h(gpui::px(28.))
                            .border_color(border_color)
                    }),
            )
            .child({
                let pane_entity = cx_entity.clone();
                let auto_colors = self.auto_node_colors;
                gpui_component::button::Button::new("toggle_auto_colors")
                    .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                        let _ = pane_entity.update(cx, |this, cx| {
                            this.auto_node_colors = !this.auto_node_colors;
                            this.auto_edge_colors = this.auto_node_colors;
                            cx.notify();
                        });
                    })
                    .label(if auto_colors { "Auto Color: ON" } else { "Auto Color: OFF" })
                    .text_size(gpui::px(10.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .absolute()
                    .top(gpui::px(8.))
                    .right(gpui::px(175.))
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(6.))
                    .px_3()
                    .py_2()
                    .text_color(fg_color)
            })
            .child({
                let on_toggle_physics = std::sync::Arc::new({
                    let pane_entity = cx_entity.clone();
                    move |_window: &mut Window, cx: &mut gpui::App| {
                        let _ = pane_entity.update(cx, |this, cx| {
                            if this.is_ticking {
                                this.is_ticking = false;
                            } else {
                                this.run_layout(cx);
                            }
                        });
                    }
                });
                let is_ticking = self.is_ticking;
                gpui_component::button::Button::new("toggle_physics")
                    .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                        on_toggle_physics(window, cx);
                    })
                    .label(if is_ticking { "Pause Physics" } else { "Play Physics" })
                    .text_size(gpui::px(10.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .absolute()
                    .top(gpui::px(8.))
                    .right(gpui::px(80.))
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(6.))
                    .px_3()
                    .py_2()
                    .text_color(fg_color)
            })
            .child({
                let on_fit_view = on_fit_view.clone();
                gpui_component::button::Button::new("fit_view")
                    .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                        on_fit_view(window, cx);
                    })
                    .label("Fit View")
                    .text_size(gpui::px(10.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .absolute()
                    .top(gpui::px(8.))
                    .right(gpui::px(8.))
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(6.))
                    .px_3()
                    .py_2()
                    .text_color(fg_color)
            })
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
            .child(if let Some(sel_nid) = selected_node_id {
                if let Some(&idx) = active_state.node_keys.get(sel_nid) {
                    let node_data = active_state.nodes.get(idx);
                    let display_name = active_state.display_label(sel_nid).unwrap_or("Selected Node");
                    let primary_label = node_data.primary_label().unwrap_or("Node");
                    let cur_exp = node_data.expansion_mode;

                    let mut prop_rows = Vec::new();
                    for (k, v) in &node_data.props {
                        if !k.starts_with('@') {
                            prop_rows.push(format!("{}: {}", k, v.to_display_string()));
                        }
                    }

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
                                .child({
                                    let pane_entity = cx_entity.clone();
                                    gpui_component::button::Button::new("exp_compact")
                                        .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                            let _ = pane_entity.update(cx, |this, cx| {
                                                let (state, view) = this.active_state_and_view();
                                                state.set_node_expansion_mode(sel_nid, DataExpansionMode::Compact);
                                                view.load_preset(state);
                                                cx.notify();
                                            });
                                        })
                                        .label(if cur_exp == DataExpansionMode::Compact { "[Compact]" } else { "Compact" })
                                        .text_size(gpui::px(9.))
                                        .px_2()
                                        .py_1()
                                })
                                .child({
                                    let pane_entity = cx_entity.clone();
                                    gpui_component::button::Button::new("exp_preview")
                                        .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                            let _ = pane_entity.update(cx, |this, cx| {
                                                let (state, view) = this.active_state_and_view();
                                                state.set_node_expansion_mode(sel_nid, DataExpansionMode::Preview);
                                                view.load_preset(state);
                                                cx.notify();
                                            });
                                        })
                                        .label(if cur_exp == DataExpansionMode::Preview { "[Preview]" } else { "Preview" })
                                        .text_size(gpui::px(9.))
                                        .px_2()
                                        .py_1()
                                })
                                .child({
                                    let pane_entity = cx_entity.clone();
                                    gpui_component::button::Button::new("exp_full")
                                        .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                            let _ = pane_entity.update(cx, |this, cx| {
                                                let (state, view) = this.active_state_and_view();
                                                state.set_node_expansion_mode(sel_nid, DataExpansionMode::Full);
                                                view.load_preset(state);
                                                cx.notify();
                                            });
                                        })
                                        .label(if cur_exp == DataExpansionMode::Full { "[Full]" } else { "Full" })
                                        .text_size(gpui::px(9.))
                                        .px_2()
                                        .py_1()
                                })
                        )
                        .children(prop_rows.into_iter().map(|row| {
                            div()
                                .text_size(gpui::px(10.))
                                .font_family("monospace")
                                .text_color(fg_color.opacity(0.9))
                                .child(row)
                        }))
                } else {
                    div()
                }
            } else {
                div()
            })
    }
}
