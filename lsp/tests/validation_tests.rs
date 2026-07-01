use tauwriter_lsp::db;
use tauwriter_lsp::RootDatabase;

/// Wrap TWXML content in the required skeleton: <document><body>...</body></document>
macro_rules! twxml {
    ($content:expr) => {
        format!("<document><body>{}</body></document>", $content)
    };
}

#[test]
fn test_twxml_invalid_tag_diagnostic() {
    let mut db = RootDatabase::default();

    // 'invalidtag' is not in the list of valid tags
    let twxml_content = twxml!("<invalidtag>Content</invalidtag>");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Unknown TWXML tag 'invalidtag'")));
}

#[test]
fn test_twxml_nesting_rule_diagnostic() {
    let mut db = RootDatabase::default();

    // <heading> is only allowed as direct child of <section>, <document> or <body>
    // Here it is inside <paragraph>, which is invalid.
    let twxml_content =
        twxml!("<section><paragraph><heading>Invalid Nesting</heading></paragraph></section>");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors
        .iter()
        .any(|e| e.message
            == "Invalid nesting: tag 'heading' is not allowed as a child of 'paragraph'"));
}

#[test]
fn test_twxml_unresolved_hubref_diagnostic() {
    let mut db = RootDatabase::default();
    let hubgs_content = "INSTANCES [ aragorn: Character {} ]";
    let twxml_content = twxml!(r#"<hubref id="gandalf"></hubref>"#);

    let hubgs_file =
        db::SourceFile::new(&mut db, "lotr.hubgs".to_string(), hubgs_content.to_string());
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file, twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors
        .iter()
        .any(|e| e.message == "Hub reference 'gandalf' not found"));
}

#[test]
fn test_twxml_self_closing_hubref_field_validation() {
    let mut db = RootDatabase::default();
    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        age: Number
    ],
    HUBS [
        Character { age }
    ]
],
INSTANCES [
    aragorn: Character { age = 87 }
]
";
    let twxml_content_valid = twxml!(r#"<hubref id="aragorn" field="age"/>"#);
    let twxml_content_invalid = twxml!(r#"<hubref id="aragorn" field="nonexistent"/>"#);

    let hubgs_file =
        db::SourceFile::new(&mut db, "lotr.hubgs".to_string(), hubgs_content.to_string());

    let twxml_file_valid = db::SourceFile::new(
        &mut db,
        "story_valid.twxml".to_string(),
        twxml_content_valid.to_string(),
    );
    let workspace_valid = db::Workspace::new(&mut db, vec![hubgs_file, twxml_file_valid]);
    let errors_valid = db::validate_file(&db, workspace_valid, twxml_file_valid);
    assert!(
        errors_valid.is_empty(),
        "Expected no errors, found {:?}",
        errors_valid
    );

    let twxml_file_invalid = db::SourceFile::new(
        &mut db,
        "story_invalid.twxml".to_string(),
        twxml_content_invalid.to_string(),
    );
    let workspace_invalid = db::Workspace::new(&mut db, vec![hubgs_file, twxml_file_invalid]);
    let errors_invalid = db::validate_file(&db, workspace_invalid, twxml_file_invalid);
    assert!(errors_invalid
        .iter()
        .any(|e| e.message.contains("Unknown field 'nonexistent'")));
}

#[test]
fn test_twxml_wrapping_hubref_sync_validation() {
    let mut db = RootDatabase::default();
    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        Character { name }
    ]
],
INSTANCES [
    aragorn: Character { name = 'Elessar' }
]
";
    let twxml_content_sync = twxml!(r#"<hubref id="aragorn" field="name">Elessar</hubref>"#);
    let twxml_content_unsync = twxml!(r#"<hubref id="aragorn" field="name">Strider</hubref>"#);

    let hubgs_file =
        db::SourceFile::new(&mut db, "lotr.hubgs".to_string(), hubgs_content.to_string());

    let twxml_file_sync = db::SourceFile::new(
        &mut db,
        "story_sync.twxml".to_string(),
        twxml_content_sync.to_string(),
    );
    let workspace_sync = db::Workspace::new(&mut db, vec![hubgs_file, twxml_file_sync]);
    let errors_sync = db::validate_file(&db, workspace_sync, twxml_file_sync);
    assert!(
        errors_sync.is_empty(),
        "Expected no errors, found {:?}",
        errors_sync
    );

    let twxml_file_unsync = db::SourceFile::new(
        &mut db,
        "story_unsync.twxml".to_string(),
        twxml_content_unsync.to_string(),
    );
    let workspace_unsync = db::Workspace::new(&mut db, vec![hubgs_file, twxml_file_unsync]);
    let errors_unsync = db::validate_file(&db, workspace_unsync, twxml_file_unsync);
    assert!(errors_unsync
        .iter()
        .any(|e| e.message == "Out of sync: expected 'Elessar', found 'Strider'"));
}

#[test]
fn test_hubgs_type_mismatch_diagnostic() {
    let mut db = RootDatabase::default();
    let hubgs_content = r#"
DEFINITIONS [
    FIELDS [
        age: Number
    ],
    HUBS [
        Character { age }
    ]
],
INSTANCES [
    frodo: Character {
        age = "fifty"
    }
]
"#;
    let hubgs_file =
        db::SourceFile::new(&mut db, "lotr.hubgs".to_string(), hubgs_content.to_string());
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(errors
        .iter()
        .any(|e| e.message == "Type mismatch for field 'age': expected 'Number'"));
}

#[test]
fn test_hubgs_fields_enums_structs_parsing() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        age: Number
    ],
    ENUMS [
        Status { Active, Inactive }
    ],
    STRUCTS [
        Address {
            street,
            city
        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "definitions.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let result = db::parse_hubgs(&db, hubgs_file);

    // 1. Verify Fields
    let fields = result.global_fields(&db);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name(&db), "name");
    assert_eq!(fields[0].type_name(&db), "Text");
    assert_eq!(fields[1].name(&db), "age");
    assert_eq!(fields[1].type_name(&db), "Number");

    // 2. Verify Enums
    let enums = result.enums(&db);
    assert_eq!(enums.len(), 1);
    assert_eq!(enums[0].name(&db), "Status");
    assert_eq!(
        enums[0].variants(&db),
        vec!["Active".to_string(), "Inactive".to_string()]
    );

    // 3. Verify Structs
    let structs = result.structs(&db);
    assert_eq!(structs.len(), 1);
    assert_eq!(structs[0].name(&db), "Address");
    assert_eq!(
        structs[0].field_names(&db),
        vec!["street".to_string(), "city".to_string()]
    );
}

