//! Adapter layer converting TauWriter HubGS and Outline data structures
//! into `graphene_core::GraphState` and layout engine constructs.

use std::collections::HashMap;
use graphene_core::math::{Size2, Vec2};
use graphene_core::{
    CompactString, EdgeData, EdgeDirection, GraphState, NodeData, NodeId, PropValue,
    Properties,
};
use graphene_style::ComputedStyle;

use crate::graph_sim::sizing::NodeContent;
use crate::graph_sim::{HubgsDefinition, HubgsInstance};
use crate::parser::OutlineNode;

/// Convert a slice of HubGS instances into a `GraphState`.
/// Returns the state along with a secondary index mapping `instance_id -> NodeId`.
pub(crate) fn hubgs_instances_to_graph_state(
    instances: &[HubgsInstance],
    definitions: &[HubgsDefinition],
    sizer: &mut impl FnMut(NodeContent) -> (f32, f32),
) -> (GraphState<ComputedStyle>, HashMap<String, NodeId>) {
    let mut state = GraphState::<ComputedStyle>::new();
    let mut id_map = HashMap::new();

    // Map relation names to arrow symbols from definition links
    let mut relation_arrows = HashMap::new();
    for def in definitions {
        for link in &def.links {
            relation_arrows.insert(link.name.as_str(), link.arrow.as_str());
        }
    }

    // 1. Add all instance nodes
    for (idx, inst) in instances.iter().enumerate() {
        let (w, h) = sizer(NodeContent {
            name: inst.name.as_ref(),
            type_name: inst.type_name.as_ref(),
            attributes: &[],
        });
        let size = Size2::new(w, h);
        let pos = Vec2::new((idx % 5) as f32 * 120.0, (idx / 5) as f32 * 100.0);

        let mut props = Properties::new();
        props.insert("@display".into(), PropValue::Text(inst.name.as_str().into()));
        props.insert("id".into(), PropValue::Text(inst.id.as_str().into()));
        props.insert("type".into(), PropValue::Text(inst.type_name.as_str().into()));

        let color_hex = inst.theme_color.map_or_else(
            || match inst.type_name.as_str() {
                "Character" => "#4169E1".to_string(),
                "Location" => "#2ECC71".to_string(),
                "Creature" => "#E67E22".to_string(),
                "Item" => "#9B59B6".to_string(),
                _ => "#7F8C8D".to_string(),
            },
            |c| format!("#{:06X}", c & 0xFFFFFF),
        );
        props.insert("@background".into(), PropValue::Text(color_hex.into()));

        let node_data = NodeData::new([CompactString::from(inst.type_name.as_str())], props);

        let node_id = state.add_node_with_data(pos, size, node_data);
        id_map.insert(inst.id.to_string(), node_id);
    }

    // 2. Add all instance edges
    for inst in instances {
        if let Some(&src_node_id) = id_map.get(inst.id.as_str()) {
            for link in &inst.links {
                if let Some(&tgt_node_id) = id_map.get(link.target.as_str()) {
                    let arrow = relation_arrows
                        .get(link.relation.as_str())
                        .copied()
                        .unwrap_or("-");

                    let direction = match arrow {
                        "->" => EdgeDirection::Directed,
                        "<-" => EdgeDirection::Reverse,
                        "<->" => EdgeDirection::Bidirectional,
                        _ => EdgeDirection::Undirected,
                    };

                    let edge_data = EdgeData::with_label(link.relation.as_str(), direction);
                    state.add_edge_with_data(src_node_id, tgt_node_id, edge_data);
                }
            }
        }
    }

    (state, id_map)
}

