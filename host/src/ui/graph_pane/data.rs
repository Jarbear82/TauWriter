//! Data types for graph pane — pure structs and enums.

use gpui::{prelude::*, Entity, SharedString, Window};

/// A single graph panel containing nodes, edges, and a label.
#[derive(IntoElement)]
pub(crate) struct GraphPanel {
    pub(crate) nodes: Vec<crate::graph_sim::GraphNode>,
    pub(crate) edges: Vec<crate::graph_sim::GraphEdge>,
    pub(crate) label: SharedString,
    pub(crate) selected_hub_id: Option<SharedString>,
    pub(crate) on_node_click: Option<
        std::sync::Arc<dyn Fn(SharedString, &mut Window, &mut gpui::App) + Send + Sync + 'static>,
    >,
    pub(crate) on_node_drag_start: Option<
        std::sync::Arc<
            dyn Fn(SharedString, gpui::Point<gpui::Pixels>, &mut Window, &mut gpui::App)
                + Send
                + Sync
                + 'static,
        >,
    >,
    pub(crate) on_mouse_move: Option<
        std::sync::Arc<
            dyn Fn(gpui::Point<gpui::Pixels>, &mut Window, &mut gpui::App) + Send + Sync + 'static,
        >,
    >,
    pub(crate) on_mouse_up:
        Option<std::sync::Arc<dyn Fn(&mut Window, &mut gpui::App) + Send + Sync + 'static>>,
    pub(crate) layout_selector: Option<gpui::AnyElement>,
    // Background pan trigger (fires when no node grabs mouse down)
    pub(crate) on_bg_mouse_down: Option<
        std::sync::Arc<
            dyn Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + Send + Sync + 'static,
        >,
    >,
    // Scroll wheel zoom trigger
    pub(crate) on_scroll_wheel: Option<
        std::sync::Arc<
            dyn Fn(&gpui::ScrollWheelEvent, &mut Window, &mut gpui::App) + Send + Sync + 'static,
        >,
    >,
    // Floating zoom controls callbacks
    pub(crate) on_zoom_in:
        Option<std::sync::Arc<dyn Fn(&mut Window, &mut gpui::App) + Send + Sync>>,
    pub(crate) on_zoom_out:
        Option<std::sync::Arc<dyn Fn(&mut Window, &mut gpui::App) + Send + Sync>>,
    pub(crate) on_fit_view:
        Option<std::sync::Arc<dyn Fn(&mut Window, &mut gpui::App) + Send + Sync>>,
    // Camera state for infinite viewport
    pub(crate) camera_offset_x: f32,
    pub(crate) camera_offset_y: f32,
    pub(crate) zoom: f32,
    // Callback to report pane content bounds back to the parent view
    pub(crate) on_bounds_changed:
        Option<std::sync::Arc<dyn Fn(f32, f32, &mut gpui::Window, &mut gpui::App) + Send + Sync>>,
}

/// Events emitted by the graph pane viewer.
#[derive(Clone)]
pub(crate) enum GraphEvent {
    NodeClicked(SharedString),
    RunLayout,
}

/// Pending layout mode for deferred execution.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum LayoutMode {
    #[default]
    None,
    RunLayout(super::super::LayoutType),
}
