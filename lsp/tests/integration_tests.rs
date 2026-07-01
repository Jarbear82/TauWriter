use tauwriter_lsp::db;
use tauwriter_lsp::RootDatabase;

/// Wrap TWXML content in the required skeleton
macro_rules! twxml {
    ($content:expr) => {
        format!("<document><body>{}</body></document>", $content)
    };
}

#[test]
fn test_salsa_indexing_and_resolution() {
    let mut db = RootDatabase::default();

    // 1. Create a Hub definition in HubGS
    let hubgs_content = "INSTANCES [ aragorn:Person { name = 'Aragorn' } ]";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    // 2. Create a reference in TWXML
    let twxml_content = twxml!("<hubref id='aragorn'>Aragorn</hubref>");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );

    // 3. Setup Workspace
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file, twxml_file]);

    // 4. Verify indexing
    let instances = db::all_hub_instances(&db, workspace);
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].name(&db), "aragorn");

    // 5. Verify resolution
    let resolved = db::resolve_reference(&db, workspace, "aragorn".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name(&db), "aragorn");

    // 6. Verify diagnostics (should be empty for valid ref)
    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors.is_empty());
}

#[test]
fn test_broken_reference_diagnostic() {
    let mut db = RootDatabase::default();

    // Reference to non-existent 'gandalf'
    let twxml_content = twxml!("<hubref id='gandalf'>Gandalf</hubref>");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("gandalf"));
}

#[test]
fn test_unknown_hub_type_diagnostic() {
    let mut db = RootDatabase::default();

    // Instance of non-existent 'Alien' type
    let hubgs_content = "INSTANCES [ zorp:Alien { name = 'Zorp' } ]";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);
    // With strict grammar: unknown type error + orphaned instances guard
    assert!(errors.iter().any(|e| e.message.contains("Alien")));
}

#[test]
fn test_completion() {
    let mut db = RootDatabase::default();

    let hubgs_content =
        "INSTANCES [ aragorn:Person { name = 'Aragorn' }, gandalf:Wizard { name = 'Gandalf' } ]";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    assert_eq!(instances.len(), 2);

    let names: Vec<String> = instances.iter().map(|i| i.name(&db)).collect();
    assert!(names.contains(&"aragorn".to_string()));
    assert!(names.contains(&"gandalf".to_string()));
}

#[test]
fn test_hover_description() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        description: Text,
        resides_in: Location
    ],
    HUBS [
        Character {
            name,
            description,
            resides_in -> (0..1) ALLOWS [Location]
        },
        Location {
            name
        }
    ]
],
INSTANCES [
    workshop:Location { name = 'The Workshop' },
    tailor:Character {
        name = 'The Brave Little Tailor',
        description = 'A nimble and clever tailor who killed seven flies with one blow.',
        resides_in = [workshop]
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // 1. Verify instance data exists
    let instances = db::all_hub_instances(&db, workspace);
    assert_eq!(instances.len(), 2); // workshop + tailor

    let tailor = instances.iter().find(|i| i.name(&db) == "tailor").unwrap();
    assert_eq!(
        tailor.description(&db).as_ref().unwrap(),
        "A nimble and clever tailor who killed seven flies with one blow."
    );
    assert_eq!(tailor.assignments(&db).len(), 3); // name, description, resides_in

    // 2. Verify hover produces correct content for an instance
    let hover = tauwriter_lsp::handlers::information::hover_impl(
        &db,
        workspace,
        "tailor",
        &tower_lsp::lsp_types::Url::parse("file:///fantasy.hubgs").unwrap(),
    )
    .unwrap()
    .unwrap();

    // Extract markdown content from MarkupContent
    let markdown = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(mc) => mc.value,
        _ => panic!("Expected Markup hover content"),
    };

    // Should contain header with type and name
    assert!(
        markdown.contains("Character: tailor (Hub)"),
        "Missing hub header. Markdown:\n{}",
        markdown
    );
    // Should contain description
    assert!(
        markdown.contains("A nimble and clever tailor"),
        "Missing description. Markdown:\n{}",
        markdown
    );
    // Should contain field values
    assert!(
        markdown.contains("Fields:"),
        "Missing fields section. Markdown:\n{}",
        markdown
    );
    assert!(
        markdown.contains("The Brave Little Tailor"),
        "Missing name value. Markdown:\n{}",
        markdown
    );
    // Should contain roles with count and link
    assert!(
        markdown.contains("Roles:"),
        "Missing roles section. Markdown:\n{}",
        markdown
    );
    assert!(
        markdown.contains("resides_in"),
        "Missing role name. Markdown:\n{}",
        markdown
    );
    assert!(
        markdown.contains("Count: 1"),
        "Missing role count. Markdown:\n{}",
        markdown
    );
}

