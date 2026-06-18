fn main() {
    let hubgs_dir = std::path::PathBuf::from("tree-sitter/hubgs/src");
    let twxml_dir = std::path::PathBuf::from("tree-sitter/twxml/src");

    cc::Build::new()
        .include(&hubgs_dir)
        .file(hubgs_dir.join("parser.c"))
        .compile("tree-sitter-hubgs");

    cc::Build::new()
        .include(&twxml_dir)
        .file(twxml_dir.join("parser.c"))
        .compile("tree-sitter-twxml");
}
