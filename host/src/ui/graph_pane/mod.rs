//! Graph pane rendering — definitions/instances relation graphs.
//!
//! Split from a single 1398-line file into focused submodules:
//! - data.rs  : pure data types (GraphPanel, GraphEvent, LayoutMode)
//! - state.rs : GraphPaneView struct + business logic
//! - render.rs: Render/RenderOnce impls + layout helpers

mod data;
mod render;
mod state;

// Re-export the public API for consumers in `ui/mod.rs` etc.
pub(crate) use data::{GraphEvent, GraphPanel, LayoutMode};
pub(crate) use state::GraphPaneView;
