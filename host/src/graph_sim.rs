//! Graph simulation — force-directed layout for HubGS definitions and instances.
//! Extracted from `main.rs` to isolate the physics engine logic.

use gpui::{Hsla, SharedString};

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
#[derive(Debug, Clone)]
pub(crate) struct GraphNode {
    pub(crate) id: SharedString,
    pub(crate) name: SharedString,
    pub(crate) type_name: SharedString,
    pub(crate) color: Hsla,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) vx: f32,
    pub(crate) vy: f32,
    /// Layout engine's computed target position for this node.
    pub(crate) anchor_x: f32,
    pub(crate) anchor_y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) attributes: Vec<String>,
}

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct GraphEdge {
    pub(crate) source: usize,
    pub(crate) target: usize,
    pub(crate) label: SharedString,
}

/// Run a force-directed layout over instance nodes.
pub(crate) fn run_graph_simulation(
    instances: &[HubgsInstance],
    definitions: &[HubgsDefinition],
    _width: f32,
    _height: f32,
    sizer: &mut sizing::NodeSizer,
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut rng = SimpleRng::new(12345);
    let n = instances.len();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut vxs = vec![0.0; n];
    let mut vys = vec![0.0; n];

    // Real measured size per node, replacing `45.0 + links.len() * 18.0`.
    let mut widths = Vec::with_capacity(n);
    let mut heights = Vec::with_capacity(n);
    for inst in instances {
        let attrs: Vec<String> = Vec::new(); // instance cards show no attribute lines today
        let (w, h) = sizer(sizing::NodeContent {
            name: inst.name.as_ref(),
            type_name: inst.type_name.as_ref(),
            attributes: &attrs,
        });
        widths.push(w);
        heights.push(h);
    }

    for _ in 0..n {
        xs.push(rng.range(-80.0, 80.0));
        ys.push(rng.range(-80.0, 80.0));
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
                    label: arrow.into(),
                });
            }
        }
    }

    simulate(
        &mut xs, &mut ys, &mut vxs, &mut vys, &edges, &widths, &heights, 80.0, 2000.0, 0.06, 0.01,
        0.85,
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
                anchor_x: xs[idx],
                anchor_y: ys[idx],
                width: widths[idx],
                height: heights[idx],
                attributes: vec![],
            }
        })
        .collect();

    (nodes, edges)
}

/// Run a force-directed layout over definition nodes.
pub(crate) fn run_def_simulation(
    definitions: &[HubgsDefinition],
    _width: f32,
    _height: f32,
    sizer: &mut sizing::NodeSizer,
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut rng = SimpleRng::new(54321);
    let n = definitions.len();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut vxs = vec![0.0; n];
    let mut vys = vec![0.0; n];

    // Real measured size per node.
    let mut widths = Vec::with_capacity(n);
    let mut heights = Vec::with_capacity(n);
    for def in definitions {
        let attrs: Vec<String> = Vec::new();
        let (w, h) = sizer(sizing::NodeContent {
            name: def.name.as_ref(),
            type_name: "HubDefinition",
            attributes: &attrs,
        });
        widths.push(w);
        heights.push(h);
    }

    for _ in 0..n {
        xs.push(rng.range(-80.0, 80.0));
        ys.push(rng.range(-80.0, 80.0));
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
        &mut xs, &mut ys, &mut vxs, &mut vys, &edges, &widths, &heights, 100.0, 2500.0, 0.08,
        0.015, 0.85,
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
                type_name: "HubDefinition".into(),
                color: color.into(),
                x: xs[idx],
                y: ys[idx],
                vx: vxs[idx],
                vy: vys[idx],
                anchor_x: xs[idx],
                anchor_y: ys[idx],
                width: widths[idx],
                height: heights[idx],
                attributes: vec![],
            }
        })
        .collect();

    (nodes, edges)
}

