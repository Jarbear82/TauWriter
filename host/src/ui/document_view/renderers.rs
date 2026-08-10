use super::collapsible::CollapsibleBlock;
use super::expansion_state::ExpandedBlocks;
use super::jump_links::{find_block_range_by_id, find_block_type_by_id, offset_to_position};
use crate::graph_sim::InstanceLink;
use crate::parser::{Block, TextRun};
use crate::ui::DocumentView;
use gpui::{
    div, prelude::*, px, AnyElement, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, SharedString, Styled,
};
use gpui_component::{scroll::ScrollableElement, table::*, Icon, IconName, Theme};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;

/// Conventionally accepted code editor colors (VS Code dark style).
static CODE_BLOCK_BG: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.0, 0.12, 1.0));
static CODE_BLOCK_HEADER_BG: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.0, 0.16, 1.0));
static CODE_BLOCK_TEXT_COLOR: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.0, 0.83, 1.0));

/// Standard warning/review background (light yellow, works on both light and dark themes).
static REVIEW_BGCOLOR: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(48.0, 1.0, 0.975, 1.0));

/// Warm amber border for aside/note blocks.
static ASIDE_BORDER_COLOR: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(43.0, 0.85, 0.47, 1.0));

/// Review warning icon/text color (warm orange).
static REVIEW_WARNING_COLOR: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(23.0, 0.82, 0.47, 1.0));

/// Review border color (soft yellow-orange).
static REVIEW_BORDER_COLOR: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(35.0, 0.97, 0.90, 1.0));

/// Build a HashMap<footnote_id, footnote_content> from blocks for O(1) lookups.
/// Returns an empty map if no footnotes are present.
#[allow(dead_code)]
pub(crate) fn build_footnote_map(blocks: &[Block]) -> HashMap<SharedString, SharedString> {
    blocks
        .iter()
        .filter_map(|b| {
            if let Block::Footnote { id, runs, .. } = b {
                let content: SharedString = runs
                    .iter()
                    .map(|r| r.text.as_ref())
                    .collect::<String>()
                    .into();
                Some((id.clone(), content))
            } else {
                None
            }
        })
        .collect()
}