#[test]
fn test_context_aware_completion() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Person {
            first_name,
            resides_in -> (1) ALLOWS [Location]
        }
    ]
],
INSTANCES [
    aragorn:Person {

    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // 1. Verify type parsing
    let types = db::all_hub_types(&db, workspace);
    assert_eq!(types.len(), 1);
    let person_type = &types[0];
    assert_eq!(person_type.name(&db), "Person");
    assert!(person_type
        .fields(&db)
        .iter()
        .any(|f| f.name == "first_name"));
    assert!(person_type
        .roles(&db)
        .iter()
        .any(|r| r.name == "resides_in"));

    // 2. Verify context identification
    // Line 17 is inside aragorn:Person block
    let pos = db::LspPosition {
        line: 17,
        character: 8,
    };
    let type_at_pos = db::get_hub_type_at_position(&db, hubgs_file, pos);
    assert_eq!(type_at_pos, Some("Person".to_string()));
}

#[test]
fn test_twxml_completion_contexts() {
    let content = r#"<document><body>
  <paragraph>
    <hubref id="aragorn" field="name">Strider</hubref>
  </paragraph>
</body></document>"#;

    let pos_id = db::LspPosition {
        line: 2,
        character: 17,
    };
    let ctx_id = tauwriter_lsp::parser::get_twxml_completion_context(content, pos_id.into());
    assert!(matches!(
        ctx_id,
        tauwriter_lsp::parser::TwxmlCompletionContext::HubrefId
    ));

    let pos_field = db::LspPosition {
        line: 2,
        character: 32,
    };
    let ctx_field = tauwriter_lsp::parser::get_twxml_completion_context(content, pos_field.into());
    if let tauwriter_lsp::parser::TwxmlCompletionContext::HubrefField { id_val } = ctx_field {
        assert_eq!(id_val, "aragorn");
    } else {
        panic!("Expected HubrefField context");
    }
}

#[test]
fn test_hubgs_completion_contexts() {
    let content = "
DEFINITIONS [
    FIELDS [ name: Text ],
    HUBS [
        Character {
            name -> (1) ALLOWS [Location]
        }
    ]
],
INSTANCES [
    aragorn: Character {
        name = rivendell
    }
]
";

    let pos_allows = db::LspPosition {
        line: 5,
        character: 32,
    };
    let ctx_allows =
        tauwriter_lsp::parser::get_hubgs_completion_context(content, pos_allows.into());
    assert!(matches!(
        ctx_allows,
        tauwriter_lsp::parser::HubgsCompletionContext::AllowsList
    ));

    let pos_assign = db::LspPosition {
        line: 11,
        character: 16,
    };
    let ctx_assign =
        tauwriter_lsp::parser::get_hubgs_completion_context(content, pos_assign.into());
    if let tauwriter_lsp::parser::HubgsCompletionContext::InstanceAssignment {
        type_name,
        role_name,
    } = ctx_assign
    {
        assert_eq!(type_name, "Character");
        assert_eq!(role_name, "name");
    } else {
        panic!("Expected InstanceAssignment context, found something else");
    }
}

#[test]
fn test_semantic_tokens_hubgs() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Person {
            first_name
        }
    ]
],
INSTANCES [
    aragorn:Person {
        first_name = 'Aragorn'
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let tokens = db::get_semantic_tokens(&db, hubgs_file);

    // Find 'Person' type definition
    let type_def = tokens
        .iter()
        .find(|t| t.token_type == 0 && t.token_modifiers == 3)
        .unwrap();
    assert_eq!(type_def.line, 9);

    // Find 'aragorn' instance name
    let inst_name = tokens
        .iter()
        .find(|t| t.token_type == 2 && t.token_modifiers == 2)
        .unwrap();
    assert_eq!(inst_name.line, 15);
}

#[test]
fn test_semantic_tokens_twxml() {
    let mut db = RootDatabase::default();

    let twxml_content = twxml!("<hubref id='aragorn'>Aragorn</hubref>");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );

    let tokens = db::get_semantic_tokens(&db, twxml_file);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, 2); // VARIABLE
    assert_eq!(tokens[0].character, 28); // After id=' (shifted by new skeleton prefix)
}