/// Shared force-directed simulation loop operating on flat layout coordinates.
/// Compute AABB hard collision response and soft proximity repulsion between two nodes.
/// Returns `(force_x, force_y)` to accumulate into node `i`'s velocity delta.
/// `is_nested_cluster` reduces repulsion to prevent sub-elements from scattering
/// when a parent is expanded (dual-gravity structural modifier).
#[inline]
fn compute_repulsion(
    xi: f32,
    yi: f32,
    half_w_i: f32,
    half_h_i: f32,
    xj: f32,
    yj: f32,
    half_w_j: f32,
    half_h_j: f32,
    rep_strength: f32,
    is_nested_cluster: bool,
) -> (f32, f32) {
    let dx = xi - xj;
    let dy = yi - yj;

    let overlap_x = (half_w_i + half_w_j) - dx.abs();
    let overlap_y = (half_h_i + half_h_j) - dy.abs();

    // AABB collision: both axes must overlap for hard response.
    // Use the least-penetration axis to minimize displacement needed to resolve.
    if overlap_x > 0.0 && overlap_y > 0.0 {
        let along_axis = overlap_x < overlap_y;
        let push_strength = 0.5;

        if along_axis {
            let dir_x = if dx.abs() > f32::EPSILON {
                dx.signum()
            } else if xi < xj {
                -1.0
            } else {
                1.0
            };
            (dir_x * overlap_x * push_strength, 0.0)
        } else {
            let dir_y = if dy.abs() > f32::EPSILON {
                dy.signum()
            } else if yi < yj {
                -1.0
            } else {
                1.0
            };
            (0.0, dir_y * overlap_y * push_strength)
        }
    } else {
        // No AABB collision — soft repulsion with smooth ramp from zero at touch.
        let edge_dist_x = dx.abs() - (half_w_i + half_w_j);
        let edge_dist_y = dy.abs() - (half_h_i + half_h_j);
        let edge_dist = edge_dist_x.max(0.0).hypot(edge_dist_y.max(0.0));

        if edge_dist < 100.0 && edge_dist > 0.1 {
            let structural_modifier = if is_nested_cluster { 0.15 } else { 1.0 };

            // Smooth ramp: zero force at perfect touch (edge_dist=0) up to full strength
            // at the inner edge of the soft zone (~10px gap)
            let ramp_factor = (edge_dist / 10.0).clamp(0.0, 1.0);
            let force = (rep_strength * structural_modifier * ramp_factor)
                / (edge_dist * edge_dist + f32::EPSILON);

            ((dx / edge_dist) * force, (dy / edge_dist) * force)
        } else {
            (0.0, 0.0)
        }
    }
}

/// Simulate the full force-directed layout.
fn simulate(
    xs: &mut [f32],
    ys: &mut [f32],
    vxs: &mut [f32],
    vys: &mut [f32],
    edges: &[GraphEdge],
    widths: &[f32],
    heights: &[f32],
    k: f32,
    rep_strength: f32,
    attr_strength: f32,
    center_pull: f32,
    damping: f32,
) {
    const MAX_ITERATIONS: usize = 500;
    const ENERGY_THRESHOLD: f32 = 0.01;

    let n = xs.len();
    for _iter in 0..MAX_ITERATIONS {
        for i in 0..n {
            let xi = xs[i];
            let yi = ys[i];
            let mut vxi = 0.0;
            let mut vyi = 0.0;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let (f_x, f_y) = compute_repulsion(
                    xi,
                    yi,
                    widths[i] / 2.0,
                    heights[i] / 2.0,
                    xs[j],
                    ys[j],
                    widths[j] / 2.0,
                    heights[j] / 2.0,
                    rep_strength,
                    false,
                );
                vxi += f_x;
                vyi += f_y;
            }
            vxs[i] += vxi;
            vys[i] += vyi;
        }

        for edge in edges {
            let src = edge.source;
            let tgt = edge.target;
            let dx = xs[tgt] - xs[src];
            let dy = ys[tgt] - ys[src];

            // Use each node's own half-extents for the edge-distance spring target.
            let half_w = (widths[src] + widths[tgt]) / 4.0;
            let half_h = (heights[src] + heights[tgt]) / 4.0;
            let edge_dist_x = dx.abs() - half_w * 2.0;
            let edge_dist_y = dy.abs() - half_h * 2.0;
            let edge_dist = edge_dist_x.max(0.0).hypot(edge_dist_y.max(0.0));

            // Target gap of k between bounding boxes, not center-to-center distance
            let force = (edge_dist - k) * attr_strength;

            if edge_dist > 0.001 {
                let fx = (dx / edge_dist) * force;
                let fy = (dy / edge_dist) * force;
                vxs[src] += fx;
                vys[src] += fy;
                vxs[tgt] -= fx;
                vys[tgt] -= fy;
            }
        }

        for i in 0..n {
            let x = xs[i];
            let y = ys[i];
            let vx = vxs[i] + (0.0 - x) * center_pull;
            let vy = vys[i] + (0.0 - y) * center_pull;

            xs[i] = x + vx;
            ys[i] = y + vy;
            vxs[i] = vx * damping;
            vys[i] = vy * damping;
        }

        // Energy-based early exit: if total kinetic energy is below threshold, the layout has converged.
        let mut total_energy = 0.0;
        for i in 0..n {
            total_energy += vxs[i] * vxs[i] + vys[i] * vys[i];
        }
        if total_energy < ENERGY_THRESHOLD {
            break;
        }
    }
}

