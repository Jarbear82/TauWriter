use crate::graph_sim::{HubgsDefinition, HubgsInstance};

#[derive(Default)]
struct ParserState {
    in_definitions: bool,
    in_fields: bool,
    in_hubs: bool,
    in_instances: bool,
}

pub(crate) fn parse_hubgs(content: &str) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let mut definitions = Vec::new();
    let mut instances = Vec::new();

    let mut state = ParserState::default();

    let mut current_hub_name = None;
    let mut current_hub_links = Vec::new();

    let mut current_id = None;
    let mut current_type = None;
    let mut current_name = String::new();
    let mut current_color = None;
    let mut current_links = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Block boundaries FSM transitions
        if update_state(&mut state, trimmed) {
            continue;
        }

        // Parse definitions
        if state.in_hubs {
            parse_definition_line(
                trimmed,
                &mut current_hub_name,
                &mut current_hub_links,
                &mut definitions,
            )?;
        }

        // Parse instances
        if state.in_instances {
            parse_instance_line(
                trimmed,
                &mut current_id,
                &mut current_type,
                &mut current_name,
                &mut current_color,
                &mut current_links,
                &mut instances,
            );
        }
    }

    Ok((definitions, instances))
}

fn update_state(state: &mut ParserState, trimmed: &str) -> bool {
    if trimmed.starts_with("DEFINITIONS [") {
        state.in_definitions = true;
        return true;
    }
    if state.in_definitions {
        if trimmed.starts_with("FIELDS [") {
            state.in_fields = true;
            return true;
        }
        if state.in_fields && (trimmed == "]" || trimmed == "],") {
            state.in_fields = false;
            return true;
        }
        if trimmed.starts_with("HUBS [") {
            state.in_hubs = true;
            return true;
        }
        if state.in_hubs && (trimmed == "]" || trimmed == "],") {
            state.in_hubs = false;
            return true;
        }
        if !state.in_fields && !state.in_hubs && (trimmed == "]" || trimmed == "],") {
            state.in_definitions = false;
            return true;
        }
    }

    if trimmed.starts_with("INSTANCES [") {
        state.in_instances = true;
        return true;
    }
    if state.in_instances && (trimmed == "]" || trimmed == "],") {
        state.in_instances = false;
        return true;
    }

    false
}

