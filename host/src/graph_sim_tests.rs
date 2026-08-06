use super::graph_adapter::{hubgs_definitions_to_graph_state, hubgs_instances_to_graph_state};
use super::graph_sim::*;

#[test]
fn test_parse_hubgs_file_valid() {
    let base = crate::utils::resolve_workspace_root()
        .expect("CARGO_MANIFEST_DIR must resolve to a parent directory");
    let hp = base.join("examples/brave_little_tailor.hubgs");
    assert!(hp.exists());

    let res = parse_hubgs_file(&hp);
    assert!(res.is_ok(), "Failed to parse hubgs: {:?}", res.err());
    let (defs, instances) = res.unwrap();

    // Check definitions
    assert!(!defs.is_empty(), "Definitions should not be empty");
    assert!(defs.iter().any(|d| d.name == "Character"));
    assert!(defs.iter().any(|d| d.name == "Location"));

    // Check instances
    assert!(!instances.is_empty(), "Instances should not be empty");
    assert!(instances.iter().any(|i| i.id == "tailor"));
    assert!(instances.iter().any(|i| i.id == "workshop"));

    // Test adapter conversions to graphene GraphState
    let mut sizer = sizing::fixed_test_sizer();
    let (inst_state, inst_map) = hubgs_instances_to_graph_state(&instances, &defs, &mut sizer);
    assert_eq!(inst_state.node_count(), instances.len());
    assert!(inst_map.contains_key("tailor"));
    assert!(inst_map.contains_key("workshop"));

    let mut sizer2 = sizing::fixed_test_sizer();
    let (def_state, def_map) = hubgs_definitions_to_graph_state(&defs, &mut sizer2);
    assert_eq!(def_state.node_count(), defs.len());
    assert!(def_map.contains_key("Character"));
    assert!(def_map.contains_key("Location"));
}