/// Single step of the physics simulation (pulling nodes towards their layout anchors and handling dragging).
/// Returns the total kinetic energy of the system to determine if it has settled.
pub(crate) fn simulate_step(
    nodes: &mut [GraphNode],
    dragged_node_id: Option<&SharedString>,
) -> f32 {
    let n = nodes.len();
    if n == 0 {
        return 0.0;
    }

    let mut dvx = vec![0.0; n];
    let mut dvy = vec![0.0; n];

    // 1. Calculate physics forces
    for i in 0..n {
        if dragged_node_id.map_or(false, |drag_id| *drag_id == nodes[i].id) {
            continue;
        }

        // Pull toward anchor layout target
        let dx = nodes[i].anchor_x - nodes[i].x;
        let dy = nodes[i].anchor_y - nodes[i].y;
        dvx[i] += dx * 0.12;
        dvy[i] += dy * 0.12;

        let mut vxi = 0.0;
        let mut vyi = 0.0;
        // Repulsion from other nodes using AABB collision detection (centralized logic)
        for j in 0..n {
            if i == j {
                continue;
            }
            let node_j = &nodes[j];
            let is_nested = false; // TODO: determine based on tree relationship
            let (f_x, f_y) = compute_repulsion(
                nodes[i].x,
                nodes[i].y,
                nodes[i].width / 2.0,
                nodes[i].height / 2.0,
                node_j.x,
                node_j.y,
                node_j.width / 2.0,
                node_j.height / 2.0,
                800.0,
                is_nested,
            );
            vxi += f_x;
            vyi += f_y;
        }
        dvx[i] += vxi;
        dvy[i] += vyi;
    }

    // 2. Apply velocities and clamp positions
    let mut total_energy = 0.0;
    let damping = 0.75;

    for i in 0..n {
        if dragged_node_id.map_or(false, |drag_id| *drag_id == nodes[i].id) {
            nodes[i].vx = 0.0;
            nodes[i].vy = 0.0;
            continue;
        }

        let node = &mut nodes[i];
        let vx = node.vx + dvx[i];
        let vy = node.vy + dvy[i];

        node.vx = vx * damping;
        node.vy = vy * damping;

        node.x += node.vx;
        node.y += node.vy;

        total_energy += node.vx * node.vx + node.vy * node.vy;
    }

    total_energy
}

