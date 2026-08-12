//! Shared utilities across TauWriter crates.

/// Removes surrounding single or double quotes from a string slice and returns a String.
pub fn unquote_string(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

/// Safely extracts text of a Tree-Sitter node given the source byte slice.
pub fn extract_node_text(node: tree_sitter::Node, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(|s| s.to_string())
}

/// Utility for safely initializing a Tree-Sitter language from a raw FFI pointer.
///
/// # Safety
/// The caller must ensure `ptr` is a valid pointer to a Tree-Sitter `TSLanguage` structure or null.
pub unsafe fn load_tree_sitter_language(ptr: *const ()) -> Option<tree_sitter::Language> {
    if ptr.is_null() {
        None
    } else {
        Some(tree_sitter::Language::from_raw(ptr.cast()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unquote_string() {
        assert_eq!(unquote_string("\"hello\""), "hello");
        assert_eq!(unquote_string("'world'"), "world");
        assert_eq!(unquote_string("no_quotes"), "no_quotes");
        assert_eq!(unquote_string("\"\""), "");
    }
}
