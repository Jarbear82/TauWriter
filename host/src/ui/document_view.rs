use crate::parser::{Block, TextRun};
use crate::ui::{DocumentHome, ParseState};
use gpui::{
    Context, Entity, Render, Window, div, prelude::*, px, rgb,
    InteractiveElement, ParentElement, Styled, AnyElement,
};
use gpui_component::scroll::ScrollableElement;

pub(crate) struct DocumentView {
    document_home: Entity<DocumentHome>,
    input_state: Entity<gpui_component::input::InputState>,
    expanded_details: std::collections::HashSet<usize>,
}

impl DocumentView {
    pub(crate) fn new(
        document_home: Entity<DocumentHome>,
        input_state: Entity<gpui_component::input::InputState>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&document_home, |_, _, cx| cx.notify())
            .detach();
        Self {
            document_home,
            input_state,
            expanded_details: std::collections::HashSet::new(),
        }
    }
}

impl Render for DocumentView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (parse_state, blocks) = {
            let doc = self.document_home.read(cx);
            (doc.parse_state.clone(), doc.blocks.clone())
        };

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
                    .bg(rgb(0xfee2e2)) // Soft red background
                    .border(px(1.))
                    .border_color(rgb(0xfecaca))
                    .text_color(rgb(0x991b1b)) // Dark red text
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
        for block in &blocks {
            if let Block::Footnote { .. } = block {
                footnote_blocks.push(block);
            } else {
                main_blocks.push(block);
            }
        }

        // Render main blocks: Paragraph + Aside side-by-side rendering peeker
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
                            .child(div().w(gpui::relative(0.75)).child(self.render_block(block, block_idx, cx)))
                            .child(div().w(gpui::relative(0.25)).child(self.render_block(aside_block, aside_idx, cx)))
                    );
                    continue;
                }
            }
            doc_container = doc_container.child(self.render_block(block, block_idx, cx));
        }

        // Render footnotes at the bottom
        if !footnote_blocks.is_empty() {
            doc_container = doc_container
                .child(div().my_6().h(px(1.)).bg(rgb(0xdddddd))) // Divider
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
                doc_container = doc_container.child(self.render_block(footnote, block_idx, cx));
            }
        }

        doc_container
    }
}

impl DocumentView {
    fn render_block(&self, block: &Block, idx: usize, cx: &mut Context<Self>) -> AnyElement {
        let doc_home = self.document_home.clone();
        let doc_blocks = doc_home.read(cx).blocks.clone();

        match block {
            Block::Heading { level, text, id: _, attributes: _, range: _ } => {
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
            Block::Paragraph { runs, id, attributes, range: _ } => {
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
                    .children(runs.iter().enumerate().map(|(run_idx, run)| self.render_run(run, idx, run_idx, &doc_blocks, cx)))
                    .into_any_element()
            }
            Block::BlockQuote { runs, id, attributes, range } => {
                let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
                let is_collapsed = self.expanded_details.contains(&start_offset);
                
                let view_handle = cx.entity().clone();
                let toggle_state = is_collapsed;
                let tooltip_text = element_tooltip("BlockQuote", id, attributes);

                let mut container = div()
                    .id(("blockquote", idx))
                    .w_full()
                    .mb_4()
                    .border_l_4()
                    .border_color(rgb(0xcccccc))
                    .bg(rgb(0xf0f0f0))
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
                        .bg(rgb(0xe5e7eb))
                        .hover(|s| s.bg(rgb(0xd1d5db)))
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
                        .child(div().italic().text_xs().text_color(rgb(0x666666)).child("Quote"))
                        .child(if is_collapsed { "▶" } else { "▼" })
                );

                if !is_collapsed {
                    container = container.child(
                        div()
                            .p_4()
                            .flex()
                            .flex_wrap()
                            .children(runs.iter().enumerate().map(|(run_idx, run)| self.render_run(run, idx, run_idx, &doc_blocks, cx)))
                    );
                }

                container.into_any_element()
            }
            Block::Aside { runs, id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("Aside", id, attributes);
                div()
                    .id(("aside", idx))
                    .w_full()
                    .mb_4()
                    .p_4()
                    .bg(rgb(0xfefcbf)) // Soft yellow aside
                    .border_l_4()
                    .border_color(rgb(0xd69e2e))
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                    })
                    .flex()
                    .flex_wrap()
                    .children(runs.iter().enumerate().map(|(run_idx, run)| self.render_run(run, idx, run_idx, &doc_blocks, cx)))
                    .into_any_element()
            }
            Block::CodeBlock { language, code, id, attributes, range } => {
                let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
                let is_collapsed = self.expanded_details.contains(&start_offset);
                let trimmed_code = trim_codeblock_indentation(code);
                
                let view_handle = cx.entity().clone();
                let toggle_state = is_collapsed;
                let tooltip_text = element_tooltip("CodeBlock", id, attributes);
                
                let lang_display = if language.is_empty() { "codeblock" } else { language };

                let mut container = div()
                    .id(("codeblock", idx))
                    .w_full()
                    .mb_4()
                    .border(px(1.))
                    .border_color(rgb(0xe2e8f0))
                    .rounded(px(4.))
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
                        .hover(|s| s.bg(rgb(0x3d3d3d)))
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
                        .child(div().child(lang_display.to_string()))
                        .child(if is_collapsed { "▶" } else { "▼" })
                );

