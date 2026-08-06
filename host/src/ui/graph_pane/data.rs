//! Data types for graph pane — pure structs and enums.

use gpui::SharedString;

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
