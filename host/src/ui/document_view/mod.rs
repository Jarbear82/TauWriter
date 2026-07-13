mod jump_links;
mod renderers;

use crate::parser::Block;
use crate::ui::{DocumentHome, ParseState, Workspace};
use gpui::{
    Context, Entity, Render, Window, div, prelude::*, px, rgb,
    InteractiveElement, ParentElement, Styled,
};

pub(crate) struct DocumentView {
    workspace: Entity<Workspace>,
    document_home: Entity<DocumentHome>,
    input_state: Entity<gpui_component::input::InputState>,
    pub(crate) expanded_details: std::collections::HashSet<usize>,
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
            expanded_details: std::collections::HashSet::new(),
        }
    }
}

impl Render for DocumentView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self.workspace.read(cx);
        let selected_path = &workspace.selected_path;
        let diagnostics = &workspace.diagnostics;
        
        let theme_val = gpui_component::Theme::global(cx);
        let sidebar_bg = theme_val.sidebar;
        let border_color = theme_val.border;
        let theme_muted_foreground = theme_val.muted_foreground;
        let theme_group_box = theme_val.group_box;
        let theme_foreground = theme_val.foreground;

        // LSP Diagnostics content
        let diagnostics_content: Vec<gpui::AnyElement> = if diagnostics.is_empty() {
            vec![gpui::div()
                .text_color(gpui::rgb(0x2ECC71))
                .text_size(gpui::px(12.))
                .child("✓ No diagnostic issues found.")
                .into_any_element()]
        } else {
            let input_state = self.input_state.clone();
            diagnostics
                .iter()
                .enumerate()
                .map(|(idx, diag)| {
                    let is_error = diag.severity == 1;
                    let severity_icon = if is_error { "🔴" } else { "🟡" };
                    let color = if is_error {
                        gpui::rgb(0xE74C3C)
                    } else {
                        gpui::rgb(0xF39C12)
                    };
                    let line_val = diag.line + 1;
                    let message = diag.message.clone();
                    let input_state = input_state.clone();
                    let diag_line = diag.line;
                    gpui::div()
                        .id(("diag", idx))
                        .flex()
                        .gap_2()
                        .py_1()
                        .px_2()
                        .rounded(gpui::px(4.))
                        .text_size(gpui::px(11.))
                        .text_color(theme_foreground)
                        .hover(|s| s.bg(gpui::rgb(0xe5e7eb)))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            let pos = gpui_component::input::Position::new(diag_line as u32, 0);
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
                        .into_any_element()
                })
                .collect::<Vec<_>>()
        };

        let viewport_width = window.viewport_size().width;
        let preview_min = viewport_width * 0.2;

        let active_file = selected_path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "No File".to_string());
        let editor_header = format!("RAW EDITOR: {}", active_file);

        let (parse_state_owned, blocks_owned, title_owned, author_owned, metadata_owned) = {
            let doc = self.document_home.read(cx);
            (
                doc.parse_state.clone(),
                doc.blocks.clone(),
                doc.title.clone(),
                doc.author.clone(),
                doc.metadata.clone(),
            )
        };
        let parse_state = &parse_state_owned;
        let blocks = &blocks_owned;
        let title = &title_owned;
        let author = &author_owned;
        let metadata = &metadata_owned;

        let preview_header = if title.is_empty() && author.is_empty() {
            "RENDERED PREVIEW".to_string()
        } else {
            format!("PREVIEW: {} by {}", title, author)
        };

        // Frontmatter
        let mut frontmatter = String::new();
        if !metadata.is_empty() {
            frontmatter.push_str("---\n");
            for (key, val) in metadata {
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
                    .child(frontmatter)
            )
        } else {
            None
        };

        // Left Pane (Editor + Diagnostics)
        let left_pane = gpui_component::resizable::v_resizable("editor-diagnostics")
            .child(
                gpui_component::resizable::resizable_panel()
                    .child(
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
                                    .p_4()
                                    .bg(theme_group_box)
                                    .child(
                                        gpui::div()
                                            .size_full()
                                            .children(frontmatter_el)
                                            .child(gpui_component::input::Input::new(&self.input_state).size_full()),
                                    ),
                            )
                    )
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
                                    .overflow_y_scroll()
                                    .p_2()
                                    .children(diagnostics_content),
                            )
                    )
            );

        // Right Pane (Rendered Preview)
        let mut doc_container = div()
            .id("doc_container")
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xfbfbfb)) // Soft light cream background
            .text_color(rgb(0x222222))
            .overflow_y_scroll()
            .p_8();

        if let ParseState::OutOfSync { .. } = parse_state {
            doc_container = doc_container.child(
                div()
                    .mb_6()
                    .p_3()
                    .rounded(px(4.))
                    .bg(rgb(0xfee2e2))
                    .border(px(1.))
                    .border_color(rgb(0xfecaca))
                    .text_color(rgb(0x991b1b))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(13.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("⚠️ Parse Error: Preview out of sync (showing last valid state)")
            );
        }

        // Separate footnotes from other blocks
        let mut main_blocks = Vec::new();
        let mut footnote_blocks = Vec::new();
        for block in blocks {
            if let Block::Footnote { .. } = block {
                footnote_blocks.push(block);
            } else {
                main_blocks.push(block);
            }
        }

        // Render main blocks
        let mut block_idx = 0;
        let mut main_iter = main_blocks.into_iter().peekable();
        while let Some(block) = main_iter.next() {
            block_idx += 1;
            if let Block::Paragraph { .. } = block {
                if let Some(Block::Aside { .. }) = main_iter.peek() {
                    let aside_block = main_iter.next().unwrap();
                    let aside_idx = block_idx + 1;
                    block_idx += 1;
                    doc_container = doc_container.child(
                        div()
                            .mb_4()
                            .flex()
                            .gap_4()
                            .w_full()
                            .child(div().w(gpui::relative(0.75)).child(renderers::render_block(
                                &self.expanded_details,
                                &self.document_home,
                                &self.input_state,
                                block,
                                block_idx,
                                blocks,
                                cx,
                            )))
                            .child(div().w(gpui::relative(0.25)).child(renderers::render_block(
                                &self.expanded_details,
                                &self.document_home,
                                &self.input_state,
                                aside_block,
                                aside_idx,
                                blocks,
                                cx,
                            )))
                    );
                    continue;
                }
            }
            doc_container = doc_container.child(renderers::render_block(
                &self.expanded_details,
                &self.document_home,
                &self.input_state,
                block,
                block_idx,
                blocks,
                cx,
            ));
        }

        // Render footnotes
        if !footnote_blocks.is_empty() {
            doc_container = doc_container
                .child(div().my_6().h(px(1.)).bg(rgb(0xdddddd)))
                .child(
                    div()
                        .mb_4()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_size(px(14.))
                        .text_color(rgb(0x666666))
                        .child("Footnotes")
                );
            for footnote in footnote_blocks {
                block_idx += 1;
                doc_container = doc_container.child(renderers::render_block(
                    &self.expanded_details,
                    &self.document_home,
                    &self.input_state,
                    footnote,
                    block_idx,
                    blocks,
                    cx,
                ));
            }
        }

        let right_pane = gpui::div()
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
                    .child(preview_header),
            )
            .child(
                gpui::div()
                    .id("preview_container")
                    .flex_1()
                    .child(doc_container),
            );

        gpui::div()
            .flex_1()
            .size_full()
            .child(
                gpui_component::resizable::h_resizable("editor-preview")
                    .child(gpui_component::resizable::resizable_panel().child(left_pane))
                    .child(gpui_component::resizable::resizable_panel().size_range(preview_min..gpui::Pixels::MAX).child(right_pane))
            )
    }
}