#[test]
fn test_hubgs_decorator_parsing() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        last_name: Text,
        full_name: Text
    ],
    HUBS [
        Person {
            first_name,
            last_name = @default('Doe'),
            full_name = @computed(first_name + ' ' + last_name)
        }
    ]
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "decorators.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let result = db::parse_hubgs(&db, hubgs_file);
    let types = result.types(&db);
    assert_eq!(types.len(), 1);

    let fields = &types[0].fields(&db);
    assert_eq!(fields.len(), 3);

    assert_eq!(fields[0].name, "first_name");
    assert_eq!(fields[0].decorator, None);

    assert_eq!(fields[1].name, "last_name");
    assert_eq!(fields[1].decorator, Some("@default".to_string()));

    assert_eq!(fields[2].name, "full_name");
    assert_eq!(fields[2].decorator, Some("@computed".to_string()));
    assert_eq!(
        fields[2].expression.as_ref().unwrap(),
        "first_name + ' ' + last_name"
    );
}

#[test]
fn test_hubgs_primitive_type_checking() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        age: Number,
        name: Text,
        is_active: Boolean
    ],
    HUBS [
        Person { age, name, is_active }
    ]
],
INSTANCES [
    aragorn:Person {
        age = 87,
        name = 'Aragorn',
        is_active = true
    },
    boromir:Person {
        age = 'Thirty',
        name = 10,
        is_active = 'yes'
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "types.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);

    // boromir should have 3 type mismatch errors
    let boromir_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.message.contains("Type mismatch"))
        .collect();
    assert_eq!(boromir_errors.len(), 3);
    assert!(boromir_errors
        .iter()
        .any(|e| e.message.contains("field 'age'") && e.message.contains("'Number'")));
    assert!(boromir_errors
        .iter()
        .any(|e| e.message.contains("field 'name'") && e.message.contains("'Text'")));
    assert!(boromir_errors
        .iter()
        .any(|e| e.message.contains("field 'is_active'") && e.message.contains("'Boolean'")));
}

