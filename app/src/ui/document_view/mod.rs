pub(crate) mod block_editor;
#[cfg(test)]
mod block_editor_tests;
mod collapsible;
mod expansion_state;
pub(crate) mod jump_links;
#[cfg(test)]
mod jump_links_tests;
mod renderers;

use crate::ui::{DocumentHome, DocumentMode, Workspace};
use expansion_state::ExpandedBlocks;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::{
    prelude::*, px, uniform_list, AnyElement, Context, Entity, InteractiveElement, ParentElement,
    Render, Styled, Window,
};
use std::collections::HashMap;

pub(crate) struct DocumentView {
    workspace: Entity<Workspace>,
    document_home: Entity<DocumentHome>,
    #[allow(dead_code)]
    editor_state: Entity<gpui_kit::component::input::EditorState>,
    #[allow(dead_code)]
    input_state: Entity<gpui_kit::component::input::InputState>,
    pub(crate) expanded_blocks: Entity<ExpandedBlocks>,
    pub(crate) focused_block_idx: Option<usize>,
    pub(crate) block_input_states: HashMap<usize, Entity<gpui_kit::component::input::InputState>>,
}

impl DocumentView {
    pub(crate) fn new(
        workspace: Entity<Workspace>,
        document_home: Entity<DocumentHome>,
        editor_state: Entity<gpui_kit::component::input::EditorState>,
        input_state: Entity<gpui_kit::component::input::InputState>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&document_home, |_, _, cx| cx.notify()).detach();
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        Self {
            workspace,
            document_home,
            editor_state,
            input_state,
            expanded_blocks: cx.new(|_cx| ExpandedBlocks::default()),
            focused_block_idx: None,
            block_input_states: HashMap::new(),
        }
    }
}

impl Render for DocumentView {
    #[allow(refining_impl_trait)]
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let theme_val = gpui_kit::component::Theme::global(cx).clone();
        let sidebar_bg = theme_val.sidebar;
        let border_color = theme_val.border;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_group_box = theme_val.group_box;
        let theme_foreground = theme_val.foreground;

        // 1. Resolve the active document tab from the workspace
        let workspace_entity = self.workspace.clone();
        let (active_doc_path, active_doc_mode, doc_home_opt, active_editor_state_opt, active_input_state_opt) = {
            let w = workspace_entity.read(cx);
            if let Some(idx) = w.active_doc_idx {
                if let Some(doc) = w.open_docs.get(idx) {
                    (
                        Some(doc.path.clone()),
                        Some(doc.mode),
                        Some(doc.document_home.clone()),
                        Some(doc.editor_state.clone()),
                        Some(doc.input_state.clone()),
                    )
                } else {
                    (None, None, None, None, None)
                }
            } else {
                (None, None, None, None, None)
            }
        };

        let (doc_home, editor_state, input_state, mode) =
            match (doc_home_opt, active_editor_state_opt, active_input_state_opt, active_doc_mode) {
                (Some(dh), Some(es), Some(is), Some(m)) => (dh, es, is, m),
                _ => {
                    return gpui_kit::div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(theme_val.background)
                        .text_color(theme_muted_foreground)
                        .child(
                            "No document open. Select a file from the sidebar explorer to open it.",
                        )
                        .into_any_element();
                }
            };

        // Extract diagnostics and selected path
        let diagnostics = {
            let w = workspace_entity.read(cx);
            w.diagnostics.clone()
        };

