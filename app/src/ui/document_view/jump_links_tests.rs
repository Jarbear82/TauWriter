use crate::parser::Block;
use crate::parser::TextRun;
use crate::ui::document_view::jump_links::*;

#[test]
fn test_jump_links_find_block_type_and_range_by_id() {
    let blocks = vec![
        Block::Heading {
            level: 1,
            text: "Introduction".into(),
            id: Some("intro".into()),
            attributes: vec![],
            range: Some(10..20),
        },
        Block::Paragraph {
            runs: vec![TextRun::new("Hello")],
            id: Some("para".into()),
            attributes: vec![],
            range: Some(30..40),
        },
    ];

    assert_eq!(find_block_type_by_id(&blocks, "intro"), Some("Heading"));
    assert_eq!(find_block_type_by_id(&blocks, "para"), Some("Paragraph"));
    assert_eq!(find_block_type_by_id(&blocks, "invalid"), None);

    assert_eq!(find_block_range_by_id(&blocks, "intro"), Some(10..20));
    assert_eq!(find_block_range_by_id(&blocks, "para"), Some(30..40));
    assert_eq!(find_block_range_by_id(&blocks, "invalid"), None);
}
