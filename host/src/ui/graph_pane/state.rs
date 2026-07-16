//! Stateful viewer for graph panes — GraphPaneView struct definition + business logic.

use crate::ui::Workspace;
use gpui::{prelude::*, Entity, SharedString};

/// Stateful viewer that owns all graph data, camera state, and layout mode.
#[derive(Debug, Clone)]
pub(crate) struct GraphPaneView {
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) window_handle: gpui::AnyWindowHandle,
    pub(crate) graph_nodes: Vec<crate::graph_sim::GraphNode>,
    pub(crate) graph_edges: Vec<crate::graph_sim::GraphEdge>,
    pub(crate) def_nodes: Vec<crate::graph_sim::GraphNode>,
    pub(crate) def_edges: Vec<crate::graph_sim::GraphEdge>,
    pub(crate) outline_nodes: Vec<crate::graph_sim::GraphNode>,
    pub(crate) outline_edges: Vec<crate::graph_sim::GraphEdge>,
    pub(crate) is_ticking: bool,
    pub(crate) dragged_node: Option<SharedString>,
    pub(crate) last_mouse_pos: gpui::Point<gpui::Pixels>,
    pub(crate) layout_mode: super::data::LayoutMode,
    // Per-tab camera state [0]=Document, [1]=Definitions, [2]=Instances.
    pub(crate) camera_states: [CameraState; 3],
    pub(crate) active_camera_idx: usize,
    // Real pane content-box bounds (set by parent via bounds callback)
    pub(crate) pane_content_width: f32,
    pub(crate) pane_content_height: f32,
}

/// Per-tab camera state so each graph pane has independent pan/zoom.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CameraState {
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) zoom: f32,
    pub(crate) is_panning: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
            is_panning: false,
        }
    }
}

impl gpui::EventEmitter<super::data::GraphEvent> for GraphPaneView {}