        // LSP Diagnostics list builder
        let make_diagnostics_content = |scroll_id: &'static str| -> AnyElement {
            if diagnostics.is_empty() {
                gpui_kit::div()
                    .text_color(theme_val.success)
                    .text_size(gpui_kit::px(12.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new(IconName::CircleCheck).size(gpui_kit::px(14.)))
                    .child("No diagnostic issues found.")
                    .into_any_element()
            } else {
                let editor_state_clone = editor_state.clone();
                let theme_val_clone = theme_val.clone();
                let theme_foreground_clone = theme_foreground;
                uniform_list(
                    scroll_id,
                    diagnostics.len(),
                    cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                        let workspace = this.workspace.read(cx);
                        let diagnostics = &workspace.diagnostics;
                        range
                            .map(|idx| {
                                let diag = &diagnostics[idx];
                                let is_error = diag.severity == 1;
                                let severity_icon = if is_error { "🔴" } else { "🟡" };
                                let color = if is_error {
                                    theme_val_clone.danger
                                } else {
                                    theme_val_clone.warning
                                };
                                let line_val = diag.line + 1;
                                let message = diag.message.clone();
                                let editor_state = editor_state_clone.clone();
                                let diag_line = diag.line;
                                gpui_kit::div()
                                    .id(("diag", idx))
                                    .flex()
                                    .gap_2()
                                    .py_1()
                                    .px_2()
                                    .rounded(gpui_kit::px(4.))
                                    .text_size(gpui_kit::px(11.))
                                    .text_color(theme_foreground_clone)
                                    .hover(|s| s.bg(theme_val_clone.accent.opacity(0.5)))
                                    .on_mouse_down(
                                        gpui_kit::MouseButton::Left,
                                        move |_, window, cx| {
                                            let pos = gpui_kit::component::input::Position::new(
                                                diag_line as u32,
                                                0,
                                            );
                                            editor_state.update(cx, |state, cx| {
                                                state.set_cursor_position(pos, window, cx);
                                            });
                                        },
                                    )
                                    .child(gpui_kit::div().text_color(color).child(severity_icon))
                                    .child(
                                        gpui_kit::div()
                                            .font_weight(gpui_kit::FontWeight::BOLD)
                                            .child(format!("Line {}:", line_val)),
                                    )
                                    .child(gpui_kit::div().child(message))
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .flex_1()
                .into_any_element()
            }
        };

        let active_file = active_doc_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "No File".to_string());
        let editor_header = format!("RAW EDITOR: {}", active_file);

        // Hoisted reads for document model
        let doc_home_borrow = doc_home.read(cx);
        let parse_state_clone = doc_home_borrow.parse_state.clone();
        let blocks_clone = doc_home_borrow.blocks.clone();
        let metadata_clone = doc_home_borrow.metadata.clone();
        let hubgs_clone = doc_home_borrow.hubgs_instances.clone();
        let _ = doc_home_borrow;

        let footnote_map = crate::ui::document_view::renderers::build_footnote_map(&blocks_clone);
        let _parse_state = &parse_state_clone;
        let blocks = &blocks_clone;

        // Frontmatter builder
        let make_frontmatter_el = || -> Option<gpui_kit::Div> {
            let mut frontmatter = String::new();
            if !metadata_clone.is_empty() {
                frontmatter.push_str("---\n");
                for (key, val) in &metadata_clone {
                    frontmatter.push_str(&format!("{}: {}\n", key, val));
                }
                frontmatter.push_str("---");
            }
            if !frontmatter.is_empty() {
                Some(
                    gpui_kit::div()
                        .mb_4()
                        .p_3()
                        .bg(sidebar_bg)
                        .border(gpui_kit::px(1.))
                        .border_color(border_color)
                        .rounded(gpui_kit::px(4.))
                        .font_family("Courier New")
                        .text_xs()
                        .text_color(theme_foreground)
                        .child(frontmatter),
                )
            } else {
                None
            }
        };

        // 1. Raw Editor Panel
        let editor_panel = gpui_kit::component::resizable::v_resizable("editor-diagnostics")
            .child(
                gpui_kit::component::resizable::resizable_panel().child(
                    gpui_kit::div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .child(
                            gpui_kit::div()
                                .p_2()
                                .bg(sidebar_bg)
                                .border_b(gpui_kit::px(1.))
                                .border_color(border_color)
                                .text_xs()
                                .font_weight(gpui_kit::FontWeight::BOLD)
                                .text_color(theme_muted_foreground)
                                .child(editor_header),
                        )
                        .child(
                            gpui_kit::div()
                                .id("source_editor_container")
                                .flex_1()
                                .h(gpui_kit::px(0.))
                                .p_4()
                                .bg(theme_group_box)
                                .child(
                                    gpui_kit::div()
                                        .size_full()
                                        .flex()
                                        .flex_col()
                                        .children(make_frontmatter_el())
                                        .child(
                                            gpui_kit::component::input::Editor::new(&editor_state)
                                                .flex_1()
                                                .w_full()
                                                .bordered(false)
                                                .aria_label("Source Editor"),
                                        ),
                                ),
                        ),
                ),
            )
            .child(
                gpui_kit::component::resizable::resizable_panel()
                    .size(gpui_kit::px(180.))
                    .size_range(gpui_kit::px(80.)..gpui_kit::px(400.))
                    .child(
                        gpui_kit::div()
                            .size_full()
                            .border_t(gpui_kit::px(1.))
                            .border_color(border_color)
                            .bg(sidebar_bg)
                            .flex()
                            .flex_col()
                            .child(
                                gpui_kit::div()
                                    .p_2()
                                    .bg(sidebar_bg)
                                    .border_b(gpui_kit::px(1.))
                                    .border_color(border_color)
                                    .text_xs()
                                    .font_weight(gpui_kit::FontWeight::BOLD)
                                    .text_color(theme_muted_foreground)
                                    .child("LSP DIAGNOSTICS"),
                            )
                            .child(
                                gpui_kit::div()
                                    .id("diagnostics_list")
                                    .flex_1()
                                    .overflow_hidden()
                                    .p_2()
                                    .child(make_diagnostics_content("diagnostics_scroll_raw")),
                            ),
                    ),
            );

        // 2. Block Editor View (using Raw Editor layout container & frontmatter)
        let block_editor_area = block_editor::render_block_editor(
            &self.workspace,
            &self.document_home,
            input_state.clone(),
            &self.expanded_blocks,
            blocks,
            &active_file,
            self.focused_block_idx,
            &self.block_input_states,
            &hubgs_clone,
            &footnote_map,
            make_frontmatter_el(),
            make_diagnostics_content("diagnostics_scroll_block"),
            &diagnostics,
            cx,
        );

        // 3. Markdown read-only panel
        let markdown_text = crate::parser::blocks_to_markdown(blocks);
        let markdown_preview = {
            let mut preview_content = gpui_kit::div()
                .flex_1()
                .h(gpui_kit::px(0.))
                .p_8()
                .bg(theme_val.background)
                .text_color(theme_val.foreground)
                .font_family("Courier New")
                .text_size(px(13.))
                .overflow_y_scrollbar();

            for line in markdown_text.lines() {
                preview_content =
                    preview_content.child(gpui_kit::div().min_h(px(18.)).child(line.to_string()));
            }

            gpui_kit::div()
                .size_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    gpui_kit::div()
                        .flex_none()
                        .p_2()
                        .bg(sidebar_bg)
                        .border_b(gpui_kit::px(1.))
                        .border_color(border_color)
                        .text_xs()
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .text_color(theme_muted_foreground)
                        .child(format!("MARKDOWN VIEW: {}", active_file)),
                )
                .child(preview_content)
        };

        // 4. FlowText Editor stub panel (Experimental / Coming Soon)
        let flow_text_stub = {
            let stub_content = gpui_kit::div()
                .flex_1()
                .h(gpui_kit::px(0.))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .p_8()
                .bg(theme_val.background)
                .text_color(theme_val.foreground)
                .child(
                    gpui_kit::div()
                        .text_lg()
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .child("FlowText Editor (Experimental / Coming Soon)"),
                )
                .child(
                    gpui_kit::div()
                        .text_sm()
                        .text_color(theme_muted_foreground)
                        .child("Character-level rich text editing powered by `gpui-flowtext` is under development. Shares the underlying rope buffer."),
                );

            gpui_kit::div()
                .size_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    gpui_kit::div()
                        .flex_none()
                        .p_2()
                        .bg(sidebar_bg)
                        .border_b(gpui_kit::px(1.))
                        .border_color(border_color)
                        .text_xs()
                        .font_weight(gpui_kit::FontWeight::BOLD)
                        .text_color(theme_muted_foreground)
                        .child(format!("FLOWTEXT EDITOR STUB: {}", active_file)),
                )
                .child(stub_content)
        };

        // Match based on active doc mode
        let content_pane = match mode {
            DocumentMode::RawEditor => editor_panel.into_any_element(),
            DocumentMode::BlockEditor => block_editor_area,
            DocumentMode::MarkdownView => markdown_preview.into_any_element(),
            DocumentMode::FlowTextEditor => flow_text_stub.into_any_element(),
        };

        gpui_kit::div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(content_pane)
            .into_any_element()
    }
}
