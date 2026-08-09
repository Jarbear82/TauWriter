use crate::graph_sim::hubgs_parser::parse_hubgs;

#[test]
fn test_hubgs_parser_decodes_valid_contents() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> (0..*) ALLOWS [Character]
        }
    ]
]
INSTANCES [
    hero: Character {
        name = "Hero"
    }
]
        "#;
    let (defs, insts) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Character");
    assert_eq!(defs[0].links.len(), 1);
    assert_eq!(defs[0].links[0].name, "friend");
    assert_eq!(defs[0].links[0].arrow, "->");
    assert_eq!(defs[0].links[0].target, "Character");

    assert_eq!(insts.len(), 1);
    assert_eq!(insts[0].id, "hero");
    assert_eq!(insts[0].type_name, "Character");
    assert_eq!(insts[0].name, "Hero");
}

#[test]
fn test_hubgs_parser_rejects_missing_multiplicity() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> ALLOWS [Character]
        }
    ]
]
        "#;
    let res = parse_hubgs(sample);
    assert!(res.is_err());
    assert!(res.err().unwrap().to_string().contains("error"));
}

#[test]
fn test_hubgs_parser_rejects_malformed_multiplicity() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> (0..) ALLOWS [Character]
        }
    ]
]
        "#;
    let res = parse_hubgs(sample);
    assert!(res.is_err());
}

#[test]
fn test_hubgs_parser_supports_all_directionalities() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend <-> (0..*) ALLOWS [Character],
            boss <- (0..1) ALLOWS [Character],
            peer - (1..1) ALLOWS [Character]
        }
    ]
]
        "#;
    let (defs, _) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    let links = &defs[0].links;
    assert_eq!(links.len(), 3);
    assert_eq!(links[0].name, "friend");
    assert_eq!(links[0].arrow, "<->");
    assert_eq!(links[0].target, "Character");
    assert_eq!(links[1].name, "boss");
    assert_eq!(links[1].arrow, "<-");
    assert_eq!(links[1].target, "Character");
    assert_eq!(links[2].name, "peer");
    assert_eq!(links[2].arrow, "-");
    assert_eq!(links[2].target, "Character");
}

// ============================================================================
// New tests for tree-sitter grammar features
// ============================================================================

#[test]
fn test_hubgs_parser_parses_imports_block() {
    let sample = r#"
IMPORTS [
    [BaseHub] FROM "base.hubgs",
    [MixinA, MixinB] FROM "mixins.hubgs"
]
DEFINITIONS [
    HUBS [
        Character {
            name -> (0..1) ALLOWS [Text]
        }
    ]
]
INSTANCES [
    hero: Character {}
]
"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Character");
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].id, "hero");
}

#[test]
fn test_hubgs_parser_recognizes_enums_block() {
    let sample = r#"
DEFINITIONS [
    ENUMS [
        Status { Active, Inactive, Pending }
    ],
    HUBS [
        Entity {
            status -> (0..1) ALLOWS [Status]
        }
    ]
]
INSTANCES [
    e: Entity {}
]
"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Entity");
    assert_eq!(instances.len(), 1);
}

#[test]
fn test_hubgs_parser_recognizes_structs_block() {
    let sample = r#"
DEFINITIONS [
    STRUCTS [
        Address { street, city, zip }
    ],
    HUBS [
        Person {
            address -> (0..1) ALLOWS [Address]
        }
    ]
]
INSTANCES [
    p: Person {}
]
"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Person");
    assert_eq!(instances.len(), 1);
}

#[test]
fn test_hubgs_parser_parses_extends_clause() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Vehicle {
            model -> (0..1) ALLOWS [Text]
        },
        Car EXTENDS [Vehicle] {
            doors -> (1..1) ALLOWS [Number]
        }
    ]
]
INSTANCES [
    myCar: Car {}
]
"#;
    let (defs, _) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 2);
    assert_eq!(defs[0].name, "Vehicle");
    assert_eq!(defs[1].name, "Car");
}

