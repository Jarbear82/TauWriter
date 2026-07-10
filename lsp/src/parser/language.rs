// Tree-sitter language getters linked from the compiled grammar shared library.
//
// # Safety
//
// These functions are `extern "C"` bindings to C symbols inside the compiled grammar
// shared library (e.g., `libtree_sitter_hubgs.so`).  Each function returns a `Language` —
// effectively a raw pointer to an immutable C struct that describes the grammar's token
// and state machine tables.
//
// The caller must uphold the following invariants:
//
// 1. **Grammar loaded**: The shared library containing the target symbol must be mapped
//    into the process address space (typically by the Rust FFI build script at link time).
//    Calling the getter before the library is loaded is UB.
//
// 2. **Grammar not unloaded**: The shared library must remain loaded for the lifetime of
//    every `Language` value obtained from it. Unloading the library (e.g. via `std::env::remove_var`
//    tricks or dlclose on a direct handle) while callers still hold `Language` values is UB.
//
// 3. **Grammar not mutated**: The grammar struct returned by these getters is immutable.
//    Tree-sitter's C API guarantees that parsing, querying, and other operations do not
//    modify the `Language` object itself — they only read its static tables and produce
//    a mutable `Tree` / `Parser` on the heap.  No valid code path in this crate mutates
//    or writes to a `Language` pointer.
//
// 4. **Zero-sized FFI type**: `tree_sitter::Language` is a `#[repr(transparent)]` wrapper
//    around a raw pointer (`*const c_void`).  Copying and dropping it is safe as long as
//    the underlying grammar struct is still loaded (invariant 2).
//
// # Correctness of callers
//
// All known callers obtain a `Language`, pass it immediately to `Parser::set_language()`,
// and then use the resulting parser tree synchronously within the same stack frame.  None
// of them store a `Language` across an await point, send it between threads, or retain it
// beyond the parsing call.  This satisfies all four invariants above.

extern "C" {
    pub fn tree_sitter_hubgs() -> tree_sitter::Language;
    pub fn tree_sitter_twxml() -> tree_sitter::Language;
}

/// Lookup a tree-sitter language by file extension name.
///
/// # Safety
///
/// Returns `None` when the requested name doesn't match a known grammar.  When it returns
/// `Some(lang)`, the caller is responsible for ensuring the FFI invariants documented on
/// this module hold while `lang` is used.
pub fn get_language(name: &str) -> Option<tree_sitter::Language> {
    match name {
        "hubgs" => Some(unsafe { tree_sitter_hubgs() }),
        "twxml" => Some(unsafe { tree_sitter_twxml() }),
        _ => None,
    }
}
