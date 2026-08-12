//! Graph pane rendering — definitions/instances relation graphs powered by graphene-rs.

mod data;
mod render;
mod state;

// Re-export the public API for consumers in `ui/mod.rs` etc.
pub(crate) use data::GraphEvent;
pub(crate) use state::GraphPaneView;
