fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR should have a workspace root parent directory");

    let twxml_parser_c = workspace_root.join("extension/languages/twxml/src/parser.c");
    assert!(
        twxml_parser_c.exists(),
        "TWXML grammar source not found at {}",
        twxml_parser_c.display()
    );
    let mut twxml_config = cc::Build::new();
    twxml_config.include(workspace_root.join("extension/languages/twxml/src"));
    twxml_config.file(&twxml_parser_c);
    twxml_config.warnings(false);
    twxml_config.compile("tree-sitter-twxml");
    println!("cargo:rerun-if-changed={}", twxml_parser_c.display());
}
