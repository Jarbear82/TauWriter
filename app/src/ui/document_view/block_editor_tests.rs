#[cfg(test)]
mod tests {
    use crate::parser::{
        convert_block_to_twxml, extract_plain_text_from_block, generate_block_skeleton,
        parse_twxml, Block,
    };

    fn wrap_doc(body: &str) -> String {
        format!("<document><body>{}</body></document>", body)
    }

    #[test]
    fn test_slash_menu_skeleton_templates_validity() {
        let kinds = [
            "paragraph", "h1", "h2", "h3", "code", "aside", "details", "list", "table", "hubref",
        ];

        for kind in kinds {
            let skeleton = generate_block_skeleton(kind);
            let full_doc = wrap_doc(skeleton);
            let result = parse_twxml(&full_doc);
            assert!(
                result.is_ok(),
                "Failed to parse skeleton template for kind: {}: {:?}",
                kind,
                result.err()
            );
            let (_title, _author, _meta, blocks) = result.unwrap();
            assert!(
                !blocks.is_empty(),
                "Skeleton template produced 0 blocks for kind: {}",
                kind
            );
        }
    }

    #[test]
    fn test_extract_plain_text_from_block() {
        let sample_xml = wrap_doc("<heading level=\"1\">My Title</heading><paragraph>First paragraph text.</paragraph>");
        let (_title, _author, _meta, blocks) = parse_twxml(&sample_xml).unwrap();

        assert_eq!(blocks.len(), 2);
        assert_eq!(extract_plain_text_from_block(&blocks[0]), "My Title");
        assert_eq!(
            extract_plain_text_from_block(&blocks[1]),
            "First paragraph text."
        );
    }

    #[test]
    fn test_convert_block_to_twxml_paragraph_to_heading() {
        let sample_xml = wrap_doc("<paragraph>Sample Paragraph</paragraph>");
        let (_title, _author, _meta, blocks) = parse_twxml(&sample_xml).unwrap();
        let target_block = &blocks[0];

        let h1_markup = convert_block_to_twxml(target_block, "heading");
        assert_eq!(h1_markup, "<heading>Sample Paragraph</heading>");

        let parsed_h1 = parse_twxml(&wrap_doc(&h1_markup)).unwrap().3;
        assert_eq!(parsed_h1.len(), 1);
        match &parsed_h1[0] {
            Block::Heading { text, .. } => {
                assert_eq!(text.as_str(), "Sample Paragraph");
            }
            _ => panic!("Expected Heading AST variant"),
        }
    }

    #[test]
    fn test_convert_block_to_twxml_heading_to_blockquote_and_code() {
        let sample_xml = wrap_doc("<heading>Chapter One</heading>");
        let (_title, _author, _meta, blocks) = parse_twxml(&sample_xml).unwrap();
        let target_block = &blocks[0];

        let bq_markup = convert_block_to_twxml(target_block, "blockquote");
        assert_eq!(bq_markup, "<blockquote>Chapter One</blockquote>");

        let code_markup = convert_block_to_twxml(target_block, "code");
        assert_eq!(code_markup, "<codeblock language=\"rust\">Chapter One</codeblock>");
    }

    #[test]
    fn test_rope_range_replacement_preserves_surrounding_text() {
        let mut document = wrap_doc("<heading>Doc Title</heading>\n<paragraph>Original Para</paragraph>\n<paragraph>End Para</paragraph>");
        let parse1 = parse_twxml(&document).unwrap().3;
        assert_eq!(parse1.len(), 3);

        // Replace second block (<paragraph>Original Para</paragraph>) with <heading>Updated Heading</heading>
        let target_range = parse1[1].range().expect("Range should exist");
        let new_block_markup = "<heading>Updated Heading</heading>";

        document.replace_range(target_range, new_block_markup);

        let parse2 = parse_twxml(&document).unwrap().3;
        assert_eq!(parse2.len(), 3);

        assert_eq!(extract_plain_text_from_block(&parse2[0]), "Doc Title");
        assert_eq!(extract_plain_text_from_block(&parse2[1]), "Updated Heading");
        assert_eq!(extract_plain_text_from_block(&parse2[2]), "End Para");
    }

