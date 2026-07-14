use super::jump_links::{find_block_range_by_id, find_block_type_by_id, offset_to_position};
use crate::parser::{Block, TextRun};
use crate::ui::{DocumentHome, DocumentView};
use gpui::{div, prelude::*, px, rgb, AnyElement, Context, Entity, SharedString};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{Icon, IconName, Theme};

pub(crate) fn render_block(
    expanded_details: &std::collections::HashSet<usize>,
    document_home: &Entity<DocumentHome>,
    input_state: &Entity<gpui_component::input::InputState>,
    block: &Block,
    idx: usize,
    doc_blocks: &[Block],
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
            div()
                .id(("heading", idx))
                .w_full()
                .mt_6()
                .mb_2()
                .font_weight(gpui::FontWeight::BOLD)
                .text_size(size)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(text.clone())
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
                        document_home,
                        input_state,
                        run,
                        idx,
                        run_idx,
                        &doc_blocks,
                        cx,
                        &theme,
                    )
                }))
                .into_any_element()
        }
        Block::BlockQuote {
            runs,
            id,
            attributes,
            range,
        } => {
            let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
            let is_collapsed = expanded_details.contains(&start_offset);

            let view_handle = cx.entity().clone();
            let toggle_state = is_collapsed;
            let tooltip_text = element_tooltip("BlockQuote", id, attributes);

            let mut container = div()
                .id(("blockquote", idx))
                .w_full()
                .mb_4()
                .border_l_4()
                .border_color(theme.border)
                .bg(theme.group_box)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                });

            // Quote Header
            container = container.child(
                div()
                    .id(("blockquote-header", idx))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_1()
                    .bg(theme.accent.opacity(0.3))
                    .hover(|s| s.bg(theme.accent.opacity(0.5)))
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                        let _ = view_handle.update(cx, |this, cx| {
                            if toggle_state {
                                this.expanded_details.remove(&start_offset);
                            } else {
                                this.expanded_details.insert(start_offset);
                            }
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .italic()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Quote"),
                    )
                    .child(if is_collapsed { "▶" } else { "▼" }),
            );

            if !is_collapsed {
                container = container.child(div().p_4().flex().flex_wrap().children(
                    runs.iter().enumerate().map(|(run_idx, run)| {
                        render_run(
                            document_home,
                            input_state,
                            run,
                            idx,
                            run_idx,
                            &doc_blocks,
                            cx,
                            &theme,
                        )
                    }),
                ));
            }

            container.into_any_element()
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
                .border_color(rgb(0xd69e2e)) // Warm amber border for aside/note semantics
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .flex()
                .flex_wrap()
                .children(runs.iter().enumerate().map(|(run_idx, run)| {
                    render_run(
                        document_home,
                        input_state,
                        run,
                        idx,
                        run_idx,
                        &doc_blocks,
                        cx,
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
            let is_collapsed = expanded_details.contains(&start_offset);
            let trimmed_code = trim_codeblock_indentation(code);

            let view_handle = cx.entity().clone();
            let toggle_state = is_collapsed;
            let tooltip_text = element_tooltip("CodeBlock", id, attributes);

            let lang_display = if language.is_empty() {
                "codeblock"
            } else {
                language
            };

            let mut container = div()
                .id(("codeblock", idx))
                .w_full()
                .mb_4()
                .border(px(1.))
                .border_color(theme.border) // Theme border for code block container
                .rounded(px(4.))
                .bg(rgb(0x1e1e1e)) // Fixed dark code editor style — conventional and unchanging
                .bg(rgb(0x1e1e1e))
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                });

            // Header
            container = container.child(
                div()
                    .id(("codeblock-header", idx))
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .bg(rgb(0x2d2d2d))
                    .text_color(rgb(0xd4d4d4))
                    .text_xs()
                    .font_family("Courier New")
                    .hover(|s| s.bg(rgb(0x3d3d3d))) // Code header hover (fixed dark scheme)
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                        let _ = view_handle.update(cx, |this, cx| {
                            if toggle_state {
                                this.expanded_details.remove(&start_offset);
                            } else {
                                this.expanded_details.insert(start_offset);
                            }
                            cx.notify();
                        });
                    })
                    .child(if is_collapsed { "▶" } else { "▼" })
                    .child(div().child(lang_display.to_string())),
            );

            if !is_collapsed {
                container = container.child(
                    div()
                        .p_4()
                        .text_color(rgb(0xd4d4d4))
                        .font_family("Courier New")
                        .text_size(px(13.))
                        .overflow_x_scrollbar()
                        .child(trimmed_code),
                );
            }

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
                                document_home,
                                input_state,
                                run,
                                idx,
                                run_idx + 100 * item_idx,
                                &doc_blocks,
                                cx,
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
                div()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .mb_1()
                            .child(term.clone()),
                    )
                    .child(div().pl_4().mb_2().flex().flex_wrap().children(
                        runs.iter().enumerate().map(|(run_idx, run)| {
                            render_run(
                                document_home,
                                input_state,
                                run,
                                idx,
                                run_idx + 100 * item_idx,
                                &doc_blocks,
                                cx,
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
            id,
            attributes,
            range: _,
        } => {
            let tooltip_text = element_tooltip("table", id, attributes);
            let mut table_container = div()
                .id(("table", idx))
                .w_full()
                .mb_4()
                .flex()
                .flex_col()
                .border(px(1.))
                .border_color(theme.border)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                });

            // Headers
            if !headers.is_empty() {
                let mut header_row = div().flex().w_full().bg(theme.accent.opacity(0.2));
                for header in headers {
                    header_row = header_row.child(
                        div()
                            .flex_1()
                            .p_2()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(header.clone()),
                    );
                }
                table_container = table_container.child(header_row);
            }

            // Rows
            for (row_idx, row) in rows.iter().enumerate() {
                let mut table_row = div()
                    .flex()
                    .w_full()
                    .border_t(px(1.))
                    .border_color(theme.border.opacity(0.5));
                for (cell_idx, cell) in row.iter().enumerate() {
                    table_row = table_row.child(div().flex_1().p_2().flex().flex_wrap().children(
                        cell.iter().enumerate().map(|(run_idx, run)| {
                            render_run(
                                document_home,
                                input_state,
                                run,
                                idx,
                                run_idx + 100 * cell_idx + 1000 * row_idx,
                                &doc_blocks,
                                cx,
                                &theme,
                            )
                        }),
                    ));
                }
                table_container = table_container.child(table_row);
            }

            table_container.into_any_element()
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
            summary,
            blocks,
            id,
            attributes,
            range,
        } => {
            let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
            let is_expanded = expanded_details.contains(&start_offset);
            let tooltip_text = element_tooltip("details", id, attributes);

            let view_handle = cx.entity().clone();
            let toggle_state = is_expanded;

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

            container = container.child(
                div()
                    .id(("details-header", idx))
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .bg(theme.accent.opacity(0.2))
                    .hover(|s| s.bg(theme.accent.opacity(0.4)))
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _window, cx| {
                        let _ = view_handle.update(cx, |this, cx| {
                            if toggle_state {
                                this.expanded_details.remove(&start_offset);
                            } else {
                                this.expanded_details.insert(start_offset);
                            }
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(summary.clone()),
                    )
                    .child(if is_expanded { "▼" } else { "▶" }),
            );

            if is_expanded {
                let mut content = div().p_3().bg(theme.background);
                for (inner_idx, inner_block) in blocks.iter().enumerate() {
                    content = content.child(render_block(
                        expanded_details,
                        document_home,
                        input_state,
                        inner_block,
                        idx + 1000 * inner_idx,
                        doc_blocks,
                        cx,
                    ));
                }
                container = container.child(content);
            }

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
                        document_home,
                        input_state,
                        run,
                        idx,
                        run_idx,
                        &doc_blocks,
                        cx,
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
                .bg(rgb(0xfffaf0)) // Soft yellow warning background
                .border(px(1.))
                .border_color(rgb(0xfeebc8))
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
                    .text_color(rgb(0xdd6b20))
                    .child(Icon::new(IconName::TriangleAlert).size(gpui::px(13.)))
                    .child("FLAG FOR REVIEW"),
            );

            for (inner_idx, inner_block) in blocks.iter().enumerate() {
                container = container.child(render_block(
                    expanded_details,
                    document_home,
                    input_state,
                    inner_block,
                    idx + 1000 * inner_idx,
                    doc_blocks,
                    cx,
                ));
            }
            container.into_any_element()
        }
    }
}

pub(crate) fn render_run(
    document_home: &Entity<DocumentHome>,
    input_state: &Entity<gpui_component::input::InputState>,
    run: &TextRun,
    block_idx: usize,
    run_idx: usize,
    blocks: &[Block],
    cx: &mut Context<DocumentView>,
    theme: &gpui_component::Theme,
) -> AnyElement {
    if run.text == "\n" {
        return div().w_full().into_any_element();
    }
    let mut text_el = div().child(run.text.clone());

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
        let hubgs_instances = document_home.read(cx).hubgs_instances.clone();

        // Build HubGS tooltip
        let mut tooltip_text = format!("HubRef: {}", hub_id);
        if let Some((type_name, name, links)) = hubgs_instances.get(hub_id.as_ref()) {
            tooltip_text = format!("Hub: {}\nname: \"{}\"", type_name, name);
            for link in links {
                tooltip_text.push_str(&format!("\n- {} -> {}", link.relation, link.target));
            }
        }
        run_tooltip = tooltip_text;

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                println!("[host] User clicked on Hub Reference ID: {}", hub_id);
            });
    } else if let Some(ref fn_id) = run.footnote_ref {
        let fn_id = fn_id.clone();

        // Build Footnote Ref tooltip
        let footnote_content = find_footnote_content(blocks, &fn_id);
        run_tooltip = format!("Footnote Ref: {}\n{}", fn_id, footnote_content);

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                println!("[host] User clicked on Footnote Reference ID: {}", fn_id);
            });
    } else if let Some(ref link) = run.link {
        let link = link.clone();
        let input_state = input_state.clone();
        let doc_home = document_home.clone();

        // Build Link tooltip
        let mut tooltip_text = format!("Link: {}", link);
        if link.starts_with('#') {
            let target_id = &link[1..];
            if let Some(target_type) = find_block_type_by_id(blocks, target_id) {
                tooltip_text = format!("Jump to element: {}\nType: {}", target_id, target_type);
            }
        }
        run_tooltip = tooltip_text;

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                if link.starts_with("http") {
                    cx.open_url(&link);
                } else if link.starts_with('#') {
                    let target_id = link[1..].to_string();
                    let current_blocks = doc_home.read(cx).blocks.clone();
                    if let Some(target_range) = find_block_range_by_id(&current_blocks, &target_id)
                    {
                        let value = input_state.read(cx).value().to_string();
                        if let Some(pos) = offset_to_position(&value, target_range.start) {
                            input_state.update(cx, |state, cx| {
                                state.set_cursor_position(pos, window, cx);
                            });
                        }
                    }
                }
            });
    }

    // Attach run tooltip
    text_el
        .id(("run", block_idx * 1000 + run_idx))
        .tooltip(move |window, cx| {
            gpui_component::tooltip::Tooltip::new(run_tooltip.clone()).build(window, cx)
        })
        .into_any_element()
}

pub(crate) fn element_tooltip(
    tag_name: &str,
    id: &Option<SharedString>,
    attributes: &[(String, String)],
) -> String {
    let mut attrs_str = String::new();
    for (k, v) in attributes {
        attrs_str.push_str(&format!("\n{}: \"{}\"", k, v));
    }
    format!("Element: {}\nid: {:?}{}", tag_name, id, attrs_str)
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

pub(crate) fn find_footnote_content(blocks: &[Block], target_id: &str) -> String {
    for block in blocks {
        if let Block::Footnote { id, runs, .. } = block {
            if id == target_id {
                return runs
                    .iter()
                    .map(|r| r.text.clone())
                    .collect::<Vec<_>>()
                    .join("");
            }
        }
    }
    "Footnote definition not found".to_string()
}
