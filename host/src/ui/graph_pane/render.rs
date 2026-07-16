//! Rendering logic for graph panes — GraphPanel (RenderOnce) and GraphPaneView (Render).

use gpui::{div, prelude::*, SharedString, Window};

use super::data::{GraphEvent, GraphPanel, LayoutMode};

// Short aliases for types used extensively in this file.
type GNode = crate::graph_sim::GraphNode;
type GEdge = crate::graph_sim::GraphEdge;

// ---------------------------------------------------------------------------
// Helpers (pure functions — no dependency on any module state)
// ---------------------------------------------------------------------------

pub(crate) fn layout_outline_tree(
    nodes: &[crate::parser::OutlineNode],
    edges: &[(usize, usize)],
    width: f32,
    height: f32,
    window: &mut Window, // needed for text measurement
) -> (Vec<GNode>, Vec<GEdge>) {
    let mut sizer = crate::graph_sim::sizing::gpui_text_sizer(window);
    layout_outline_tree_with_sizer_inner(nodes, edges, width, height, &mut sizer)
}

/// Variant that takes a pre-constructed sizer closure (for use when the caller
/// already has a `NodeSizer` bound to a Window).
pub(crate) fn layout_outline_tree_with_sizer(
    nodes: &[crate::parser::OutlineNode],
    edges: &[(usize, usize)],
    width: f32,
    height: f32,
    sizer: &mut impl FnMut(crate::graph_sim::sizing::NodeContent) -> (f32, f32),
) -> (Vec<GNode>, Vec<GEdge>) {
    layout_outline_tree_with_sizer_inner(nodes, edges, width, height, sizer)
}

fn layout_outline_tree_with_sizer_inner(
    nodes: &[crate::parser::OutlineNode],
    edges: &[(usize, usize)],
    width: f32,
    height: f32,
    sizer: &mut impl FnMut(crate::graph_sim::sizing::NodeContent) -> (f32, f32),
) -> (Vec<GNode>, Vec<GEdge>) {
    let n_len = nodes.len();
    if n_len == 0 {
        return (Vec::new(), Vec::new());
    }

    let mut in_degrees = vec![0; n_len];
    let mut adj = vec![Vec::new(); n_len];
    for &(parent, child) in edges {
        if parent < n_len && child < n_len {
            adj[parent].push(child);
            in_degrees[child] += 1;
        }
    }

    let mut depths = vec![0; n_len];
    let mut queue = std::collections::VecDeque::new();
    for i in 0..n_len {
        if in_degrees[i] == 0 {
            queue.push_back((i, 0));
        }
    }

    while let Some((node_idx, d)) = queue.pop_front() {
        depths[node_idx] = d;
        for &child in &adj[node_idx] {
            queue.push_back((child, d + 1));
        }
    }

    let max_depth = *depths.iter().max().unwrap_or(&0);

    let mut level_nodes = vec![Vec::new(); max_depth + 1];
    for i in 0..n_len {
        let d = depths[i];
        if d <= max_depth {
            level_nodes[d].push(i);
        }
    }

    let mut out_nodes = Vec::with_capacity(n_len);
    let mut coords = vec![(0.0f32, 0.0f32); n_len];

    for d in 0..=max_depth {
        let count = level_nodes[d].len();
        let y = ((d + 1) as f32 * height) / ((max_depth + 2) as f32);
        for (col, &node_idx) in level_nodes[d].iter().enumerate() {
            let x = ((col + 1) as f32 * width) / ((count + 1) as f32);
            coords[node_idx] = (x, y);

            let color_hsla = match nodes[node_idx].kind.as_str() {
                "section" => gpui::hsla(0.6, 0.8, 0.5, 1.0),
                "heading" => gpui::hsla(0.3, 0.8, 0.5, 1.0),
                "paragraph" => gpui::hsla(0.0, 0.0, 0.8, 1.0),
                "hubref" => gpui::hsla(0.1, 0.8, 0.5, 1.0),
                _ => gpui::hsla(0.8, 0.8, 0.5, 1.0),
            };

            let (measured_w, measured_h) = sizer(crate::graph_sim::sizing::NodeContent {
                name: &nodes[node_idx].name,
                type_name: &nodes[node_idx].kind,
                attributes: &[],
            });

            out_nodes.push(GNode {
                id: nodes[node_idx].id.clone().into(),
                name: nodes[node_idx].name.clone().into(),
                type_name: nodes[node_idx].kind.clone().into(),
                color: color_hsla,
                x: coords[node_idx].0,
                y: coords[node_idx].1,
                vx: 0.0,
                vy: 0.0,
                anchor_x: coords[node_idx].0,
                anchor_y: coords[node_idx].1,
                width: measured_w,
                height: measured_h,
                attributes: vec![],
            });
        }
    }

    let mut out_edges = Vec::with_capacity(edges.len());
    for &(parent, child) in edges {
        if parent < n_len && child < n_len {
            let (source_x, source_y) = coords[parent];
            let (target_x, target_y) = coords[child];
            out_edges.push(GEdge {
                source: parent,
                target: child,
                label: "->".into(),
            });
        }
    }

    (out_nodes, out_edges)
}

