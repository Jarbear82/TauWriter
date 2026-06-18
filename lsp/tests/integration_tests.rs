use tauwriter_lsp::db;
use tauwriter_lsp::RootDatabase;

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
    let twxml_content = "<hubref id='aragorn'>Aragorn</hubref>";
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
    let twxml_content = "<hubref id='gandalf'>Gandalf</hubref>";
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
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("Alien"));
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

    let hubgs_content =
        "INSTANCES [ aragorn:Person { name = 'Aragorn', description = 'Heir of Isildur' } ]";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    assert_eq!(instances.len(), 1);
    assert_eq!(
        instances[0].description(&db).as_ref().unwrap(),
        "Heir of Isildur"
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

    let twxml_content = "<hubref id='aragorn'>Aragorn</hubref>";
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );

    let tokens = db::get_semantic_tokens(&db, twxml_file);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, 2); // VARIABLE
    assert_eq!(tokens[0].character, 12); // After id='
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
fn test_type_hover() {
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
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

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
