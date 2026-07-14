use gpui::{div, AnyElement, Context, Entity};
use gpui::{prelude::*, SharedString};

use super::expansion_state::{ToggleEvent, ToggleState};

/// Closure type for lazy body rendering (no Send bound — AnyElement isn't Send).
type ChildrenFn = Box<dyn FnOnce() -> Vec<AnyElement>>;

/// Reusable wrapper component for all collapsible block types.
/// Stores children as a builder closure to avoid storing AnyElement directly.
pub(crate) struct CollapsibleBlock {
    /// Whether the block starts collapsed.
    is_collapsed: bool,
    /// Label shown in the header row.
    header_label: SharedString,
    /// Left border color.
    border_color: gpui::Hsla,
    /// Background color of the container body.
    bg_color: gpui::Hsla,
    /// Children builder closure for lazy rendering.
    children_fn: Option<ChildrenFn>,
}

impl CollapsibleBlock {
    pub fn new(
        is_collapsed: bool,
        header_label: SharedString,
        border_color: gpui::Hsla,
        bg_color: gpui::Hsla,
    ) -> Self {
        Self {
            is_collapsed,
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

    /// Render this collapsible block into an AnyElement using the provided toggle entity.
    pub(crate) fn render_with_toggle(
        mut self,
        toggle: Entity<ToggleState>,
        cx: &mut Context<impl gpui::Render>,
    ) -> AnyElement {
        let is_expanded = toggle.read(cx).is_expanded;

        // Subscribe to child toggle changes so the parent re-renders on collapse/expand.
        let _sub = cx.subscribe(&toggle, |_this, _target, ev: &ToggleEvent, cx| match ev {
            ToggleEvent::Toggled { .. } => cx.notify(),
        });

        let header_label = self.header_label;
        let border_color = self.border_color;
        let bg_color = self.bg_color;
        let is_expanded_clone = is_expanded;

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
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                let toggle_clone = toggle.clone();
                let new_is_expanded = !is_expanded_clone;
                toggle_clone.update(cx, |ts, cx| {
                    ts.toggle();
                    cx.emit(ToggleEvent::Toggled {
                        is_expanded: new_is_expanded,
                    });
                });
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
