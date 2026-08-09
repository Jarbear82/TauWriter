//! Stateful viewer for graph panes — GraphPaneView struct definition + business logic.

use std::collections::HashMap;
use gpui::{prelude::*, Entity};
use graphene_core::{math::Vec2, GraphState, NodeId};
use graphene_gpui::render::draw_pipeline::Viewport;
use graphene_gpui::{ExpansionState, GraphCanvasController, GraphView, InteractionState};
use graphene_style::ComputedStyle;

use crate::graph_adapter::{
    hubgs_definitions_to_graph_state, hubgs_instances_to_graph_state, outline_to_graph_state,
};
use crate::ui::Workspace;

/// Stateful viewer that owns all graph states, views, camera states, layout mode, and library interaction controllers.
#[derive(Debug, Clone)]
pub(crate) struct GraphPaneView {
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) window_handle: gpui::AnyWindowHandle,

    // Three graph tab states & views (0=Document Outline, 1=Definitions Schema, 2=Instances Relation)
    pub(crate) outline_state: GraphState<ComputedStyle>,
    pub(crate) outline_view: GraphView<ComputedStyle>,

    pub(crate) def_state: GraphState<ComputedStyle>,
    pub(crate) def_view: GraphView<ComputedStyle>,

    pub(crate) instances_state: GraphState<ComputedStyle>,
    pub(crate) instances_view: GraphView<ComputedStyle>,

    // Id mapping tables (external string ID -> graphene NodeId)
    pub(crate) outline_id_map: HashMap<String, NodeId>,
    pub(crate) def_id_map: HashMap<String, NodeId>,
    pub(crate) inst_id_map: HashMap<String, NodeId>,

    pub(crate) is_ticking: bool,
    pub(crate) layout_mode: super::data::LayoutMode,

    /// Library interaction state & controller (persistent per pane view)
    pub(crate) controller: GraphCanvasController,
    pub(crate) interaction_state: InteractionState,
    pub(crate) expansion_state: ExpansionState,

    pub(crate) selected_node: Option<NodeId>,

    pub(crate) auto_node_colors: bool,
    pub(crate) auto_edge_colors: bool,
    pub(crate) wcag_contrast_auto: bool,

    // Per-tab camera state [0]=Document, [1]=Definitions, [2]=Instances.
    pub(crate) camera_states: [CameraState; 3],
    pub(crate) active_camera_idx: usize,

    // Real pane content-box bounds in window space (set by parent via bounds callback)
    pub(crate) pane_bounds: gpui::Bounds<f32>,
}

/// Per-tab camera state so each graph pane has independent pan/zoom.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CameraState {
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) zoom: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl gpui::EventEmitter<super::data::GraphEvent> for GraphPaneView {}