    #[test]
    fn test_reorder_blocks_downward() {
        use crate::parser::reorder_blocks_in_document;

        let doc = wrap_doc("<heading>Block A</heading>\n<paragraph>Block B</paragraph>\n<paragraph>Block C</paragraph>");
        let blocks = parse_twxml(&doc).unwrap().3;
        assert_eq!(blocks.len(), 3);

        let src_range = blocks[0].range().unwrap();
        let target_range = blocks[2].range().unwrap();

        let reordered_doc = reorder_blocks_in_document(&doc, src_range, target_range);
        let parse2 = parse_twxml(&reordered_doc).unwrap().3;
        assert_eq!(parse2.len(), 3);

        assert_eq!(extract_plain_text_from_block(&parse2[0]), "Block B");
        assert_eq!(extract_plain_text_from_block(&parse2[1]), "Block C");
        assert_eq!(extract_plain_text_from_block(&parse2[2]), "Block A");
    }

    #[test]
    fn test_reorder_blocks_upward() {
        use crate::parser::reorder_blocks_in_document;

        let doc = wrap_doc("<heading>Block A</heading>\n<paragraph>Block B</paragraph>\n<paragraph>Block C</paragraph>");
        let blocks = parse_twxml(&doc).unwrap().3;
        assert_eq!(blocks.len(), 3);

        let src_range = blocks[2].range().unwrap();
        let target_range = blocks[0].range().unwrap();

        let reordered_doc = reorder_blocks_in_document(&doc, src_range, target_range);
        let parse2 = parse_twxml(&reordered_doc).unwrap().3;
        assert_eq!(parse2.len(), 3);

        assert_eq!(extract_plain_text_from_block(&parse2[0]), "Block C");
        assert_eq!(extract_plain_text_from_block(&parse2[1]), "Block A");
        assert_eq!(extract_plain_text_from_block(&parse2[2]), "Block B");
    }

    #[test]
    fn test_wrap_text_in_inline_format() {
        use crate::parser::wrap_text_in_inline_format;

        assert_eq!(wrap_text_in_inline_format("bold text", "bold", None), "<bold>bold text</bold>");
        assert_eq!(wrap_text_in_inline_format("italic text", "italic", None), "<italic>italic text</italic>");
        assert_eq!(wrap_text_in_inline_format("code snippet", "code", None), "<code>code snippet</code>");
        assert_eq!(wrap_text_in_inline_format("underlined", "underline", None), "<u>underlined</u>");
        assert_eq!(
            wrap_text_in_inline_format("Aragorn", "hubref", Some("aragorn_instance")),
            "<hubref id=\"aragorn_instance\">Aragorn</hubref>"
        );
    }

    #[test]
    fn test_detect_markdown_prefix_trigger() {
        use crate::parser::{detect_markdown_prefix_trigger, MarkdownTriggerResult};

        assert_eq!(
            detect_markdown_prefix_trigger("# Main Title"),
            MarkdownTriggerResult::Heading("Main Title".to_string())
        );
        assert_eq!(
            detect_markdown_prefix_trigger("## Sub Section"),
            MarkdownTriggerResult::Section("Sub Section".to_string())
        );
        assert_eq!(
            detect_markdown_prefix_trigger("> Wise quote"),
            MarkdownTriggerResult::BlockQuote("Wise quote".to_string())
        );
        assert_eq!(
            detect_markdown_prefix_trigger("- First bullet"),
            MarkdownTriggerResult::UnorderedList("First bullet".to_string())
        );
        assert_eq!(
            detect_markdown_prefix_trigger("1. Numbered item"),
            MarkdownTriggerResult::OrderedList("Numbered item".to_string())
        );
        assert_eq!(
            detect_markdown_prefix_trigger("Plain text without trigger"),
            MarkdownTriggerResult::NoMatch
        );
    }

    #[test]
    fn test_detect_hubref_completion_trigger() {
        use crate::parser::detect_hubref_completion_trigger;

        let input = "Mentioning @aragorn in paragraph";
        assert_eq!(detect_hubref_completion_trigger(input, 19), Some("aragorn"));

        let input_hash = "Linking #gondor_hub here";
        assert_eq!(detect_hubref_completion_trigger(input_hash, 19), Some("gondor_hub"));

        let plain_input = "Standard text no trigger";
        assert_eq!(detect_hubref_completion_trigger(plain_input, 12), None);
    }

