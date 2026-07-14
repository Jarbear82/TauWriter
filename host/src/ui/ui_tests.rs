use crate::parser::Block;
use crate::ui::{DocumentHome, ParseState};

#[test]
fn test_ui_document_home_state_transitions_correctly() {
    // Setup initial DocumentHome state (Synced)
    let mut doc = DocumentHome {
        title: "Test".into(),
        author: "Author".into(),
        metadata: Vec::new(),
        blocks: vec![],
        parse_state: ParseState::Synced,
        hubgs_instances: std::collections::HashMap::new(),
    };
    assert_eq!(doc.parse_state, ParseState::Synced);

    // Exercise: Transition to OutOfSync due to a parse error
    doc.parse_state = ParseState::OutOfSync {
        error: "Unclosed tag <bold>".to_string(),
    };

    // Verify: Ensure state is OutOfSync with the correct error payload
    match &doc.parse_state {
        ParseState::OutOfSync { error } => {
            assert_eq!(error, "Unclosed tag <bold>");
        }
        _ => panic!("Expected OutOfSync state"),
    }

    // Exercise: Transition back to Synced
    doc.blocks = vec![Block::Heading {
        level: 1,
        text: "Hello".into(),
        id: None,
        attributes: Vec::new(),
        range: None,
    }];
    doc.parse_state = ParseState::Synced;

    // Verify: Ensure state is Synced and blocks updated
    assert_eq!(doc.parse_state, ParseState::Synced);
    assert_eq!(doc.blocks.len(), 1);
}
