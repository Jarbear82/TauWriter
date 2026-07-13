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
            );
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
    current_hub_links: &mut Vec<(String, String)>,
    definitions: &mut Vec<HubgsDefinition>,
) {
    if trimmed.ends_with('{') && !trimmed.contains(':') {
        let hub_name = trimmed.trim_end_matches('{').trim().to_string();
        *current_hub_name = Some(hub_name);
        current_hub_links.clear();
        return;
    }
    if trimmed == "}" || trimmed == "}," {
        if let Some(name) = current_hub_name.take() {
            definitions.push(HubgsDefinition {
                name,
                links: current_hub_links.clone(),
            });
        }
        return;
    }

    if trimmed.contains("ALLOWS [") {
        if let Some(arrow_idx) = trimmed.find("->") {
            let rel_name = trimmed[..arrow_idx].trim().to_string();
            if let Some(allows_idx) = trimmed.find("ALLOWS [") {
                let target_part = &trimmed[allows_idx + 8..];
                if let Some(end_bracket) = target_part.find(']') {
                    let target_hub = target_part[..end_bracket].trim().to_string();
                    current_hub_links.push((rel_name, target_hub));
                }
            }
        } else if let Some(double_arrow_idx) = trimmed.find("<->") {
            let rel_name = trimmed[..double_arrow_idx].trim().to_string();
            if let Some(allows_idx) = trimmed.find("ALLOWS [") {
                let target_part = &trimmed[allows_idx + 8..];
                if let Some(end_bracket) = target_part.find(']') {
                    let target_hub = target_part[..end_bracket].trim().to_string();
                    current_hub_links.push((rel_name, target_hub));
                }
            }
        }
    }
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
            friend -> ALLOWS [Character]
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
        assert_eq!(defs[0].links[0], ("friend".to_string(), "Character".to_string()));

        assert_eq!(insts.len(), 1);
        assert_eq!(insts[0].id, "hero");
        assert_eq!(insts[0].type_name, "Character");
        assert_eq!(insts[0].name, "Hero");
    }
}
