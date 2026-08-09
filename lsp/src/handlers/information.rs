//! Thin re-export module for hover and code-action handlers.
//!
//! Public API surface: `hover()`, `code_action()`, `format_hub_value()`.

pub use crate::handlers::code_actions as code_action;
pub use crate::handlers::hover::{format_hub_value, hover, hover_impl};
pub use crate::handlers::markdown::MarkdownContent;

#[cfg(test)]
#[path = "information_tests.rs"]
mod information_test_module;
