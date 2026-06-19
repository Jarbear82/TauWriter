use tauwriter_lsp::db;
use tauwriter_lsp::RootDatabase;

#[test]
fn test_twxml_invalid_tag_diagnostic() {
    let mut db = RootDatabase::default();

    // 'invalidtag' is not in the list of valid tags
    let twxml_content = "<document><invalidtag>Content</invalidtag></document>";
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

    // <heading> is only allowed as direct child of <section> or <document>
    // Here it is inside <paragraph>, which is invalid.
    let twxml_content = "<document><section><paragraph><heading>Invalid Nesting</heading></paragraph></section></document>";
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
    let twxml_content = r#"<document><hubref id="gandalf"></hubref></document>"#;

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
fn test_hubgs_type_mismatch_diagnostic() {
    let mut db = RootDatabase::default();
    let hubgs_content = r#"
DEFINITIONS [
    FIELDS [
        age: Number
    ]
    HUBS [
        Character { age }
    ]
]
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
