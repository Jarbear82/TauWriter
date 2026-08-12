//! Raw FFI bindings for tree-sitter grammar libraries.
//! All `unsafe extern "C"` declarations live here to minimize the unsafe surface area.

/// Load a `tree_sitter::Language` from a raw pointer, asserting layout compatibility.
pub(crate) fn load_ts_language(
    ptr: *const std::ffi::c_void,
) -> anyhow::Result<tree_sitter::Language> {
    if ptr.is_null() {
        anyhow::bail!("tree-sitter language pointer is NULL");
    }
    Ok(unsafe { tree_sitter::Language::from_raw(ptr.cast()) })
}

unsafe extern "C" {
    /// Safety: Returns a static, read-only TSLanguage pointer for the TWXML grammar.
    fn tree_sitter_twxml() -> *const std::ffi::c_void;
}

/// Load the TWXML tree-sitter language. Returns `None` if the symbol is missing.
pub(crate) fn load_twxml_language() -> Option<tree_sitter::Language> {
    let ptr = unsafe { tree_sitter_twxml() };
    if ptr.is_null() {
        None
    } else {
        load_ts_language(ptr).ok()
    }
}
