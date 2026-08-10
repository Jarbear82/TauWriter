//! Block Editor view for TauWriter (Notion / Craft style block-card renderer).
//!
//! Provides a continuous vertical scroll of block cards, each with:
//! - A left gutter containing drag handle, add block (+), edit pencil, and drag to reorder.
//! - A main content card containing styled read-only projections of TWXML AST blocks.

use gpui::{
    div, prelude::*, px, AnyElement, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{Icon, IconName, Theme};
use std::collections::HashMap;

use super::expansion_state::ExpandedBlocks;
use crate::graph_sim::InstanceLink;
use crate::parser::Block;
use crate::ui::DocumentView;

/// Payload attached during block drag-to-reorder operations.
#[derive(Clone, Copy)]
pub struct DragBlock {
    pub src_idx: usize,
}

/// Visual drag feedback element displayed under the cursor during reordering.
struct DragBlockView {
    src_idx: usize,
}

impl Render for DragBlockView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::global(cx);
        div()
            .p_2()
            .px_4()
            .rounded(px(6.))
            .bg(theme.accent.opacity(0.9))
            .text_color(theme.accent_foreground)
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .shadow_md()
            .child(format!("Reordering Block #{}", self.src_idx + 1))
    }
}

/// Helper to map LSP diagnostics to a block's line range.
pub(crate) fn match_diagnostics_to_block<'a>(
    doc_text: &str,
    block_range: &Option<std::ops::Range<usize>>,
    diagnostics: &'a [crate::lsp_client::Diagnostic],
) -> Option<&'a crate::lsp_client::Diagnostic> {
    let r = block_range.as_ref()?;
    let safe_start = r.start.min(doc_text.len());
    let safe_end = r.end.min(doc_text.len());
    let start_line = doc_text[..safe_start].lines().count().saturating_sub(1);
    let end_line = doc_text[..safe_end].lines().count().saturating_sub(1);

    diagnostics
        .iter()
        .filter(|d| (start_line..=end_line).contains(&d.line))
        .min_by_key(|d| d.severity)
}
/// Calculates the visible windowed block range and spacer heights for virtual scroll optimization.
pub(crate) fn compute_virtual_viewport(
    total_blocks: usize,
    scroll_offset_y: f32,
    container_height: f32,
    estimated_block_height: f32,
    overscan_count: usize,
) -> (std::ops::Range<usize>, f32, f32) {
    if total_blocks == 0 {
        return (0..0, 0.0, 0.0);
    }
    let block_h = estimated_block_height.max(1.0);
    let raw_start = (scroll_offset_y.max(0.0) / block_h).floor() as usize;
    let visible_count = (container_height.max(block_h) / block_h).ceil() as usize;

    let start_idx = raw_start.saturating_sub(overscan_count).min(total_blocks);
    let end_idx = (raw_start + visible_count + overscan_count).min(total_blocks);

    let top_spacer = start_idx as f32 * block_h;
    let bottom_spacer = (total_blocks.saturating_sub(end_idx)) as f32 * block_h;

    (start_idx..end_idx, top_spacer, bottom_spacer)
}

/// Render the complete Block Editor interface.
pub(crate) fn render_block_editor(
    _workspace_entity: &Entity<crate::ui::Workspace>,
    _document_home: &Entity<crate::ui::DocumentHome>,
    input_state: Entity<gpui_component::input::InputState>,
    expanded_blocks: &Entity<ExpandedBlocks>,
    blocks: &[Block],
    active_file: &str,
    focused_block_idx: Option<usize>,
    block_input_states: &HashMap<usize, Entity<gpui_component::input::InputState>>,
    hubgs_instances: &HashMap<SharedString, (SharedString, SharedString, Vec<InstanceLink>)>,
    footnote_map: &HashMap<SharedString, SharedString>,
    frontmatter_el: Option<gpui::Div>,
    diagnostics_content: AnyElement,
    diagnostics: &[crate::lsp_client::Diagnostic],
    cx: &mut Context<DocumentView>,
) -> AnyElement {
    let theme_val = Theme::global(cx).clone();
    let border_color = theme_val.border;
    let sidebar_bg = theme_val.sidebar;
    let theme_muted_foreground = theme_val.muted_foreground;
    let theme_group_box = theme_val.group_box;

    let block_editor_header = format!("BLOCK EDITOR: {} ({} blocks)", active_file, blocks.len());

    let content_area: AnyElement = if blocks.is_empty() {
        gpui::div()
            .id("block_editor_empty")
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .p_8()
            .text_color(theme_muted_foreground)
            .child(Icon::new(IconName::File).size(px(32.)))
            .child(div().text_sm().child("Empty document. Click '+' or type '/' to insert a block."))
            .into_any_element()
    } else {
        let mut scroll_container = gpui::div()
            .id("block_editor_scroll_container")
            .flex_1()
            .h(px(0.))
            .w_full()
            .overflow_y_scrollbar();

        for (idx, block) in blocks.iter().enumerate() {
            scroll_container = scroll_container.child(render_block_card(
                expanded_blocks,
                blocks,
                hubgs_instances,
                footnote_map,
                input_state.clone(),
                block,
                idx,
                focused_block_idx,
                block_input_states,
                diagnostics,
                cx,
            ));
        }

        scroll_container.into_any_element()
    };

    gpui_component::resizable::v_resizable("block-editor-diagnostics")
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
                            .child(block_editor_header),
                    )
                    .child(
                        gpui::div()
                            .id("block_editor_container")
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
                                    .child(content_area),
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
                                .id("block_diagnostics_list")
                                .flex_1()
                                .overflow_hidden()
                                .p_2()
                                .child(diagnostics_content),
                        ),
                ),
        )
        .into_any_element()
}

