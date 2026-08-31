use gpui::prelude::*;
use gpui::{div, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui_component::dock::{Panel, PanelEvent};

use crate::ui::document_tabs;
use crate::ui::document_view::DocumentView;
use crate::ui::mode_dropdown;
use crate::ui::{MainView, Workspace};

pub(crate) struct EditorPanel {
    pub(crate) document_view: Entity<DocumentView>,
    pub(crate) workspace: Entity<Workspace>,
    pub(crate) main_view: Entity<MainView>,
    pub(crate) focus_handle: FocusHandle,
}

impl EditorPanel {
    pub(crate) fn new(
        document_view: Entity<DocumentView>,
        workspace: Entity<Workspace>,
        main_view: Entity<MainView>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        Self {
            document_view,
            workspace,
            main_view,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl EventEmitter<PanelEvent> for EditorPanel {}

impl Focusable for EditorPanel {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for EditorPanel {
    fn panel_name(&self) -> &'static str {
        "EditorPanel"
    }

    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Document Editor"
    }
}

impl Render for EditorPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_val = gpui_component::Theme::global(cx);
        let fg_color = theme_val.foreground;
        let border_color = theme_val.border;
        let sidebar_bg = theme_val.sidebar;
        let theme_muted_foreground = theme_val.muted_foreground;

        let workspace = self.workspace.read(cx);
        let active_doc_idx = workspace.active_doc_idx;

        let mut doc_tab_bar = document_tabs::render_doc_tab_bar(
            theme_val.background,
            sidebar_bg,
            border_color,
            fg_color,
            theme_muted_foreground,
            &workspace.open_docs,
            active_doc_idx,
            self.main_view.clone(),
        );

        if let Some(idx) = active_doc_idx {
            if workspace.open_docs.get(idx).is_some() {
                doc_tab_bar = doc_tab_bar.child(div().px_3().flex().items_center().child(
                    mode_dropdown::render_mode_selector(
                        workspace.open_docs[idx].mode,
                        idx,
                        self.main_view.clone(),
                    ),
                ));
            }
        }

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(doc_tab_bar)
            .child(
                div()
                    .flex_1()
                    .h(gpui::px(0.))
                    .w_full()
                    .overflow_hidden()
                    .child(self.document_view.clone()),
            )
    }
}