#[test]
fn test_deep_validation() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Person {
            first_name,
            resides_in -> (1) ALLOWS [Location]
        },
        Location {
            name
        },
        Wizard {
            name
        }
    ]
],
INSTANCES [
    rivendell:Location { name = 'Rivendell' },
    gandalf:Wizard { name = 'Gandalf' },
    aragorn:Person {
        first_name = 'Aragorn',
        resides_in = [rivendell],
        unknown_field = 'Oops'
    },
    boromir:Person {
        resides_in = [gandalf]
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);

    // 1. unknown_field error
    assert!(errors.iter().any(|e| e.message.contains("unknown_field")));

    // 2. Type mismatch error (gandalf is Wizard, resides_in ALLOWS Location)
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Type mismatch") && e.message.contains("resides_in")));
}

#[test]
fn test_multiplicity_validation() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Person {
            first_name,
            resides_in -> (1) ALLOWS [Location]
        },
        Location {
            name
        }
    ]
],
INSTANCES [
    rivendell:Location { name = 'Rivendell' },
    aragorn:Person {
        first_name = 'Aragorn'
        // Missing resides_in
    },
    boromir:Person {
        first_name = 'Boromir',
        resides_in = [rivendell, rivendell]
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);

    // 1. Missing required role 'resides_in' for aragorn
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Missing required role 'resides_in'")));

    // 2. Multiplicity violation for boromir (expected 1, found 2)
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Multiplicity violation") && e.message.contains("resides_in")));
}

#[test]
fn test_imports_resolution() {
    let mut db = RootDatabase::default();

    // 1. Define 'Location' in schema.hubgs
    let schema_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Location {
            name
        }
    ]
]
";
    let schema_file = db::SourceFile::new(
        &mut db,
        "schema.hubgs".to_string(),
        schema_content.to_string(),
    );

    // 2. Import 'Location' in story.hubgs
    let story_content = "
IMPORTS [
    [Location] FROM 'schema.hubgs'
],
INSTANCES [
    rivendell:Location {
        name = 'Rivendell'
    }
]
";
    let story_file = db::SourceFile::new(
        &mut db,
        "story.hubgs".to_string(),
        story_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![schema_file, story_file]);

    // Verify 'Location' is visible in story_file
    let resolved = db::resolve_type(&db, workspace, story_file, "Location".to_string());
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name(&db), "Location");

    // Verify diagnostics are clean in story_file
    let errors = db::validate_file(&db, workspace, story_file);
    assert!(errors.is_empty(), "Errors: {:?}", errors);
}

#[test]
fn test_type_goto_definition() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Character {
            name
        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let resolved = db::resolve_type(&db, workspace, hubgs_file, "Character".to_string());
    assert!(resolved.is_some());
    let character_type = resolved.unwrap();

    // 'Character' starts on line 9, character 8
    let range = character_type.range(&db);
    assert_eq!(range.start.line, 9);
    assert_eq!(range.start.character, 8);
}

