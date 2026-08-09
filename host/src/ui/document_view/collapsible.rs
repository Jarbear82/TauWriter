use gpui::prelude::*;
use gpui::{div, AnyElement, Context, Entity};

use super::expansion_state::ExpandedBlocks;

/// Closure type for lazy body rendering (no Send bound — AnyElement isn't Send).
type ChildrenFn = Box<dyn FnOnce() -> Vec<AnyElement>>;

/// Reusable wrapper component for all collapsible block types.
/// Reads/writes expansion state from a shared ExpandedBlocks registry keyed by document offset.
pub(crate) struct CollapsibleBlock {
    /// Header label shown in the toggle row.
    header_label: String,
    /// Left border color.
    border_color: gpui::Hsla,
    /// Background color of the container body.
    bg_color: gpui::Hsla,
    /// Children builder closure for lazy rendering.
    children_fn: Option<ChildrenFn>,
}

impl CollapsibleBlock {
    pub fn new(header_label: String, border_color: gpui::Hsla, bg_color: gpui::Hsla) -> Self {
        Self {
            header_label,
            border_color,
            bg_color,
            children_fn: None,
        }
    }

    /// Attach the children rendered when this block is expanded.
    pub fn with_body(mut self, children: Vec<AnyElement>) -> Self {
        self.children_fn = Some(Box::new(move || children));
        self
    }

    /// Render this collapsible block into an AnyElement using the shared ExpandedBlocks registry.
    pub(crate) fn render(
        mut self,
        toggle_offset: usize,
        expanded_blocks: Entity<ExpandedBlocks>,
        cx: &mut Context<impl gpui::Render>,
    ) -> AnyElement {
        let is_expanded = expanded_blocks.read(cx).expanded.contains(&toggle_offset);

        // Subscribe to registry changes so the parent re-renders on collapse/expand.
        cx.subscribe(&expanded_blocks, |_this, _target, _ev: &(), cx| {
            cx.notify();
        })
        .detach();

        let header_label = self.header_label.clone();
        let border_color = self.border_color;
        let bg_color = self.bg_color;
        let toggle_key = toggle_offset;
        let blocks_key = expanded_blocks.clone();

        let mut container = div()
            .id("collapsible")
            .w_full()
            .mb_4()
            .border_l_4()
            .border_color(border_color)
            .bg(bg_color);

        // Header row — always visible, serves as the collapse/expand toggle handle.
        let header = div()
            .id("collapse-toggle")
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_1()
            .bg(gpui_component::Theme::global(cx).accent.opacity(0.3))
            .hover(|s| s.bg(gpui_component::Theme::global(cx).accent.opacity(0.5)))
            .cursor_pointer()
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                blocks_key.update(cx, |eb, _cx| eb.toggle(toggle_key));
                // No explicit notify needed — EventEmitter<()> on ExpandedBlocks triggers
                // all subscriber re-renders implicitly via .update().
            })
            .child(if is_expanded { "▼" } else { "▶" })
            .child(
                div()
                    .italic()
                    .text_xs()
                    .text_color(gpui_component::Theme::global(cx).muted_foreground)
                    .child(header_label),
            );

        container = container.child(header);

        if is_expanded {
            if let Some(children_fn) = self.children_fn.take() {
                let body_children = children_fn();
                container = container.child(div().p_4().flex().flex_wrap().children(body_children));
            }
        }

        container.into_any_element()
    }
}
