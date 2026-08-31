//! HubGS data types and parser/sizing integrations for TauWriter.
//! Physics and canvas rendering are delegated to graphene-rs (`graphene_core`, `graphene_layout`, `graphene_gpui`).

use gpui::SharedString;

#[cfg(test)]
mod graph_sim_tests;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HubgsLink {
    pub(crate) name: SharedString,
    pub(crate) arrow: SharedString,
    pub(crate) target: SharedString,
    pub(crate) multiplicity: SharedString,
}

pub(crate) struct HubgsDefinition {
    pub(crate) name: SharedString,
    pub(crate) links: Vec<HubgsLink>,
    #[allow(dead_code)]
    pub(crate) parents: Vec<SharedString>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InstanceLink {
    pub(crate) relation: SharedString,
    pub(crate) target: SharedString,
}

pub(crate) struct HubgsInstance {
    pub(crate) id: SharedString,
    pub(crate) type_name: SharedString,
    pub(crate) name: SharedString,
    pub(crate) theme_color: Option<u32>,
    pub(crate) links: Vec<InstanceLink>,
}

/// Recursively search a directory tree for the first `.hubgs` file.
pub(crate) fn find_any_hubgs(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    crate::utils::find_first_file(dir, Some("hubgs"))
}

pub(crate) mod hubgs_parser;
#[cfg(test)]
pub(crate) mod hubgs_parser_tests;
pub(crate) mod sizing;

/// Parse a HubGS file into definitions and instances.
pub(crate) fn parse_hubgs_file(
    path: &std::path::Path,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let content = std::fs::read_to_string(path)?;
    hubgs_parser::parse_hubgs(&content)
}