/// Renders a single block card container with left gutter and main content projection.
pub(crate) fn render_block_card(
    expanded_blocks: &Entity<ExpandedBlocks>,
    doc_blocks: &[Block],
    hubgs_instances: &HashMap<SharedString, (SharedString, SharedString, Vec<InstanceLink>)>,
    footnote_map: &HashMap<SharedString, SharedString>,
    input_state: Entity<gpui_component::input::InputState>,
    block: &Block,
    idx: usize,
    focused_block_idx: Option<usize>,
    block_input_states: &HashMap<usize, Entity<gpui_component::input::InputState>>,
    diagnostics: &[crate::lsp_client::Diagnostic],
    cx: &mut Context<DocumentView>,
) -> AnyElement {
    let theme = Theme::global(cx).clone();
    let is_collapsible = matches!(
        block,
        Block::Details { .. } | Block::CodeBlock { .. } | Block::BlockQuote { .. }
    );

    // Render left gutter controls
    let gutter = render_gutter(idx, is_collapsible, &theme, block, input_state.clone(), diagnostics, cx);

    // Render block content read-only projection
    let content_projection = super::renderers::render_block(
        expanded_blocks,
        doc_blocks,
        hubgs_instances,
        footnote_map,
        input_state.clone(),
        block,
        idx,
        cx,
    );

    let is_focused = focused_block_idx == Some(idx);
    let is_codeblock = matches!(block, Block::CodeBlock { .. });
    let card_range_done = block.range();
    let card_range_click = block.range();
    let block_fallback_text = crate::parser::extract_plain_text_from_block(block);
    let view_weak_header = cx.entity().downgrade();

    let edit_header = if is_focused {
        Some(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_2()
                .py_1()
                .mb_1()
                .rounded(px(4.))
                .bg(theme.accent.opacity(0.15))
                .text_color(theme.accent)
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .child(format!("EDITING BLOCK #{}", idx + 1))
                .child(
                    div()
                        .cursor_pointer()
                        .px_2()
                        .py_0p5()
                        .rounded(px(3.))
                        .bg(theme.accent)
                        .text_color(theme.accent_foreground)
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            if let Some(v) = view_weak_header.upgrade() {
                                v.update(cx, |this, cx| {
                                    if let Some(block_state) = this.block_input_states.get(&idx) {
                                        let new_block_text = block_state.read(cx).value().to_string();
                                        if let Some(r) = card_range_done.clone() {
                                            this.input_state.update(cx, |state, cx| {
                                                let doc_text = state.value().to_string();
                                                if r.end <= doc_text.len() {
                                                    let mut new_doc = doc_text;
                                                    new_doc.replace_range(r, &new_block_text);
                                                    state.set_value(new_doc, window, cx);
                                                    cx.emit(gpui_component::input::InputEvent::Change);
                                                }
                                            });
                                        }
                                    }
                                    this.focused_block_idx = None;
                                    cx.notify();
                                });
                            }
                        })
                        .child("Done (Ctrl+Enter)"),
                ),
        )
    } else {
        None
    };

    let input_state_drop = input_state.clone();
    let input_state_click = input_state.clone();
    let doc_blocks_vec = doc_blocks.to_vec();
    let target_idx = idx;
    let view_weak_click = cx.entity().downgrade();

    let block_input_entity = block_input_states.get(&idx).cloned();

    let content_area: AnyElement = if is_focused {
        let input_elem = if let Some(ref b_state) = block_input_entity {
            gpui_component::input::Input::new(b_state).w_full().into_any_element()
        } else {
            gpui_component::input::Input::new(&input_state).w_full().into_any_element()
        };

        let input_box: AnyElement = if is_codeblock {
            div()
                .p_2()
                .rounded(px(6.))
                .border(px(1.))
                .border_color(theme.accent)
                .bg(theme.background)
                .w_full()
                .overflow_x_scrollbar()
                .child(input_elem)
                .into_any_element()
        } else {
            div()
                .p_2()
                .rounded(px(6.))
                .border(px(1.))
                .border_color(theme.accent)
                .bg(theme.background)
                .w_full()
                .child(input_elem)
                .into_any_element()
        };

        div()
            .flex_1()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .children(edit_header)
            .child(input_box)
            .into_any_element()
    } else {
        div()
            .flex_1()
            .w_full()
            .cursor_text()
            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                let raw_text = if let Some(r) = card_range_click.clone() {
                    let full_doc = input_state_click.read(cx).value().to_string();
                    if r.end <= full_doc.len() {
                        full_doc[r].to_string()
                    } else {
                        block_fallback_text.clone()
                    }
                } else {
                    block_fallback_text.clone()
                };

                let block_text = if is_codeblock {
                    raw_text
                } else {
                    crate::parser::normalize_block_text_for_editing(&raw_text)
                };

                if let Some(v) = view_weak_click.upgrade() {
                    v.update(cx, |this, cx| {
                        let block_state = cx.new(|cx| {
                            let mut state = gpui_component::input::InputState::new(window, cx).multi_line(true);
                            state.set_value(block_text, window, cx);
                            state
                        });
                        this.block_input_states.insert(idx, block_state);
                        this.focused_block_idx = Some(idx);
                        cx.notify();
                    });
                }
            })
            .children(edit_header)
            .child(content_projection)
            .into_any_element()
    };

    let (card_bg, card_border) = if is_focused {
        (theme.accent.opacity(0.06), theme.accent)
    } else {
        (gpui::hsla(0.0, 0.0, 0.0, 0.0), gpui::hsla(0.0, 0.0, 0.0, 0.0))
    };

    div()
        .id(("block_card", idx))
        .group("block_card_group")
        .relative()
        .w_full()
        .my_1()
        .px_2()
        .py_1()
        .rounded(px(6.))
        .bg(card_bg)
        .border(px(1.))
        .border_color(card_border)
        .hover(|s| s.bg(theme.accent.opacity(0.08)).border_color(theme.accent.opacity(0.5)))
        .drag_over::<DragBlock>(move |div, _dragged, _window, cx| {
            let theme = Theme::global(cx);
            div.bg(theme.accent.opacity(0.12)).border_color(theme.accent)
        })
        .on_drop(move |dragged: &DragBlock, window, cx| {
            let src_idx = dragged.src_idx;
            if src_idx != target_idx && src_idx < doc_blocks_vec.len() && target_idx < doc_blocks_vec.len() {
                let src_range = doc_blocks_vec[src_idx].range();
                let target_range = doc_blocks_vec[target_idx].range();
                if let (Some(sr), Some(tr)) = (src_range, target_range) {
                    input_state_drop.update(cx, |state, cx| {
                        let text = state.value().to_string();
                        let new_doc = crate::parser::reorder_blocks_in_document(&text, sr, tr);
                        state.set_value(new_doc, window, cx);
                        cx.emit(gpui_component::input::InputEvent::Change);
                    });
                }
            }
        })
        .flex()
        .items_start()
        .gap_2()
        .child(gutter)
        .child(content_area)
        .into_any_element()
}

