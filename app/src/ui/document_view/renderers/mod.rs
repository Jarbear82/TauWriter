pub(crate) mod tables;

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
use gpui_component::description_list::{DescriptionItem, DescriptionList};
use gpui_component::{scroll::ScrollableElement, Icon, IconName, Theme};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Conventionally accepted code editor colors (VS Code dark style).
static CODE_BLOCK_BG: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.0, 0.12, 1.0));
static CODE_BLOCK_TEXT_COLOR: Lazy<gpui::Hsla> = Lazy::new(|| gpui::hsla(0.0, 0.0, 0.83, 1.0));


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
                .with_body(vec![div()
                    .flex()
                    .flex_wrap()
                    .w_full()
                    .children(body_runs)
                    .into_any_element()])
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
                .bg(theme.accent.opacity(0.12))
                .border_l_4()
                .border_color(theme.accent)
                .rounded_r(px(4.))
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
            attributes: _,
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

            let mut container = div()
                .id("codeblock")
                .w_full()
                .mb_4()
                .border(px(1.))
                .border_color(theme.border)
                .rounded(px(4.))
                .bg(*CODE_BLOCK_BG)
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

            let collapsible_content = CollapsibleBlock::new(
                lang_display.to_string(),
                gpui::hsla(0.0, 0.0, 0.0, 0.0),
                *CODE_BLOCK_BG,
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
            let mut desc_list = DescriptionList::new().bordered(true);
            for (_item_idx, (term, runs)) in items.iter().enumerate() {
                let clean_term = term.replace("\r\n", " ").replace('\n', " ");
                let val_str: String = runs.iter().map(|r| r.text.as_ref()).collect();
                desc_list = desc_list.child(
                    DescriptionItem::new(clean_term).value(val_str),
                );
            }

            div()
                .id(("dl", idx))
                .w_full()
                .mb_4()
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(desc_list)
                .into_any_element()
        }
        Block::Table {
            headers,
            rows,
            id: _,
            attributes: _,
            range,
        } => tables::render_table_block(
            expanded_blocks,
            doc_blocks,
            hubgs_instances,
            footnote_map,
            input_state,
            headers,
            rows,
            range,
            idx,
            cx,
        ),
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
            let review_children: Vec<AnyElement> = blocks
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
                .collect();

            div()
                .id(("review", idx))
                .w_full()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(theme.warning.opacity(0.1))
                .border_l_4()
                .border_color(theme.warning)
                .tooltip(move |window, cx| {
                    gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                })
                .child(
                    div()
                        .mb_2()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.warning)
                        .child(Icon::new(IconName::TriangleAlert).size(gpui::px(13.)))
                        .child("FLAG FOR REVIEW"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .gap_2()
                        .children(review_children),
                )
                .into_any_element()
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

    let mut run_tooltip = element_tooltip("text", &run.id, &run.attributes);

    if let Some(ref hub_id) = run.hubref {
        let hub_id = hub_id.clone();

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
                log::debug!("[app] User clicked on Hub Reference ID: {}", hub_id);
            });
    } else if let Some(ref fn_id) = run.footnote_ref {
        let fn_id = fn_id.clone();

        let footnote_content = footnote_map
            .get(fn_id.as_ref())
            .cloned()
            .unwrap_or_else(|| -> SharedString { "Footnote definition not found".into() });
        run_tooltip = format!(
            "Footnote Ref: {} | {}",
            fn_id,
            footnote_content.replace("\r\n", " ").replace('\n', " ")
        );

        text_el = text_el
            .text_color(theme.accent)
            .underline()
            .hover(|s| s.text_color(theme.accent.opacity(0.8)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                log::debug!("[app] User clicked on Footnote Reference ID: {}", fn_id);
            });
    } else if let Some(ref link) = run.link {
        let link = link.clone();

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
                log::debug!("[app] User clicked internal/external link: {}", link);
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

    text_el
        .id(("run", block_idx * 1000 + run_idx))
        .tooltip(move |window, cx| {
            gpui_component::tooltip::Tooltip::new(clean_run_tooltip.clone()).build(window, cx)
        })
        .on_mouse_down(gpui::MouseButton::Right, move |_, window, cx| {
            if let Some(ref r) = run_range_fmt {
                let formatted =
                    crate::parser::wrap_text_in_inline_format(&run_text_fmt, "bold", None);
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