#[test]
fn test_hubgs_computed_field_evaluation() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        last_name: Text,
        full_name: Text
    ],
    HUBS [
        Person {
            first_name,
            last_name,
            full_name = @computed(first_name)
        }
    ]
],
INSTANCES [
    aragorn:Person {
        first_name = 'Aragorn',
        last_name = 'Elessar'
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "computed.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let instance = instances[0];

    // full_name should evaluate to 'Aragorn' (as it's computed from first_name)
    let full_name_val = db::compute_field_value(&db, workspace, instance, "full_name".to_string());
    assert_eq!(
        full_name_val,
        Some(db::HubValue::String("Aragorn".to_string()))
    );
}

#[test]
fn test_hubgs_orphaned_instances() {
    let mut db = RootDatabase::default();

    // File with INSTANCES but no DEFINITIONS or IMPORTS
    let hubgs_content = "INSTANCES [ hero:Person { name = 'Hero' } ]";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "orphan.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("no definitions or imports")));
}

#[test]
fn test_hubgs_instances_resolved_via_imports() {
    let mut db = RootDatabase::default();

    // File B has DEFINITIONS
    let schema_content = "
DEFINITIONS [
    FIELDS [ name: Text ],
    HUBS [ Person { name } ]
]
";
    let schema_file = db::SourceFile::new(
        &mut db,
        "schema.hubgs".to_string(),
        schema_content.to_string(),
    );

    // File A has INSTANCES + IMPORTS
    let story_content = "
IMPORTS [ [Person] FROM 'schema.hubgs' ],
INSTANCES [ hero:Person { name = 'Hero' } ]
";
    let story_file = db::SourceFile::new(
        &mut db,
        "story.hubgs".to_string(),
        story_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![schema_file, story_file]);

    // Story file should have zero errors because Person is resolved via import
    let errors = db::validate_file(&db, workspace, story_file);
    assert!(errors.is_empty(), "Expected no errors, found: {:?}", errors);
}

#[test]
fn test_twxml_skeleton_missing_metadata() {
    let mut db = RootDatabase::default();

    // Document with <body> but no <meta /> — still valid (meta tags are optional)
    let twxml_content = "<document><body><paragraph>Hello</paragraph></body></document>";
    let twxml_file = db::SourceFile::new(
        &mut db,
        "no_meta.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(
        errors.is_empty(),
        "Expected no errors for valid document without meta tags, found: {:?}",
        errors
    );
}

#[test]
fn test_twxml_skeleton_missing_body() {
    let mut db = RootDatabase::default();

    // Document with <meta /> but no <body>
    let twxml_content = "<document><meta /></document>";
    let twxml_file = db::SourceFile::new(
        &mut db,
        "no_body.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors
        .iter()
        .any(|e| e.message == "Document missing required <body> block"));
}

#[test]
fn test_twxml_skeleton_valid() {
    let mut db = RootDatabase::default();

    // Valid document without <metadata> — meta tags are optional
    let twxml_content = "<document><body><paragraph>Hello</paragraph></body></document>";
    let twxml_file = db::SourceFile::new(
        &mut db,
        "valid.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(
        !errors
            .iter()
            .any(|e| e.message.contains("missing required") || e.message.contains("Duplicate")),
        "Expected no skeleton errors, found: {:?}",
        errors
    );
}

#[test]
fn test_twxml_invalid_self_closing_tag() {
    let mut db = RootDatabase::default();
    let twxml_content = twxml!("<invalidtag />");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "story.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Unknown TWXML tag 'invalidtag'")));
}

#[test]
fn test_twxml_meta_invalid_nesting() {
    let mut db = RootDatabase::default();
    // <meta /> nested inside <body>/paragraph is invalid
    let twxml_content = "<document><body><paragraph><meta name=\"author\" content=\"Tolkien\"/></paragraph></body></document>";
    let twxml_file = db::SourceFile::new(
        &mut db,
        "nested_meta.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(errors
        .iter()
        .any(|e| e.message == "Invalid nesting: tag 'meta' is only allowed as a direct child of 'document'"));
}

