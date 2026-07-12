fn main() {
    let mut config = cc::Build::new();
    config.include("../extension/languages/twxml/src");
    config.file("../extension/languages/twxml/src/parser.c");
    // Disable compiler warnings
    config.warnings(false);
    config.compile("tree-sitter-twxml");

    println!("cargo:rerun-if-changed=../extension/languages/twxml/src/parser.c");
}
