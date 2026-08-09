use lsp_types::Position;

#[derive(Debug)]
pub enum HubgsCompletionContext {
    AllowsList,
    InstanceAssignment {
        type_name: String,
        role_name: String,
    },
    None,
}

pub fn get_hubgs_completion_context(contents: &str, pos: Position) -> HubgsCompletionContext {
    let language = match super::super::get_hubgs_language() {
        Some(lang) => lang,
        None => return HubgsCompletionContext::None,
    };
    let mut parser = tree_sitter::Parser::new();
    if let Err(_) = parser.set_language(&language) {
        return HubgsCompletionContext::None;
    }
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return HubgsCompletionContext::None,
    };

    let ts_pos = tree_sitter::Point {
        row: pos.line as usize,
        column: pos.character as usize,
    };

    let node = match tree.root_node().descendant_for_point_range(ts_pos, ts_pos) {
        Some(n) => n,
        None => return HubgsCompletionContext::None,
    };

    // Check if cursor is after ALLOWS keyword in a hub_role
    let mut current = node;
    while current.kind() != "source_file" {
        if current.kind() == "hub_role" {
            let mut allows_node = None;
            let mut cursor = current.walk();
            for child in current.children(&mut cursor) {
                if child.kind() == "ALLOWS" || &contents[child.byte_range()] == "ALLOWS" {
                    allows_node = Some(child);
                    break;
                }
            }
            if let Some(an) = allows_node {
                if node.start_byte() > an.end_byte() {
                    return HubgsCompletionContext::AllowsList;
                }
            }
            break;
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    // Check for instance assignment context
    let mut current = node;
    let mut assignment_node = None;
    while current.kind() != "source_file" {
        if current.kind() == "instance_assignment" {
            assignment_node = Some(current);
            break;
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    if let Some(assign) = assignment_node {
        if let Some(id_node) = assign.child(0) {
            let role_name = contents[id_node.byte_range()].trim().to_string();

            let mut inst_block = assign;
            while inst_block.kind() != "instance_block" && inst_block.kind() != "source_file" {
                if let Some(p) = inst_block.parent() {
                    inst_block = p;
                } else {
                    break;
                }
            }

            if inst_block.kind() == "instance_block" {
                if let Some(type_node) = inst_block.child_by_field_name("type") {
                    let type_name = contents[type_node.byte_range()].trim().to_string();
                    return HubgsCompletionContext::InstanceAssignment {
                        type_name,
                        role_name,
                    };
                }
            }
        }
    }

    HubgsCompletionContext::None
}
