mod collapsible;
mod expansion_state;
pub(crate) mod jump_links;
#[cfg(test)]
mod jump_links_tests;
mod renderers;

use crate::parser::Block;
use crate::ui::{DocumentHome, DocumentMode, ParseState, Workspace};
use expansion_state::ExpandedBlocks;
use gpui::{
    div, prelude::*, px, uniform_list, Context, Entity, InteractiveElement, ParentElement, Render,
    Styled, Window,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{alert::Alert, Icon, IconName};
use once_cell::sync::Lazy;

/// Parse error warning colors (soft red palette).
static PARSE_ERROR_BG: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.85, 0.95, 1.0));
static PARSE_ERROR_BORDER: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.82, 0.92, 1.0));

pub(crate) struct DocumentView {
    workspace: Entity<Workspace>,
    document_home: Entity<DocumentHome>,
    input_state: Entity<gpui_component::input::InputState>,
    pub(crate) expanded_blocks: Entity<ExpandedBlocks>,
}

impl DocumentView {
    pub(crate) fn new(
        workspace: Entity<Workspace>,
        document_home: Entity<DocumentHome>,
        input_state: Entity<gpui_component::input::InputState>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&document_home, |_, _, cx| cx.notify()).detach();
        cx.observe(&workspace, |_, _, cx| cx.notify()).detach();
        Self {
            workspace,
            document_home,
            input_state,
            expanded_blocks: cx.new(|_cx| ExpandedBlocks::default()),
        }
    }

    /// Handle a hub reference click by propagating to the workspace for graph pane coordination.
    pub(crate) fn on_hubref_clicked(
        &mut self,
        hub_id: gpui::SharedString,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let _ = {
            let w = self.workspace.read(cx);
            if let Some(idx) = w.active_doc_idx {
                if let Some(doc) = w.open_docs.get(idx) {
                    (doc.path.clone(), doc.input_state.clone())
                } else {
                    return;
                }
            } else {
                return;
            }
        };
        // Update workspace selected hub so the graph pane can center on it
        self.workspace.update(cx, |w, cx| {
            w.selected_hub_id = Some(hub_id);
            cx.notify();
        });
    }
}

impl Render for DocumentView {
    #[allow(refining_impl_trait)]
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme_val = gpui_component::Theme::global(cx).clone();
        let sidebar_bg = theme_val.sidebar;
        let border_color = theme_val.border;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_group_box = theme_val.group_box;
        let theme_foreground = theme_val.foreground;

        // 1. Resolve the active document tab from the workspace
        let workspace_entity = self.workspace.clone();
        let (active_doc_path, active_doc_mode, doc_home_opt, active_input_state_opt) = {
            let w = workspace_entity.read(cx);
            if let Some(idx) = w.active_doc_idx {
                if let Some(doc) = w.open_docs.get(idx) {
                    (
                        Some(doc.path.clone()),
                        Some(doc.mode),
                        Some(doc.document_home.clone()),
                        Some(doc.input_state.clone()),
                    )
                } else {
                    (None, None, None, None)
                }
            } else {
                (None, None, None, None)
            }
        };

