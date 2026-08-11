mod features;
mod hubgs;
pub mod language;
mod twxml;

pub use features::*;
// Re-export all public items from submodules for flat crate::parser access
pub use hubgs::{get_hub_type_at_position, is_in_hub_definition, parse_hubgs_ast};
pub use hubgs::{get_hubgs_completion_context, HubgsCompletionContext};
pub use twxml::find_review_at_position;
pub use twxml::{get_all_twxml_tags, parse_twxml_ast};
pub use twxml::{get_twxml_completion_context, TwxmlCompletionContext};
// Attribute parsing utility shared across TWXML consumers.
pub use twxml::{get_all_attributes, get_attribute};

// Re-export tree-sitter language bindings from the canonical module.
pub use language::{get_language, tree_sitter_hubgs, tree_sitter_twxml};

// Thin wrappers for internal callers that prefer typed helpers over string lookup.
pub fn get_hubgs_language() -> Option<tree_sitter::Language> {
    get_language("hubgs")
}
pub fn get_twxml_language() -> Option<tree_sitter::Language> {
    get_language("twxml")
}

pub fn ts_range_to_lsp(range: tree_sitter::Range) -> crate::db::LspRange {
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
