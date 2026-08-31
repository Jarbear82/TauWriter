use gpui::prelude::*;
use gpui::{div, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui_component::dock::{Panel, PanelEvent};

use crate::ui::sidebar::SidebarView;

pub(crate) struct FilesPanel {
    pub(crate) sidebar: Entity<SidebarView>,
    pub(crate) focus_handle: FocusHandle,
}

impl FilesPanel {
    pub(crate) fn new(sidebar: Entity<SidebarView>, cx: &mut Context<Self>) -> Self {
        Self {
            sidebar,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for FilesPanel {}

impl Focusable for FilesPanel {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for FilesPanel {
    fn panel_name(&self) -> &'static str {
        "FilesPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Workspace Files"
    }
}

impl Render for FilesPanel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.sidebar.clone())
    }
}
