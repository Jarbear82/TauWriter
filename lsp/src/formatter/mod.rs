mod expression;
mod hubgs;
mod twxml;

use crate::parser::language::{tree_sitter_hubgs, tree_sitter_twxml};

pub fn format_source(contents: &str, file_type: &str) -> String {
    match file_type {
        "twxml" => twxml::format_twxml(contents),
        "hubgs" => hubgs::format_hubgs(contents),
        _ => contents.to_string(),
    }
}