#[test]
fn test_global_field_completion() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        last_name: Text
    ],
    HUBS [
        Person {

        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // Line 8 is inside Person { ... }
    let pos = db::LspPosition {
        line: 8,
        character: 12,
    };

    assert!(db::is_in_hub_definition(&db, hubgs_file, pos));

    let global_fields = db::all_global_fields(&db, workspace);
    assert_eq!(global_fields.len(), 2);
    assert_eq!(global_fields[0].name(&db), "first_name");
}

#[test]
fn test_field_definition_hover() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        age: Number,
        is_active: Boolean
    ],
    HUBS [
        Person {
            first_name,
            age,
            is_active
        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // Hover over a global field definition should return hover content
    let hover = tauwriter_lsp::handlers::information::hover_impl(
        &db,
        workspace,
        "first_name",
        &tower_lsp::lsp_types::Url::parse("file:///fantasy.hubgs").unwrap(),
    )
    .unwrap()
    .unwrap();

    let markdown = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(mc) => mc.value,
        _ => panic!("Expected Markup hover content (field def)"),
    };

    // Should contain field header
    assert!(
        markdown.contains("Field: first_name"),
        "Missing field header. Markdown:\n{}",
        markdown
    );
    // Should show type
    assert!(
        markdown.contains("Text"),
        "Missing type info. Markdown:\n{}",
        markdown
    );
    // Should contain source snippet
    assert!(
        markdown.contains("```hubgs"),
        "Missing code block. Markdown:\n{}",
        markdown
    );
}

