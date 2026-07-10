//! Graph simulation — force-directed layout for HubGS definitions and instances.
//! Extracted from `main.rs` to isolate the physics engine logic.

use gpui::Hsla;

pub(crate) struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub(crate) fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }

    pub(crate) fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct GraphNode {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) color: Hsla,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) vx: f32,
    pub(crate) vy: f32,
}

pub(crate) struct HubgsDefinition {
    pub(crate) name: String,
    pub(crate) links: Vec<(String, String)>,
}

pub(crate) struct HubgsInstance {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) name: String,
    pub(crate) theme_color: Option<u32>,
    pub(crate) links: Vec<(String, String)>, // (relation, target_id)
}

/// Run a force-directed layout over instance nodes.
pub(crate) fn run_graph_simulation(
    instances: &[HubgsInstance],
    width: f32,
    height: f32,
) -> (Vec<GraphNode>, Vec<(usize, usize, String)>) {
    let mut rng = SimpleRng::new(12345);

    let mut nodes: Vec<GraphNode> = instances
        .iter()
        .map(|inst| {
            let rx = width / 2.0 + rng.range(-50.0, 50.0);
            let ry = height / 2.0 + rng.range(-50.0, 50.0);

            let color = inst.theme_color.map_or_else(
                || match inst.type_name.as_str() {
                    "Character" => gpui::rgb(0x4169E1),
                    "Location" => gpui::rgb(0x2ECC71),
                    "Creature" => gpui::rgb(0xE67E22),
                    "Item" => gpui::rgb(0x9B59B6),
                    _ => gpui::rgb(0x7F8C8D),
                },
                gpui::rgb,
            );

            GraphNode {
                id: inst.id.clone(),
                name: inst.name.clone(),
                type_name: inst.type_name.clone(),
                color: color.into(),
                x: rx,
                y: ry,
                vx: 0.0,
                vy: 0.0,
            }
        })
        .collect();

    let mut edges = Vec::new();
    for (src_idx, inst) in instances.iter().enumerate() {
        for (_, target_id) in &inst.links {
            if let Some(tgt_idx) = instances.iter().position(|i| &i.id == target_id) {
                edges.push((src_idx, tgt_idx, String::new()));
            }
        }
    }

    simulate(&mut nodes, &edges, width, height, 80.0, 2000.0, 0.06, 0.01, 0.85, 300.0);

    (nodes, edges)
}

/// Run a force-directed layout over definition nodes.
pub(crate) fn run_def_simulation(
    definitions: &[HubgsDefinition],
    width: f32,
    height: f32,
) -> (Vec<GraphNode>, Vec<(usize, usize, String)>) {
    let mut rng = SimpleRng::new(54321);

    let mut nodes: Vec<GraphNode> = definitions
        .iter()
        .map(|def| {
            let rx = width / 2.0 + rng.range(-50.0, 50.0);
            let ry = height / 2.0 + rng.range(-50.0, 50.0);

            let color = match def.name.as_str() {
                "Character" => gpui::rgb(0x4169E1),
                "Location" => gpui::rgb(0x2ECC71),
                "Creature" => gpui::rgb(0xE67E22),
                "Item" => gpui::rgb(0x9B59B6),
                _ => gpui::rgb(0x7F8C8D),
            };

            GraphNode {
                id: def.name.clone(),
                name: def.name.clone(),
                type_name: "HubDefinition".to_string(),
                color: color.into(),
                x: rx,
                y: ry,
                vx: 0.0,
                vy: 0.0,
            }
        })
        .collect();

    let mut edges = Vec::new();
    for (src_idx, def) in definitions.iter().enumerate() {
        for (_, target_hub) in &def.links {
            if let Some(tgt_idx) = definitions.iter().position(|d| &d.name == target_hub) {
                edges.push((src_idx, tgt_idx, String::new()));
            }
        }
    }

    simulate(&mut nodes, &edges, width, height, 100.0, 2500.0, 0.08, 0.015, 0.85, 400.0);

    (nodes, edges)
}

