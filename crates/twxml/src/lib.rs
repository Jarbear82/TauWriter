//! `tauwriter-twxml` — Standalone TWXML document parser, block DOM types, outline extraction, and tree-sitter AST queries.

pub mod markdown;
pub mod outline;
pub mod parse;
pub mod ts_query;
pub mod types;

pub use markdown::blocks_to_markdown;
pub use outline::parse_document_outline;
pub use parse::{load_and_parse_twxml, parse_twxml, parse_twxml_internal};
pub use ts_query::{get_all_attributes, get_all_twxml_tags, get_attribute, parse_hub_references};
pub use types::{Block, HubReferenceInfo, ListItem, OutlineNode, TextRun, TwxmlTagInfo};

unsafe extern "C" {
    fn tree_sitter_twxml() -> *const std::ffi::c_void;
}

/// Load the TWXML tree-sitter language. Returns `None` if the symbol is missing or NULL.
pub fn load_twxml_language() -> Option<tree_sitter::Language> {
    let ptr = unsafe { tree_sitter_twxml() };
    if ptr.is_null() {
        None
    } else {
        unsafe { Some(tree_sitter::Language::from_raw(ptr.cast())) }
    }
}