        let (doc_home, input_state, mode) =
            match (doc_home_opt, active_input_state_opt, active_doc_mode) {
                (Some(dh), Some(is), Some(m)) => (dh, is, m),
                _ => {
                    return gpui::div()
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

        // LSP Diagnostics list
        let diagnostics_content = if diagnostics.is_empty() {
            gpui::div()
                .text_color(theme_val.success)
                .text_size(gpui::px(12.))
                .flex()
                .items_center()
                .gap_2()
                .child(Icon::new(IconName::CircleCheck).size(gpui::px(14.)))
                .child("No diagnostic issues found.")
                .into_any_element()
        } else {
            let input_state_clone = input_state.clone();
            let theme_val_clone = theme_val.clone();
            let theme_foreground_clone = theme_foreground;
            uniform_list(
                "diagnostics_scroll",
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
                            let input_state = input_state_clone.clone();
                            let diag_line = diag.line;
                            gpui::div()
                                .id(("diag", idx))
                                .flex()
                                .gap_2()
                                .py_1()
                                .px_2()
                                .rounded(gpui::px(4.))
                                .text_size(gpui::px(11.))
                                .text_color(theme_foreground_clone)
                                .hover(|s| s.bg(theme_val_clone.accent.opacity(0.5)))
                                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                    let pos =
                                        gpui_component::input::Position::new(diag_line as u32, 0);
                                    input_state.update(cx, |state, cx| {
                                        state.set_cursor_position(pos, window, cx);
                                    });
                                })
                                .child(gpui::div().text_color(color).child(severity_icon))
                                .child(
                                    gpui::div()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .child(format!("Line {}:", line_val)),
                                )
                                .child(gpui::div().child(message))
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .flex_1()
            .into_any_element()
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
        let title_clone = doc_home_borrow.title.clone();
        let author_clone = doc_home_borrow.author.clone();
        let metadata_clone = doc_home_borrow.metadata.clone();
        let hubgs_clone = doc_home_borrow.hubgs_instances.clone();
        drop(doc_home_borrow);

        let footnote_map = crate::ui::document_view::renderers::build_footnote_map(&blocks_clone);
        let parse_state = &parse_state_clone;
        let blocks = &blocks_clone;
        let title = &title_clone;
        let author = &author_clone;

        let preview_header = if title.is_empty() && author.is_empty() {
            "RENDERED PREVIEW".to_string()
        } else {
            format!("PREVIEW: {} by {}", title, author)
        };

        // Frontmatter
        let mut frontmatter = String::new();
        if !metadata_clone.is_empty() {
            frontmatter.push_str("---\n");
            for (key, val) in &metadata_clone {
                frontmatter.push_str(&format!("{}: {}\n", key, val));
            }
            frontmatter.push_str("---");
        }
        let frontmatter_el = if !frontmatter.is_empty() {
            Some(
                gpui::div()
                    .mb_4()
                    .p_3()
                    .bg(sidebar_bg)
                    .border(gpui::px(1.))
                    .border_color(border_color)
                    .rounded(gpui::px(4.))
                    .font_family("Courier New")
                    .text_xs()
                    .text_color(theme_foreground)
                    .child(frontmatter),
            )
        } else {
            None
        };

        // 1. Raw Editor Panel
        let editor_panel = gpui_component::resizable::v_resizable("editor-diagnostics")
            .child(
                gpui_component::resizable::resizable_panel().child(
                    gpui::div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .child(
                            gpui::div()
                                .p_2()
                                .bg(sidebar_bg)
                                .border_b(gpui::px(1.))
                                .border_color(border_color)
                                .text_xs()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme_muted_foreground)
                                .child(editor_header),
                        )
                        .child(
                            gpui::div()
                                .id("source_editor_container")
                                .flex_1()
                                .h(gpui::px(0.))
                                .p_4()
                                .bg(theme_group_box)
                                .child(
                                    gpui::div()
                                        .size_full()
                                        .flex()
                                        .flex_col()
                                        .children(frontmatter_el)
                                        .child(
                                            gpui_component::input::Input::new(&input_state)
                                                .flex_1()
                                                .w_full(),
                                        ),
                                ),
                        ),
                ),
            )
            .child(
                gpui_component::resizable::resizable_panel()
                    .size(gpui::px(180.))
                    .size_range(gpui::px(80.)..gpui::px(400.))
                    .child(
                        gpui::div()
                            .size_full()
                            .border_t(gpui::px(1.))
                            .border_color(border_color)
                            .bg(sidebar_bg)
                            .flex()
                            .flex_col()
                            .child(
                                gpui::div()
                                    .p_2()
                                    .bg(sidebar_bg)
                                    .border_b(gpui::px(1.))
                                    .border_color(border_color)
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme_muted_foreground)
                                    .child("LSP DIAGNOSTICS"),
                            )
                            .child(
                                gpui::div()
                                    .id("diagnostics_list")
                                    .flex_1()
                                    .overflow_hidden()
                                    .p_2()
                                    .child(diagnostics_content),
                            ),
                    ),
            );

        // 2. WYSIWYG Preview Panel
        let mut preview_content = div()
            .id("preview_content")
            .w_full()
            .flex()
            .flex_col()
            .bg(theme_val.background)
            .text_color(theme_val.foreground)
            .p_8();

        if let ParseState::OutOfSync { .. } = parse_state {
            preview_content = preview_content.child(
                Alert::warning(
                    "parse-error",
                    "Parse Error: Preview out of sync (showing last valid state)",
                )
                .banner(),
            );
        }

        // Render main blocks, separating footnotes
        let mut main_blocks = Vec::new();
        let mut footnote_blocks = Vec::new();
        for block in blocks {
            if let Block::Footnote { .. } = block {
                footnote_blocks.push(block.clone());
            } else {
                main_blocks.push(block.clone());
            }
        }