fn compute_fit_zoom(nodes: &[GNode], view_width: f32, view_height: f32) -> Option<(f32, f32, f32)> {
    if nodes.is_empty() {
        return Some((0.0, 0.0, 1.0));
    }

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for node in nodes {
        let hw = node.width / 2.0;
        let hh = node.height / 2.0;
        min_x = min_x.min(node.x - hw);
        max_x = max_x.max(node.x + hw);
        min_y = min_y.min(node.y - hh);
        max_y = max_y.max(node.y + hh);
    }

    let content_w = (max_x - min_x).max(1.0);
    let content_h = (max_y - min_y).max(1.0);
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    let zoom_x = (view_width / content_w) * 0.9;
    let zoom_y = (view_height / content_h) * 0.9;
    let fit_zoom = (zoom_x.min(zoom_y)).max(0.1);

    let cam_x = center_x - (view_width / 2.0) / fit_zoom;
    let cam_y = center_y - (view_height / 2.0) / fit_zoom;

    Some((cam_x, cam_y, fit_zoom))
}

// ---------------------------------------------------------------------------
// GraphPanel :: RenderOnce
// ---------------------------------------------------------------------------

impl RenderOnce for GraphPanel {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let border_color = theme.border;
        let sidebar_bg = theme.sidebar;
        let fg_color = theme.foreground;
        let active_accent = theme.primary;
        let nodes = &self.nodes;
        let edges = &self.edges;
        let label = self.label;

        let zoom = self.zoom;
        let cam_x = self.camera_offset_x;
        let cam_y = self.camera_offset_y;