fn parse_definition_line(
    trimmed: &str,
    current_hub_name: &mut Option<String>,
    current_hub_links: &mut Vec<(String, String, String)>,
    definitions: &mut Vec<HubgsDefinition>,
) -> anyhow::Result<()> {
    if trimmed.ends_with('{') && !trimmed.contains(':') {
        let hub_name = trimmed.trim_end_matches('{').trim().to_string();
        *current_hub_name = Some(hub_name);
        current_hub_links.clear();
        return Ok(());
    }
    if trimmed == "}" || trimmed == "}," {
        if let Some(name) = current_hub_name.take() {
            definitions.push(HubgsDefinition {
                name,
                links: current_hub_links.clone(),
            });
        }
        return Ok(());
    }

    if trimmed.contains("ALLOWS [") {
        let (arrow_end, rel_name, arrow_str) = if let Some(double_arrow_idx) = trimmed.find("<->") {
            (double_arrow_idx + 3, trimmed[..double_arrow_idx].trim().to_string(), "<->".to_string())
        } else if let Some(arrow_idx) = trimmed.find("->") {
            (arrow_idx + 2, trimmed[..arrow_idx].trim().to_string(), "->".to_string())
        } else if let Some(rarrow_idx) = trimmed.find("<-") {
            (rarrow_idx + 2, trimmed[..rarrow_idx].trim().to_string(), "<-".to_string())
        } else if let Some(dash_idx) = trimmed.find(" - ") {
            (dash_idx + 3, trimmed[..dash_idx].trim().to_string(), "-".to_string())
        } else if let Some(dash_idx) = trimmed.find('-') {
            let before = &trimmed[..dash_idx];
            let after = &trimmed[dash_idx + 1..];
            if (before.ends_with(' ') || before.is_empty()) && (after.starts_with(' ') || after.starts_with('(')) {
                (dash_idx + 1, before.trim().to_string(), "-".to_string())
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        if let Some(allows_idx) = trimmed.find("ALLOWS [") {
            if allows_idx < arrow_end {
                anyhow::bail!("Invalid HubGS syntax: ALLOWS [ must be after arrow");
            }
            let middle = trimmed[arrow_end..allows_idx].trim();
            if !middle.starts_with('(') || !middle.ends_with(')') {
                anyhow::bail!("Invalid HubGS syntax: missing multiplicity bounds (e.g. (0..1)) in relationship: {}", trimmed);
            }
            let inner = &middle[1..middle.len() - 1];
            let parts: Vec<&str> = inner.split("..").collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid HubGS syntax: multiplicity must be formatted as 'min..max' (e.g. (0..1)) in relationship: {}", trimmed);
            }
            let min = parts[0].trim();
            let max = parts[1].trim();
            if min.is_empty() || max.is_empty() {
                anyhow::bail!("Invalid HubGS syntax: multiplicity bounds cannot be empty: {}", trimmed);
            }

            let target_part = &trimmed[allows_idx + 8..];
            if let Some(end_bracket) = target_part.find(']') {
                let target_hub = target_part[..end_bracket].trim().to_string();
                current_hub_links.push((rel_name, arrow_str, target_hub));
            } else {
                anyhow::bail!("Invalid HubGS syntax: missing closing bracket in target: {}", trimmed);
            }
        }
    }
    Ok(())
}

fn parse_instance_line(
    trimmed: &str,
    current_id: &mut Option<String>,
    current_type: &mut Option<String>,
    current_name: &mut String,
    current_color: &mut Option<u32>,
    current_links: &mut Vec<(String, String)>,
    instances: &mut Vec<HubgsInstance>,
) {
    if trimmed.contains(':') && trimmed.ends_with('{') {
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() >= 2 {
            let id = parts[0].trim().to_string();
            let type_part = parts[1].trim().trim_end_matches('{').trim().to_string();
            *current_id = Some(id);
            *current_type = Some(type_part);
            *current_name = String::new();
            *current_color = None;
            current_links.clear();
        }
        return;
    }

    if trimmed == "}" || trimmed == "}," {
        if let (Some(id), Some(type_name)) = (current_id.take(), current_type.take()) {
            let name = if current_name.is_empty() {
                id.clone()
            } else {
                current_name.clone()
            };
            instances.push(HubgsInstance {
                id,
                type_name,
                name,
                theme_color: *current_color,
                links: current_links.clone(),
            });
        }
        return;
    }

    if let Some(eq_idx) = trimmed.find('=') {
        let key = trimmed[..eq_idx].trim();
        let val_part = trimmed[eq_idx + 1..].trim().trim_end_matches(',').trim();

        if key == "name" {
            *current_name = val_part.trim_matches('"').to_string();
        } else if key == "theme_color" {
            let clean_val = val_part.trim_start_matches("0x").trim_start_matches("0X");
            if let Ok(color_val) = u32::from_str_radix(clean_val, 16) {
                *current_color = Some(color_val);
            }
        } else if val_part.starts_with('[') && val_part.ends_with(']') {
            let inside = &val_part[1..val_part.len() - 1];
            for t_id in inside.split(',') {
                let cleaned = t_id.trim();
                if !cleaned.is_empty() {
                    current_links.push((key.to_string(), cleaned.to_string()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hubgs_parser_decodes_valid_contents() {
        let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> (0..*) ALLOWS [Character]
        }
    ]
]
INSTANCES [
    hero: Character {
        name = "Hero"
    }
]
        "#;
        let (defs, insts) = parse_hubgs(sample).unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "Character");
        assert_eq!(defs[0].links.len(), 1);
        assert_eq!(defs[0].links[0], ("friend".to_string(), "->".to_string(), "Character".to_string()));

        assert_eq!(insts.len(), 1);
        assert_eq!(insts[0].id, "hero");
        assert_eq!(insts[0].type_name, "Character");
        assert_eq!(insts[0].name, "Hero");
    }

    #[test]
    fn test_hubgs_parser_rejects_missing_multiplicity() {
        let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> ALLOWS [Character]
        }
    ]
]
        "#;
        let res = parse_hubgs(sample);
        assert!(res.is_err());
        assert!(res.err().unwrap().to_string().contains("missing multiplicity bounds"));
    }

    #[test]
    fn test_hubgs_parser_rejects_malformed_multiplicity() {
        let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> (0..) ALLOWS [Character]
        }
    ]
]
        "#;
        let res = parse_hubgs(sample);
        assert!(res.is_err());
    }

    #[test]
    fn test_hubgs_parser_supports_all_directionalities() {
        let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend <-> (0..*) ALLOWS [Character],
            boss <- (0..1) ALLOWS [Character],
            peer - (1..1) ALLOWS [Character]
        }
    ]
]
        "#;
        let (defs, _) = parse_hubgs(sample).unwrap();
        assert_eq!(defs.len(), 1);
        let links = &defs[0].links;
        assert_eq!(links.len(), 3);
        assert_eq!(links[0], ("friend".to_string(), "<->".to_string(), "Character".to_string()));
        assert_eq!(links[1], ("boss".to_string(), "<-".to_string(), "Character".to_string()));
        assert_eq!(links[2], ("peer".to_string(), "-".to_string(), "Character".to_string()));
    }
}