        let mut block_idx = 0;
        let mut main_iter = main_blocks.into_iter().peekable();
        while let Some(block) = main_iter.next() {
            block_idx += 1;
            if let Block::Paragraph { .. } = block {
                if let Some(Block::Aside { .. }) = main_iter.peek() {
                    let aside_block = main_iter.next().unwrap();
                    let aside_idx = block_idx + 1;
                    block_idx += 1;
                    preview_content = preview_content.child(
                        div()
                            .mb_4()
                            .flex()
                            .gap_4()
                            .w_full()
                            .child(div().w(gpui::relative(0.75)).child(renderers::render_block(
                                &self.expanded_blocks,
                                blocks,
                                &hubgs_clone,
                                &footnote_map,
                                input_state.clone(),
                                &block,
                                block_idx,
                                cx,
                            )))
                            .child(div().w(gpui::relative(0.25)).child(renderers::render_block(
                                &self.expanded_blocks,
                                blocks,
                                &hubgs_clone,
                                &footnote_map,
                                input_state.clone(),
                                &aside_block,
                                aside_idx,
                                cx,
                            ))),
                    );
                    continue;
                }
            }
            // Check for Block::Include rendering in WYSIWYG
            if let Block::Include {
                resolved_blocks: Some(ref inner_blocks),
                ..
            } = block
            {
                for inner_block in inner_blocks {
                    block_idx += 1;
                    preview_content = preview_content.child(renderers::render_block(
                        &self.expanded_blocks,
                        blocks,
                        &hubgs_clone,
                        &footnote_map,
                        input_state.clone(),
                        inner_block,
                        block_idx,
                        cx,
                    ));
                }
            } else {
                preview_content = preview_content.child(renderers::render_block(
                    &self.expanded_blocks,
                    blocks,
                    &hubgs_clone,
                    &footnote_map,
                    input_state.clone(),
                    &block,
                    block_idx,
                    cx,
                ));
            }
        }

        if !footnote_blocks.is_empty() {
            preview_content = preview_content
                .child(div().my_6().h(px(1.)).bg(theme_val.border.opacity(0.5)))
                .child(
                    div()
                        .mb_4()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_size(px(14.))
                        .text_color(theme_muted_foreground)
                        .child("Footnotes"),
                );
            for footnote in footnote_blocks {
                block_idx += 1;
                preview_content = preview_content.child(renderers::render_block(
                    &self.expanded_blocks,
                    blocks,
                    &hubgs_clone,
                    &footnote_map,
                    input_state.clone(),
                    &footnote,
                    block_idx,
                    cx,
                ));
            }
        }

        let preview_area = gpui::div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                gpui::div()
                    .flex_none()
                    .p_2()
                    .bg(sidebar_bg)
                    .border_b(gpui::px(1.))
                    .border_color(border_color)
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme_muted_foreground)
                    .child(preview_header),
            )
            .child(
                gpui::div()
                    .id("preview_container")
                    .flex_1()
                    .h(gpui::px(0.))
                    .w_full()
                    .overflow_y_scrollbar()
                    .child(preview_content),
            );

        // 3. Markdown read-only panel
        let markdown_text = crate::parser::blocks_to_markdown(blocks);
        let markdown_preview = {
            let mut preview_content = gpui::div()
                .flex_1()
                .h(gpui::px(0.))
                .p_8()
                .bg(theme_val.background)
                .text_color(theme_val.foreground)
                .font_family("Courier New")
                .text_size(px(13.))
                .overflow_y_scrollbar();

            for line in markdown_text.lines() {
                preview_content =
                    preview_content.child(gpui::div().min_h(px(18.)).child(line.to_string()));
            }

            gpui::div()
                .size_full()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    gpui::div()
                        .flex_none()
                        .p_2()
                        .bg(sidebar_bg)
                        .border_b(gpui::px(1.))
                        .border_color(border_color)
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme_muted_foreground)
                        .child(format!("MARKDOWN VIEW: {}", active_file)),
                )
                .child(preview_content)
        };

        // Match based on active doc mode
        let content_pane = match mode {
            DocumentMode::RawEditor => editor_panel.into_any_element(),
            DocumentMode::WysiwygPreview => preview_area.into_any_element(),
            DocumentMode::MarkdownView => markdown_preview.into_any_element(),
        };

        gpui::div()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(content_pane)
            .into_any_element()
    }
}
