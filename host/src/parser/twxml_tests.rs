use crate::parser::{parse_twxml, Block};

#[test]
fn test_parse_twxml_basic_structures_match() {
    let xml = r#"
        <document>
          <meta name="title" content="Test Document"/>
          <meta name="author" content="Test Author"/>
          <body>
            <heading>Chapter 1</heading>
            <paragraph>Hello <bold>world</bold>!</paragraph>
          </body>
        </document>
        "#;
    let (title, author, _metadata, blocks) = parse_twxml(xml).unwrap();
    assert_eq!(title, "Test Document");
    assert_eq!(author, "Test Author");
    assert_eq!(blocks.len(), 2);

    match &blocks[0] {
        Block::Heading { level, text, .. } => {
            assert_eq!(level, &1);
            assert_eq!(text, "Chapter 1");
        }
        _ => panic!("Expected Heading"),
    }

    match &blocks[1] {
        Block::Paragraph { runs, .. } => {
            assert_eq!(runs.len(), 3);
            assert_eq!(runs[0].text, "Hello ");
            assert!(!runs[0].bold);
            assert_eq!(runs[1].text, "world");
            assert!(runs[1].bold);
            assert_eq!(runs[2].text, "!");
            assert!(!runs[2].bold);
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_parser_whitespace_normalization_and_br_handling_succeeds() {
    let xml = r#"
        <document>
          <body>
            <paragraph>
              This is a very long paragraph that has
              been split across multiple lines for source
              readability.
              It also contains  intentional  double spaces.
              And punctuation,
              like this!
              Here is a line break:<br/>
              And another line break:<br />
              And some <bold>bold
              text</bold> on multiple lines.
            </paragraph>
          </body>
        </document>
        "#;
    let (_, _, _metadata, blocks) = parse_twxml(xml).unwrap();
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::Paragraph { runs, .. } => {
            let text_content: String = runs.iter().map(|r| r.text.as_str()).collect();
            // Check if newlines are removed, double spaces preserved, br tags are \n, and punctuation has no extra space.
            let expected = "This is a very long paragraph that has been split across multiple lines for source readability. It also contains  intentional  double spaces. And punctuation, like this! Here is a line break:\nAnd another line break:\nAnd some bold text on multiple lines.";
            assert_eq!(text_content, expected);

            // Verify that bold style run was correctly normalized and preserved.
            let bold_run = runs.iter().find(|r| r.bold).unwrap();
            assert_eq!(bold_run.text, "bold text");
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_parser_extracts_correct_character_ranges() {
    let xml = "<document><body><heading>Chapter 1</heading><paragraph>Hello <bold>world</bold>!</paragraph></body></document>";
    let (_, _, _metadata, blocks) = parse_twxml(xml).unwrap();
    assert_eq!(blocks.len(), 2);

    // Heading block
    match &blocks[0] {
        Block::Heading { text, range, .. } => {
            assert_eq!(text, "Chapter 1");
            let range = range.as_ref().unwrap();
            assert_eq!(&xml[range.clone()], "<heading>Chapter 1</heading>");
        }
        _ => panic!("Expected Heading"),
    }

    // Paragraph block
    match &blocks[1] {
        Block::Paragraph { runs, range, .. } => {
            let range = range.as_ref().unwrap();
            assert_eq!(
                &xml[range.clone()],
                "<paragraph>Hello <bold>world</bold>!</paragraph>"
            );

            assert_eq!(runs.len(), 3);
            // "Hello " text run
            let r0 = runs[0].range.as_ref().unwrap();
            assert_eq!(&xml[r0.clone()], "Hello ");

            // "<bold>world</bold>" -> text run "world"
            let r1 = runs[1].range.as_ref().unwrap();
            assert_eq!(&xml[r1.clone()], "world");

            // "!" text run
            let r2 = runs[2].range.as_ref().unwrap();
            assert_eq!(&xml[r2.clone()], "!");
        }
        _ => panic!("Expected Paragraph"),
    }
}

#[test]
fn test_parser_handles_details_footnotes_and_reviews_properly() {
    let xml = r#"
        <document>
          <body>
            <details>
              <summary>Show details</summary>
              <paragraph>Hidden paragraph.</paragraph>
            </details>
            <paragraph>Inline footnote reference: <fr id="99"/>.</paragraph>
            <footnote id="99">
              Footnote content.
            </footnote>
            <review>
              <paragraph>To be reviewed.</paragraph>
            </review>
          </body>
        </document>
        "#;
    let (_, _, _metadata, blocks) = parse_twxml(xml).unwrap();
    assert_eq!(blocks.len(), 4);

    // 1. Details block
    match &blocks[0] {
        Block::Details {
            summary,
            blocks: inner_blocks,
            ..
        } => {
            assert_eq!(summary, "Show details");
            assert_eq!(inner_blocks.len(), 1);
            match &inner_blocks[0] {
                Block::Paragraph { runs, .. } => {
                    assert_eq!(runs[0].text, "Hidden paragraph.");
                }
                _ => panic!("Expected paragraph inside details"),
            }
        }
        _ => panic!("Expected Details block"),
    }

    // 2. Paragraph with fr tag
    match &blocks[1] {
        Block::Paragraph { runs, .. } => {
            assert_eq!(runs.len(), 3);
            assert_eq!(runs[0].text, "Inline footnote reference: ");
            assert_eq!(runs[1].text, "[99]");
            assert_eq!(runs[1].footnote_ref.as_deref(), Some("99"));
            assert!(runs[1].subscript);
            assert_eq!(runs[2].text, ".");
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 3. Footnote definition block
    match &blocks[2] {
        Block::Footnote { id, runs, .. } => {
            assert_eq!(id, "99");
            assert_eq!(runs[0].text, "Footnote content.");
        }
        _ => panic!("Expected Footnote block"),
    }

    // 4. Review block
    match &blocks[3] {
        Block::Review {
            blocks: inner_blocks,
            ..
        } => {
            assert_eq!(inner_blocks.len(), 1);
            match &inner_blocks[0] {
                Block::Paragraph { runs, .. } => {
                    assert_eq!(runs[0].text, "To be reviewed.");
                }
                _ => panic!("Expected paragraph inside review"),
            }
        }
        _ => panic!("Expected Review block"),
    }
}