/// Helper to convert a byte offset in document text to InputState Position.
fn offset_to_position(text: &str, offset: usize) -> gpui_component::input::Position {
    let safe_offset = offset.min(text.len());
    let prefix = &text[..safe_offset];
    let line = prefix.lines().count().saturating_sub(1);
    let last_line = prefix.lines().last().unwrap_or("");
    let col = last_line.len();
    gpui_component::input::Position::new(line as u32, col as u32)
}

/// Renders the left gutter controls for a block card.
fn render_gutter(
    idx: usize,
    _is_collapsible: bool,
    theme: &Theme,
    block: &Block,
    input_state: Entity<gpui_component::input::InputState>,
    diagnostics: &[crate::lsp_client::Diagnostic],
    cx: &mut Context<DocumentView>,
) -> AnyElement {
    let muted_color = theme.muted_foreground;
    let block_range = block.range();
    let is_codeblock = matches!(block, Block::CodeBlock { .. });

    let doc_text = input_state.read(cx).value().to_string();
    let diag_opt = match_diagnostics_to_block(&doc_text, &block_range, diagnostics);

    let input_state_add = input_state.clone();
    let add_offset = block_range.as_ref().map(|r| r.end);

    let input_state_edit = input_state;
    let card_range_edit = block_range;
    let block_fallback_edit = crate::parser::extract_plain_text_from_block(block);
    let view_weak_edit = cx.entity().downgrade();

    let mut gutter_el = div()
        .id(("gutter", idx))
        .flex_none()
        .w(px(72.))
        .pt_1()
        .flex()
        .items_center()
        .justify_end()
        .gap_1()
        .opacity(0.3) // muted until hovered
        .group_hover("block_card_group", |s| s.opacity(1.0));

    if let Some(diag) = diag_opt {
        let color = if diag.severity == 1 { theme.danger } else { theme.warning };
        let msg = diag.message.clone();
        gutter_el = gutter_el.child(
            div()
                .id(("diag_badge", idx))
                .cursor_pointer()
                .tooltip(move |w, cx| gpui_component::tooltip::Tooltip::new(msg.clone()).build(w, cx))
                .child(Icon::new(IconName::TriangleAlert).size(px(12.)).text_color(color)),
        );
    }

    gutter_el
        // Drag Handle (Drag to reorder block)
        .child(
            div()
                .id(("drag_handle", idx))
                .cursor_pointer()
                .on_drag(DragBlock { src_idx: idx }, move |dragged, _offset, _window, cx| {
                    cx.new(|_| DragBlockView {
                        src_idx: dragged.src_idx,
                    })
                })
                .tooltip(|w, cx| gpui_component::tooltip::Tooltip::new("Drag to reorder block").build(w, cx))
                .child(Icon::new(IconName::Menu).size(px(12.)).text_color(muted_color)),
        )
        // Insert Block Button (+)
        .child(
            div()
                .id(("add_block_btn", idx))
                .cursor_pointer()
                .tooltip(|w, cx| gpui_component::tooltip::Tooltip::new("Add block below (+)").build(w, cx))
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    if let Some(offset) = add_offset {
                        let skeleton = format!("\n{}", crate::parser::generate_block_skeleton("paragraph"));
                        input_state_add.update(cx, |state, cx| {
                            let text = state.value().to_string();
                            let pos = offset_to_position(&text, offset);
                            state.set_cursor_position(pos, window, cx);
                            state.replace(&skeleton, window, cx);
                        });
                    }
                })
                .child(Icon::new(IconName::Plus).size(px(12.)).text_color(muted_color)),
        )
        // Edit Pencil / Caret Jump Button
        .child(
            div()
                .id(("edit_block_btn", idx))
                .cursor_pointer()
                .tooltip(|w, cx| gpui_component::tooltip::Tooltip::new("Edit block").build(w, cx))
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    let raw_text = if let Some(r) = card_range_edit.clone() {
                        let full_doc = input_state_edit.read(cx).value().to_string();
                        if r.end <= full_doc.len() {
                            full_doc[r].to_string()
                        } else {
                            block_fallback_edit.clone()
                        }
                    } else {
                        block_fallback_edit.clone()
                    };

                    let block_text = if is_codeblock {
                        raw_text
                    } else {
                        crate::parser::normalize_block_text_for_editing(&raw_text)
                    };

                    if let Some(v) = view_weak_edit.upgrade() {
                        v.update(cx, |this, cx| {
                            let block_state = cx.new(|cx| {
                                let mut state = gpui_component::input::InputState::new(window, cx).multi_line(true);
                                state.set_value(block_text, window, cx);
                                state
                            });
                            this.block_input_states.insert(idx, block_state);
                            this.focused_block_idx = Some(idx);
                            cx.notify();
                        });
                    }
                })
                .child(Icon::new(IconName::Settings).size(px(12.)).text_color(muted_color)),
        )
        .into_any_element()
}