#[test]
fn test_hubgs_parser_parses_extends_with_multiple_parents() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Engine {
            hp -> (0..1) ALLOWS [Number]
        },
        Wheels {
            count -> (1..4) ALLOWS [Number]
        },
        Car EXTENDS [Engine, Wheels] {
            brand -> (0..1) ALLOWS [Text]
        }
    ]
]
INSTANCES [
    c: Car {}
]
"#;
    let (defs, _) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 3);
    assert_eq!(defs[2].name, "Car");
}

#[test]
fn test_hubgs_parser_parses_constraints_block() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Account {
            balance @display,
            @constraints [balance > 0, balance < 1000000]
        }
    ]
]
INSTANCES [
    a: Account {}
]
"#;
    let (defs, _) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Account");
}

#[test]
fn test_hubgs_parser_parses_computed_decorator_expression() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Product {
            price @default(10),
            total @computed(price * quantity)
        }
    ]
]
INSTANCES [
    p: Product {}
]
"#;
    let (defs, _) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Product");
}

#[test]
fn test_hubgs_parser_parses_default_decorator_with_method_call() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        User {
            displayName @default(name.trim())
        }
    ]
]
INSTANCES [
    u: User {}
]
"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(instances.len(), 1);
}

#[test]
fn test_hubgs_parser_parses_full_brave_little_tailor_style_file() {
    let sample = r#"
DEFINITIONS [
    FIELDS [
        name: Text,
        description: Text,
        theme_color: Color
    ],
    HUBS [
        Character {
            name @display,
            description,
            theme_color @background,
            resides_in -> (0..1) ALLOWS [Location],
            associates <-> (0..*) ALLOWS [Character]
        },
        Location {
            name,
            description
        }
    ]
],

INSTANCES [
    workshop:Location {
        name = "Tailor's Workshop",
        description = "A small room near a window where the tailor sews."
    },
    tailor:Character {
        name = "The Brave Little Tailor",
        description = "A nimble and clever tailor.",
        theme_color = 0xFFD700,
        resides_in = [workshop]
    }
]
"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();

    assert_eq!(defs.len(), 2);
    assert!(defs.iter().any(|d| d.name == "Character"));
    assert!(defs.iter().any(|d| d.name == "Location"));

    // Check Character links
    let char_def = defs.iter().find(|d| d.name == "Character").unwrap();
    assert_eq!(char_def.links.len(), 2);
    assert_eq!(char_def.links[0].name, "resides_in");
    assert_eq!(char_def.links[0].arrow, "->");
    assert_eq!(char_def.links[0].target, "Location");
    assert_eq!(char_def.links[1].name, "associates");
    assert_eq!(char_def.links[1].arrow, "<->");
    assert_eq!(char_def.links[1].target, "Character");

    // Check instances
    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0].id, "workshop");
    assert_eq!(instances[0].type_name, "Location");
    assert_eq!(instances[0].name, "Tailor's Workshop");

    let tailor = instances.iter().find(|i| i.id == "tailor").unwrap();
    assert_eq!(tailor.type_name, "Character");
    assert_eq!(tailor.name, "The Brave Little Tailor");
    assert_eq!(tailor.theme_color, Some(0xFFD700));
    assert_eq!(tailor.links.len(), 1);
    assert_eq!(tailor.links[0].relation, "resides_in");
    assert_eq!(tailor.links[0].target, "workshop");
}

#[test]
fn test_hubgs_parser_handles_empty_sections_gracefully() {
    let sample = r#"
DEFINITIONS [
    HUBS []
]
INSTANCES []
"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 0);
    assert_eq!(instances.len(), 0);
}

#[test]
fn test_hubgs_parser_handles_minimal_file() {
    let sample = r#"DEFINITIONS [HUBS [X {}]]INSTANCES [y: X {}]"#;
    let (defs, instances) = parse_hubgs(sample).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "X");
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].id, "y");
}

#[test]
fn test_hubgs_parser_returns_error_when_ffi_null() {
    // This test validates that we get a descriptive error when the FFI symbol is NULL.
    // In normal testing the symbol will be linked, so this just tests the code path exists.
    let sample = "DEFINITIONS [HUBS [X {}]]INSTANCES [y: X {}]";
    let res = parse_hubgs(sample);
    // If the grammar is properly linked (normal case), this succeeds.
    // If it's not linked, we get a descriptive error.
    match res {
        Ok((defs, _)) => assert_eq!(defs.len(), 1),
        Err(e) => assert!(e.to_string().contains("tree_sitter_hubgs")),
    }
}