#[test]
fn test_twxml_meta_after_body() {
    let mut db = RootDatabase::default();
    // <meta /> placed after <body> is invalid
    let twxml_content = "<document><body><paragraph>Hello</paragraph></body><meta name=\"author\" content=\"Tolkien\"/></document>";
    let twxml_file = db::SourceFile::new(
        &mut db,
        "ordered_meta.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);
    let language = unsafe { tauwriter_lsp::parser::tree_sitter_twxml() };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(&twxml_content, None).unwrap();
    println!("AST TREE: {}", tree.root_node().to_sexp());

    let errors = db::validate_file(&db, workspace, twxml_file);
    println!("ERRORS: {:?}", errors);
    assert!(errors
        .iter()
        .any(|e| e.message == "Invalid positioning: tag 'meta' must precede the <body> block"));
}

#[test]
fn test_twxml_include_tag_valid() {
    let mut db = RootDatabase::default();
    let twxml_content = twxml!("<include src=\"chapter2.twxml\" />");
    let twxml_file = db::SourceFile::new(
        &mut db,
        "include_valid.twxml".to_string(),
        twxml_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![twxml_file]);

    let errors = db::validate_file(&db, workspace, twxml_file);
    assert!(!errors
        .iter()
        .any(|e| e.message.contains("Invalid include") || e.message.contains("Unknown TWXML tag")));
}

#[test]
fn test_twxml_include_tag_invalid() {
    let mut db = RootDatabase::default();
    // 1. Missing src attribute
    let twxml_content1 = twxml!("<include />");
    let twxml_file1 = db::SourceFile::new(
        &mut db,
        "include_no_src.twxml".to_string(),
        twxml_content1.to_string(),
    );
    let workspace1 = db::Workspace::new(&mut db, vec![twxml_file1]);
    let errors1 = db::validate_file(&db, workspace1, twxml_file1);
    assert!(errors1
        .iter()
        .any(|e| e.message == "Invalid include: tag 'include' must have a non-empty 'src' attribute"));

    // 2. Block-style include is invalid
    let twxml_content2 = twxml!("<include src=\"chapter2.twxml\">Nested content</include>");
    let twxml_file2 = db::SourceFile::new(
        &mut db,
        "include_block.twxml".to_string(),
        twxml_content2.to_string(),
    );
    let workspace2 = db::Workspace::new(&mut db, vec![twxml_file2]);
    let errors2 = db::validate_file(&db, workspace2, twxml_file2);
    assert!(errors2
        .iter()
        .any(|e| e.message == "Invalid include: tag 'include' must be self-closing"));
}


#[test]
fn test_hubgs_unknown_instance_type() {
    let mut db = RootDatabase::default();

    // Instance references a type that doesn't exist anywhere
    let hubgs_content = "
DEFINITIONS [
    FIELDS [ name: Text ],
    HUBS [ Character { name } ]
],
INSTANCES [
    hero:NonExistentType { name = 'Hero' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "unknown_type.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(errors
        .iter()
        .any(|e| e.message == "Unknown Hub type 'NonExistentType'"));
}

#[test]
fn test_hubgs_dependency_validation_imports_satisfy() {
    let mut db = RootDatabase::default();

    // File A defines types
    let defs_content = "
DEFINITIONS [
    FIELDS [ name: Text ],
    HUBS [ Character { name } ]
]
";
    let defs_file =
        db::SourceFile::new(&mut db, "defs.hubgs".to_string(), defs_content.to_string());

    // File B imports types and uses them in INSTANCES — should be valid
    let inst_content = "
IMPORTS [ [Character] FROM 'defs.hubgs' ],
INSTANCES [
    hero:Character { name = 'Hero' }
]
";
    let inst_file = db::SourceFile::new(
        &mut db,
        "instances.hubgs".to_string(),
        inst_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![defs_file, inst_file]);

    let errors = db::validate_file(&db, workspace, inst_file);
    assert!(
        errors.is_empty(),
        "Expected no errors when imports satisfy instance types, found: {:?}",
        errors
    );
}

// ============================================================
// Dynamic Evaluation Engine Tests
// ============================================================

#[test]
fn test_computed_arithmetic_addition() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        price: Number,
        quantity: Number,
        total: Number
    ],
    HUBS [
        OrderItem {
            price,
            quantity,
            total = @computed(price * quantity)
        }
    ]
],
INSTANCES [
    item1:OrderItem { price = 5, quantity = 3 }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "arithmetic.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let instance = instances[0];

    let total_val = db::compute_field_value(&db, workspace, instance, "total".to_string());
    assert_eq!(total_val, Some(db::HubValue::Number("15".to_string())));
}

