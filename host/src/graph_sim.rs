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
    pub(crate) links: Vec<(String, String, String)>, // (relation, arrow, target_hub)
}

pub(crate) struct HubgsInstance {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) name: String,
    pub(crate) theme_color: Option<u32>,
    pub(crate) links: Vec<(String, String)>, // (relation, target_id)
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct GraphEdge {
    pub(crate) source: usize,
    pub(crate) target: usize,
    pub(crate) label: String,
}

/// Run a force-directed layout over instance nodes.
pub(crate) fn run_graph_simulation(
    instances: &[HubgsInstance],
    definitions: &[HubgsDefinition],
    width: f32,
    height: f32,
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
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

    // Map relation names to their defined arrow type
    let mut relation_arrows = std::collections::HashMap::new();
    for def in definitions {
        for (rel_name, arrow, _) in &def.links {
            relation_arrows.insert(rel_name.as_str(), arrow.as_str());
        }
    }

    let id_to_index: std::collections::HashMap<&str, usize> = instances
        .iter()
        .enumerate()
        .map(|(idx, inst)| (inst.id.as_str(), idx))
        .collect();

    let mut edges = Vec::new();
    for (src_idx, inst) in instances.iter().enumerate() {
        for (rel_name, target_id) in &inst.links {
            if let Some(&tgt_idx) = id_to_index.get(target_id.as_str()) {
                let arrow = relation_arrows.get(rel_name.as_str()).copied().unwrap_or("-");
                edges.push(GraphEdge {
                    source: src_idx,
                    target: tgt_idx,
                    label: arrow.to_string(),
                });
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
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
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

    let name_to_index: std::collections::HashMap<&str, usize> = definitions
        .iter()
        .enumerate()
        .map(|(idx, def)| (def.name.as_str(), idx))
        .collect();

    let mut edges = Vec::new();
    for (src_idx, def) in definitions.iter().enumerate() {
        for (_, arrow, target_hub) in &def.links {
            if let Some(&tgt_idx) = name_to_index.get(target_hub.as_str()) {
                edges.push(GraphEdge {
                    source: src_idx,
                    target: tgt_idx,
                    label: arrow.clone(),
                });
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
    edges: &[GraphEdge],
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

        for edge in edges {
            let src = edge.source;
            let tgt = edge.target;
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

mod hubgs_parser;

/// Parse a HubGS file into definitions and instances.
pub(crate) fn parse_hubgs_file(
    path: &std::path::Path,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let content = std::fs::read_to_string(path)?;
    hubgs_parser::parse_hubgs(&content)
}
