use crate::graph_sim::InstanceLink;
use crate::parser::{Block, TextRun};
use crate::ui::DocumentView;
use gpui::{div, prelude::*, px, AnyElement, Context, Entity, ParentElement, SharedString, Styled};
use gpui_component::table::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::render_run;
use super::super::expansion_state::ExpandedBlocks;

pub(crate) fn render_table_block(
    _expanded_blocks: &Entity<ExpandedBlocks>,
    doc_blocks: &[Block],
    hubgs_instances: &HashMap<SharedString, (SharedString, SharedString, Vec<InstanceLink>)>,
    footnote_map: &HashMap<SharedString, SharedString>,
    input_state: Entity<gpui_component::input::InputState>,
    headers: &[SharedString],
    rows: &[Vec<Vec<TextRun>>],
    range: &Option<std::ops::Range<usize>>,
    idx: usize,
    cx: &mut Context<DocumentView>,
) -> AnyElement {
    let theme = gpui_component::Theme::global(cx).clone();

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
