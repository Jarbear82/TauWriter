/// Tree-sitter language getters delegated to shared parser crates.

pub fn tree_sitter_hubgs() -> tree_sitter::Language {
    tauwriter_hubgs::load_hubgs_language().expect("tree_sitter_hubgs symbol linked")
}

pub fn tree_sitter_twxml() -> tree_sitter::Language {
    tauwriter_twxml::load_twxml_language().expect("tree_sitter_twxml symbol linked")
}

pub fn get_language(name: &str) -> Option<tree_sitter::Language> {
    match name {
        "hubgs" => tauwriter_hubgs::load_hubgs_language(),
        "twxml" => tauwriter_twxml::load_twxml_language(),
        _ => None,
    }
}
