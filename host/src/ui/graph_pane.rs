//! Graph pane rendering — definitions/instances relation graphs.
//!
//! Extracted from `ui/mod.rs` to eliminate ~90 lines of near-duplicate rendering logic
//! between the definitions and instances graph panels, and to reduce `mod.rs` file length.
//! [user-review: split required] See task ticket for splitting rationale.

use gpui::{div, prelude::*, Hsla, Entity, Render, Context, Window};
use std::path::Path;

use super::super::graph_sim::{GraphNode, GraphEdge, run_graph_simulation, run_def_simulation, parse_hubgs_file};
use super::Workspace;

/// A single graph panel containing nodes, edges, and a label.
pub(crate) struct GraphPanel {
    pub(crate) nodes: Vec<GraphNode>,
    pub(crate) edges: Vec<GraphEdge>,
    pub(crate) label: &'static str,
}

// ─── GraphPaneView View ──────────────────────────────────────────────────────

pub(crate) struct GraphPaneView {
    _workspace: Entity<Workspace>,
    graph_nodes: Vec<GraphNode>,
    graph_edges: Vec<GraphEdge>,
    def_nodes: Vec<GraphNode>,
    def_edges: Vec<GraphEdge>,
}

impl GraphPaneView {
    pub(crate) fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let ws = workspace.clone();
        cx.observe(&workspace, move |this, _, cx| {
            this.recalculate_layout(&ws, cx);
        }).detach();

        let mut this = Self {
            _workspace: workspace.clone(),
            graph_nodes: Vec::new(),
            graph_edges: Vec::new(),
            def_nodes: Vec::new(),
            def_edges: Vec::new(),
        };
        this.recalculate_layout(&workspace, cx);
        this
    }

    fn recalculate_layout(&mut self, workspace: &Entity<Workspace>, cx: &mut Context<Self>) {
        let selected_path = workspace.read(cx).selected_path.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let layout_data = cx.background_executor().spawn(async move {
                    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent()
                        .unwrap()
                        .to_path_buf();

                    let hubgs_path = selected_path.as_ref()
                        .map(|p| p.with_extension("hubgs"));

                    let target_hubgs = if let Some(ref hp) = hubgs_path {
                        if hp.exists() {
                            Some(hp.clone())
                        } else {
                            crate::graph_sim::find_any_hubgs(&workspace_root)
                        }
                    } else {
                        crate::graph_sim::find_any_hubgs(&workspace_root)
                    };

                    let mut graph_nodes = Vec::new();
                    let mut graph_edges = Vec::new();
                    let mut def_nodes = Vec::new();
                    let mut def_edges = Vec::new();

                    if let Some(hp) = target_hubgs {
                        if let Ok((defs, instances)) = parse_hubgs_file(&hp) {
                            let (n, e) = run_graph_simulation(&instances, &defs, 500.0, 500.0);
                            graph_nodes = n;
                            graph_edges = e;

                            let (dn, de) = run_def_simulation(&defs, 500.0, 500.0);
                            def_nodes = dn;
                            def_edges = de;
                        }
                    }

                    (graph_nodes, graph_edges, def_nodes, def_edges)
                }).await;

                let _ = this.update(&mut cx, |this, cx| {
                    this.graph_nodes = layout_data.0;
                    this.graph_edges = layout_data.1;
                    this.def_nodes = layout_data.2;
                    this.def_edges = layout_data.3;
                    cx.notify();
                });
            }
        }).detach();
    }
}

impl Render for GraphPaneView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = gpui_component::Theme::global(cx);
        let bg_color = theme.background;
        let fg_color = theme.foreground;
        let border_color = theme.border;
        let sidebar_bg = theme.sidebar;
        let active_accent = theme.primary;
        let theme_muted_foreground = theme.muted_foreground;

        let left_panel = GraphPanel {
            nodes: self.def_nodes.clone(),
            edges: self.def_edges.clone(),
            label: "DEFINITIONS SCHEMA GRAPH",
        };
        let right_panel = GraphPanel {
            nodes: self.graph_nodes.clone(),
            edges: self.graph_edges.clone(),
            label: "INSTANCES RELATION GRAPH",
        };

        render_graph_panels(
            left_panel,
            right_panel,
            &bg_color,
            &fg_color,
            &border_color,
            &sidebar_bg,
            &active_accent,
            &theme_muted_foreground,
        )
    }
}

// ─── Rendering helpers ───────────────────────────────────────────────────────

/// Render a pair of graph panels (definitions + instances) as a split layout.
pub(crate) fn render_graph_panels(
    left_panel: GraphPanel,
    right_panel: GraphPanel,
    _bg_color: &gpui::Hsla,
    fg_color: &gpui::Hsla,
    border_color: &gpui::Hsla,
    sidebar_bg: &gpui::Hsla,
    active_accent: &gpui::Hsla,
    theme_muted_foreground: &Hsla,
) -> impl IntoElement {
    let border_color = *border_color;
    let sidebar_bg = *sidebar_bg;
    let fg_color = *fg_color;
    let active_accent = *active_accent;

    let left_content = render_single_panel(
        left_panel,
        &border_color,
        &sidebar_bg,
        &fg_color,
        &active_accent,
        theme_muted_foreground,
    );
    let right_content = render_single_panel(
        right_panel,
        &border_color,
        &sidebar_bg,
        &fg_color,
        &active_accent,
        theme_muted_foreground,
    );

    gpui_component::resizable::h_resizable("graph-panels-group")
        .child(gpui_component::resizable::resizable_panel().child(left_content))
        .child(gpui_component::resizable::resizable_panel().child(right_content))
}

