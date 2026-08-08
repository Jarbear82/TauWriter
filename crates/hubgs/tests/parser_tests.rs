use tauwriter_hubgs::parse_hubgs;

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
fn test_hubgs_parser_handles_escaped_backslash() {
    let sample = r#"
DEFINITIONS [
    HUBS [
        Doc { content }
    ]
]
INSTANCES [
    d: Doc { name = "\\test" }
]
"#;
    let (_defs, instances) = parse_hubgs(sample).unwrap();
    let d = instances.iter().find(|i| i.id == "d").unwrap();
    assert_eq!(d.name, "\\test");
}