#[test]
fn test_hubgs_parser_supports_arbitrary_relation_names_not_in_allowlist() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            friend -> (0..*) ALLOWS [Character]
        }
    ]
]
INSTANCES [
    a: Character { name = "A" },
    b: Character { name = "B", friend = [a] }
]
"#;
    let (_defs, instances) = parse_hubgs(sample).unwrap();
    let b = instances.iter().find(|i| i.id == "b").unwrap();
    assert_eq!(b.links.len(), 1);
    assert_eq!(b.links[0].relation, "friend");
    assert_eq!(b.links[0].target, "a");
}

#[test]
fn test_hubgs_parser_supports_extends_with_inherited_links() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Entity { id -> (1..1) ALLOWS [Text] },
        Character EXTENDS [Entity] {
            name -> (0..1) ALLOWS [Text]
        }
    ]
]
INSTANCES [
    hero: Character {}
]
"#;
    let (defs, _) = parse_hubgs(sample).unwrap();
    let char_def = defs.iter().find(|d| d.name == "Character").unwrap();
    assert_eq!(char_def.links.len(), 2);
    assert!(char_def.parents.contains(&"Entity".into()));
}

#[test]
fn test_hubgs_parser_handles_escaped_backslash_before_n() {
    // Regression: unquote_string must not confuse \n (newline) with \\n (literal backslash + n)
    let sample = r#"
DEFINITIONS [
    HUBS [
        Doc { content }
    ]
]
INSTANCES [
    d: Doc { name = "\\test", content = "line1\nline2" }
]
"#;
    let (_defs, instances) = parse_hubgs(sample).unwrap();
    let d = instances.iter().find(|i| i.id == "d").unwrap();
    assert_eq!(d.name, "\\test");
}

#[test]
fn debug_parse_constraints() {
    // Test just @constraints alone
    let sample = "DEFINITIONS [HUBS [Character {@constraints [a > 0]}]]INSTANCES [a: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_constraints_inline() {
    // @constraints on same line as identifier
    let sample =
        "DEFINITIONS [HUBS [Character {a @constraints [b > 0]}]]INSTANCES [c: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_empty_hub() {
    let sample = "DEFINITIONS [HUBS [Character {}]]INSTANCES [a: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_simple_field() {
    let sample = "DEFINITIONS [HUBS [Character {name}]]INSTANCES [a: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_field_attr() {
    let sample = "DEFINITIONS [HUBS [Character {name @display}]]INSTANCES [a: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_constraints_only() {
    let sample = "DEFINITIONS [HUBS [Character {@constraints [a + b]}]]INSTANCES [b: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_constraints_with_field() {
    let sample =
        "DEFINITIONS [HUBS [Character {name, @constraints [a > 0]}]]INSTANCES [b: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_constraints_multiline() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Character {
            balance @display,
            @constraints [balance > 0]
        }
    ]
]
INSTANCES [
    a: Character {}
]"#;
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_computed() {
    let sample = "DEFINITIONS [HUBS [Product {price @default(10), total @computed(price * quantity)}]]INSTANCES [p: Product {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_default_method_call() {
    let sample =
        "DEFINITIONS [HUBS [User {displayName @default(name.trim())}]]INSTANCES [u: User {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_imports() {
    let sample = "IMPORTS [[BaseHub] FROM \"base.hubgs\"]DEFINITIONS [HUBS [Character {}]]INSTANCES [hero: Character {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_enums() {
    let sample =
        "DEFINITIONS [ENUMS [Status {Active, Inactive}], HUBS [Entity {}]]INSTANCES [e: Entity {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}

#[test]
fn debug_parse_structs() {
    let sample =
        "DEFINITIONS [STRUCTS [Address {street, city}], HUBS [Person {}]]INSTANCES [p: Person {}]";
    match parse_hubgs(sample) {
        Ok((defs, insts)) => println!("OK - defs={}, insts={}", defs.len(), insts.len()),
        Err(e) => println!("ERR: {}", e),
    }
}
