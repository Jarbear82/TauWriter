//! Raw FFI bindings for tree-sitter grammar libraries.
//! All `unsafe extern "C"` declarations live here to minimize the unsafe surface area.

/// Load a `tree_sitter::Language` from a raw pointer, asserting layout compatibility.
pub(crate) fn load_ts_language(
    ptr: *const std::ffi::c_void,
) -> anyhow::Result<tree_sitter::Language> {
    if ptr.is_null() {
        anyhow::bail!("tree-sitter language pointer is NULL");
    }
    // `tree_sitter::Language` is a thin newtype wrapper around a raw
    // pointer in tree-sitter 0.26; assert that invariant explicitly so a
    // future dependency bump fails loudly here instead of silently producing UB.
    const _: () = assert!(
        std::mem::size_of::<*const std::ffi::c_void>()
            == std::mem::size_of::<tree_sitter::Language>(),
        "tree_sitter::Language layout changed — transmute is no longer valid"
    );
    // SAFETY: size assertion above guarantees layout compatibility for this
    // tree-sitter version; ptr is verified non-null above and is expected
    // to be a `TSLanguage*` returned by a `tree_sitter_<lang>()` FFI symbol.
    Ok(unsafe { std::mem::transmute::<*const std::ffi::c_void, tree_sitter::Language>(ptr) })
}

unsafe extern "C" {
    /// Safety: Returns a static, read-only TSLanguage pointer for the TWXML grammar.
    fn tree_sitter_twxml() -> *const std::ffi::c_void;

    /// Safety: Returns a static, read-only TSLanguage pointer for the HubGS grammar.
    fn tree_sitter_hubgs() -> *const std::ffi::c_void;
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

/// Load the HubGS tree-sitter language. Returns `None` if the symbol is missing.
pub(crate) fn load_hubgs_language() -> Option<tree_sitter::Language> {
    let ptr = unsafe { tree_sitter_hubgs() };
    if ptr.is_null() {
        None
    } else {
        load_ts_language(ptr).ok()
    }
}
