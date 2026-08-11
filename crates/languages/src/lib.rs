#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    HubGS,
    TWXML,
}

impl LanguageId {
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("hubgs") => Some(Self::HubGS),
            Some("twxml") => Some(Self::TWXML),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::HubGS => "HubGS",
            Self::TWXML => "TWXML",
        }
    }
}

pub fn hubgs_language() -> Option<tree_sitter::Language> {
    Some(tauwriter_analysis::parser::tree_sitter_hubgs())
}

pub fn twxml_language() -> Option<tree_sitter::Language> {
    Some(tauwriter_analysis::parser::tree_sitter_twxml())
}