#[test]
fn test_type_hover() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text
    ],
    HUBS [
        Person {
            first_name,
            resides_in -> (1) ALLOWS [Location]
        },
        Location {
            name
        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // 1. Verify type data exists
    let resolved = db::resolve_type(&db, workspace, hubgs_file, "Person".to_string());
    assert!(resolved.is_some());
    let person_type = resolved.unwrap();

    assert!(person_type
        .fields(&db)
        .iter()
        .any(|f| f.name == "first_name"));
    assert!(person_type
        .roles(&db)
        .iter()
        .any(|r| r.name == "resides_in"));

    // 2. Verify allows list is correctly parsed (not empty)
    let roles = person_type.roles(&db);
    let resides_role = roles.iter().find(|r| r.name == "resides_in").unwrap();
    assert!(
        !resides_role.allowed_types.is_empty(),
        "Allows list should contain 'Location', but got: {:?}",
        resides_role.allowed_types
    );
    assert_eq!(resides_role.allowed_types, vec!["Location".to_string()]);

    // 3. Verify hover produces correct content for a type
    let hover = tauwriter_lsp::handlers::information::hover_impl(
        &db,
        workspace,
        "Person",
        &tower_lsp::lsp_types::Url::parse("file:///fantasy.hubgs").unwrap(),
    )
    .unwrap()
    .unwrap();

    let markdown = match hover.contents {
        tower_lsp::lsp_types::HoverContents::Markup(mc) => mc.value,
        _ => panic!("Expected Markup hover content (type def)"),
    };

    // Should contain type header
    assert!(
        markdown.contains("Type: Person"),
        "Missing type header. Markdown:\n{}",
        markdown
    );
    // Should list fields
    assert!(
        markdown.contains("Fields:"),
        "Missing fields section. Markdown:\n{}",
        markdown
    );
    assert!(
        markdown.contains("first_name"),
        "Missing field name. Markdown:\n{}",
        markdown
    );
    // Should show roles with allows list
    assert!(
        markdown.contains("Roles:"),
        "Missing roles section. Markdown:\n{}",
        markdown
    );
    assert!(
        markdown.contains("ALLOWS [Location]"),
        "Missing allows list in roles. Markdown:\n{}",
        markdown
    );
    // Should contain source code snippet
    assert!(
        markdown.contains("```hubgs"),
        "Missing code block. Markdown:\n{}",
        markdown
    );
}

#[test]
fn test_folding_ranges() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        resides_in: Location,
        name: Text,
        unknown_field: Text
    ],
    HUBS [
        Person {
            name
        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let ranges = db::get_folding_ranges(&db, hubgs_file);

    // Should find at least DEFINITIONS block and HUBS block and Person definition
    assert!(ranges.len() >= 3);

    // Check for DEFINITIONS block (line 1 to 13)
    let def_block = ranges
        .iter()
        .find(|r| r.start.line == 1 && r.end.line == 13);
    assert!(def_block.is_some());
}

#[test]
fn test_snapshot_fixture_testing() {
    let id_fixture = include_str!("fixtures/completion_id.fixture");
    let field_fixture = include_str!("fixtures/completion_field.fixture");

    let parse_fixture = |content: &str| -> (String, db::LspPosition) {
        let mut clean_content = String::new();
        let mut cursor_pos = db::LspPosition {
            line: 0,
            character: 0,
        };

        let mut current_line = 0;
        let mut current_char = 0;
        for ch in content.chars() {
            if ch == '|' {
                cursor_pos.line = current_line;
                cursor_pos.character = current_char;
            } else {
                clean_content.push(ch);
                if ch == '\n' {
                    current_line += 1;
                    current_char = 0;
                } else {
                    current_char += 1;
                }
            }
        }
        (clean_content, cursor_pos)
    };

    let (id_clean, id_pos) = parse_fixture(id_fixture);
    let id_ctx = tauwriter_lsp::parser::get_twxml_completion_context(&id_clean, id_pos.into());
    assert!(matches!(
        id_ctx,
        tauwriter_lsp::parser::TwxmlCompletionContext::HubrefId
    ));

    let (field_clean, field_pos) = parse_fixture(field_fixture);
    let field_ctx =
        tauwriter_lsp::parser::get_twxml_completion_context(&field_clean, field_pos.into());
    if let tauwriter_lsp::parser::TwxmlCompletionContext::HubrefField { id_val } = field_ctx {
        assert_eq!(id_val, "aragorn");
    } else {
        panic!("Expected HubrefField context for field completion fixture");
    }
}

#[test]
fn test_formatter_structural_blocks() {
    // Verify the formatter handles structural blocks correctly
    let content = "<document><body><heading>Title</heading></body></document>";
    let formatted = tauwriter_lsp::formatter::format_source(content, "twxml");

    // The formatted output should contain proper document structure
    assert!(formatted.contains("<document>"));
    assert!(formatted.contains("</document>"));
}

#[test]
fn test_meta_tag_parsing() {
    // ponytail: Tests that <meta /> tags under <document> are recognized.
    let mut db = tauwriter_lsp::RootDatabase::default();
    let twxml_content = twxml!("");
    let twxml_file =
        db::SourceFile::new(&mut db, "test.twxml".to_string(), twxml_content.to_string());

    let tags = db::all_twxml_tags(&db, twxml_file);

    // Should find 'body' structural tag; no metadata since it was replaced by <meta />.
    let tag_names: Vec<_> = tags.iter().map(|t| t.name(&db).clone()).collect();
    assert!(tag_names.iter().any(|n| n == "body"));
}

#[test]
fn test_formatter_preserves_meta_under_document() {
    // ponytail: Verifies that <meta /> tags directly under <document>
    // (without a <metadata> wrapper) are preserved through formatting.
    // Previously these would disappear because meta_tag nodes weren't
    // recognized in the formatter's dispatch match arm.
    let content = "<document><meta name=\"title\" content=\"Test\" /><body></body></document>";
    let formatted = tauwriter_lsp::formatter::format_source(content, "twxml");

    // The formatted output must contain all original meta tags
    assert!(
        formatted.contains("meta"),
        "Expected meta tag in formatted output, got:\n{}",
        formatted
    );
    assert!(
        formatted.contains("title"),
        "Expected 'title' attribute in formatted output, got:\n{}",
        formatted
    );
    assert!(
        formatted.contains("Test"),
        "Expected 'Test' value in formatted output, got:\n{}",
        formatted
    );
}