#[allow(dead_code)]
impl GraphPaneView {
    pub(crate) fn new(
        workspace: Entity<Workspace>,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            workspace: workspace.clone(),
            window_handle: window.window_handle(),
            graph_nodes: Vec::new(),
            graph_edges: Vec::new(),
            def_nodes: Vec::new(),
            def_edges: Vec::new(),
            outline_nodes: Vec::new(),
            outline_edges: Vec::new(),
            is_ticking: false,
            dragged_node: None,
            last_mouse_pos: gpui::point(gpui::px(0.), gpui::px(0.)),
            layout_mode: super::data::LayoutMode::default(),
            camera_states: [CameraState::default(); 3],
            active_camera_idx: 0,
            pane_content_width: 0.0,
            pane_content_height: 0.0,
        };
        // Only load data on creation — no auto-layout trigger
        this.recalculate_data(&workspace, cx);
        this
    }

    pub(crate) fn trigger_run_layout(&mut self, cx: &mut Context<Self>) {
        let layout_type = self.workspace.read(cx).layout_type;
        self.layout_mode = super::data::LayoutMode::RunLayout(layout_type);
        // Note: data reload is handled in render() when layout_mode is checked
    }

    pub(crate) fn active_camera(&self) -> &CameraState {
        &self.camera_states[self.active_camera_idx]
    }

    pub(crate) fn active_camera_mut(&mut self) -> &mut CameraState {
        &mut self.camera_states[self.active_camera_idx]
    }

    /// Select which tab's camera to use. Called when switching tabs.
    pub(crate) fn select_tab_camera(&mut self, idx: usize) {
        if idx < 3 {
            self.active_camera_idx = idx;
        }
    }

    fn start_ticking(&mut self, cx: &mut Context<Self>) {
        if self.is_ticking {
            return;
        }
        self.is_ticking = true;

        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(16))
                        .await;

                    let mut energy = 0.0;
                    let success = cx.update(|cx| {
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |view, cx| {
                                // Tick instances graph
                                let energy_graph = crate::graph_sim::simulate_step(
                                    &mut view.graph_nodes,
                                    view.dragged_node.as_ref(),
                                );

                                // Tick definitions graph
                                let energy_def = crate::graph_sim::simulate_step(
                                    &mut view.def_nodes,
                                    view.dragged_node.as_ref(),
                                );

                                // Tick outline nodes (was missing — Document Graph never animated)
                                let energy_outline = crate::graph_sim::simulate_step(
                                    &mut view.outline_nodes,
                                    view.dragged_node.as_ref(),
                                );

                                energy = energy_graph + energy_def + energy_outline;
                                cx.notify();
                            });
                            true
                        } else {
                            false
                        }
                    });

                    if !success {
                        break;
                    }

                    if energy < 0.05 {
                        let _ = cx.update(|cx| {
                            if let Some(this) = this.upgrade() {
                                this.update(cx, |view, _| {
                                    view.is_ticking = false;
                                });
                            }
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    fn update_graphs(
        &mut self,
        definitions: Vec<crate::graph_sim::HubgsDefinition>,
        instances: Vec<crate::graph_sim::HubgsInstance>,
        inst_widths: &[f32],
        inst_heights: &[f32],
        def_widths: &[f32],
        def_heights: &[f32],
        cx: &mut Context<Self>,
    ) {
        let mut rng = crate::graph_sim::SimpleRng::new(12345);

        // 1. Build new instance nodes list (preserving existing coordinates)
        let current_map: std::collections::HashMap<&str, &crate::graph_sim::GraphNode> = self
            .graph_nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        let mut next_graph_nodes = Vec::with_capacity(instances.len());
        for (idx, inst) in instances.iter().enumerate() {
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
            let width = inst_widths[idx];
            let height = inst_heights[idx];
            if let Some(existing) = current_map.get(inst.id.as_str()) {
                next_graph_nodes.push(crate::graph_sim::GraphNode {
                    id: inst.id.clone(),
                    name: inst.name.clone(),
                    type_name: inst.type_name.clone(),
                    color: color.into(),
                    x: existing.x,
                    y: existing.y,
                    vx: existing.vx,
                    vy: existing.vy,
                    anchor_x: existing.anchor_x,
                    anchor_y: existing.anchor_y,
                    width,
                    height,
                    attributes: vec![],
                });
            } else {
                next_graph_nodes.push(crate::graph_sim::GraphNode {
                    id: inst.id.clone(),
                    name: inst.name.clone(),
                    type_name: inst.type_name.clone(),
                    color: color.into(),
                    x: rng.range(-50.0, 50.0),
                    y: rng.range(-50.0, 50.0),
                    vx: 0.0,
                    vy: 0.0,
                    anchor_x: rng.range(-50.0, 50.0),
                    anchor_y: rng.range(-50.0, 50.0),
                    width,
                    height,
                    attributes: vec![],
                });
            }
        }
        self.graph_nodes = next_graph_nodes;

        // 2. Build new instances edges list
        let mut relation_arrows = std::collections::HashMap::new();
        for def in &definitions {
            for link in &def.links {
                relation_arrows.insert(link.name.as_str(), link.arrow.as_str());
            }
        }

        let id_to_index: std::collections::HashMap<&str, usize> = instances
            .iter()
            .enumerate()
            .map(|(idx, inst)| (inst.id.as_str(), idx))
            .collect();

        let mut next_graph_edges = Vec::new();
        for (src_idx, inst) in instances.iter().enumerate() {
            for link in &inst.links {
                if let Some(&tgt_idx) = id_to_index.get(link.target.as_str()) {
                    let arrow = relation_arrows
                        .get(link.relation.as_str())
                        .copied()
                        .unwrap_or("-");
                    next_graph_edges.push(crate::graph_sim::GraphEdge {
                        source: src_idx,
                        target: tgt_idx,
                        label: arrow.into(),
                    });
                }
            }
        }
        self.graph_edges = next_graph_edges;

        // 3. Build new definition nodes list (preserving coordinates)
        let current_defs_map: std::collections::HashMap<&str, &crate::graph_sim::GraphNode> = self
            .def_nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        let mut next_def_nodes = Vec::with_capacity(definitions.len());
        for (idx, def) in definitions.iter().enumerate() {
            let color = match def.name.as_str() {
                "Character" => gpui::rgb(0x4169E1),
                "Location" => gpui::rgb(0x2ECC71),
                "Creature" => gpui::rgb(0xE67E22),
                "Item" => gpui::rgb(0x9B59B6),
                _ => gpui::rgb(0x7F8C8D),
            };
            let width = def_widths[idx];
            let height = def_heights[idx];
            if let Some(existing) = current_defs_map.get(def.name.as_str()) {
                next_def_nodes.push(crate::graph_sim::GraphNode {
                    id: def.name.clone(),
                    name: def.name.clone(),
                    type_name: "HubDefinition".into(),
                    color: color.into(),
                    x: existing.x,
                    y: existing.y,
                    vx: existing.vx,
                    vy: existing.vy,
                    anchor_x: existing.anchor_x,
                    anchor_y: existing.anchor_y,
                    width,
                    height,
                    attributes: vec![],
                });
            } else {
                next_def_nodes.push(crate::graph_sim::GraphNode {
                    id: def.name.clone(),
                    name: def.name.clone(),
                    type_name: "HubDefinition".into(),
                    color: color.into(),
                    x: rng.range(-50.0, 50.0),
                    y: rng.range(-50.0, 50.0),
                    vx: 0.0,
                    vy: 0.0,
                    anchor_x: rng.range(-50.0, 50.0),
                    anchor_y: rng.range(-50.0, 50.0),
                    width,
                    height,
                    attributes: vec![],
                });
            }
        }
        self.def_nodes = next_def_nodes;

        // 4. Build new definition edges list
        let name_to_index: std::collections::HashMap<&str, usize> = definitions
            .iter()
            .enumerate()
            .map(|(idx, def)| (def.name.as_str(), idx))
            .collect();

        let mut next_def_edges = Vec::new();
        for (src_idx, def) in definitions.iter().enumerate() {
            for link in &def.links {
                if let Some(&tgt_idx) = name_to_index.get(link.target.as_str()) {
                    next_def_edges.push(crate::graph_sim::GraphEdge {
                        source: src_idx,
                        target: tgt_idx,
                        label: link.arrow.clone(),
                    });
                }
            }
        }
        self.def_edges = next_def_edges;
    }

    /// Run the selected layout type — computes target positions for ALL nodes, then starts physics to animate toward them.
    pub(crate) fn run_layout(&mut self, cx: &mut Context<Self>) {
        // Force restart ticking even if a prior loop got stuck (defense-in-depth).
        self.is_ticking = false;

        let layout_type = self.workspace.read(cx).layout_type;
        let pw = self.pane_content_width.max(400.0);
        let ph = self.pane_content_height.max(300.0);

        // Compute targets for instance nodes (with locks)
        let locked_ids: Vec<SharedString> = self.dragged_node.iter().cloned().collect();
        if !self.graph_nodes.is_empty() {
            let targets = crate::graph_sim::compute_layout_with_locks(
                layout_type,
                &self.graph_nodes,
                &self.graph_edges,
                pw,
                ph,
                &locked_ids,
            );
            for (id, tx, ty) in targets {
                if let Some(node) = self.graph_nodes.iter_mut().find(|n| n.id == id) {
                    node.anchor_x = tx;
                    node.anchor_y = ty;
                }
            }
        }

        // For definition nodes
        let locked_ids: Vec<SharedString> = self.dragged_node.iter().cloned().collect();
        if !self.def_nodes.is_empty() {
            let targets = crate::graph_sim::compute_layout_with_locks(
                layout_type,
                &self.def_nodes,
                &self.def_edges,
                pw,
                ph,
                &locked_ids,
            );
            for (id, tx, ty) in targets {
                if let Some(node) = self.def_nodes.iter_mut().find(|n| n.id == id) {
                    node.anchor_x = tx;
                    node.anchor_y = ty;
                }
            }
        }

        // For outline nodes (empty edges since they're a tree structure)
        let outline_edges_empty: Vec<crate::graph_sim::GraphEdge> = Vec::new();
        let locked_ids: Vec<SharedString> = self.dragged_node.iter().cloned().collect();
        if !self.outline_nodes.is_empty() {
            let targets = crate::graph_sim::compute_layout_with_locks(
                layout_type,
                &self.outline_nodes,
                &outline_edges_empty,
                pw,
                ph,
                &locked_ids,
            );
            for (id, tx, ty) in targets {
                if let Some(node) = self.outline_nodes.iter_mut().find(|n| n.id == id) {
                    node.anchor_x = tx;
                    node.anchor_y = ty;
                }
            }
        }

        self.start_ticking(cx);
        cx.notify();
    }

    fn recalculate_data(&mut self, workspace: &Entity<Workspace>, cx: &mut Context<Self>) {
        let active_doc_text = {
            let w = workspace.read(cx);
            if let Some(idx) = w.active_doc_idx {
                w.open_docs
                    .get(idx)
                    .map(|doc| doc.input_state.read(cx).value().to_string())
            } else {
                None
            }
        };

        let window_handle = self.window_handle; // capture before the async block

        cx.spawn(
            move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    let parse_result = cx
                        .background_executor()
                        .spawn(async move {
                            let workspace_root = crate::utils::resolve_workspace_root()
                                .expect("CARGO_MANIFEST_DIR must resolve to a parent directory");

                            let hubgs_files = crate::utils::find_files_by_extension(
                                &workspace_root,
                                Some("hubgs"),
                            );

                            let mut all_defs = Vec::new();
                            let mut all_instances = Vec::new();

                            for hp in hubgs_files {
                                if let Ok((defs, instances)) =
                                    crate::graph_sim::parse_hubgs_file(&hp)
                                {
                                    all_defs.extend(defs);
                                    all_instances.extend(instances);
                                }
                            }

                            // Deduplicate definitions by name
                            let mut defs_map = std::collections::HashMap::new();
                            for def in all_defs {
                                defs_map.insert(def.name.clone(), def);
                            }
                            let definitions: Vec<_> = defs_map.into_values().collect();

                            // Deduplicate instances by id
                            let mut insts_map = std::collections::HashMap::new();
                            for inst in all_instances {
                                insts_map.insert(inst.id.clone(), inst);
                            }
                            let instances: Vec<_> = insts_map.into_values().collect();

                            // Parse document outline only — no layout yet (needs Window).
                            let outline_nodes_raw = if let Some(ref t) = active_doc_text {
                                crate::parser::parse_document_outline(t)
                            } else {
                                (Vec::new(), Vec::new())
                            };

                            (definitions, instances, outline_nodes_raw)
                        })
                        .await;

                    // Now in UI context — measure node sizes using GPUI text shaping.
                    let _ = this.update(&mut cx, |this, cx| {
                        if let Ok((
                            inst_widths,
                            inst_heights,
                            def_widths,
                            def_heights,
                            outline_nodes,
                            outline_edges,
                        )) = window_handle.update(cx, |_root, window, _app| {
                            let mut sizer = crate::graph_sim::sizing::gpui_text_sizer(window);

                            // Measure instance nodes
                            let mut iw = Vec::with_capacity(parse_result.1.len());
                            let mut ih = Vec::with_capacity(parse_result.1.len());
                            for inst in &parse_result.1 {
                                let (w, h) = sizer(crate::graph_sim::sizing::NodeContent {
                                    name: inst.name.as_ref(),
                                    type_name: inst.type_name.as_ref(),
                                    attributes: &[],
                                });
                                iw.push(w);
                                ih.push(h);
                            }

                            // Measure definition nodes
                            let mut dw = Vec::with_capacity(parse_result.0.len());
                            let mut dh = Vec::with_capacity(parse_result.0.len());
                            for def in &parse_result.0 {
                                let (w, h) = sizer(crate::graph_sim::sizing::NodeContent {
                                    name: def.name.as_ref(),
                                    type_name: "HubDefinition",
                                    attributes: &[],
                                });
                                dw.push(w);
                                dh.push(h);
                            }

                            // Layout outline tree with real text measurement.
                            let (nodes_raw, edges_raw) = parse_result.2;
                            let (out_nodes, out_edges) = if nodes_raw.is_empty() {
                                (Vec::new(), Vec::new())
                            } else {
                                super::render::layout_outline_tree_with_sizer(
                                    &nodes_raw, &edges_raw, 500.0, 500.0, &mut sizer,
                                )
                            };

                            (iw, ih, dw, dh, out_nodes, out_edges)
                        }) {
                            this.update_graphs(
                                parse_result.0,
                                parse_result.1,
                                &inst_widths,
                                &inst_heights,
                                &def_widths,
                                &def_heights,
                                cx,
                            );

                            this.outline_nodes = outline_nodes;
                            this.outline_edges = outline_edges;

                            cx.notify();
                        }
                    });
                }
            },
        )
        .detach();
    }
}