fn render_single_panel(
    panel: GraphPanel,
    border_color: &Hsla,
    sidebar_bg: &Hsla,
    fg_color: &Hsla,
    active_accent: &Hsla,
    _theme_muted_foreground: &Hsla,
) -> gpui::Div {
    let border_color = *border_color;
    let sidebar_bg = *sidebar_bg;
    let fg_color = *fg_color;
    let active_accent = *active_accent;
    let nodes = panel.nodes;
    let edges = panel.edges;
    let label = panel.label;

    // Canvas that draws edges as paths (needs owned data via clone)
    let edge_data = edges.clone();
    let node_ref = nodes.clone();
    let canvas = gpui::canvas(
        move |_, _, _| {},
        move |bounds, _, window, _| {
            for edge in &edge_data {
                let src_idx = edge.source;
                let tgt_idx = edge.target;
                if src_idx < node_ref.len() && tgt_idx < node_ref.len() {
                    let x1 = node_ref[src_idx].x;
                    let y1 = node_ref[src_idx].y;
                    let x2 = node_ref[tgt_idx].x;
                    let y2 = node_ref[tgt_idx].y;
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let dist = (dx * dx + dy * dy).sqrt();

                    let p1 = gpui::point(
                        bounds.origin.x + gpui::px(x1),
                        bounds.origin.y + gpui::px(y1),
                    );
                    let p2 = gpui::point(
                        bounds.origin.x + gpui::px(x2),
                        bounds.origin.y + gpui::px(y2),
                    );

                    let mut builder = gpui::PathBuilder::stroke(gpui::px(2.0));
                    builder.move_to(p1);
                    builder.line_to(p2);
                    if let Ok(path) = builder.build() {
                        window.paint_path(path, border_color);
                    }

                    if dist > 0.001 {
                        let ux = dx / dist;
                        let uy = dy / dist;
                        let px = -uy;
                        let py = ux;

                        let draw_arrowhead = |window: &mut gpui::Window, tip_x: f32, tip_y: f32, base_x: f32, base_y: f32| {
                            let left_x = base_x + 6.0 * px;
                            let left_y = base_y + 6.0 * py;
                            let right_x = base_x - 6.0 * px;
                            let right_y = base_y - 6.0 * py;

                            let mut arrow_builder = gpui::PathBuilder::stroke(gpui::px(2.0));
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
                            let tip_x = x2 - 26.0 * ux;
                            let tip_y = y2 - 26.0 * uy;
                            let base_x = tip_x - 10.0 * ux;
                            let base_y = tip_y - 10.0 * uy;
                            draw_arrowhead(window, tip_x, tip_y, base_x, base_y);
                        }
                        if edge.label == "<-" || edge.label == "<->" {
                            let tip_x = x1 + 26.0 * ux;
                            let tip_y = y1 + 26.0 * uy;
                            let base_x = tip_x + 10.0 * ux;
                            let base_y = tip_y + 10.0 * uy;
                            draw_arrowhead(window, tip_x, tip_y, base_x, base_y);
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
    for (idx, node) in nodes.iter().enumerate() {
        let x = node.x;
        let y = node.y;

        let initials = if node.name.len() > 3 {
            node.name.chars().take(2).collect::<String>().to_uppercase()
        } else {
            node.name.clone()
        };

        let color = node.color;

        let node_div = gpui::div()
            .id(format!("{}_{}", label.to_lowercase(), idx))
            .absolute()
            .left(gpui::px(x - 24.))
            .top(gpui::px(y - 24.))
            .w(gpui::px(48.))
            .h(gpui::px(48.))
            .rounded_full()
            .bg(color)
            .flex()
            .items_center()
            .justify_center()
            .shadow_md()
            .border(gpui::px(2.))
            .border_color(border_color)
            .hover(|s| s.border_color(active_accent))
            .child(
                gpui::div()
                    .text_color(fg_color)
                    .text_size(gpui::px(11.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(initials),
            )
            .child(
                gpui::div()
                    .absolute()
                    .top(gpui::px(52.))
                    .w(gpui::px(120.))
                    .left(gpui::px(-36.))
                    .flex()
                    .justify_center()
                    .child(
                        gpui::div()
                            .bg(sidebar_bg.opacity(0.9))
                            .rounded(gpui::px(4.))
                            .px_2()
                            .py_0p5()
                            .text_size(gpui::px(10.))
                            .text_color(fg_color)
                            .border(gpui::px(1.))
                            .border_color(border_color)
                            .child(node.name.clone()),
                    ),
            );

        node_elements.push(node_div);
    }

    div()
        .flex_1()
        .h_full()
        .border_r(gpui::px(1.))
        .border_color(border_color)
        .flex()
        .flex_col()
        .child(
            gpui::div()
                .p_2()
                .bg(sidebar_bg)
                .border_b(gpui::px(1.))
                .border_color(border_color)
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .child(label),
        )
        .child(
            gpui::div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .bg(sidebar_bg)
                .child(
                    gpui::div()
                        .w(gpui::px(500.))
                        .h(gpui::px(500.))
                        .relative()
                        .child(canvas)
                        .children(node_elements),
                ),
        )
}