        // Canvas that draws edges as paths (needs owned data via to_vec)
        let edge_data = edges.to_vec();
        let node_ref = nodes.to_vec();
        let on_bounds_changed = self.on_bounds_changed.clone();
        let canvas = gpui::canvas(
            move |_, _, _| {},
            move |bounds, _, window, cx| {
                if let Some(ref cb) = on_bounds_changed {
                    cb(
                        bounds.size.width.as_f32(),
                        bounds.size.height.as_f32(),
                        window,
                        cx,
                    );
                }
                for edge in &edge_data {
                    let src_idx = edge.source;
                    let tgt_idx = edge.target;
                    if src_idx < node_ref.len() && tgt_idx < node_ref.len() {
                        let src = &node_ref[src_idx];
                        let tgt = &node_ref[tgt_idx];

                        // AABB edge attachment points (world space)
                        let dx_world = tgt.x - src.x;
                        let dy_world = tgt.y - src.y;

                        let exit_x = if dx_world.abs() * src.height > dy_world.abs() * src.width {
                            if dx_world > 0.0 {
                                src.x + src.width / 2.0
                            } else {
                                src.x - src.width / 2.0
                            }
                        } else {
                            src.x
                        };
                        let exit_y = if dx_world.abs() * src.height > dy_world.abs() * src.width {
                            src.y
                        } else if dy_world > 0.0 {
                            src.y + src.height / 2.0
                        } else {
                            src.y - src.height / 2.0
                        };

                        let enter_x = if dx_world.abs() * tgt.height > dy_world.abs() * tgt.width {
                            if dx_world < 0.0 {
                                tgt.x + tgt.width / 2.0
                            } else {
                                tgt.x - tgt.width / 2.0
                            }
                        } else {
                            tgt.x
                        };
                        let enter_y = if dx_world.abs() * tgt.height > dy_world.abs() * tgt.width {
                            tgt.y
                        } else if dx_world < 0.0 {
                            tgt.y + tgt.height / 2.0
                        } else {
                            tgt.y - tgt.height / 2.0
                        };

                        let p1 = gpui::point(
                            bounds.origin.x + gpui::px(((exit_x - cam_x) * zoom).round()),
                            bounds.origin.y + gpui::px(((exit_y - cam_y) * zoom).round()),
                        );
                        let p2 = gpui::point(
                            bounds.origin.x + gpui::px(((enter_x - cam_x) * zoom).round()),
                            bounds.origin.y + gpui::px(((enter_y - cam_y) * zoom).round()),
                        );

                        let dx_screen = (enter_x - exit_x) * zoom;
                        let dy_screen = (enter_y - exit_y) * zoom;
                        let dist = (dx_screen * dx_screen + dy_screen * dy_screen).sqrt();

                        let scaled_line_width = gpui::px((2.0 * zoom).max(1.0));
                        let mut builder = gpui::PathBuilder::stroke(scaled_line_width);
                        builder.move_to(p1);
                        builder.line_to(p2);
                        if let Ok(path) = builder.build() {
                            window.paint_path(path, border_color);
                        }

                        if dist > 0.001 {
                            let ux = dx_screen / dist;
                            let uy = dy_screen / dist;
                            let px = -uy;
                            let py = ux;

                            let arrow_len = (26.0 * zoom).max(14.0);
                            let arm_len = (10.0 * zoom).max(5.0);
                            let arrow_width = 6.0 * zoom;

                            let draw_arrowhead =
                                |window: &mut gpui::Window,
                                 tip_x: f32,
                                 tip_y: f32,
                                 base_x: f32,
                                 base_y: f32| {
                                    let left_x = base_x + arrow_width * px;
                                    let left_y = base_y + arrow_width * py;
                                    let right_x = base_x - arrow_width * px;
                                    let right_y = base_y - arrow_width * py;

                                    let mut arrow_builder =
                                        gpui::PathBuilder::stroke(scaled_line_width);
                                    arrow_builder.move_to(gpui::point(
                                        bounds.origin.x + gpui::px(left_x),
                                        bounds.origin.y + gpui::px(left_y),
                                    ));
                                    arrow_builder.line_to(gpui::point(
                                        bounds.origin.x + gpui::px(tip_x),
                                        bounds.origin.y + gpui::px(tip_y),
                                    ));
                                    arrow_builder.line_to(gpui::point(
                                        bounds.origin.x + gpui::px(right_x),
                                        bounds.origin.y + gpui::px(right_y),
                                    ));
                                    if let Ok(arrow_path) = arrow_builder.build() {
                                        window.paint_path(arrow_path, border_color);
                                    }
                                };

                            if edge.label == "->" || edge.label == "<->" {
                                let tip_fx = ((enter_x - cam_x) * zoom - arrow_len * ux).round();
                                let tip_fy = ((enter_y - cam_y) * zoom - arrow_len * uy).round();
                                let base_fx = tip_fx - (arm_len * ux).round() as f32;
                                let base_fy = tip_fy - (arm_len * uy).round() as f32;
                                draw_arrowhead(window, tip_fx, tip_fy, base_fx, base_fy);
                            }
                            if edge.label == "<-" || edge.label == "<->" {
                                let tip_fx = ((exit_x - cam_x) * zoom + arrow_len * ux).round();
                                let tip_fy = ((exit_y - cam_y) * zoom + arrow_len * uy).round();
                                let base_fx = tip_fx + (arm_len * ux).round() as f32;
                                let base_fy = tip_fy + (arm_len * uy).round() as f32;
                                draw_arrowhead(window, tip_fx, tip_fy, base_fx, base_fy);
                            }
                        }
                    }
                }
            },
        )
        .absolute()
        .size_full();

        // Build node div elements from the original owned data
        let mut node_elements = Vec::new();
        let on_drag_start = self.on_node_drag_start.clone();
        let on_click = self.on_node_click.clone();

