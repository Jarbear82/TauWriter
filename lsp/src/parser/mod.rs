mod features;
mod hubgs;
mod twxml;

pub use features::*;
pub use hubgs::*;
pub use twxml::*;

// Actual C-linked languages
extern "C" {
    pub fn tree_sitter_hubgs() -> tree_sitter::Language;
    pub fn tree_sitter_twxml() -> tree_sitter::Language;
}

pub(crate) fn ts_range_to_lsp(range: tree_sitter::Range) -> crate::db::LspRange {
    crate::db::LspRange {
        start: crate::db::LspPosition {
            line: range.start_point.row as u32,
            character: range.start_point.column as u32,
        },
        end: crate::db::LspPosition {
            line: range.end_point.row as u32,
            character: range.end_point.column as u32,
        },
    }
}