#[test]
fn test_computed_arithmetic_subtraction_division() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        start: Number,
        end: Number,
        diff: Number,
        half_diff: Number
    ],
    HUBS [
        Range {
            start,
            end,
            diff = @computed(end - start),
            half_diff = @computed(diff / 2)
        }
    ]
],
INSTANCES [
    r1:Range { start = 10, end = 30 }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "arith2.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let instance = instances[0];

    let diff_val = db::compute_field_value(&db, workspace, instance, "diff".to_string());
    assert_eq!(diff_val, Some(db::HubValue::Number("20".to_string())));

    let half_val = db::compute_field_value(&db, workspace, instance, "half_diff".to_string());
    assert_eq!(half_val, Some(db::HubValue::Number("10".to_string())));
}

#[test]
fn test_computed_string_concatenation() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        last_name: Text,
        full_name: Text
    ],
    HUBS [
        Person {
            first_name,
            last_name,
            full_name = @computed(first_name + ' ' + last_name)
        }
    ]
],
INSTANCES [
    p1:Person { first_name = 'Aragorn', last_name = 'Elessar' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "concat.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let instance = instances[0];

    let full_val = db::compute_field_value(&db, workspace, instance, "full_name".to_string());
    assert_eq!(
        full_val,
        Some(db::HubValue::String("Aragorn Elessar".to_string()))
    );
}

#[test]
fn test_computed_parenthesized_expression() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        a: Number,
        b: Number,
        c: Number,
        result: Number
    ],
    HUBS [
        Calc {
            a,
            b,
            c,
            result = @computed((a + b) * c)
        }
    ]
],
INSTANCES [
    c1:Calc { a = 2, b = 3, c = 4 }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "parens.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let instance = instances[0];

    // (2 + 3) * 4 = 20
    let result_val = db::compute_field_value(&db, workspace, instance, "result".to_string());
    assert_eq!(result_val, Some(db::HubValue::Number("20".to_string())));
}

#[test]
fn test_computed_unary_negation() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        value: Number,
        negated: Number
    ],
    HUBS [
        Signed {
            value,
            negated = @computed(-value)
        }
    ]
],
INSTANCES [
    s1:Signed { value = 42 }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "unary.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let instance = instances[0];

    let neg_val = db::compute_field_value(&db, workspace, instance, "negated".to_string());
    assert_eq!(neg_val, Some(db::HubValue::Number("-42".to_string())));
}

#[test]
fn test_cross_hub_role_access_length() {
    let mut db = RootDatabase::default();

    // Test this.companions.length where companions is a role
    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        companion_count: Number
    ],
    HUBS [
        Character {
            name,
            companion_count = @computed(this.associates.length),
            associates <-> (0..*) ALLOWS [Character]
        }
    ]
],
INSTANCES [
    tailor:Character {
        name = 'The Tailor',
        associates = [giant1, giant2]
    },
    giant1:Character { name = 'First Giant' },
    giant2:Character { name = 'Second Giant' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "roles.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let tailor = instances[0];

    let count_val = db::compute_field_value(&db, workspace, tailor, "companion_count".to_string());
    assert_eq!(count_val, Some(db::HubValue::Number("2".to_string())));
}

#[test]
fn test_cross_hub_role_access_field() {
    let mut db = RootDatabase::default();

    // Test cross-Hub field access: owner.name through a role
    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        owner_name: Text
    ],
    HUBS [
        Character {
            name
        },
        Item {
            name,
            owner -> (0..1) ALLOWS [Character],
            owner_name = @computed(this.owner.name)
        }
    ]
],
INSTANCES [
    tailor:Character { name = 'The Tailor' },
    jam:Item {
        name = 'Good Jam',
        owner = [tailor]
    }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "crosshub.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let jam = instances[1];

    // owner_name should resolve to 'The Tailor' via the owner role
    let owner_name_val = db::compute_field_value(&db, workspace, jam, "owner_name".to_string());
    assert_eq!(
        owner_name_val,
        Some(db::HubValue::String("The Tailor".to_string()))
    );
}

