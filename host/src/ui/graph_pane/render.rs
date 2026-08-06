//! Rendering logic for graph panes — GraphPaneView implementation using graphene_gpui::GraphCanvas.

use std::collections::{HashMap, HashSet};
use gpui::{div, prelude::*, SharedString, Window};
use graphene_core::math::Vec2;
use graphene_core::{NodeId, PropValue};
use graphene_gpui::render::draw_pipeline::Viewport;
use graphene_gpui::{CanvasConfig, GraphCanvas, InteractionState};
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
        match active_tab {
            crate::ui::GraphTab::DocumentGraph => self.select_tab_camera(0),
            crate::ui::GraphTab::DefinitionsSchema => self.select_tab_camera(1),
            crate::ui::GraphTab::InstancesRelation => self.select_tab_camera(2),
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

        // Determine selected graphene NodeId from workspace selected_hub_id
        let selected_node_id: Option<NodeId> = selected_hub_id
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
            });

        let cam = self.camera_states[active_tab_idx];
        let pane_w = self.pane_content_width.max(100.0);
        let pane_h = self.pane_content_height.max(100.0);

        let viewport = Viewport {
            offset: Vec2::new(cam.offset_x, cam.offset_y),
            zoom: cam.zoom,
            bounds: gpui::Bounds {
                origin: gpui::point(0.0, 0.0),
                size: gpui::size(pane_w, pane_h),
            },
        };

        let interaction_state = InteractionState::new(50.0);
        let theme = Theme::catppuccin_mocha();
        let node_labels = HashMap::new();
        let edge_labels = HashMap::new();
        let collapsed_parents = HashSet::new();

        let canvas_element = GraphCanvas::new(
            active_view,
            &viewport,
            &interaction_state,
            &theme,
            selected_node_id,
            &node_labels,
            &edge_labels,
            30,
            &collapsed_parents,
        )
        .with_config(CanvasConfig {
            edge_stroke_width: 2.0,
            arrow_length: 10.0,
            arrow_width: 8.0,
            node_border_width: 2.0,
            node_font_size: 11.0,
            ..CanvasConfig::default()
        })
        .into_element();

        // Mouse handlers
        let on_mouse_down = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            let viewport_clone = viewport.clone();

            move |ev: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut gpui::App| {
                let mouse_p = gpui::point(f32::from(ev.position.x), f32::from(ev.position.y));
                let model_p = viewport_clone.screen_to_model(mouse_p);

                let mut clicked_id_str: Option<SharedString> = None;
                let mut hit_node_id: Option<NodeId> = None;

                let _ = pane_entity.update(cx, |this, _| {
                    let (state, view, _) = match this.active_camera_idx {
                        0 => (&this.outline_state, &this.outline_view, &this.outline_id_map),
                        1 => (&this.def_state, &this.def_view, &this.def_id_map),
                        _ => (&this.instances_state, &this.instances_view, &this.inst_id_map),
                    };

                    for &nid in view.node_order.iter().rev() {
                        if let Some(node) = view.nodes.get(&nid) {
                            let hw = node.size.w / 2.0;
                            let hh = node.size.h / 2.0;
                            if model_p.x >= node.pos.x - hw
                                && model_p.x <= node.pos.x + hw
                                && model_p.y >= node.pos.y - hh
                                && model_p.y <= node.pos.y + hh
                            {
                                hit_node_id = Some(nid);

                                // Retrieve primary display string ID
                                if let Some(PropValue::Text(id_val)) = state.get_node_prop(nid, "id") {
                                    clicked_id_str = Some(SharedString::from(id_val.as_str().to_string()));
                                } else if let Some(lbl) = state.display_label(nid) {
                                    clicked_id_str = Some(SharedString::from(lbl.to_string()));
                                }
                                break;
                            }
                        }
                    }

                    if let Some(nid) = hit_node_id {
                        this.dragged_node = Some(nid);
                        this.last_mouse_pos = ev.position;
                    } else {
                        let cam_mut = this.active_camera_mut();
                        cam_mut.is_panning = true;
                        this.last_mouse_pos = ev.position;
                    }
                });

                if let Some(node_str) = clicked_id_str {
                    let _ = pane_entity.update(cx, |_, cx| {
                        cx.emit(GraphEvent::NodeClicked(node_str));
                    });
                }
            }
        });

        let on_mouse_move = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |ev: &gpui::MouseMoveEvent, _window: &mut Window, cx: &mut gpui::App| {
                let pos = ev.position;
                let _ = pane_entity.update(cx, |this, _| {
                    if let Some(dragged_id) = this.dragged_node {
                        let zoom = this.active_camera().zoom.max(0.01);
                        let dx = (f32::from(pos.x) - f32::from(this.last_mouse_pos.x)) / zoom;
                        let dy = (f32::from(pos.y) - f32::from(this.last_mouse_pos.y)) / zoom;
                        this.last_mouse_pos = pos;

                        let (state, view) = this.active_state_and_view();
                        if let Some(&idx) = state.node_keys.get(dragged_id) {
                            let cur_pos = *state.positions.get(idx);
                            let new_pos = Vec2::new(cur_pos.x + dx, cur_pos.y + dy);
                            state.set_node_position(dragged_id, new_pos);
                            if let Some(v_node) = view.nodes.get_mut(&dragged_id) {
                                v_node.pos = new_pos;
                            }
                        }
                    } else if this.active_camera().is_panning {
                        let zoom = this.active_camera().zoom.max(0.01);
                        let dx = (f32::from(pos.x) - f32::from(this.last_mouse_pos.x)) / zoom;
                        let dy = (f32::from(pos.y) - f32::from(this.last_mouse_pos.y)) / zoom;
                        this.last_mouse_pos = pos;

                        let cam_mut = this.active_camera_mut();
                        cam_mut.offset_x += dx;
                        cam_mut.offset_y += dy;
                    }
                });
            }
        });

        let on_mouse_up = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    this.dragged_node = None;
                    this.active_camera_mut().is_panning = false;
                });
            }
        });

        let on_scroll_wheel = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |ev: &gpui::ScrollWheelEvent, _window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    let cam = this.active_camera();
                    let old_z = cam.zoom;
                    let pixel_delta = ev.delta.pixel_delta(gpui::px(20.0));
                    let zoom_delta = -f32::from(pixel_delta.y) * 0.05;
                    let new_zoom = (old_z + zoom_delta).clamp(0.1, 5.0);

                    let cam_mut = this.active_camera_mut();
                    cam_mut.zoom = new_zoom;
                });
            }
        });

        let on_zoom_in = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    let cam_mut = this.active_camera_mut();
                    cam_mut.zoom = (cam_mut.zoom + 0.2).min(5.0);
                });
            }
        });

        let on_zoom_out = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    let cam_mut = this.active_camera_mut();
                    cam_mut.zoom = (cam_mut.zoom - 0.2).max(0.1);
                });
            }
        });

        let on_fit_view = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    let active_idx = this.active_camera_idx;
                    let (view, pane_w, pane_h) = (
                        match active_idx {
                            0 => &this.outline_view,
                            1 => &this.def_view,
                            _ => &this.instances_view,
                        },
                        this.pane_content_width.max(100.0),
                        this.pane_content_height.max(100.0),
                    );

                    let mut vp = Viewport {
                        offset: Vec2::default(),
                        zoom: 1.0,
                        bounds: gpui::Bounds {
                            origin: gpui::point(0.0, 0.0),
                            size: gpui::size(pane_w, pane_h),
                        },
                    };
                    vp.fit_to_graph(view);

                    let cam_mut = this.active_camera_mut();
                    cam_mut.offset_x = vp.offset.x;
                    cam_mut.offset_y = vp.offset.y;
                    cam_mut.zoom = vp.zoom;
                });
            }
        });

        let on_bounds_changed = std::sync::Arc::new({
            let pane_entity = cx_entity.clone();
            move |w: f32, h: f32, _window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_entity.update(cx, |this, _| {
                    this.pane_content_width = w;
                    this.pane_content_height = h;
                });
            }
        });

        let bounds_reporter = gpui::canvas(
            move |_, _, _| {},
            move |bounds, _, window, cx| {
                on_bounds_changed(
                    bounds.size.width.as_f32(),
                    bounds.size.height.as_f32(),
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
                    .left(gpui::px(196.))
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
    }
}