                if !is_collapsed {
                    container = container.child(
                        div()
                            .p_4()
                            .text_color(rgb(0xd4d4d4))
                            .font_family("Courier New")
                            .text_size(px(13.))
                            .overflow_x_scrollbar()
                            .child(trimmed_code)
                    );
                }

                container.into_any_element()
            }
            Block::List { ordered, items, id, attributes, range: _ } => {
                let start_offset = if items.is_empty() { 0 } else { 1 };
                let tooltip_text = element_tooltip(if *ordered { "ol" } else { "ul" }, id, attributes);
                let items_elements = items.iter().enumerate().map(|(item_idx, item)| {
                    let bullet_el = if let Some(checked) = item.checked {
                        gpui_component::checkbox::Checkbox::new(format!("chk-{}-{}", start_offset, item_idx))
                            .checked(checked)
                            .mr_2()
                            .into_any_element()
                    } else if *ordered {
                        div().child(format!("{}. ", item_idx + 1)).mr_2().into_any_element()
                    } else {
                        div().child("• ").mr_2().into_any_element()
                    };
                    div()
                        .flex()
                        .items_center()
                        .mb_1()
                        .child(bullet_el)
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .children(item.runs.iter().enumerate().map(|(run_idx, run)| {
                                    self.render_run(run, idx, run_idx + 100 * item_idx, &doc_blocks, cx)
                                }))
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
            Block::DescriptionList { items, id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("dl", id, attributes);
                let items_elements = items.iter().enumerate().map(|(item_idx, (term, runs))| {
                    div()
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::BOLD)
                                .mb_1()
                                .child(term.clone())
                        )
                        .child(
                            div()
                                .pl_4()
                                .mb_2()
                                .flex()
                                .flex_wrap()
                                .children(runs.iter().enumerate().map(|(run_idx, run)| {
                                    self.render_run(run, idx, run_idx + 100 * item_idx, &doc_blocks, cx)
                                }))
                        )
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
            Block::Table { headers, rows, id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("table", id, attributes);
                let mut table_container = div()
                    .id(("table", idx))
                    .w_full()
                    .mb_4()
                    .flex()
                    .flex_col()
                    .border(px(1.))
                    .border_color(rgb(0xdddddd))
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                    });
                
                // Headers
                if !headers.is_empty() {
                    let mut header_row = div().flex().w_full().bg(rgb(0xeeeeee));
                    for header in headers {
                        header_row = header_row.child(
                            div()
                                .flex_1()
                                .p_2()
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(header.clone())
                        );
                    }
                    table_container = table_container.child(header_row);
                }

                // Rows
                for (row_idx, row) in rows.iter().enumerate() {
                    let mut table_row = div().flex().w_full().border_t(px(1.)).border_color(rgb(0xdddddd));
                    for (cell_idx, cell) in row.iter().enumerate() {
                        table_row = table_row.child(
                            div()
                                .flex_1()
                                .p_2()
                                .flex()
                                .flex_wrap()
                                .children(cell.iter().enumerate().map(|(run_idx, run)| {
                                    self.render_run(run, idx, run_idx + 100 * cell_idx + 1000 * row_idx, &doc_blocks, cx)
                                }))
                        );
                    }
                    table_container = table_container.child(table_row);
                }

                table_container.into_any_element()
            }
            Block::HorizontalRule { id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("hr", id, attributes);
                div()
                    .id(("hr", idx))
                    .w_full()
                    .my_4()
                    .h(px(1.))
                    .bg(rgb(0xdddddd))
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                    })
                    .into_any_element()
            }
            Block::Image { src, alt, id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("image", id, attributes);
                div()
                    .id(("image", idx))
                    .w_full()
                    .mb_4()
                    .p_4()
                    .rounded(px(4.))
                    .bg(rgb(0xeeeeee))
                    .flex()
                    .flex_col()
                    .items_center()
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                    })
                    .child(format!("Image: {} (Alt: {})", src, alt))
                    .into_any_element()
            }
            Block::Audio { src, alt, id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("audio", id, attributes);
                div()
                    .id(("audio", idx))
                    .w_full()
                    .mb_4()
                    .p_4()
                    .rounded(px(4.))
                    .bg(rgb(0xeeeeee))
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                    })
                    .child(format!("Audio: {} (Alt: {})", src, alt))
                    .into_any_element()
            }
            Block::Video { src, alt, id, attributes, range: _ } => {
                let tooltip_text = element_tooltip("video", id, attributes);
                div()
                    .id(("video", idx))
                    .w_full()
                    .mb_4()
                    .p_4()
                    .rounded(px(4.))
                    .bg(rgb(0xeeeeee))
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(tooltip_text.clone()).build(window, cx)
                    })
                    .child(format!("Video: {} (Alt: {})", src, alt))
                    .into_any_element()
            }
            Block::Details { summary, blocks, id, attributes, range } => {
                let start_offset = range.as_ref().map(|r| r.start).unwrap_or(0);
                let is_expanded = self.expanded_details.contains(&start_offset);
                let tooltip_text = element_tooltip("details", id, attributes);

                let view_handle = cx.entity().clone();
                let toggle_state = is_expanded;

                let mut container = div()
                    .id(("details", idx))
                    .w_full()
                    .mb_4()
                    .border(px(1.))
                    .border_color(rgb(0xe2e8f0))
                    .rounded(px(4.))
                    .bg(rgb(0xf8fafc))
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
                        .bg(rgb(0xf1f5f9))
                        .hover(|s| s.bg(rgb(0xe2e8f0)))
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
                        .child(div().font_weight(gpui::FontWeight::BOLD).child(summary.clone()))
                        .child(if is_expanded { "▼" } else { "▶" })
                );

                if is_expanded {
                    let mut content = div().p_3().bg(rgb(0xffffff));
                    for (inner_idx, inner_block) in blocks.iter().enumerate() {
                        content = content.child(self.render_block(inner_block, idx + 1000 * inner_idx, cx));
                    }
                    container = container.child(content);
                }

                container.into_any_element()
            }
            Block::Footnote { id, runs, attributes, range: _ } => {
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
                    .child(div().font_weight(gpui::FontWeight::BOLD).child(format!("{}:", id)))
                    .children(runs.iter().enumerate().map(|(run_idx, run)| self.render_run(run, idx, run_idx, &doc_blocks, cx)))
                    .into_any_element()
            }
            Block::Review { blocks, id, attributes, range: _ } => {
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
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(0xdd6b20))
                        .child("⚠️ FLAG FOR REVIEW")
                );

                for (inner_idx, inner_block) in blocks.iter().enumerate() {
                    container = container.child(self.render_block(inner_block, idx + 1000 * inner_idx, cx));
                }
                container.into_any_element()
            }
        }
    }

    fn render_run(&self, run: &TextRun, block_idx: usize, run_idx: usize, blocks: &[Block], cx: &mut Context<Self>) -> AnyElement {
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
                .bg(rgb(0xf0f0f0))
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
            let hubgs_instances = self.document_home.read(cx).hubgs_instances.clone();
            
            // Build HubGS tooltip
            let mut tooltip_text = format!("HubRef: {}", hub_id);
            if let Some((type_name, name, links)) = hubgs_instances.get(&hub_id) {
                tooltip_text = format!("Hub: {}\nname: \"{}\"", type_name, name);
                for (rel, target) in links {
                    tooltip_text.push_str(&format!("\n- {} -> {}", rel, target));
                }
            }
            run_tooltip = tooltip_text;

            text_el = text_el
                .text_color(rgb(0x0066cc))
                .underline()
                .hover(|s| s.text_color(rgb(0x004499)))
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                    println!("[host] User clicked on Hub Reference ID: {}", hub_id);
                });
        } else if let Some(ref fn_id) = run.footnote_ref {
            let fn_id = fn_id.clone();
            let blocks_clone = blocks.to_vec();

            // Build Footnote Ref tooltip
            let footnote_content = find_footnote_content(&blocks_clone, &fn_id);
            run_tooltip = format!("Footnote Ref: {}\n{}", fn_id, footnote_content);

            text_el = text_el
                .text_color(rgb(0x0066cc))
                .underline()
                .hover(|s| s.text_color(rgb(0x004499)))
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                    println!("[host] User clicked on Footnote Reference ID: {}", fn_id);
                });
        } else if let Some(ref link) = run.link {
            let link = link.clone();
            let blocks_clone = blocks.to_vec();
            let input_state = self.input_state.clone();
            let doc_home = self.document_home.clone();

            // Build Link tooltip
            let mut tooltip_text = format!("Link: {}", link);
            if link.starts_with('#') {
                let target_id = &link[1..];
                if let Some(target_type) = find_block_type_by_id(&blocks_clone, target_id) {
                    tooltip_text = format!("Jump to element: {}\nType: {}", target_id, target_type);
                }
            }
            run_tooltip = tooltip_text;

            text_el = text_el
                .text_color(rgb(0x0066cc))
                .underline()
                .hover(|s| s.text_color(rgb(0x004499)))
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    if link.starts_with("http") {
                        cx.open_url(&link);
                    } else if link.starts_with('#') {
                        let target_id = link[1..].to_string();
                        let current_blocks = doc_home.read(cx).blocks.clone();
                        if let Some(target_range) = find_block_range_by_id(&current_blocks, &target_id) {
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
}

// ─── Tooltip & Jump Link Helpers ─────────────────────────────────────────────

fn element_tooltip(tag_name: &str, id: &Option<String>, attributes: &[(String, String)]) -> String {
    let mut attrs_str = String::new();
    for (k, v) in attributes {
        attrs_str.push_str(&format!("\n{}: \"{}\"", k, v));
    }
    format!("Element: {}\nid: {:?}{}", tag_name, id, attrs_str)
}

fn trim_codeblock_indentation(code: &str) -> String {
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

    let trimmed_lines: Vec<String> = lines.into_iter().map(|line| {
        if line.trim().is_empty() {
            String::new()
        } else if line.len() >= min_indent {
            line[min_indent..].to_string()
        } else {
            line.to_string()
        }
    }).collect();

    trimmed_lines.join("\n")
}

fn find_footnote_content(blocks: &[Block], target_id: &str) -> String {
    for block in blocks {
        if let Block::Footnote { id, runs, .. } = block {
            if id == target_id {
                return runs.iter().map(|r| r.text.clone()).collect::<Vec<_>>().join("");
            }
        }
    }
    "Footnote definition not found".to_string()
}

fn find_block_type_by_id(blocks: &[Block], target_id: &str) -> Option<&'static str> {
    for block in blocks {
        match block {
            Block::Heading { id, .. } if id.as_deref() == Some(target_id) => return Some("Heading"),
            Block::Paragraph { id, .. } if id.as_deref() == Some(target_id) => return Some("Paragraph"),
            Block::BlockQuote { id, .. } if id.as_deref() == Some(target_id) => return Some("BlockQuote"),
            Block::Aside { id, .. } if id.as_deref() == Some(target_id) => return Some("Aside"),
            Block::CodeBlock { id, .. } if id.as_deref() == Some(target_id) => return Some("CodeBlock"),
            Block::List { id, .. } if id.as_deref() == Some(target_id) => return Some("List"),
            Block::DescriptionList { id, .. } if id.as_deref() == Some(target_id) => return Some("DescriptionList"),
            Block::Table { id, .. } if id.as_deref() == Some(target_id) => return Some("Table"),
            Block::HorizontalRule { id, .. } if id.as_deref() == Some(target_id) => return Some("HorizontalRule"),
            Block::Image { id, .. } if id.as_deref() == Some(target_id) => return Some("Image"),
            Block::Audio { id, .. } if id.as_deref() == Some(target_id) => return Some("Audio"),
            Block::Video { id, .. } if id.as_deref() == Some(target_id) => return Some("Video"),
            Block::Details { id, blocks: inner, .. } => {
                if id.as_deref() == Some(target_id) { return Some("Details"); }
                if let Some(t) = find_block_type_by_id(inner, target_id) { return Some(t); }
            }
            Block::Footnote { id, .. } if id == target_id => return Some("Footnote"),
            Block::Review { id, blocks: inner, .. } => {
                if id.as_deref() == Some(target_id) { return Some("Review"); }
                if let Some(t) = find_block_type_by_id(inner, target_id) { return Some(t); }
            }
            _ => {}
        }
    }
    None
}

fn find_block_range_by_id(blocks: &[Block], target_id: &str) -> Option<std::ops::Range<usize>> {
    for block in blocks {
        match block {
            Block::Heading { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Paragraph { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::BlockQuote { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Aside { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::CodeBlock { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::List { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::DescriptionList { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Table { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::HorizontalRule { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Image { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Audio { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Video { id, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
            }
            Block::Details { id, blocks: inner, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
                if let Some(r) = find_block_range_by_id(inner, target_id) { return Some(r); }
            }
            Block::Footnote { id, range, .. } => {
                if id == target_id { return range.clone(); }
            }
            Block::Review { id, blocks: inner, range, .. } => {
                if id.as_deref() == Some(target_id) { return range.clone(); }
                if let Some(r) = find_block_range_by_id(inner, target_id) { return Some(r); }
            }
        }
    }
    None
}

fn offset_to_position(text: &str, offset: usize) -> Option<gpui_component::input::Position> {
    let mut row = 0;
    let mut col = 0;
    for (i, c) in text.char_indices() {
        if i >= offset {
            return Some(gpui_component::input::Position::new(row, col));
        }
        if c == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Some(gpui_component::input::Position::new(row, col))
}
