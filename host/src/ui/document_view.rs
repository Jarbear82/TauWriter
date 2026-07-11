use crate::parser::{Block, TextRun};
use crate::ui::DocumentHome;
use gpui::{
    Context, Entity, Render, Window, div, prelude::*, px, rgb,
    InteractiveElement,
};

pub(crate) struct DocumentView {
    document_home: Entity<DocumentHome>,
}

impl DocumentView {
    pub(crate) fn new(document_home: Entity<DocumentHome>, cx: &mut Context<Self>) -> Self {
        cx.observe(&document_home, |_, _, cx| cx.notify())
            .detach();
        Self { document_home }
    }
}

impl Render for DocumentView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let doc = self.document_home.read(cx);

        let mut doc_container = div()
            .id("doc_container")
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xfbfbfb)) // Soft light cream background
            .text_color(rgb(0x222222))
            .overflow_y_scroll()
            .p_8();

        // Document header (Title & Author)
        if !doc.title.is_empty() {
            doc_container = doc_container.child(
                div()
                    .mb_4()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(28.))
                    .child(doc.title.clone()),
            );
        }
        if !doc.author.is_empty() {
            doc_container = doc_container.child(
                div()
                    .mb_8()
                    .text_color(rgb(0x666666))
                    .text_size(px(14.))
                    .child(format!("By {}", doc.author)),
            );
        }

        // Render each block
        for block in &doc.blocks {
            doc_container = doc_container.child(render_block(block));
        }

        doc_container
    }
}

fn render_block(block: &Block) -> impl IntoElement {
    match block {
        Block::Heading { level, text } => {
            let size = match level {
                1 => px(24.),
                2 => px(20.),
                3 => px(18.),
                _ => px(16.),
            };
            div()
                .mt_6()
                .mb_2()
                .font_weight(gpui::FontWeight::BOLD)
                .text_size(size)
                .child(text.clone())
        }
        Block::Paragraph { runs } => {
            div()
                .mb_4()
                .flex()
                .flex_wrap()
                .children(runs.iter().map(|run| render_run(run)))
        }
        Block::BlockQuote { runs } => {
            div()
                .mb_4()
                .p_4()
                .bg(rgb(0xf0f0f0))
                .border_l_4()
                .border_color(rgb(0xcccccc))
                .flex()
                .flex_wrap()
                .children(runs.iter().map(|run| render_run(run)))
        }
        Block::Aside { runs } => {
            div()
                .mb_4()
                .p_4()
                .bg(rgb(0xfefcbf)) // Soft yellow aside
                .border_l_4()
                .border_color(rgb(0xd69e2e))
                .flex()
                .flex_wrap()
                .children(runs.iter().map(|run| render_run(run)))
        }
        Block::CodeBlock { language: _, code } => {
            div()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(rgb(0x1e1e1e))
                .text_color(rgb(0xd4d4d4))
                .font_family("Courier New")
                .text_size(px(13.))
                .child(code.clone())
        }
        Block::List { ordered, items } => {
            let mut list_container = div().mb_4().flex().flex_col().pl_4();
            for (idx, item) in items.iter().enumerate() {
                let bullet = if let Some(checked) = item.checked {
                    if checked { "[x] ".to_string() } else { "[ ] ".to_string() }
                } else if *ordered {
                    format!("{}. ", idx + 1)
                } else {
                    "• ".to_string()
                };
                list_container = list_container.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .child(bullet)
                        .children(item.runs.iter().map(|run| render_run(run)))
                );
            }
            list_container
        }
        Block::DescriptionList { items } => {
            let mut dl_container = div().mb_4().flex().flex_col().pl_4();
            for (term, runs) in items {
                dl_container = dl_container
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
                            .children(runs.iter().map(|run| render_run(run)))
                    );
            }
            dl_container
        }
        Block::Table { headers, rows } => {
            let mut table_container = div().mb_4().flex().flex_col().border(px(1.)).border_color(rgb(0xdddddd));
            
            // Headers
            if !headers.is_empty() {
                let mut header_row = div().flex().bg(rgb(0xeeeeee));
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
            for row in rows {
                let mut table_row = div().flex().border_t(px(1.)).border_color(rgb(0xdddddd));
                for cell in row {
                    table_row = table_row.child(
                        div()
                            .flex_1()
                            .p_2()
                            .flex()
                            .flex_wrap()
                            .children(cell.iter().map(|run| render_run(run)))
                    );
                }
                table_container = table_container.child(table_row);
            }

            table_container
        }
        Block::HorizontalRule => {
            div()
                .my_4()
                .h(px(1.))
                .bg(rgb(0xdddddd))
        }
        Block::Image { src, alt } => {
            div()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(rgb(0xeeeeee))
                .flex()
                .flex_col()
                .items_center()
                .child(format!("Image: {} (Alt: {})", src, alt))
        }
        Block::Audio { src, alt } => {
            div()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(rgb(0xeeeeee))
                .child(format!("Audio: {} (Alt: {})", src, alt))
        }
        Block::Video { src, alt } => {
            div()
                .mb_4()
                .p_4()
                .rounded(px(4.))
                .bg(rgb(0xeeeeee))
                .child(format!("Video: {} (Alt: {})", src, alt))
        }
    }
}

fn render_run(run: &TextRun) -> impl IntoElement {
    if run.text == "\n" {
        return div().w_full();
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

    if let Some(ref hub_id) = run.hubref {
        let hub_id = hub_id.clone();
        text_el = text_el
            .text_color(rgb(0x0066cc))
            .underline()
            .hover(|s| s.text_color(rgb(0x004499)))
            .on_mouse_down(gpui::MouseButton::Left, move |_, _, _cx| {
                println!(
                    "[host] User clicked on Hub Reference ID: {}",
                    hub_id
                );
            });
    } else if let Some(ref _link) = run.link {
        text_el = text_el
            .text_color(rgb(0x0066cc))
            .underline()
            .hover(|s| s.text_color(rgb(0x004499)));
    }

    text_el
}