    #[test]
    fn test_multiline_xml_paragraph_run_sanitization() {
        let twxml_sample = wrap_doc(r#"<paragraph>
        One summer morning, a little <bold><italic>tailor</italic></bold> sat on his bench by the door,
        working with <underline>remarkable cheerfulness</underline>. Along came a country woman selling jam
        cakes. "Come, <hubref id="tailor">little tailor</hubref>, try some!"
      </paragraph>"#);

        let parsed = parse_twxml(&twxml_sample).expect("TWXML should parse cleanly");
        assert_eq!(parsed.3.len(), 1);
        let block = &parsed.3[0];
        let plain_text = extract_plain_text_from_block(block);
        assert!(plain_text.contains("One summer morning"));
        assert!(plain_text.contains("little tailor"));
    }

    #[test]
    fn test_normalize_block_text_for_editing() {
        use crate::parser::normalize_block_text_for_editing;

        let multiline_input = "<paragraph>\n        One summer morning, a little <bold><italic>tailor</italic></bold> sat on his bench by the door,\n        working with <underline>remarkable cheerfulness</underline>. Along came a country woman selling jam\n        cakes. \"Come, <hubref id=\"tailor\">little tailor</hubref>, try some!\"\n      </paragraph>";

        let normalized = normalize_block_text_for_editing(multiline_input);
        assert!(!normalized.contains('\n'));
        assert_eq!(
            normalized,
            "<paragraph> One summer morning, a little <bold><italic>tailor</italic></bold> sat on his bench by the door, working with <underline>remarkable cheerfulness</underline>. Along came a country woman selling jam cakes. \"Come, <hubref id=\"tailor\">little tailor</hubref>, try some!\" </paragraph>"
        );
    }

    #[test]
    fn test_table_cell_text_extraction_and_update() {
        use crate::parser::{table_to_twxml, TextRun};

        let headers: Vec<gpui::SharedString> = vec!["Col 1".into(), "Col 2".into()];
        let mut rows: Vec<Vec<Vec<TextRun>>> = vec![
            vec![vec![TextRun::new("A1")], vec![TextRun::new("B1")]],
            vec![vec![TextRun::new("A2")], vec![TextRun::new("B2")]],
        ];

        // Update cell (1, 0)
        rows[1][0] = vec![TextRun::new("Updated A2")];

        let twxml = table_to_twxml(&headers, &rows);
        assert!(twxml.contains("<table>"));
        assert!(twxml.contains("<header>Col 1</header>"));
        assert!(twxml.contains("<cell>Updated A2</cell>"));
    }

    #[test]
    fn test_table_add_row_and_column() {
        use crate::parser::{table_add_column, table_add_row, table_to_twxml, TextRun};

        let mut headers: Vec<gpui::SharedString> = vec!["Name".into(), "Role".into()];
        let mut rows: Vec<Vec<Vec<TextRun>>> = vec![
            vec![vec![TextRun::new("Tailor")], vec![TextRun::new("Hero")]],
        ];

        // Add a row at index 1
        table_add_row(headers.len(), &mut rows, 1);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].len(), 2);

        // Add a column at index 2
        table_add_column(&mut headers, &mut rows, 2);
        assert_eq!(headers.len(), 3);
        assert_eq!(rows[0].len(), 3);
        assert_eq!(rows[1].len(), 3);

