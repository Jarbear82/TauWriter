use crate::graph_sim::{HubgsDefinition, HubgsInstance, HubgsLink, InstanceLink};
use gpui::SharedString;

pub(crate) fn parse_hubgs(
    content: &str,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let (raw_defs, raw_insts) = tauwriter_hubgs::parse_hubgs(content)?;

    let defs = raw_defs
        .into_iter()
        .map(|d| HubgsDefinition {
            name: SharedString::from(d.name),
            links: d
                .links
                .into_iter()
                .map(|l| HubgsLink {
                    name: SharedString::from(l.name),
                    arrow: SharedString::from(l.arrow),
                    target: SharedString::from(l.target),
                    multiplicity: SharedString::from(l.multiplicity),
                })
                .collect(),
            parents: d.parents.into_iter().map(SharedString::from).collect(),
        })
        .collect();

    let instances = raw_insts
        .into_iter()
        .map(|i| HubgsInstance {
            id: SharedString::from(i.id),
            type_name: SharedString::from(i.type_name),
            name: SharedString::from(i.name),
            theme_color: i.theme_color,
            links: i
                .links
                .into_iter()
                .map(|l| InstanceLink {
                    relation: SharedString::from(l.relation),
                    target: SharedString::from(l.target),
                })
                .collect(),
        })
        .collect();

    Ok((defs, instances))
}