/// Shared force-directed simulation loop.  Parameters mirror the original
/// Kamada-Kawai / repulsion-attraction model from main.rs.
fn simulate(
    nodes: &mut [GraphNode],
    edges: &[(usize, usize, String)],
    width: f32,
    height: f32,
    k: f32,
    rep_strength: f32,
    attr_strength: f32,
    center_pull: f32,
    damping: f32,
    repulsion_range: f32,
) {
    for _ in 0..200 {
        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i == j {
                    continue;
                }
                let dx = nodes[i].x - nodes[j].x;
                let dy = nodes[i].y - nodes[j].y;
                let dist_sq = dx * dx + dy * dy + 0.1;
                let dist = dist_sq.sqrt();
                if dist < repulsion_range {
                    let force = rep_strength / dist_sq;
                    nodes[i].vx += (dx / dist) * force;
                    nodes[i].vy += (dy / dist) * force;
                }
            }
        }

        for &(src, tgt, _) in edges {
            let dx = nodes[tgt].x - nodes[src].x;
            let dy = nodes[tgt].y - nodes[src].y;
            let dist = (dx * dx + dy * dy + 0.1).sqrt();
            let force = (dist - k) * attr_strength;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;
            nodes[src].vx += fx;
            nodes[src].vy += fy;
            nodes[tgt].vx -= fx;
            nodes[tgt].vy -= fy;
        }

        let cx = width / 2.0;
        let cy = height / 2.0;
        for node in &mut *nodes {
            node.vx += (cx - node.x) * center_pull;
            node.vy += (cy - node.y) * center_pull;

            node.x += node.vx;
            node.y += node.vy;

            node.vx *= damping;
            node.vy *= damping;

            node.x = node.x.clamp(24.0, width - 24.0);
            node.y = node.y.clamp(24.0, height - 24.0);
        }
    }
}

/// Recursively search a directory tree for the first `.hubgs` file.
pub(crate) fn find_any_hubgs(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.starts_with('.') || name == "target" || name == "vendor" {
                continue;
            }
            if path.is_dir() {
                if let Some(found) = find_any_hubgs(&path) {
                    return Some(found);
                }
            } else if path.extension().map_or(false, |ext| ext == "hubgs") {
                return Some(path);
            }
        }
    }
    None
}

/// Parse a HubGS file into definitions and instances.
pub(crate) fn parse_hubgs_file(
    path: &std::path::Path,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let content = std::fs::read_to_string(path)?;
    let mut definitions = Vec::new();
    let mut instances = Vec::new();

    let mut in_definitions = false;
    let mut in_fields = false;
    let mut in_hubs = false;
    let mut in_instances = false;

    let mut current_hub_name = None;
    let mut current_hub_links = Vec::new();

    let mut current_id = None;
    let mut current_type = None;
    let mut current_name = String::new();
    let mut current_color = None;
    let mut current_links = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Block boundaries
        if trimmed.starts_with("DEFINITIONS [") {
            in_definitions = true;
            continue;
        }
        if in_definitions {
            if trimmed.starts_with("FIELDS [") {
                in_fields = true;
                continue;
            }
            if in_fields && (trimmed == "]" || trimmed == "],") {
                in_fields = false;
                continue;
            }
            if trimmed.starts_with("HUBS [") {
                in_hubs = true;
                continue;
            }
            if in_hubs && (trimmed == "]" || trimmed == "],") {
                in_hubs = false;
                continue;
            }
            if !in_fields && !in_hubs && (trimmed == "]" || trimmed == "],") {
                in_definitions = false;
                continue;
            }
        }

        if trimmed.starts_with("INSTANCES [") {
            in_instances = true;
            continue;
        }
        if in_instances && (trimmed == "]" || trimmed == "],") {
            in_instances = false;
            continue;
        }

        // Parse Definitions
        if in_hubs {
            if trimmed.ends_with('{') && !trimmed.contains(':') {
                let hub_name = trimmed.trim_end_matches('{').trim().to_string();
                current_hub_name = Some(hub_name);
                current_hub_links.clear();
                continue;
            }
            if trimmed == "}" || trimmed == "}," {
                if let Some(name) = current_hub_name.take() {
                    definitions.push(HubgsDefinition {
                        name,
                        links: current_hub_links.clone(),
                    });
                }
                continue;
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

        // Parse Instances
        if in_instances {
            if trimmed.contains(':') && trimmed.ends_with('{') {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 2 {
                    let id = parts[0].trim().to_string();
                    let type_part = parts[1].trim().trim_end_matches('{').trim().to_string();
                    current_id = Some(id);
                    current_type = Some(type_part);
                    current_name = String::new();
                    current_color = None;
                    current_links.clear();
                }
                continue;
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
                        theme_color: current_color,
                        links: current_links.clone(),
                    });
                }
                continue;
            }

            if let Some(eq_idx) = trimmed.find('=') {
                let key = trimmed[..eq_idx].trim();
                let val_part = trimmed[eq_idx + 1..].trim().trim_end_matches(',').trim();

                if key == "name" {
                    current_name = val_part.trim_matches('"').to_string();
                } else if key == "theme_color" {
                    let clean_val = val_part.trim_start_matches("0x").trim_start_matches("0X");
                    if let Ok(color_val) = u32::from_str_radix(clean_val, 16) {
                        current_color = Some(color_val);
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
    }

    Ok((definitions, instances))
}
