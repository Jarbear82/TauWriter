use gpui::{prelude::*, EventEmitter};
use std::collections::HashSet;

/// Central expansion registry — tracks which block offsets are expanded.
#[derive(Default)]
pub(crate) struct ExpandedBlocks {
    pub expanded: HashSet<usize>,
}

impl ExpandedBlocks {
    /// Toggle the expansion state for a given document offset.
    pub fn toggle(&mut self, offset: usize) {
        if self.expanded.contains(&offset) {
            self.expanded.remove(&offset);
        } else {
            self.expanded.insert(offset);
        }
    }

    /// Check whether the block at `offset` is currently expanded.
    pub fn is_expanded(&self, offset: usize) -> bool {
        self.expanded.contains(&offset)
    }
}

impl EventEmitter<()> for ExpandedBlocks {}
