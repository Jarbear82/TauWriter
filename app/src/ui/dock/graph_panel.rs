use gpui::prelude::*;
use gpui::{div, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui_component::button::{Button, ButtonGroup, DropdownButton as GpuiDropdownButton};
use gpui_component::dock::{Panel, PanelEvent};
use gpui_component::menu::PopupMenuItem;
use gpui_component::IconName;

use crate::ui::graph_pane::GraphPaneView;
use crate::ui::{GraphTab, LayoutType, MainView, Workspace};

pub(crate) struct GraphPanel {
    pub(crate) graph_pane: Entity<GraphPaneView>,
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) main_view: Entity<MainView>,
    pub(crate) focus_handle: FocusHandle,
}

impl GraphPanel {
    pub(crate) fn new(
        graph_pane: Entity<GraphPaneView>,
        workspace: Entity<Workspace>,
        main_view: Entity<MainView>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        Self {
            graph_pane,
            workspace,
            main_view,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for GraphPanel {}

impl Focusable for GraphPanel {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for GraphPanel {
    fn panel_name(&self) -> &'static str {
        "GraphPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Knowledge Graph"
    }
}

impl Render for GraphPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_val = gpui_component::Theme::global(cx);
        let border_color = theme_val.border;

        let workspace = self.workspace.read(cx);
        let active_graph_tab = workspace.active_graph_tab;
        let selected_graph_index = match active_graph_tab {
            GraphTab::DocumentGraph => 0,
            GraphTab::DefinitionsSchema => 1,
            GraphTab::InstancesRelation => 2,
        };

        let view_clone_graph = self.main_view.clone();
        let graph_tab_configs = vec![
            ("Document Graph", IconName::File),
            ("Definitions Schema", IconName::LayoutDashboard),
            ("Instances Relation", IconName::Network),
        ];

        let graph_tab_bar = gpui_component::tab::TabBar::new("graph-tab-bar")
            .selected_index(selected_graph_index)
            .on_click(move |index, _, cx| {
                let tab = match index {
                    0 => GraphTab::DocumentGraph,
                    1 => GraphTab::DefinitionsSchema,
                    2 => GraphTab::InstancesRelation,
                    _ => return,
                };
                view_clone_graph.update(cx, |this, cx| {
                    this.workspace.update(cx, |w, cx| {
                        w.active_graph_tab = tab;
                        cx.notify();
                    });
                    cx.notify();
                });
            })
            .children(
                graph_tab_configs
                    .into_iter()
                    .map(|(label, icon)| gpui_component::tab::Tab::new().icon(icon).label(label)),
            );

        let current_layout_type = workspace.layout_type;
        let graph_pane_for_layout = self.graph_pane.clone();
        let graph_pane_for_dropdown = graph_pane_for_layout.clone();

        let layout_label = match current_layout_type {
            LayoutType::ForceDirected => "Force",
            LayoutType::Sugiyama => "Tree",
            LayoutType::Cose => "Compound",
            LayoutType::Circular => "Circle",
            LayoutType::Grid => "Grid",
        };

        let layout_selector_bar = div()
            .flex()
            .items_center()
            .gap_1p5()
            .px_2()
            .py_1()
            .border_b(gpui::px(1.))
            .border_color(border_color)
            .child(
                GpuiDropdownButton::new("layout-mode-selector")
                    .button(Button::new("layout-btn").label(layout_label))
                    .dropdown_menu(move |menu, _event, _cx| {
                        let graph_pane = graph_pane_for_dropdown.clone();
                        [
                            ("Force (ForceAtlas2)", LayoutType::ForceDirected),
                            ("Tree (Sugiyama)", LayoutType::Sugiyama),
                            ("Compound (CoSE)", LayoutType::Cose),
                            ("Circle", LayoutType::Circular),
                            ("Grid", LayoutType::Grid),
                        ]
                        .into_iter()
                        .fold(menu, |menu, (label, layout_type)| {
                            let graph_pane = graph_pane.clone();
                            menu.item(PopupMenuItem::new(label).on_click(
                                move |_event, _window, cx| {
                                    let _ = graph_pane.update(cx, |pane, cx| {
                                        pane.workspace.update(cx, |w, _| {
                                            w.layout_type = layout_type;
                                        });
                                    });
                                },
                            ))
                        })
                    }),
            )
            .child(
                ButtonGroup::new("button-group")
                    .child(Button::new("run-layout").label("Run Layout").on_click(
                        move |_, _, cx| {
                            let _ =
                                graph_pane_for_layout.update(cx, |pane, cx| pane.run_layout(cx));
                        },
                    ))
                    .child({
                        let pane_entity = self.graph_pane.clone();
                        let auto_colors = self.graph_pane.read(cx).auto_node_colors;
                        Button::new("toggle_auto_colors")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                let _ = pane_entity.update(cx, |this, cx| {
                                    this.auto_node_colors = !this.auto_node_colors;
                                    this.auto_edge_colors = this.auto_node_colors;
                                    cx.notify();
                                });
                            })
                            .label(if auto_colors {
                                "Auto Color: ON"
                            } else {
                                "Auto Color: OFF"
                            })
                    })
                    .child({
                        let pane_entity = self.graph_pane.clone();
                        let is_ticking = self.graph_pane.read(cx).is_ticking;
                        Button::new("toggle_physics")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                let _ = pane_entity.update(cx, |this, cx| {
                                    if this.is_ticking {
                                        this.is_ticking = false;
                                    } else {
                                        this.run_layout(cx);
                                    }
                                });
                            })
                            .label(if is_ticking {
                                "Pause Physics"
                            } else {
                                "Play Physics"
                            })
                    })
                    .child({
                        let pane_entity = self.graph_pane.clone();
                        Button::new("fit_view")
                            .on_mouse_down(gpui::MouseButton::Left, move |_ev, _window, cx| {
                                let _ = pane_entity.update(cx, |this, cx| {
                                    this.fit_view(cx);
                                });
                            })
                            .label("Fit View")
                    }),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(graph_tab_bar)
            .child(layout_selector_bar)
            .child(
                div()
                    .flex_1()
                    .h(gpui::px(0.))
                    .child(self.graph_pane.clone()),
            )
    }
}
