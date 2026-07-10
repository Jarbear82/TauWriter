use super::graph_sim::*;

#[test]
fn test_parse_hubgs_file_valid() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
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
    
    // Check simulator layout coordinate bounds
    let (nodes, _edges) = run_graph_simulation(&instances, 500.0, 500.0);
    assert_eq!(nodes.len(), instances.len());
    for node in &nodes {
        assert!(node.x >= 24.0 && node.x <= 476.0, "Node x out of bounds: {}", node.x);
        assert!(node.y >= 24.0 && node.y <= 476.0, "Node y out of bounds: {}", node.y);
    }
    
    let (dnodes, _dedges) = run_def_simulation(&defs, 500.0, 500.0);
    assert_eq!(dnodes.len(), defs.len());
    for node in &dnodes {
        assert!(node.x >= 24.0 && node.x <= 476.0, "Node x out of bounds: {}", node.x);
        assert!(node.y >= 24.0 && node.y <= 476.0, "Node y out of bounds: {}", node.y);
    }
}