        let twxml = table_to_twxml(&headers, &rows);
        assert!(twxml.contains("<header>Header</header>"));
    }

    #[test]
    fn test_table_delete_row_and_column() {
        use crate::parser::{table_delete_column, table_delete_row, TextRun};

        let mut headers: Vec<gpui::SharedString> = vec!["H1".into(), "H2".into(), "H3".into()];
        let mut rows: Vec<Vec<Vec<TextRun>>> = vec![
            vec![vec![TextRun::new("A")], vec![TextRun::new("B")], vec![TextRun::new("C")]],
            vec![vec![TextRun::new("D")], vec![TextRun::new("E")], vec![TextRun::new("F")]],
        ];

        // Delete row 0
        table_delete_row(&mut rows, 0);
        assert_eq!(rows.len(), 1);

        // Delete col 2
        table_delete_column(&mut headers, &mut rows, 2);
        assert_eq!(headers.len(), 2);
        assert_eq!(rows[0].len(), 2);
    }

    #[test]
    fn test_context_menu_wrap_selection() {
        use crate::parser::wrap_text_in_inline_format;

        let sample = "selected prose";
        assert_eq!(wrap_text_in_inline_format(sample, "bold", None), "<bold>selected prose</bold>");
        assert_eq!(wrap_text_in_inline_format(sample, "italic", None), "<italic>selected prose</italic>");
        assert_eq!(wrap_text_in_inline_format(sample, "code", None), "<code>selected prose</code>");
        assert_eq!(wrap_text_in_inline_format(sample, "underline", None), "<u>selected prose</u>");
        assert_eq!(
            wrap_text_in_inline_format(sample, "hubref", Some("hero_instance")),
            "<hubref id=\"hero_instance\">selected prose</hubref>"
        );
    }

    #[test]
    fn test_context_menu_nested_formatting() {
        use crate::parser::wrap_text_in_inline_format;

        let italic_run = "<italic>emphasized text</italic>";
        let nested = wrap_text_in_inline_format(italic_run, "bold", None);
        assert_eq!(nested, "<bold><italic>emphasized text</italic></bold>");
    }

    #[test]
    fn test_match_diagnostics_to_block_ranges() {
        use crate::lsp_client::Diagnostic;
        use crate::ui::document_view::block_editor::match_diagnostics_to_block;

        let doc_text = "<paragraph>First paragraph</paragraph>\n<paragraph>Second paragraph</paragraph>";
        let block_0_range = Some(0..37);
        let block_1_range = Some(38..74);

        let diagnostics = vec![
            Diagnostic { line: 1, severity: 1, message: "Error in line 2".to_string() },
        ];

        assert!(match_diagnostics_to_block(doc_text, &block_0_range, &diagnostics).is_none());
        let matched = match_diagnostics_to_block(doc_text, &block_1_range, &diagnostics);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().message, "Error in line 2");
    }

    #[test]
    fn test_gutter_badge_severity_prioritization() {
        use crate::lsp_client::Diagnostic;
        use crate::ui::document_view::block_editor::match_diagnostics_to_block;

        let doc_text = "<paragraph>Line 1\nLine 2</paragraph>";
        let block_range = Some(0..doc_text.len());

        let diagnostics = vec![
            Diagnostic { line: 0, severity: 2, message: "Warning msg".to_string() },
            Diagnostic { line: 1, severity: 1, message: "Error msg".to_string() },
        ];

        let matched = match_diagnostics_to_block(doc_text, &block_range, &diagnostics);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().severity, 1);
        assert_eq!(matched.unwrap().message, "Error msg");
    }

    #[test]
    fn test_virtual_list_viewport_calculation() {
        use crate::ui::document_view::block_editor::compute_virtual_viewport;

        let total_blocks = 1000;
        let scroll_y = 480.0;
        let container_h = 480.0;
        let block_h = 48.0;
        let overscan = 5;

        let (range, top_spacer, bottom_spacer) =
            compute_virtual_viewport(total_blocks, scroll_y, container_h, block_h, overscan);

        assert_eq!(range, 5..25);
        assert_eq!(top_spacer, 240.0);
        assert_eq!(bottom_spacer, (1000 - 25) as f32 * 48.0);
    }

    #[test]
    fn test_virtual_list_spacer_heights() {
        use crate::ui::document_view::block_editor::compute_virtual_viewport;

        // At top of document
        let (range_top, top_sp_0, _bot_sp_0) =
            compute_virtual_viewport(100, 0.0, 500.0, 50.0, 2);
        assert_eq!(range_top.start, 0);
        assert_eq!(top_sp_0, 0.0);

        // Empty document fallback
        let (range_empty, top_empty, bot_empty) =
            compute_virtual_viewport(0, 100.0, 500.0, 50.0, 5);
        assert_eq!(range_empty, 0..0);
        assert_eq!(top_empty, 0.0);
        assert_eq!(bot_empty, 0.0);
    }
}