/// Render a single Block into an AnyElement.
///
/// Hoisted reads: `blocks`, `hubgs_instances`, and `footnote_map` are passed
/// as references from the caller's single Entity read (in DocumentView::render).
pub(crate) fn render_block(
    expanded_blocks: &Entity<ExpandedBlocks>,
    doc_blocks: &[Block],
    hubgs_instances: &HashMap<SharedString, (SharedString, SharedString, Vec<InstanceLink>)>,
    footnote_map: &HashMap<SharedString, SharedString>,
    input_state: Entity<gpui_component::input::InputState>,
    block: &Block,
    idx: usize,
    cx: &mut Context<DocumentView>,
) -> AnyElement {
    let theme = Theme::global(cx).clone();

    match block {
        Block::Heading {
            level,
            text,
            id: _,
            attributes: _,
            range: _,
        } => {
            let shifted_level = level.saturating_sub(1);
            let size = match shifted_level {
                0 => px(28.), // H0 (Title)
                1 => px(24.), // H1
                2 => px(20.), // H2
                3 => px(18.), // H3
                _ => px(16.),
            };
            let tooltip_text = format!("Element: Heading\nLevel: {}", shifted_level);
            let clean_heading = text.replace("\r\n", " ").replace('\n', " ");
            div()
                .id(("heading", idx))
                .w_full()
                .mt_6()
                .mb_2()
                .font_weight(gpui::FontWeight::BOLD)
                .text_size(size)
                .flex()
                .flex_wrap()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(clean_heading)
                .into_any_element()
        }
        Block::Paragraph {
            runs,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("Paragraph", id, attributes);
            div()
                .id(("paragraph", idx))
                .w_full()
                .mb_4()
                .flex()
                .flex_wrap()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .children(runs.iter().enumerate().map(|(run_idx, run)| {
                    render_run(
                        doc_blocks,
                        hubgs_instances,
                        footnote_map,
                        input_state.clone(),
                        run,
                        idx,
                        run_idx,
                        &theme,
                    )
                }))
                .into_any_element()
        }
        Block::BlockQuote {
            runs,
            id: _,
            attributes: _,
            range,
        } => {
            let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);

            let body_runs: Vec<AnyElement> = runs
                .iter()
                .enumerate()
                .map(|(run_idx, run)| {
                    render_run(
                        doc_blocks,
                        hubgs_instances,
                        footnote_map,
                        input_state.clone(),
                        run,
                        idx,
                        run_idx,
                        &theme,
                    )
                })
                .collect();

            CollapsibleBlock::new("Quote".to_string(), theme.border, theme.group_box)
                .with_body(vec![
                    div().flex().flex_wrap().w_full().children(body_runs).into_any_element()
                ])
                .render(start_offset, expanded_blocks.clone(), cx)
        }
        Block::Aside {
            runs,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("Aside", id, attributes);
            div()
                .id(("aside", idx))
                .w_full()
                .mb_4()
                .p_4()
                .bg(theme.accent.opacity(0.15)) // Soft tinted aside/note bg
                .border_l_4()
                .border_color(*ASIDE_BORDER_COLOR) // Warm amber border for aside/note semantics
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .flex()
                .flex_wrap()
                .children(runs.iter().enumerate().map(|(run_idx, run)| {
                    render_run(
                        doc_blocks,
                        hubgs_instances,
                        footnote_map,
                        input_state.clone(),
                        run,
                        idx,
                        run_idx,
                        &theme,
                    )
                }))
                .into_any_element()
        }
        Block::CodeBlock {
            language,
            code,
            id,
            attributes,
            range,
        } => {
            let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
            let trimmed_code = trim_codeblock_indentation(code);

            let lang_display = if language.is_empty() {
                "codeblock"
            } else {
                language
            };

            let id_clone = id.clone();
            let attrs_clone: Vec<_> = attributes.iter().cloned().collect();

            // Outer wrapper for CodeBlock's unique styling (dark bg, rounded corners)
            let mut container = div()
                .id("codeblock")
                .w_full()
                .mb_4()
                .border(px(1.))
                .border_color(theme.border)
                .rounded(px(4.))
                .bg(*CODE_BLOCK_BG) // Conventionally accepted VS Code dark background
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(format!(
                        "Element: CodeBlock\nid: {:?}",
                        id_clone
                    ))
                    .build(window, cx)
                });

            let code_lines: Vec<AnyElement> = trimmed_code
                .lines()
                .map(|line| div().child(line.to_string()).into_any_element())
                .collect();

            // Use CollapsibleBlock for the toggle/header logic
            let collapsible_content = CollapsibleBlock::new(
                lang_display.to_string(),
                gpui::hsla(0.0, 0.0, 0.0, 0.0), // transparent (outer container provides border)
                *CODE_BLOCK_BG,                 // inner bg (VS Code dark style)
            )
            .with_body(vec![div()
                .flex()
                .flex_col()
                .p_4()
                .text_color(*CODE_BLOCK_TEXT_COLOR)
                .font_family("Courier New")
                .text_size(px(13.))
                .overflow_x_scrollbar()
                .children(code_lines)
                .into_any_element()]);

            // Wrap in outer container with border/tooltip
            let content = collapsible_content.render(start_offset, expanded_blocks.clone(), cx);
            container = container.child(content);

            container.into_any_element()
        }
        Block::List {
            ordered,
            items,
            id,
            attributes,
            range: _,
        } => {
            let start_offset = if items.is_empty() { 0 } else { 1 };
            let tooltip_text = element_tooltip(if *ordered { "ol" } else { "ul" }, id, attributes);
            let items_elements = items.iter().enumerate().map(|(item_idx, item)| {
                let bullet_el = if let Some(checked) = item.checked {
                    gpui_component::checkbox::Checkbox::new(format!(
                        "chk-{}-{}",
                        start_offset, item_idx
                    ))
                    .checked(checked)
                    .mr_2()
                    .into_any_element()
                } else if *ordered {
                    div()
                        .child(format!("{}. ", item_idx + 1))
                        .mr_2()
                        .into_any_element()
                } else {
                    div().child("• ").mr_2().into_any_element()
                };
                div().flex().items_center().mb_1().child(bullet_el).child(
                    div()
                        .flex()
                        .flex_wrap()
                        .children(item.runs.iter().enumerate().map(|(run_idx, run)| {
                            render_run(
                                doc_blocks,
                                hubgs_instances,
                                footnote_map,
                                input_state.clone(),
                                run,
                                idx,
                                run_idx + 100 * item_idx,
                                &theme,
                            )
                        })),
                )
            });

            div()
                .id(("list", idx))
                .w_full()
                .mb_4()
                .flex()
                .flex_col()
                .pl_4()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .children(items_elements)
                .into_any_element()
        }
        Block::DescriptionList {
            items,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("dl", id, attributes);
            let items_elements = items.iter().enumerate().map(|(item_idx, (term, runs))| {
                let clean_term = term.replace("\r\n", " ").replace('\n', " ");
                div()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .mb_1()
                            .child(clean_term),
                    )
                    .child(div().pl_4().mb_2().flex().flex_wrap().children(
                        runs.iter().enumerate().map(|(run_idx, run)| {
                            render_run(
                                doc_blocks,
                                hubgs_instances,
                                footnote_map,
                                input_state.clone(),
                                run,
                                idx,
                                run_idx + 100 * item_idx,
                                &theme,
                            )
                        }),
                    ))
            });

            div()
                .id(("dl", idx))
                .w_full()
                .mb_4()
                .flex()
                .flex_col()
                .pl_4()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .children(items_elements)
                .into_any_element()
        }
        Block::Table {
            headers,
            rows,
            id: _,
            attributes: _,
            range,
        } => {
            // Build header cells — simple clones.
            let header_cells: Vec<_> = headers
                .iter()
                .map(|h| TableHead::new().child(h.replace("\r\n", " ").replace('\n', " ")))
                .collect();

            // Arc-wrapped references for cheap per-cell closure captures.
            let doc_blocks_arc = Arc::new(doc_blocks.to_vec());
            let hubgs_arc = Arc::new(
                hubgs_instances
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>(),
            );
            let fn_map_arc = Arc::new(
                footnote_map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>(),
            );

            // Build table body row-by-row via explicit loops.
            // Each cell wraps its runs in an AnyElement iterator via .children().
            let mut table_body = TableBody::new();
            for (row_idx, row) in rows.iter().enumerate() {
                let mut table_row = TableRow::new();
                for (cell_idx, cell) in row.iter().enumerate() {
                    let runs: Vec<_> = cell
                        .iter()
                        .enumerate()
                        .map(|(run_idx, run)| {
                            render_run(
                                &*doc_blocks_arc,
                                &*hubgs_arc,
                                &*fn_map_arc,
                                input_state.clone(),
                                run,
                                idx + 1000 * row_idx,
                                run_idx + 100 * cell_idx,
                                &theme,
                            )
                        })
                        .collect();
                    table_row = table_row.child(TableCell::new().children(runs));
                }
                table_body = table_body.child(table_row);
            }

            let table_range = range.clone();
            let headers_clone = headers.to_vec();
            let rows_clone = rows.to_vec();

            let input_state_row_add = input_state.clone();
            let headers_row_add = headers_clone.clone();
            let rows_row_add = rows_clone.clone();
            let range_row_add = table_range.clone();

            let input_state_col_add = input_state.clone();
            let headers_col_add = headers_clone.clone();
            let rows_col_add = rows_clone.clone();
            let range_col_add = table_range.clone();

            let input_state_row_del = input_state.clone();
            let headers_row_del = headers_clone.clone();
            let rows_row_del = rows_clone.clone();
            let range_row_del = table_range.clone();

            let input_state_col_del = input_state.clone();
            let headers_col_del = headers_clone.clone();
            let rows_col_del = rows_clone.clone();
            let range_col_del = table_range.clone();

            let toolbar = div()
                .flex()
                .items_center()
                .gap_2()
                .mb_2()
                .text_xs()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(3.))
                        .bg(theme.accent)
                        .text_color(theme.accent_foreground)
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            if let Some(ref r) = range_row_add {
                                let h = headers_row_add.clone();
                                let mut rw = rows_row_add.clone();
                                let target_len = rw.len();
                                crate::parser::table_add_row(h.len(), &mut rw, target_len);
                                let new_twxml = crate::parser::table_to_twxml(&h, &rw);
                                input_state_row_add.update(cx, |state, cx| {
                                    let full_text = state.value().to_string();
                                    if r.end <= full_text.len() {
                                        let mut updated = full_text;
                                        updated.replace_range(r.clone(), &new_twxml);
                                        state.set_value(updated, window, cx);
                                        cx.emit(gpui_component::input::InputEvent::Change);
                                    }
                                });
                            }
                        })
                        .child("+ Row"),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(3.))
                        .bg(theme.accent)
                        .text_color(theme.accent_foreground)
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            if let Some(ref r) = range_col_add {
                                let mut h = headers_col_add.clone();
                                let mut rw = rows_col_add.clone();
                                let target_len = h.len();
                                crate::parser::table_add_column(&mut h, &mut rw, target_len);
                                let new_twxml = crate::parser::table_to_twxml(&h, &rw);
                                input_state_col_add.update(cx, |state, cx| {
                                    let full_text = state.value().to_string();
                                    if r.end <= full_text.len() {
                                        let mut updated = full_text;
                                        updated.replace_range(r.clone(), &new_twxml);
                                        state.set_value(updated, window, cx);
                                        cx.emit(gpui_component::input::InputEvent::Change);
                                    }
                                });
                            }
                        })
                        .child("+ Col"),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(3.))
                        .bg(theme.muted_foreground.opacity(0.3))
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            if let Some(ref r) = range_row_del {
                                let h = headers_row_del.clone();
                                let mut rw = rows_row_del.clone();
                                if rw.len() > 1 {
                                    let last_idx = rw.len() - 1;
                                    crate::parser::table_delete_row(&mut rw, last_idx);
                                    let new_twxml = crate::parser::table_to_twxml(&h, &rw);
                                    input_state_row_del.update(cx, |state, cx| {
                                        let full_text = state.value().to_string();
                                        if r.end <= full_text.len() {
                                            let mut updated = full_text;
                                            updated.replace_range(r.clone(), &new_twxml);
                                            state.set_value(updated, window, cx);
                                            cx.emit(gpui_component::input::InputEvent::Change);
                                        }
                                    });
                                }
                            }
                        })
                        .child("- Row"),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(3.))
                        .bg(theme.muted_foreground.opacity(0.3))
                        .cursor_pointer()
                        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                            if let Some(ref r) = range_col_del {
                                let mut h = headers_col_del.clone();
                                let mut rw = rows_col_del.clone();
                                if h.len() > 1 {
                                    let last_idx = h.len() - 1;
                                    crate::parser::table_delete_column(&mut h, &mut rw, last_idx);
                                    let new_twxml = crate::parser::table_to_twxml(&h, &rw);
                                    input_state_col_del.update(cx, |state, cx| {
                                        let full_text = state.value().to_string();
                                        if r.end <= full_text.len() {
                                            let mut updated = full_text;
                                            updated.replace_range(r.clone(), &new_twxml);
                                            state.set_value(updated, window, cx);
                                            cx.emit(gpui_component::input::InputEvent::Change);
                                        }
                                    });
                                }
                            }
                        })
                        .child("- Col"),
                );

            div()
                .id(("table_wrapper", idx))
                .w_full()
                .mb_4()
                .child(toolbar)
                .child(
                    Table::new()
                        .w_full()
                        .overflow_x_hidden()
                        .child(TableHeader::new().children(header_cells))
                        .child(table_body),
                )
                .into_any_element()
        }
        Block::HorizontalRule {
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("hr", id, attributes);
            div()
                .id(("hr", idx))
                .w_full()
                .my_4()
                .h(px(1.))
                .bg(theme.muted_foreground.opacity(0.3))
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .into_any_element()
        }
        Block::Image {
            src,
            alt,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("image", id, attributes);
            let alt_display = alt.as_deref().unwrap_or("[No Alt Text]");
            div()
                .id(("image", idx))
                .w_full()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(theme.group_box)
                .flex()
                .flex_col()
                .items_center()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(format!("Image: {} (Alt: {})", src, alt_display))
                .into_any_element()
        }
        Block::Audio {
            src,
            alt,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("audio", id, attributes);
            let alt_display = alt.as_deref().unwrap_or("[No Alt Text]");
            div()
                .id(("audio", idx))
                .w_full()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(theme.group_box)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(format!("Audio: {} (Alt: {})", src, alt_display))
                .into_any_element()
        }
        Block::Video {
            src,
            alt,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("video", id, attributes);
            let alt_display = alt.as_deref().unwrap_or("[No Alt Text]");
            div()
                .id(("video", idx))
                .w_full()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(theme.group_box)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(format!("Video: {} (Alt: {})", src, alt_display))
                .into_any_element()
        }
        Block::Details {
            summary: _,
            blocks,
            id,
            attributes,
            range,
        } => {
            let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
            let tooltip_text = element_tooltip("details", &id.clone(), attributes);

            let mut container = div()
                .id(("details", idx))
                .w_full()
                .mb_4()
                .border(px(1.))
                .border_color(theme.border)
                .rounded(px(4.))
                .bg(theme.sidebar.opacity(0.3))
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                });

            let collapsible_content =
                CollapsibleBlock::new("Details".to_string(), theme.border, theme.background)
                    .with_body(
                        blocks
                            .iter()
                            .enumerate()
                            .map(|(inner_idx, inner_block)| {
                                render_block(
                                    expanded_blocks,
                                    doc_blocks,
                                    hubgs_instances,
                                    footnote_map,
                                    input_state.clone(),
                                    inner_block,
                                    idx + 1000 * inner_idx,
                                    cx,
                                )
                            })
                            .collect(),
                    );

            let content = collapsible_content.render(start_offset, expanded_blocks.clone(), cx);
            container = container.child(content);
            container.into_any_element()
        }
        Block::Footnote {
            id,
            runs,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("footnote", &Some(id.clone()), attributes);
            div()
                .id(("footnote", idx))
                .w_full()
                .mb_2()
                .flex()
                .flex_wrap()
                .gap_2()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(
                    div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(format!("{}:", id)),
                )
                .children(runs.iter().enumerate().map(|(run_idx, run)| {
                    render_run(
                        doc_blocks,
                        hubgs_instances,
                        footnote_map,
                        input_state.clone(),
                        run,
                        idx,
                        run_idx,
                        &theme,
                    )
                }))
                .into_any_element()
        }
        Block::Review {
            blocks,
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("review", id, attributes);
            let mut container = div()
                .id(("review", idx))
                .w_full()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(*REVIEW_BGCOLOR) // Soft yellow warning background
                .border(px(1.))
                .border_color(*ASIDE_BORDER_COLOR)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                });

            container = container.child(
                div()
                    .mb_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(*REVIEW_WARNING_COLOR)
                    .child(Icon::new(IconName::TriangleAlert).size(gpui::px(13.)))
                    .child("FLAG FOR REVIEW"),
            );

            for (inner_idx, inner_block) in blocks.iter().enumerate() {
                container = container.child(render_block(
                    expanded_blocks,
                    doc_blocks,
                    hubgs_instances,
                    footnote_map,
                    input_state.clone(),
                    inner_block,
                    idx + 1000 * inner_idx,
                    cx,
                ));
            }
            container.into_any_element()
        }
        Block::Include {
            src: _,
            id: _,
            attributes: _,
            range: _,
            resolved_blocks,
        } => {
            let mut container = div().flex().flex_col().w_full().gap_2();
            if let Some(blocks) = resolved_blocks {
                for (inner_idx, inner_block) in blocks.iter().enumerate() {
                    container = container.child(render_block(
                        expanded_blocks,
                        doc_blocks,
                        hubgs_instances,
                        footnote_map,
                        input_state.clone(),
                        inner_block,
                        idx + 1000 * (inner_idx + 1),
                        cx,
                    ));
                }
            }
            container.into_any_element()
        }
    }
}