/// Convert a slice of HubGS definitions into a `GraphState`.
/// Returns the state along with a secondary index mapping `definition_name -> NodeId`.
pub(crate) fn hubgs_definitions_to_graph_state(
    definitions: &[HubgsDefinition],
    sizer: &mut impl FnMut(NodeContent) -> (f32, f32),
) -> (GraphState<ComputedStyle>, HashMap<String, NodeId>) {
    let mut state = GraphState::<ComputedStyle>::new();
    let mut id_map = HashMap::new();

    // 1. Add definition nodes
    for (idx, def) in definitions.iter().enumerate() {
        let (w, h) = sizer(NodeContent {
            name: def.name.as_ref(),
            type_name: "HubDefinition",
            attributes: &[],
        });
        let size = Size2::new(w, h);
        let pos = Vec2::new((idx % 4) as f32 * 140.0, (idx / 4) as f32 * 110.0);

        let mut props = Properties::new();
        props.insert("@display".into(), PropValue::Text(def.name.as_str().into()));
        props.insert("name".into(), PropValue::Text(def.name.as_str().into()));

        let color_hex = match def.name.as_str() {
            "Character" => "#4169E1",
            "Location" => "#2ECC71",
            "Creature" => "#E67E22",
            "Item" => "#9B59B6",
            _ => "#7F8C8D",
        };
        props.insert("@background".into(), PropValue::Text(color_hex.into()));

        let node_data = NodeData::new([CompactString::from("HubDefinition")], props);

        let node_id = state.add_node_with_data(pos, size, node_data);
        id_map.insert(def.name.to_string(), node_id);
    }

    // 2. Add definition edges
    for def in definitions {
        if let Some(&src_node_id) = id_map.get(def.name.as_str()) {
            for link in &def.links {
                if let Some(&tgt_node_id) = id_map.get(link.target.as_str()) {
                    let direction = match link.arrow.as_str() {
                        "->" => EdgeDirection::Directed,
                        "<-" => EdgeDirection::Reverse,
                        "<->" => EdgeDirection::Bidirectional,
                        _ => EdgeDirection::Undirected,
                    };
                    let edge_data = EdgeData::with_label(link.name.as_str(), direction);
                    state.add_edge_with_data(src_node_id, tgt_node_id, edge_data);
                }
            }
        }
    }

    (state, id_map)
}

/// Convert outline nodes & tree edges into a `GraphState` with hierarchy relationships.
pub(crate) fn outline_to_graph_state(
    nodes: &[OutlineNode],
    edges: &[(usize, usize)],
    sizer: &mut impl FnMut(NodeContent) -> (f32, f32),
) -> (GraphState<ComputedStyle>, HashMap<String, NodeId>) {
    let mut state = GraphState::<ComputedStyle>::new();
    let mut index_to_node_id = HashMap::new();
    let mut string_id_map = HashMap::new();

    // 1. Add nodes
    for (idx, node) in nodes.iter().enumerate() {
        let (w, h) = sizer(NodeContent {
            name: &node.name,
            type_name: &node.kind,
            attributes: &[],
        });
        let size = Size2::new(w, h);
        let pos = Vec2::new((idx % 3) as f32 * 160.0, (idx / 3) as f32 * 90.0);

        let mut props = Properties::new();
        props.insert("@display".into(), PropValue::Text(node.name.as_str().into()));
        props.insert("kind".into(), PropValue::Text(node.kind.as_str().into()));

        let color_hex = match node.kind.as_str() {
            "section" => "#3B82F6",
            "heading" => "#10B981",
            "paragraph" => "#9CA3AF",
            "hubref" => "#F59E0B",
            _ => "#8B5CF6",
        };
        props.insert("@background".into(), PropValue::Text(color_hex.into()));

        let node_data = NodeData::new([CompactString::from(node.kind.as_str())], props);

        let node_id = state.add_node_with_data(pos, size, node_data);
        index_to_node_id.insert(idx, node_id);
        string_id_map.insert(node.id.clone(), node_id);
    }

    // 2. Set hierarchy parent-child links & tree edges
    for &(parent_idx, child_idx) in edges {
        if let (Some(&parent_id), Some(&child_id)) = (
            index_to_node_id.get(&parent_idx),
            index_to_node_id.get(&child_idx),
        ) {
            state.reparent_node(child_id, Some(parent_id));
            let edge_data = EdgeData::with_label("child", EdgeDirection::Directed);
            state.add_edge_with_data(parent_id, child_id, edge_data);
        }
    }

    (state, string_id_map)
}
