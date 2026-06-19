use tauwriter_lsp::db;
use tauwriter_lsp::RootDatabase;

#[test]
fn test_workspace_symbols() {
    let mut db = RootDatabase::default();

    let hubgs_content = "
DEFINITIONS [
    HUBS [
        Character {
            name
        }
    ]
],
INSTANCES [
    aragorn:Character { name = 'Aragorn' }
]
";
    let hubgs_file = db::SourceFile::new(
        &mut db,
        "fantasy.hubgs".to_string(),
        hubgs_content.to_string(),
    );

    let workspace = db::Workspace::new(&mut db, vec![hubgs_file]);

    // Test finding Hub Instances
    let instances = db::all_hub_instances(&db, workspace);
    assert!(instances.iter().any(|i| i.name(&db) == "aragorn"));

    // Test finding Hub Types
    let types = db::all_hub_types(&db, workspace);
    assert!(types.iter().any(|t| t.name(&db) == "Character"));
}
