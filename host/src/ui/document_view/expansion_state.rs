use gpui::prelude::*;
use gpui::EventEmitter;
use std::collections::HashSet;

/// Global expansion tracking entity — holds all expanded block offsets.
#[derive(Default)]
pub(crate) struct ExpandedBlocks {
    pub expanded: HashSet<usize>,
}

impl ExpandedBlocks {
    pub fn toggle(&mut self, offset: usize) {
        if self.expanded.contains(&offset) {
            self.expanded.remove(&offset);
        } else {
            self.expanded.insert(offset);
        }
    }

    pub fn is_expanded(&self, offset: usize) -> bool {
        self.expanded.contains(&offset)
    }
}

impl EventEmitter<ExpandedBlocksEvent> for ExpandedBlocks {}

#[derive(Clone, PartialEq)]
pub(crate) enum ExpandedBlocksEvent {
    Toggled { offset: usize },
}

/// Per-block toggle state — each interactive block gets its own Entity<ToggleState>.
pub(crate) struct ToggleState {
    pub is_expanded: bool,
}

impl ToggleState {
    pub fn new(is_expanded: bool) -> Self {
        Self { is_expanded }
    }

    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
    }
}

impl EventEmitter<ToggleEvent> for ToggleState {}

#[derive(Clone, PartialEq)]
pub(crate) enum ToggleEvent {
    Toggled { is_expanded: bool },
}