/// Compute node layout target positions based on layout type.
pub(crate) fn compute_layout(
    layout_type: crate::ui::LayoutType,
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    width: f32,
    height: f32,
) -> Vec<(SharedString, f32, f32)> {
    match layout_type {
        crate::ui::LayoutType::Circular => {
            let n = nodes.len();
            if n == 0 {
                return Vec::new();
            }
            let cx = width / 2.0;
            let cy = height / 2.0;

            // Size circle from total node "footprint" arc length needed
            let avg_node_width = nodes.iter().map(|node| node.width).sum::<f32>() / n as f32;
            let required_radius =
                (avg_node_width * n as f32 / (2.0 * std::f32::consts::PI)).max(50.0);
            // Also respect the canvas constraint
            let canvas_radius = (width.min(height) / 2.0 - 50.0).max(50.0);
            let radius = canvas_radius.max(required_radius);

            let mut targets = Vec::with_capacity(n);
            for (i, node) in nodes.iter().enumerate() {
                let angle = (i as f32 / n as f32) * 2.0 * std::f32::consts::PI;
                let x = cx + radius * angle.cos();
                let y = cy + radius * angle.sin();
                targets.push((node.id.clone(), x, y));
            }
            targets
        }
        crate::ui::LayoutType::Grid => {
            let n = nodes.len();
            if n == 0 {
                return Vec::new();
            }

            // Use actual node dimensions for cell sizing with a gap
            let max_w = nodes.iter().map(|node| node.width).fold(0.0, f32::max);
            let max_h = nodes.iter().map(|node| node.height).fold(0.0, f32::max);
            let cell_gap_x = 20.0;
            let cell_gap_y = 16.0;

            // Layout grid: spread across available space using canvas size for positioning
            // but use actual node dimensions so there's no overlap
            let cols = ((width / (max_w + cell_gap_x)).ceil() as usize).max(1);
            let rows = ((height / (max_h + cell_gap_y)).ceil() as usize).max(1);

            let mut targets = Vec::with_capacity(n);
            for (i, node) in nodes.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                let x = (col as f32 + 0.5) * (width / cols as f32);
                let y = (row as f32 + 0.5) * (height / rows as f32);
                targets.push((node.id.clone(), x, y));
            }
            targets
        }
        crate::ui::LayoutType::ForceDirected => {
            let n = nodes.len();
            if n == 0 {
                return Vec::new();
            }
            let mut xs: Vec<_> = nodes.iter().map(|n| n.x).collect();
            let mut ys: Vec<_> = nodes.iter().map(|n| n.y).collect();
            let mut vxs = vec![0.0; n];
            let mut vys = vec![0.0; n];

            simulate(
                &mut xs,
                &mut ys,
                &mut vxs,
                &mut vys,
                edges,
                &nodes.iter().map(|n| n.width).collect::<Vec<_>>(),
                &nodes.iter().map(|n| n.height).collect::<Vec<_>>(),
                80.0,
                2000.0,
                0.06,
                0.01,
                0.85,
            );

            nodes
                .iter()
                .enumerate()
                .map(|(idx, node)| (node.id.clone(), xs[idx], ys[idx]))
                .collect()
        }
    }
}

/// Compute node layout target positions, preserving targets for any locked nodes.
pub(crate) fn compute_layout_with_locks(
    layout_type: crate::ui::LayoutType,
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    width: f32,
    height: f32,
    locked_node_ids: &[SharedString],
) -> Vec<(SharedString, f32, f32)> {
    let targets = compute_layout(layout_type, nodes, edges, width, height);

    // Build a quick-lookup set for locked node IDs
    let locked_set: std::collections::HashSet<&str> =
        locked_node_ids.iter().map(|s| s.as_ref()).collect();

    // For locked nodes, override computed target with their current (x, y) position
    targets
        .into_iter()
        .map(|(id, tx, ty)| {
            if locked_set.contains(id.as_str()) {
                let cur = nodes
                    .iter()
                    .find(|n| n.id == id)
                    .map_or((tx, ty), |n| (n.x, n.y));
                (id, cur.0, cur.1)
            } else {
                (id, tx, ty)
            }
        })
        .collect()
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
