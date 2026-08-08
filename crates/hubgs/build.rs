fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("CARGO_MANIFEST_DIR should have a workspace root parent directory");

    let hubgs_parser_c = workspace_root.join("extension/languages/hubgs/src/parser.c");
    assert!(
        hubgs_parser_c.exists(),
        "HubGS grammar source not found at {}",
        hubgs_parser_c.display()
    );
    let mut hubgs_config = cc::Build::new();
    hubgs_config.include(workspace_root.join("extension/languages/hubgs/src"));
    hubgs_config.file(&hubgs_parser_c);
    hubgs_config.warnings(false);
    hubgs_config.compile("tree-sitter-hubgs");
    println!("cargo:rerun-if-changed={}", hubgs_parser_c.display());
}