        for (idx, node) in nodes.iter().enumerate() {
            let color = node.color;
            let node_id = node.id.clone();
            let is_selected = Some(&node.id) == self.selected_hub_id.as_ref();

            let sx = ((node.x - cam_x) * zoom).round() as i32;
            let sy = ((node.y - cam_y) * zoom).round() as i32;
            let scaled_w = (node.width * zoom).max(40.0);
            let scaled_h = (node.height * zoom).max(30.0);
            let half_sx = sx - (scaled_w / 2.0) as i32;
            let half_sy = sy - (scaled_h / 2.0) as i32;

            let border_color_val = if is_selected {
                active_accent
            } else {
                border_color
            };

            let font_size = (13.0 * zoom).max(9.0);
            let header_font_size = (10.0 * zoom).max(9.0);
            let attr_font_size = (11.0 * zoom).max(9.0);
            let scaled_border = gpui::px((2.0 * zoom).max(1.0));
            let scaled_rounded = gpui::px((6.0 * zoom).max(3.0));

            let node_on_drag_start = on_drag_start.clone();
            let node_on_click = on_click.clone();
            let node_id_clone = node_id.clone();

            let node_div = gpui::div()
                .id(format!("node-{}", node_id))
                .absolute()
                .left(gpui::px(half_sx as f32))
                .top(gpui::px(half_sy as f32))
                .w(gpui::px(scaled_w))
                .h(gpui::px(scaled_h))
                .flex()
                .flex_col()
                .bg(color)
                .border(scaled_border)
                .border_color(border_color_val)
                .rounded(scaled_rounded)
                .shadow_md()
                .hover(|s| s.border_color(active_accent))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    move |_ev: &gpui::MouseDownEvent, window, cx| {
                        if let Some(ref cb) = node_on_click {
                            cb(node_id_clone.clone(), window, cx);
                        }
                        if let Some(ref cb) = node_on_drag_start {
                            cb(
                                node_id_clone.clone(),
                                gpui::point(gpui::px(half_sx as f32), gpui::px(half_sy as f32)),
                                window,
                                cx,
                            );
                        }
                    },
                )
                .child(
                    gpui::div()
                        .bg(color.opacity(0.2))
                        .border_b(gpui::px((1.0 * zoom).max(1.0)))
                        .border_color(border_color_val)
                        .px_4()
                        .py_2()
                        .flex()
                        .flex_col()
                        .items_center()
                        .child(
                            gpui::div()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_size(gpui::px(font_size))
                                .text_color(fg_color)
                                .child(node.name.clone()),
                        )
                        .child(
                            gpui::div()
                                .text_size(gpui::px(header_font_size))
                                .text_color(fg_color.opacity(0.7))
                                .child(format!("«{}»", node.type_name)),
                        ),
                )
                .child(gpui::div().px_4().py_1().flex().flex_col().children(
                    node.attributes.iter().map(|attr| {
                        gpui::div()
                            .text_size(gpui::px(attr_font_size))
                            .font_family("monospace")
                            .text_color(fg_color)
                            .child(attr.clone())
                    }),
                ));