#[test]
fn test_default_applied_when_not_overridden() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        status: Text
    ],
    HUBS [
        Person {
            name,
            status = @default('Active')
        }
    ]
],
INSTANCES [
    p1:Person { name = 'Aragorn' },
    p2:Person { name = 'Boromir', status = 'Inactive' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "defaults.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let p1 = instances[0];
    let p2 = instances[1];

    // p1 did not assign status -> should get default 'Active'
    let p1_status = db::compute_field_value(&db, workspace, p1, "status".to_string());
    assert_eq!(p1_status, Some(db::HubValue::String("Active".to_string())));

    // p2 explicitly assigned status -> should keep 'Inactive'
    let p2_status = db::compute_field_value(&db, workspace, p2, "status".to_string());
    assert_eq!(
        p2_status,
        Some(db::HubValue::String("Inactive".to_string()))
    );
}

#[test]
fn test_default_with_expression() {
    let mut db = RootDatabase::default();

    // @default can reference other fields via expression
    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        first_name: Text,
        last_name: Text,
        display_name: Text
    ],
    HUBS [
        Person {
            first_name,
            last_name = @default('Doe'),
            display_name = @computed(first_name + ' ' + last_name)
        }
    ]
],
INSTANCES [
    p1:Person { first_name = 'John' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "defexpr.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let p1 = instances[0];

    // last_name should get default 'Doe', then display_name computes to 'John Doe'
    let last_val = db::compute_field_value(&db, workspace, p1, "last_name".to_string());
    assert_eq!(last_val, Some(db::HubValue::String("Doe".to_string())));

    let display_val = db::compute_field_value(&db, workspace, p1, "display_name".to_string());
    assert_eq!(
        display_val,
        Some(db::HubValue::String("John Doe".to_string()))
    );
}

#[test]
fn test_computed_literal_expression() {
    let mut db = RootDatabase::default();

    // Test that @computed with a pure literal expression works
    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        version: Number
    ],
    HUBS [
        Artifact {
            name,
            version = @computed(1)
        }
    ]
],
INSTANCES [
    a1:Artifact { name = 'Test' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "literal.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let a1 = instances[0];

    let version_val = db::compute_field_value(&db, workspace, a1, "version".to_string());
    assert_eq!(version_val, Some(db::HubValue::Number("1".to_string())));
}

// ============================================================
// Polymorphic Type Resolution Tests (P2/P3)
// ============================================================

#[test]
fn test_polymorphic_validation_child_satisfies_parent_role() {
    // P3: Child type extending parent should validate when role accepts parent
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        Animal {
            name
        },
        Dog EXTENDS [Animal] {
            name,
            role -> (1) ALLOWS [Animal]
        }
    ]
],
INSTANCES [
    buddy:Dog { name = 'Buddy', role = [rex] },
    rex:Animal { name = 'Rex' }
]
";
    let hubgs_file =
        db::SourceFile::new(&mut db, "poly.hubgs".to_string(), hubgs_content.to_string());
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // buddy.role references rex (type Animal), which is in Dog's allowed_types [Animal]
    // This should validate successfully since Buddy is a Dog and Dog extends Animal
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        errors.is_empty(),
        "Expected no validation errors for polymorphic assignment, found: {:?}",
        errors
    );
}

#[test]
fn test_polymorphic_validation_no_extends_fails() {
    // Robot does NOT extend Animal -> should fail role assignment when role expects [Animal]
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        Animal {
            name
        },
        Robot {
            name,
            serve -> (1) ALLOWS [Animal]
        }
    ]
],
INSTANCES [
    robby:Robot { name = 'Robby', serve = [robby] }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "poly_fail.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // whiskers.serve role accepts [Animal], robby is Robot (does NOT extend Animal)
    // This should produce a type mismatch error
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        !errors.is_empty(),
        "Expected type mismatch error for non-extended type"
    );
    assert!(
        errors.iter().any(|e| e.message.contains("Type mismatch")),
        "Expected type mismatch error, found: {:?}",
        errors
    );
}

#[test]
fn test_polymorphic_validation_multi_extends_satisfies_any_parent() {
    // P2: Multi-parent EXTENDS - child should satisfy any parent's role
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        Animal {
            name
        },
        Mammal {
            name
        },
        Dragon EXTENDS [Animal, Mammal] {
            name,
            roar_to -> (1) ALLOWS [Mammal],
            fly_to -> (1) ALLOWS [Animal]
        }
    ]
],
INSTANCES [
    draco:Dragon { name = 'Draco', roar_to = [spike], fly_to = [sparky] },
    spike:Mammal { name = 'Spike' },
    sparky:Animal { name = 'Sparky' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "poly_multi.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // Dragon extends both Animal and Mammal - should satisfy either parent's role
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        errors.is_empty(),
        "Expected no validation errors for multi-extends polymorphic assignment, found: {:?}",
        errors
    );
}