impl GraphPaneView {
    pub(crate) fn new(
        workspace: Entity<Workspace>,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            workspace: workspace.clone(),
            window_handle: window.window_handle(),

            outline_state: GraphState::new(),
            outline_view: GraphView::new(),

            def_state: GraphState::new(),
            def_view: GraphView::new(),

            instances_state: GraphState::new(),
            instances_view: GraphView::new(),

            outline_id_map: HashMap::new(),
            def_id_map: HashMap::new(),
            inst_id_map: HashMap::new(),

            is_ticking: false,
            layout_mode: super::data::LayoutMode::default(),
            controller: GraphCanvasController::new(),
            interaction_state: InteractionState::new(64.0),
            expansion_state: ExpansionState::new(),
            selected_node: None,
            auto_node_colors: true,
            auto_edge_colors: true,
            wcag_contrast_auto: true,
            camera_states: [CameraState::default(); 3],
            active_camera_idx: 0,
            pane_bounds: gpui::Bounds::default(),
        };
        // Load graph data on creation
        this.recalculate_data(&workspace, cx);
        this
    }

    pub(crate) fn active_viewport(&self) -> Viewport {
        let cam = self.active_camera();
        let bounds = if self.pane_bounds.size.width <= 0.0 || self.pane_bounds.size.height <= 0.0 {
            gpui::Bounds {
                origin: gpui::point(0.0, 0.0),
                size: gpui::size(100.0, 100.0),
            }
        } else {
            self.pane_bounds
        };
        Viewport {
            offset: Vec2::new(cam.offset_x, cam.offset_y),
            zoom: cam.zoom,
            bounds,
        }
    }

    pub(crate) fn write_viewport_to_camera(&mut self, vp: &Viewport) {
        let cam = self.active_camera_mut();
        cam.offset_x = vp.offset.x;
        cam.offset_y = vp.offset.y;
        cam.zoom = vp.zoom;
    }

    /// Rebuild interaction state spatial grid for the active tab's view
    pub(crate) fn rebuild_interaction_grid(&mut self) {
        let view = match self.active_camera_idx {
            0 => &self.outline_view,
            1 => &self.def_view,
            _ => &self.instances_view,
        };
        self.interaction_state.rebuild_grid(view);
    }

    pub(crate) fn fit_view(&mut self, cx: &mut Context<Self>) {
        let view = match self.active_camera_idx {
            0 => &self.outline_view,
            1 => &self.def_view,
            _ => &self.instances_view,
        };
        let mut vp = self.active_viewport();
        vp.fit_to_graph(view);
        self.write_viewport_to_camera(&vp);
        cx.notify();
    }

    pub(crate) fn trigger_run_layout(&mut self, cx: &mut Context<Self>) {
        let layout_type = self.workspace.read(cx).layout_type;
        self.layout_mode = super::data::LayoutMode::RunLayout(layout_type);
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

    /// Helper to get active state and view pair
    pub(crate) fn active_state_and_view(
        &mut self,
    ) -> (
        &mut GraphState<ComputedStyle>,
        &mut GraphView<ComputedStyle>,
    ) {
        match self.active_camera_idx {
            0 => (&mut self.outline_state, &mut self.outline_view),
            1 => (&mut self.def_state, &mut self.def_view),
            _ => (&mut self.instances_state, &mut self.instances_view),
        }
    }

    fn start_ticking(&mut self, cx: &mut Context<Self>) {
        if self.is_ticking {
            return;
        }
        self.is_ticking = true;

        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let mut sim_instances = graphene_layout::livesim::LiveForceSimulation::new();
                let mut sim_defs = graphene_layout::livesim::LiveForceSimulation::new();
                let mut sim_outline = graphene_layout::livesim::LiveForceSimulation::new();

                for _step in 0..150 {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(16))
                        .await;

                    let success = cx.update(|cx| {
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |view, cx| {
                                sim_instances.tick(&mut view.instances_state);
                                view.instances_view.load_preset(&view.instances_state);

                                sim_defs.tick(&mut view.def_state);
                                view.def_view.load_preset(&view.def_state);

                                sim_outline.tick(&mut view.outline_state);
                                view.outline_view.load_preset(&view.outline_state);

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
                }

                let _ = cx.update(|cx| {
                    if let Some(this) = this.upgrade() {
                        this.update(cx, |view, _| {
                            view.is_ticking = false;
                        });
                    }
                });
            }
        })
        .detach();
    }

    /// Run the selected layout algorithm on active or all graph states.
    pub(crate) fn run_layout(&mut self, cx: &mut Context<Self>) {
        self.is_ticking = false;
        let layout_type = self.workspace.read(cx).layout_type;

        use graphene_layout::basic::{CircleLayout, GridLayout};
        use graphene_layout::cose::CoseLayout;
        use graphene_layout::force::ForceDirectedLayout;
        use graphene_layout::hierarchical::SugiyamaLayout;
        use graphene_layout::traits::Layout;

        match layout_type {
            crate::ui::LayoutType::Circular => {
                let mut layout = CircleLayout::default();
                layout.compute(&mut self.instances_state);
                layout.compute(&mut self.def_state);
                layout.compute(&mut self.outline_state);
            }
            crate::ui::LayoutType::Grid => {
                let mut layout = GridLayout::default();
                layout.compute(&mut self.instances_state);
                layout.compute(&mut self.def_state);
                layout.compute(&mut self.outline_state);
            }
            crate::ui::LayoutType::ForceDirected => {
                let mut force_layout = ForceDirectedLayout::default().with_iterations(120);
                force_layout.compute(&mut self.instances_state);
                force_layout.compute(&mut self.def_state);

                let mut tree_layout = SugiyamaLayout::default();
                tree_layout.compute(&mut self.outline_state);
            }
            crate::ui::LayoutType::Sugiyama => {
                let mut tree_layout = SugiyamaLayout::default();
                tree_layout.compute(&mut self.outline_state);
                tree_layout.compute(&mut self.def_state);
                tree_layout.compute(&mut self.instances_state);
            }
            crate::ui::LayoutType::Cose => {
                let mut cose_layout = CoseLayout::default();
                cose_layout.compute(&mut self.instances_state);
                cose_layout.compute(&mut self.def_state);
                cose_layout.compute(&mut self.outline_state);
            }
        }

        self.instances_view.load_preset(&self.instances_state);
        self.def_view.load_preset(&self.def_state);
        self.outline_view.load_preset(&self.outline_state);
        self.rebuild_interaction_grid();

        self.start_ticking(cx);
        cx.notify();
    }

    pub(crate) fn recalculate_data(&mut self, workspace: &Entity<Workspace>, cx: &mut Context<Self>) {
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

        let window_handle = self.window_handle;

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

                            let outline_raw = if let Some(ref t) = active_doc_text {
                                crate::parser::parse_document_outline(t)
                            } else {
                                (Vec::new(), Vec::new())
                            };

                            (definitions, instances, outline_raw)
                        })
                        .await;

                    // Measure node sizes & build GraphState structures
                    let _ = this.update(&mut cx, |this, cx| {
                        if let Ok((
                            inst_state,
                            inst_map,
                            def_state,
                            def_map,
                            out_state,
                            out_map,
                        )) = window_handle.update(cx, |_root, window, _app| {
                            let mut sizer = crate::graph_sim::sizing::gpui_text_sizer(window);

                            let (inst_state, inst_map) = hubgs_instances_to_graph_state(
                                &parse_result.1,
                                &parse_result.0,
                                &mut sizer,
                            );

                            let (def_state, def_map) = hubgs_definitions_to_graph_state(
                                &parse_result.0,
                                &mut sizer,
                            );

                            let (out_nodes, out_edges) = parse_result.2;
                            let (out_state, out_map) = outline_to_graph_state(
                                &out_nodes,
                                &out_edges,
                                &mut sizer,
                            );

                            (
                                inst_state, inst_map, def_state, def_map, out_state, out_map,
                            )
                        }) {
                            this.instances_view = GraphView::from_state(&inst_state);
                            this.instances_state = inst_state;
                            this.inst_id_map = inst_map;

                            this.def_view = GraphView::from_state(&def_state);
                            this.def_state = def_state;
                            this.def_id_map = def_map;

                            this.outline_view = GraphView::from_state(&out_state);
                            this.outline_state = out_state;
                            this.outline_id_map = out_map;

                            this.rebuild_interaction_grid();
                            cx.notify();
                        }
                    });
                }
            },
        )
        .detach();
    }
}