            node_elements.push(node_div);
        }

        let on_mouse_move = self.on_mouse_move.clone();
        let on_mouse_up = self.on_mouse_up.clone();
        let on_bg_mouse_down = self.on_bg_mouse_down.clone();
        let on_scroll_wheel = self.on_scroll_wheel.clone();
        let on_zoom_in = self.on_zoom_in.clone();
        let on_zoom_out = self.on_zoom_out.clone();
        let on_fit_view = self.on_fit_view.clone();

        div()
            .flex_1()
            .h_full()
            .border_r(gpui::px(1.))
            .border_color(border_color)
            .flex()
            .flex_col()
            .on_mouse_down(
                gpui::MouseButton::Left,
                move |ev: &gpui::MouseDownEvent, window, cx| {
                    if let Some(ref cb) = on_bg_mouse_down {
                        cb(ev, window, cx);
                    }
                },
            )
            .on_mouse_move(move |ev: &gpui::MouseMoveEvent, window, cx| {
                if let Some(ref cb) = on_mouse_move {
                    cb(ev.position, window, cx);
                }
            })
            .on_mouse_up(gpui::MouseButton::Left, move |_ev, window, cx| {
                if let Some(ref cb) = on_mouse_up {
                    cb(window, cx);
                }
            })
            .on_scroll_wheel(move |ev: &gpui::ScrollWheelEvent, window, cx| {
                if let Some(ref cb) = on_scroll_wheel {
                    cb(ev, window, cx);
                }
            })
            .child(canvas)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .bg(sidebar_bg)
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .children(node_elements),
            )
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
                    .child(format!("{}", label)),
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
                    .child(
                        gpui_component::button::Button::new("zoom_in")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                                if let Some(ref cb) = on_zoom_in {
                                    cb(window, cx);
                                }
                            })
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
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                                if let Some(ref cb) = on_zoom_out {
                                    cb(window, cx);
                                }
                            })
                            .label("-")
                            .text_size(gpui::px(12.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .min_w(gpui::px(28.))
                            .h(gpui::px(28.))
                            .border_color(border_color),
                    )
                    .child(
                        gpui::div()
                            .w(gpui::px(1.))
                            .h(gpui::px(28.))
                            .bg(border_color),
                    ),
            )
            .child(
                gpui_component::button::Button::new("fit_view")
                    .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, cx| {
                        if let Some(ref cb) = on_fit_view {
                            cb(window, cx);
                        }
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
                    .text_color(fg_color),
            )
            .child(if let Some(layout_selector) = self.layout_selector {
                layout_selector.into_any_element()
            } else {
                gpui::div().into_any_element()
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
                    .child(format!("Zoom: {:.0}%", (zoom * 100.0).round()))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(gpui::px(14.))
                            .h(gpui::px(14.))
                            .border(gpui::px(2.))
                            .border_color(fg_color)
                            .rounded(gpui::px(2.)),
                    ),
            )
    }
}

// ---------------------------------------------------------------------------
// GraphPaneView :: Render
// ---------------------------------------------------------------------------

