//! Node sizing — computes real on-screen dimensions for graph nodes via GPUI's
//! text shaping system, instead of guessing from content length.
//!
//! Kept separate from the rest of `graph_sim`, which is pure math and must stay
//! unit-testable without a live window. This module is the only place that
//! touches `Window`/`App`.

use gpui::{px, Font, FontStyle, FontWeight, TextRun as GpuiTextRun, Window};

/// Visual constants shared between measurement and the actual node renderer in
/// `ui/graph_pane/render.rs`. If you touch padding/font sizes there, touch them
/// here too — these two places must never drift apart.
pub(crate) const NODE_H_PADDING: f32 = 32.0; // px_4 left + right
pub(crate) const NODE_HEADER_V_PADDING: f32 = 16.0; // py_2 top + bottom
pub(crate) const NODE_BODY_V_PADDING: f32 = 8.0; // py_1 top + bottom
pub(crate) const NODE_ATTR_LINE_HEIGHT: f32 = 18.0;
pub(crate) const NODE_NAME_FONT_SIZE: f32 = 13.0;
pub(crate) const NODE_TYPE_FONT_SIZE: f32 = 10.0;
pub(crate) const NODE_ATTR_FONT_SIZE: f32 = 11.0;
pub(crate) const NODE_HEADER_GAP: f32 = 4.0;
pub(crate) const NODE_MIN_WIDTH: f32 = 40.0;

/// The raw text content a node needs measured — mirrors exactly what
/// `render.rs` paints, so measured size == painted size.
pub(crate) struct NodeContent<'a> {
    pub(crate) name: &'a str,
    pub(crate) type_name: &'a str,
    pub(crate) attributes: &'a [String],
}


fn measure_line_width(window: &mut Window, text: &str, font_size: f32, bold: bool) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let mut font: Font = window.text_style().font();
    font.weight = if bold {
        FontWeight::BOLD
    } else {
        FontWeight::NORMAL
    };
    font.style = FontStyle::Normal;

    let run = GpuiTextRun {
        len: text.len(),
        font,
        color: gpui::black(), // color doesn't affect shaping metrics
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let shaped =
        window
            .text_system()
            .shape_line(text.to_string().into(), px(font_size), &[run], None);
    f32::from(shaped.width)
}

/// Build a real GPUI-backed `NodeSizer` bound to the given window. This is what
/// production code passes into the graph_sim layout/physics entry points.
pub(crate) fn gpui_text_sizer<'w>(
    window: &'w mut Window,
) -> impl FnMut(NodeContent) -> (f32, f32) + 'w {
    move |content: NodeContent| {
        let name_w = measure_line_width(window, content.name, NODE_NAME_FONT_SIZE, true);
        let type_label = format!("«{}»", content.type_name);
        let type_w = measure_line_width(window, &type_label, NODE_TYPE_FONT_SIZE, false);

        let mut max_attr_w: f32 = 0.0;
        for attr in content.attributes {
            max_attr_w =
                max_attr_w.max(measure_line_width(window, attr, NODE_ATTR_FONT_SIZE, false));
        }

        let width = (name_w.max(type_w).max(max_attr_w) + NODE_H_PADDING).max(NODE_MIN_WIDTH);

        let header_h =
            NODE_NAME_FONT_SIZE + NODE_HEADER_GAP + NODE_TYPE_FONT_SIZE + NODE_HEADER_V_PADDING;
        let body_h = if content.attributes.is_empty() {
            0.0
        } else {
            content.attributes.len() as f32 * NODE_ATTR_LINE_HEIGHT + NODE_BODY_V_PADDING
        };

        (width, header_h + body_h)
    }
}

/// Deterministic, GPUI-free sizer for unit tests — proportional to text length
/// so tests can still assert relative ordering (long names -> wider nodes)
/// without a live window.
#[cfg(test)]
pub(crate) fn fixed_test_sizer() -> impl FnMut(NodeContent) -> (f32, f32) {
    move |content: NodeContent| {
        let w = (content.name.len().max(content.type_name.len()) as f32 * 7.0 + NODE_H_PADDING)
            .max(NODE_MIN_WIDTH);
        let h = NODE_HEADER_V_PADDING
            + 20.0
            + if content.attributes.is_empty() {
                0.0
            } else {
                content.attributes.len() as f32 * NODE_ATTR_LINE_HEIGHT + NODE_BODY_V_PADDING
            };
        (w, h)
    }
}
