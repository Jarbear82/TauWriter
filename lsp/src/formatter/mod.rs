mod hubgs;
mod twxml;

use tree_sitter::Language;

extern "C" {
    pub(crate) fn tree_sitter_hubgs() -> Language;
    pub(crate) fn tree_sitter_twxml() -> Language;
}

pub fn format_source(contents: &str, file_type: &str) -> String {
    match file_type {
        "twxml" => twxml::format_twxml(contents),
        "hubgs" => hubgs::format_hubgs(contents),
        _ => contents.to_string(),
    }
}