impl Render for super::state::GraphPaneView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check if a layout run is pending and execute it synchronously
        let should_run_layout = matches!(self.layout_mode, LayoutMode::RunLayout(_));
        if should_run_layout {
            self.layout_mode = LayoutMode::None;
            self.run_layout(cx);
        }

        let (active_tab, selected_hub_id) = {
            let w = self.workspace.read(cx);
            (w.active_graph_tab, w.selected_hub_id.clone())
        };

        // Ensure active_camera_idx matches the visible tab so user input targets the right pane.
        match active_tab {
            super::super::GraphTab::DocumentGraph => self.select_tab_camera(0),
            super::super::GraphTab::DefinitionsSchema => self.select_tab_camera(1),
            super::super::GraphTab::InstancesRelation => self.select_tab_camera(2),
        }

        let cx_entity = cx.entity().clone();

        // Node click handler
        let on_node_click = std::sync::Arc::new({
            let cx_entity = cx_entity.clone();
            move |node_id: SharedString, _window: &mut Window, cx: &mut gpui::App| {
                let _ = cx_entity.update(cx, |_this, cx| {
                    cx.emit(GraphEvent::NodeClicked(node_id));
                });
            }
        });

        // Drag state management
        let on_mouse_down = std::sync::Arc::new({
            let pane_d = cx_entity.clone();
            move |node_id: SharedString,
                  ev: gpui::Point<gpui::Pixels>,
                  _window: &mut Window,
                  cx: &mut gpui::App| {
                let _ = pane_d.update(cx, |this, _cx| {
                    this.dragged_node = Some(node_id.clone());
                    this.last_mouse_pos = ev;
                });
            }
        });

        let on_mouse_move = std::sync::Arc::new({
            let pane_m = cx_entity.clone();
            move |pos: gpui::Point<gpui::Pixels>, _window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_m.update(cx, |this, _cx| {
                    if let Some(ref dragged_id) = this.dragged_node {
                        let delta_x = (f32::from(pos.x) - f32::from(this.last_mouse_pos.x))
                            / this.active_camera().zoom;
                        let delta_y = (f32::from(pos.y) - f32::from(this.last_mouse_pos.y))
                            / this.active_camera().zoom;
                        this.last_mouse_pos = pos;

                        if let Some(node) = this
                            .graph_nodes
                            .iter_mut()
                            .find(|n| n.id == *dragged_id)
                            .or_else(|| this.def_nodes.iter_mut().find(|n| n.id == *dragged_id))
                            .or_else(|| this.outline_nodes.iter_mut().find(|n| n.id == *dragged_id))
                        {
                            node.x += delta_x;
                            node.y += delta_y;
                            node.anchor_x = node.x;
                            node.anchor_y = node.y;
                        }
                    } else if this.active_camera().is_panning {
                        let zoom_for_calc = this.active_camera().zoom.max(0.01);
                        let pan_dx =
                            -(f32::from(pos.x) - f32::from(this.last_mouse_pos.x)) / zoom_for_calc;
                        let pan_dy =
                            -(f32::from(pos.y) - f32::from(this.last_mouse_pos.y)) / zoom_for_calc;
                        this.last_mouse_pos = pos;
                        let cam = this.active_camera_mut();
                        cam.offset_x += pan_dx;
                        cam.offset_y += pan_dy;
                    }
                });
            }
        });

        let on_mouse_up = std::sync::Arc::new({
            let pane_u = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_u.update(cx, |this, _cx| {
                    this.dragged_node = None;
                    let cam = this.active_camera_mut();
                    cam.is_panning = false;
                });
            }
        });

        // SHARED VIEWPORT CONTROLS
        let on_bg_mouse_down_shared = std::sync::Arc::new({
            let pane_bg = cx_entity.clone();
            move |ev: &gpui::MouseDownEvent, _window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_bg.update(cx, |this, _| {
                    let cam = this.active_camera_mut();
                    cam.is_panning = true;
                    this.last_mouse_pos = ev.position;
                });
            }
        });

        let on_scroll_wheel_shared = std::sync::Arc::new({
            let pane_zoom = cx_entity.clone();
            move |ev: &gpui::ScrollWheelEvent, _window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_zoom.update(cx, |this, _cx| {
                    let cam = this.active_camera();
                    let old_z = cam.zoom;
                    let offset_x = cam.offset_x;
                    let offset_y = cam.offset_y;

                    let pixel_delta = ev.delta.pixel_delta(gpui::px(20.0));
                    let zoom_delta = -f32::from(pixel_delta.y) * 0.1;
                    let new_zoom = (old_z + zoom_delta).clamp(0.1, 5.0);

                    if (new_zoom - old_z).abs() < f32::EPSILON {
                        return;
                    }

                    let mouse_container_x = f32::from(ev.position.x);
                    let mouse_container_y = f32::from(ev.position.y);

                    let world_x = mouse_container_x / old_z + offset_x;
                    let world_y = mouse_container_y / old_z + offset_y;

                    let cam_mut = this.active_camera_mut();
                    cam_mut.offset_x = world_x - mouse_container_x / new_zoom;
                    cam_mut.offset_y = world_y - mouse_container_y / new_zoom;
                    cam_mut.zoom = new_zoom;
                });
            }
        });

        let on_zoom_in_shared = std::sync::Arc::new({
            let pane_zoom = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_zoom.update(cx, |view, _| {
                    let cam = view.active_camera();
                    let old_z = cam.zoom;
                    let offset_x = cam.offset_x;
                    let offset_y = cam.offset_y;

                    let new_zoom = (old_z + 0.2).min(5.0);
                    if (new_zoom - old_z).abs() < f32::EPSILON {
                        return;
                    }

                    // Re-anchor around panel center for consistency with scroll-wheel
                    let pan_w = view.pane_content_width.max(100.0);
                    let pan_h = view.pane_content_height.max(100.0);
                    let center_x = pan_w / 2.0;
                    let center_y = pan_h / 2.0;

                    let world_x = center_x / old_z + offset_x;
                    let world_y = center_y / old_z + offset_y;

                    let cam_mut = view.active_camera_mut();
                    cam_mut.offset_x = world_x - center_x / new_zoom;
                    cam_mut.offset_y = world_y - center_y / new_zoom;
                    cam_mut.zoom = new_zoom;
                });
            }
        });

        let on_zoom_out_shared = std::sync::Arc::new({
            let pane_zoom = cx_entity.clone();
            move |_window: &mut Window, cx: &mut gpui::App| {
                let _ = pane_zoom.update(cx, |view, _| {
                    let cam = view.active_camera();
                    let old_z = cam.zoom;
                    let offset_x = cam.offset_x;
                    let offset_y = cam.offset_y;

                    let new_zoom = (old_z - 0.2).max(0.1);
                    if (new_zoom - old_z).abs() < f32::EPSILON {
                        return;
                    }

                    let pan_w = view.pane_content_width.max(100.0);
                    let pan_h = view.pane_content_height.max(100.0);
                    let center_x = pan_w / 2.0;
                    let center_y = pan_h / 2.0;

                    let world_x = center_x / old_z + offset_x;
                    let world_y = center_y / old_z + offset_y;

                    let cam_mut = view.active_camera_mut();
                    cam_mut.offset_x = world_x - center_x / new_zoom;
                    cam_mut.offset_y = world_y - center_y / new_zoom;
                    cam_mut.zoom = new_zoom;
                });
            }
        });

        match active_tab {
            super::super::GraphTab::DocumentGraph => GraphPanel {
                nodes: self.outline_nodes.clone(),
                edges: self.outline_edges.clone(),
                label: "TWXML DOCUMENT GRAPH".into(),
                selected_hub_id: selected_hub_id.clone(),
                on_node_click: Some(on_node_click.clone()),
                on_node_drag_start: Some(on_mouse_down.clone()),
                on_mouse_move: Some(on_mouse_move.clone()),
                on_mouse_up: Some(on_mouse_up.clone()),
                on_bg_mouse_down: Some(on_bg_mouse_down_shared.clone()),
                on_scroll_wheel: Some(on_scroll_wheel_shared.clone()),
                on_zoom_in: Some(on_zoom_in_shared.clone()),
                on_zoom_out: Some(on_zoom_out_shared.clone()),
                layout_selector: Some(
                    gpui_component::button::Button::new("run_layout_document_graph")
                        .label("Run Layout")
                        .on_click({
                            let pane = cx_entity.clone();
                            move |_ev, _window, cx| {
                                let _ = pane.update(cx, |view, cx| {
                                    view.run_layout(cx);
                                    cx.notify();
                                });
                            }
                        })
                        .into_any_element(),
                ),
                camera_offset_x: self.camera_states[0].offset_x,
                camera_offset_y: self.camera_states[0].offset_y,
                zoom: self.camera_states[0].zoom,
                on_fit_view: Some(std::sync::Arc::new({
                    let pane_fit = cx_entity.clone();
                    move |_window, cx| {
                        let _ = pane_fit.update(cx, |view, _| {
                            let (fit_cam_x, fit_cam_y, fit_zoom) = compute_fit_zoom(
                                &view.outline_nodes,
                                view.pane_content_width.max(200.0),
                                view.pane_content_height.max(200.0),
                            )
                            .unwrap_or((0.0, 0.0, 1.0));
                            let cam = view.active_camera_mut();
                            cam.offset_x = fit_cam_x;
                            cam.offset_y = fit_cam_y;
                            cam.zoom = fit_zoom;
                        });
                    }
                })),
                on_bounds_changed: Some(std::sync::Arc::new({
                    let pane_bounds = cx_entity.clone();
                    move |w, h, _window, cx| {
                        let _ = pane_bounds.update(cx, |view, _| {
                            view.pane_content_width = w;
                            view.pane_content_height = h;
                        });
                    }
                })),
            }
            .into_any_element(),

            super::super::GraphTab::DefinitionsSchema => GraphPanel {
                nodes: self.def_nodes.clone(),
                edges: self.def_edges.clone(),
                label: "HUBGS DEFINITIONS SCHEMA".into(),
                selected_hub_id: selected_hub_id.clone(),
                on_node_click: Some(on_node_click.clone()),
                on_node_drag_start: Some(on_mouse_down.clone()),
                on_mouse_move: Some(on_mouse_move.clone()),
                on_mouse_up: Some(on_mouse_up.clone()),
                on_bg_mouse_down: Some(on_bg_mouse_down_shared.clone()),
                on_scroll_wheel: Some(on_scroll_wheel_shared.clone()),
                on_zoom_in: Some(on_zoom_in_shared.clone()),
                on_zoom_out: Some(on_zoom_out_shared.clone()),
                layout_selector: Some(
                    gpui_component::button::Button::new("run_layout_definitions_schema")
                        .label("Run Layout")
                        .on_click({
                            let pane = cx_entity.clone();
                            move |_ev, _window, cx| {
                                let _ = pane.update(cx, |view, cx| {
                                    view.run_layout(cx);
                                    cx.notify();
                                });
                            }
                        })
                        .into_any_element(),
                ),
                camera_offset_x: self.camera_states[1].offset_x,
                camera_offset_y: self.camera_states[1].offset_y,
                zoom: self.camera_states[1].zoom,
                on_fit_view: Some(std::sync::Arc::new({
                    let pane_fit = cx_entity.clone();
                    move |_window, cx| {
                        let _ = pane_fit.update(cx, |view, _| {
                            let (fit_cam_x, fit_cam_y, fit_zoom) = compute_fit_zoom(
                                &view.def_nodes,
                                view.pane_content_width.max(200.0),
                                view.pane_content_height.max(200.0),
                            )
                            .unwrap_or((0.0, 0.0, 1.0));
                            let cam = view.active_camera_mut();
                            cam.offset_x = fit_cam_x;
                            cam.offset_y = fit_cam_y;
                            cam.zoom = fit_zoom;
                        });
                    }
                })),
                on_bounds_changed: Some(std::sync::Arc::new({
                    let pane_bounds = cx_entity.clone();
                    move |w, h, _window, cx| {
                        let _ = pane_bounds.update(cx, |view, _| {
                            view.pane_content_width = w;
                            view.pane_content_height = h;
                        });
                    }
                })),
            }
            .into_any_element(),

            super::super::GraphTab::InstancesRelation => GraphPanel {
                nodes: self.graph_nodes.clone(),
                edges: self.graph_edges.clone(),
                label: "HUBGS INSTANCES RELATION".into(),
                selected_hub_id: selected_hub_id.clone(),
                on_node_click: Some(on_node_click.clone()),
                on_node_drag_start: Some(on_mouse_down.clone()),
                on_mouse_move: Some(on_mouse_move.clone()),
                on_mouse_up: Some(on_mouse_up.clone()),
                on_bg_mouse_down: Some(on_bg_mouse_down_shared.clone()),
                on_scroll_wheel: Some(on_scroll_wheel_shared.clone()),
                on_zoom_in: Some(on_zoom_in_shared.clone()),
                on_zoom_out: Some(on_zoom_out_shared.clone()),
                layout_selector: Some(
                    gpui_component::button::Button::new("run_layout_instances_relation")
                        .label("Run Layout")
                        .on_click({
                            let pane = cx_entity.clone();
                            move |_ev, _window, cx| {
                                let _ = pane.update(cx, |view, cx| {
                                    view.run_layout(cx);
                                    cx.notify();
                                });
                            }
                        })
                        .into_any_element(),
                ),
                camera_offset_x: self.camera_states[2].offset_x,
                camera_offset_y: self.camera_states[2].offset_y,
                zoom: self.camera_states[2].zoom,
                on_fit_view: Some(std::sync::Arc::new({
                    let pane_fit = cx_entity.clone();
                    move |_window, cx| {
                        let _ = pane_fit.update(cx, |view, _| {
                            let (fit_cam_x, fit_cam_y, fit_zoom) = compute_fit_zoom(
                                &view.graph_nodes,
                                view.pane_content_width.max(200.0),
                                view.pane_content_height.max(200.0),
                            )
                            .unwrap_or((0.0, 0.0, 1.0));
                            let cam = view.active_camera_mut();
                            cam.offset_x = fit_cam_x;
                            cam.offset_y = fit_cam_y;
                            cam.zoom = fit_zoom;
                        });
                    }
                })),
                on_bounds_changed: Some(std::sync::Arc::new({
                    let pane_bounds = cx_entity.clone();
                    move |w, h, _window, cx| {
                        let _ = pane_bounds.update(cx, |view, _| {
                            view.pane_content_width = w;
                            view.pane_content_height = h;
                        });
                    }
                })),
            }
            .into_any_element(),
        }
    }
}
