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
        error: "Unclosed tag <bold>".into(),
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

#[test]
fn test_twxml_language_config_rules() {
    let rules = crate::ui::create_twxml_language_config();
    assert_eq!(rules.brackets.len(), 4);
    assert!(rules.auto_closing_pairs.is_some());
    let pairs = rules.auto_closing_pairs.as_ref().unwrap();
    assert_eq!(pairs.len(), 6);
    assert!(pairs.iter().any(|p| p.open == "<" && p.close == ">"));
    assert!(pairs.iter().any(|p| p.open == "\"" && p.close == "\""));
    assert!(rules.auto_close_before.contains('>'));
}

#[test]
fn test_language_for_path_mapping() {
    use std::path::Path;

    assert_eq!(crate::ui::language_for_path(Path::new("doc.twxml")), "twxml");
    assert_eq!(crate::ui::language_for_path(Path::new("test.xml")), "twxml");
    assert_eq!(crate::ui::language_for_path(Path::new("src/main.rs")), "rust");
    assert_eq!(crate::ui::language_for_path(Path::new("package.json")), "json");
    assert_eq!(crate::ui::language_for_path(Path::new("README.md")), "markdown");
    assert_eq!(crate::ui::language_for_path(Path::new("unknown.xyz")), "plaintext");
    assert_eq!(crate::ui::language_for_path(Path::new("no_extension")), "plaintext");
}

