//! Graph pane rendering — definitions/instances relation graphs.
//!
//! Extracted from `ui/mod.rs` to eliminate ~90 lines of near-duplicate rendering logic
//! between the definitions and instances graph panels, and to reduce `mod.rs` file length.
//! [user-review: split required] See task ticket for splitting rationale.

use gpui::{div, prelude::*, Hsla};

use super::super::graph_sim::GraphNode;

/// A single graph panel containing nodes, edges, and a label.
pub(crate) struct GraphPanel {
    pub(crate) nodes: Vec<GraphNode>,
    pub(crate) edges: Vec<(usize, usize, String)>,
    pub(crate) label: &'static str,
}

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
            for &(src_idx, tgt_idx, _) in &edge_data {
                if src_idx < node_ref.len() && tgt_idx < node_ref.len() {
                    let p1 = gpui::point(
                        bounds.origin.x + gpui::px(node_ref[src_idx].x),
                        bounds.origin.y + gpui::px(node_ref[src_idx].y),
                    );
                    let p2 = gpui::point(
                        bounds.origin.x + gpui::px(node_ref[tgt_idx].x),
                        bounds.origin.y + gpui::px(node_ref[tgt_idx].y),
                    );

                    let mut builder = gpui::PathBuilder::stroke(gpui::px(2.0));
                    builder.move_to(p1);
                    builder.line_to(p2);
                    if let Ok(path) = builder.build() {
                        window.paint_path(path, border_color);
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