#[test]
fn test_polymorphic_validation_transitive_extends() {
    // Role accepts [LivingThing], instance is Animal (extends LivingThing) -> valid
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        LivingThing {
            name,
            keeper -> (0..1) ALLOWS [LivingThing]
        },
        Animal EXTENDS [LivingThing] {
            name
        }
    ]
],
INSTANCES [
    leo:Animal { name = 'Leo', keeper = [keeper_inst] },
    keeper_inst:LivingThing { name = 'Keeper' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "poly_transitive.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // keeper role accepts [LivingThing], leo's keeper is LivingThing - exact match works
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        errors.is_empty(),
        "Expected no validation errors for transitive extends with direct grandparent instance, found: {:?}",
        errors
    );
}

#[test]
fn test_polymorphic_validation_grandchild_role_match() {
    // When role accepts [LivingThing], an Animal instance (extends LivingThing) should satisfy it
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        LivingThing {
            name,
            keeper -> (0..1) ALLOWS [LivingThing]
        },
        Animal EXTENDS [LivingThing] {
            name,
            parent -> (0..1) ALLOWS [LivingThing]
        }
    ]
],
INSTANCES [
    leo:Animal { name = 'Leo', keeper = [grandparent] },
    grandparent:LivingThing { name = 'Grandparent' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "poly_grandchild.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // keeper role accepts [LivingThing], grandparent is LivingThing - exact match works
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        errors.is_empty(),
        "Expected no validation errors for grandchild-instance filling grandparent-role, found: {:?}",
        errors
    );
}

#[test]
fn test_polymorphic_validation_child_as_parent_instance() {
    // True polymorphism: role accepts [Animal], instance is Dog (extends Animal)
    // This validates the extends_parents traversal works correctly
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        Animal {
            name,
            keeper -> (0..1) ALLOWS [Animal]
        },
        Dog EXTENDS [Animal] {
            name
        }
    ]
],
INSTANCES [
    buddy:Dog { name = 'Buddy' },
    whiskers:Animal { name = 'Whiskers', keeper = [buddy] }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "poly_child_as_parent.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // whiskers.keeper role accepts [Animal], buddy is Dog which extends Animal
    // This should pass via polymorphism (no exact 'Animal' instance needed)
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        errors.is_empty(),
        "Expected no validation errors for child-instance filling parent-role, found: {:?}",
        errors
    );
}

#[test]
fn test_polymorphic_validation_grandchild_as_grandparent_instance() {
    // Grandchild fills grandparent role (two levels of inheritance)
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text
    ],
    HUBS [
        LivingThing {
            name,
            caretaker -> (0..1) ALLOWS [LivingThing]
        },
        Animal EXTENDS [LivingThing] {
            name
        },
        Dog EXTENDS [Animal] {
            name
        }
    ]
],
INSTANCES [
    buddy:Dog { name = 'Buddy' },
    guardian:LivingThing { name = 'Guardian', caretaker = [buddy] }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "poly_grandchild_as_gp.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // caretaker role accepts [LivingThing], buddy is Dog which extends Animal which extends LivingThing
    // This validates the full chain traversal works
    let errors = db::validate_file(&db, workspace, hubgs_file);
    assert!(
        errors.is_empty(),
        "Expected no validation errors for grandchild-instance filling grandparent-role, found: {:?}",
        errors
    );
}

#[test]
fn test_computed_collection_operators() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    FIELDS [
        name: Text,
        companions_count: Number,
        companions_joined: Text
    ],
    HUBS [
        Person {
            name,
            companions <-> (0..*) ALLOWS [Person],
            companions_count = @computed(this.companions.len()),
            companions_joined = @computed(this.companions.map(c => c.name).join(', '))
        }
    ]
],
INSTANCES [
    aragorn:Person { name = 'Aragorn', companions = [gandalf, legolas] },
    gandalf:Person { name = 'Gandalf' },
    legolas:Person { name = 'Legolas' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "collections.hubgs".to_string(),
        hubgs_content.to_string(),
    );
    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    let instances = db::all_hub_instances(&db, workspace);
    let aragorn_inst = instances.iter().find(|i| i.name(&db) == "aragorn").cloned().unwrap();

    let count_val = db::compute_field_value(&db, workspace, aragorn_inst, "companions_count".to_string());
    let joined_val = db::compute_field_value(&db, workspace, aragorn_inst, "companions_joined".to_string());

    assert_eq!(count_val, Some(db::HubValue::Number("2".to_string())));
    assert_eq!(
        joined_val,
        Some(db::HubValue::String("Gandalf, Legolas".to_string()))
    );
}