/// Render a single TextRun into an AnyElement.
///
/// Hoisted reads: `doc_blocks`, `hubgs_instances`, and `footnote_map` are passed
/// as references from the caller's single Entity read (in DocumentView::render).
pub(crate) fn render_run(
    doc_blocks: &[Block],
    hubgs_instances: &HashMap<SharedString, (SharedString, SharedString, Vec<InstanceLink>)>,
    footnote_map: &HashMap<SharedString, SharedString>,
    input_state: Entity<gpui_component::input::InputState>,
    run: &TextRun,
    block_idx: usize,
    run_idx: usize,
    theme: &gpui_component::Theme,
) -> AnyElement {
    let clean_text = run.text.replace("\r\n", " ").replace('\n', " ");
    if clean_text.trim().is_empty() && run.text.contains('\n') {
        return div().w_full().into_any_element();
    }
    let mut text_el = div().child(clean_text);

    if run.bold {
        text_el = text_el.font_weight(gpui::FontWeight::BOLD);
    }
    if run.italic {
        text_el = text_el.italic();
    }
    if run.underline {
        text_el = text_el.underline();
    }
    if run.code {
        text_el = text_el
            .bg(theme.group_box)
            .p_0p5()
            .rounded(px(2.))
            .font_family("Courier New")
            .text_size(px(13.));
    }
    if run.superscript {
        text_el = text_el.text_size(px(10.)).mb_2();
    }
    if run.subscript {
        text_el = text_el.text_size(px(10.)).mt_2();
    }

    // Default Element Tooltip fallback for run elements
    let mut run_tooltip = element_tooltip("text", &run.id, &run.attributes);

    if let Some(ref hub_id) = run.hubref {
        let hub_id = hub_id.clone();

        // O(1) HashMap lookup instead of Entity::read on every render frame
        let mut tooltip_text = format!("HubRef: {}", hub_id);
        if let Some((type_name, name, links)) = hubgs_instances.get(hub_id.as_ref()) {
            tooltip_text = format!("Hub: {} | name: \"{}\"", type_name, name);
            for link in links {
                tooltip_text.push_str(&format!(" | {} -> {}", link.relation, link.target));
            }
        }
        run_tooltip = tooltip_text;

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| {
                log::debug!("[host] User clicked on Hub Reference ID: {}", hub_id);
            });
    } else if let Some(ref fn_id) = run.footnote_ref {
        let fn_id = fn_id.clone();

        // O(1) HashMap lookup instead of O(n) linear scan per footnote reference
        let footnote_content = footnote_map
            .get(fn_id.as_ref())
            .cloned()
            .unwrap_or_else(|| -> SharedString { "Footnote definition not found".into() });
        run_tooltip = format!("Footnote Ref: {} | {}", fn_id, footnote_content.replace("\r\n", " ").replace('\n', " "));

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                log::debug!("[host] User clicked on Footnote Reference ID: {}", fn_id);
            });
    } else if let Some(ref link) = run.link {
        let link = link.clone();

        // Build Link tooltip — synchronous lookup, no closures needed
        let mut tooltip_text = format!("Link: {}", link);
        if link.starts_with('#') {
            let target_id = &link[1..];
            if let Some(target_type) = find_block_type_by_id(doc_blocks, target_id) {
                tooltip_text = format!("Jump to element: {} | Type: {}", target_id, target_type);
            }
        }
        run_tooltip = tooltip_text;

        let input_state_link = input_state.clone();
        let doc_blocks_vec = doc_blocks.to_vec();

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .cursor_pointer()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                log::debug!("[host] User clicked internal/external link: {}", link);
                if link.starts_with("http") {
                    cx.open_url(&link);
                } else if link.starts_with('#') {
                    let target_id = &link[1..];
                    if let Some(target_range) = find_block_range_by_id(&doc_blocks_vec, target_id) {
                        input_state_link.update(cx, |state, cx| {
                            let text = state.value().to_string();
                            if let Some(pos) = offset_to_position(&text, target_range.start) {
                                state.set_cursor_position(pos, window, cx);
                            }
                        });
                    }
                }
            });
    }

    let clean_run_tooltip = run_tooltip.replace("\r\n", " | ").replace('\n', " | ");

    let run_range_fmt = run.range.clone();
    let run_text_fmt = run.text.to_string();
    let input_state_fmt = input_state.clone();

    // Attach run tooltip and context menu action
    text_el
        .id(("run", block_idx * 1000 + run_idx))
        .tooltip(move |window, cx| {
            gpui_component::tooltip::Tooltip::new(clean_run_tooltip.clone()).build(window, cx)
        })
        .on_mouse_down(gpui::MouseButton::Right, move |_, window, cx| {
            if let Some(ref r) = run_range_fmt {
                let formatted = crate::parser::wrap_text_in_inline_format(&run_text_fmt, "bold", None);
                input_state_fmt.update(cx, |state, cx| {
                    let full_text = state.value().to_string();
                    if r.end <= full_text.len() {
                        let mut updated = full_text;
                        updated.replace_range(r.clone(), &formatted);
                        state.set_value(updated, window, cx);
                        cx.emit(gpui_component::input::InputEvent::Change);
                    }
                });
            }
        })
        .into_any_element()
}

pub(crate) fn element_tooltip(
    tag_name: &str,
    id: &Option<SharedString>,
    attributes: &[(SharedString, SharedString)],
) -> String {
    let mut attrs_str = String::new();
    for (k, v) in attributes {
        attrs_str.push_str(&format!(" | {}: \"{}\"", k, v));
    }
    format!("Element: {} | id: {:?}{}", tag_name, id, attrs_str)
}

pub(crate) fn trim_codeblock_indentation(code: &str) -> String {
    let lines: Vec<&str> = code.split('\n').collect();
    let mut min_indent = usize::MAX;
    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        if indent < min_indent {
            min_indent = indent;
        }
    }

    if min_indent == usize::MAX {
        return code.to_string();
    }

    let trimmed_lines: Vec<String> = lines
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    trimmed_lines.join("\n")
}
