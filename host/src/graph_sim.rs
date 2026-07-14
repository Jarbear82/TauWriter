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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HubgsLink {
    pub(crate) name: String,
    pub(crate) arrow: String,
    pub(crate) target: String,
}

pub(crate) struct HubgsDefinition {
    pub(crate) name: String,
    pub(crate) links: Vec<HubgsLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InstanceLink {
    pub(crate) relation: String,
    pub(crate) target: String,
}

pub(crate) struct HubgsInstance {
    pub(crate) id: String,
    pub(crate) type_name: String,
    pub(crate) name: String,
    pub(crate) theme_color: Option<u32>,
    pub(crate) links: Vec<InstanceLink>,
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
    let n = instances.len();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut vxs = vec![0.0; n];
    let mut vys = vec![0.0; n];

    for _ in 0..n {
        xs.push(width / 2.0 + rng.range(-50.0, 50.0));
        ys.push(height / 2.0 + rng.range(-50.0, 50.0));
    }

    // Map relation names to their defined arrow type
    let mut relation_arrows = std::collections::HashMap::new();
    for def in definitions {
        for link in &def.links {
            relation_arrows.insert(link.name.as_str(), link.arrow.as_str());
        }
    }

    let id_to_index: std::collections::HashMap<&str, usize> = instances
        .iter()
        .enumerate()
        .map(|(idx, inst)| (inst.id.as_str(), idx))
        .collect();

    let mut edges = Vec::new();
    for (src_idx, inst) in instances.iter().enumerate() {
        for link in &inst.links {
            if let Some(&tgt_idx) = id_to_index.get(link.target.as_str()) {
                let arrow = relation_arrows
                    .get(link.relation.as_str())
                    .copied()
                    .unwrap_or("-");
                edges.push(GraphEdge {
                    source: src_idx,
                    target: tgt_idx,
                    label: arrow.to_string(),
                });
            }
        }
    }

    simulate(
        &mut xs, &mut ys, &mut vxs, &mut vys, &edges, width, height, 80.0, 2000.0, 0.06, 0.01,
        0.85, 300.0,
    );

    let nodes = instances
        .iter()
        .enumerate()
        .map(|(idx, inst)| {
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
                x: xs[idx],
                y: ys[idx],
                vx: vxs[idx],
                vy: vys[idx],
            }
        })
        .collect();

    (nodes, edges)
}

/// Run a force-directed layout over definition nodes.
pub(crate) fn run_def_simulation(
    definitions: &[HubgsDefinition],
    width: f32,
    height: f32,
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut rng = SimpleRng::new(54321);
    let n = definitions.len();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut vxs = vec![0.0; n];
    let mut vys = vec![0.0; n];

    for _ in 0..n {
        xs.push(width / 2.0 + rng.range(-50.0, 50.0));
        ys.push(height / 2.0 + rng.range(-50.0, 50.0));
    }

    let name_to_index: std::collections::HashMap<&str, usize> = definitions
        .iter()
        .enumerate()
        .map(|(idx, def)| (def.name.as_str(), idx))
        .collect();

    let mut edges = Vec::new();
    for (src_idx, def) in definitions.iter().enumerate() {
        for link in &def.links {
            if let Some(&tgt_idx) = name_to_index.get(link.target.as_str()) {
                edges.push(GraphEdge {
                    source: src_idx,
                    target: tgt_idx,
                    label: link.arrow.clone(),
                });
            }
        }
    }

    simulate(
        &mut xs, &mut ys, &mut vxs, &mut vys, &edges, width, height, 100.0, 2500.0, 0.08, 0.015,
        0.85, 400.0,
    );

    let nodes = definitions
        .iter()
        .enumerate()
        .map(|(idx, def)| {
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
                x: xs[idx],
                y: ys[idx],
                vx: vxs[idx],
                vy: vys[idx],
            }
        })
        .collect();

    (nodes, edges)
}

/// Shared force-directed simulation loop operating on flat layout coordinates.
fn simulate(
    xs: &mut [f32],
    ys: &mut [f32],
    vxs: &mut [f32],
    vys: &mut [f32],
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
    let n = xs.len();
    for _ in 0..200 {
        for i in 0..n {
            let xi = xs[i];
            let yi = ys[i];
            let mut vxi = 0.0;
            let mut vyi = 0.0;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let dx = xi - xs[j];
                let dy = yi - ys[j];
                let dist_sq = dx * dx + dy * dy + 0.1;
                let dist = dist_sq.sqrt();
                if dist < repulsion_range {
                    let force = rep_strength / dist_sq;
                    vxi += (dx / dist) * force;
                    vyi += (dy / dist) * force;
                }
            }
            vxs[i] += vxi;
            vys[i] += vyi;
        }

        for edge in edges {
            let src = edge.source;
            let tgt = edge.target;
            let dx = xs[tgt] - xs[src];
            let dy = ys[tgt] - ys[src];
            let dist = (dx * dx + dy * dy + 0.1).sqrt();
            let force = (dist - k) * attr_strength;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;
            vxs[src] += fx;
            vys[src] += fy;
            vxs[tgt] -= fx;
            vys[tgt] -= fy;
        }

        let cx = width / 2.0;
        let cy = height / 2.0;
        for i in 0..n {
            let x = xs[i];
            let y = ys[i];
            let vx = vxs[i] + (cx - x) * center_pull;
            let vy = vys[i] + (cy - y) * center_pull;

            xs[i] = (x + vx).clamp(24.0, width - 24.0);
            ys[i] = (y + vy).clamp(24.0, height - 24.0);
            vxs[i] = vx * damping;
            vys[i] = vy * damping;
        }
    }
}

/// Recursively search a directory tree for the first `.hubgs` file.
pub(crate) fn find_any_hubgs(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    crate::utils::find_first_file(dir, Some("hubgs"))
}

mod hubgs_parser;

/// Parse a HubGS file into definitions and instances.
pub(crate) fn parse_hubgs_file(
    path: &std::path::Path,
) -> anyhow::Result<(Vec<HubgsDefinition>, Vec<HubgsInstance>)> {
    let content = std::fs::read_to_string(path)?;
    hubgs_parser::parse_hubgs(&content)
}
