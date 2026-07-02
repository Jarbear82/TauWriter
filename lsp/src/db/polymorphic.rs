/// ponytail: Polymorphic type helpers for EXTENDS-based compatibility checking
use super::resolution::all_hub_types;
use super::types::{HubFieldDef, HubRoleDef, Workspace};

// Re-export the core type from salsa so callers can hold references
pub use super::types::HubType;

/// Check if a HubType is compatible with any of the allowed types in a role's allows list.
/// Compatible means: the type itself or any ancestor in its EXTENDS chain matches.
pub fn hub_type_allows(
    db: &dyn super::Db,
    workspace: Workspace,
    hub_type: &HubType<'_>,
    allowed_types: &[String],
) -> bool {
    if allowed_types.is_empty() {
        return true;
    }

    let all_types = all_hub_types(db, workspace);

    // BFS walk through extends_parents chain; check each type against allowed_types
    let mut queue = hub_type.extends_parents(db).clone();
    queue.push(hub_type.name(db).clone());
    let mut visited = std::collections::HashSet::new();

    while let Some(name) = queue.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        if allowed_types.contains(&name) {
            return true; // Compatible found!
        }
        // Resolve parent type by name (search all workspace types, not just visible)
        if let Some(parent_type) = all_types.iter().find(|t| t.name(db) == name).cloned() {
            for parent in &parent_type.extends_parents(db) {
                queue.push(parent.clone());
            }
        }
    }

    false
}

// ---- Polymorphic field/role resolution (extends inheritance) ----

/// Collect all fields from a type's extends_parents chain.
/// Child definitions override parent definitions for same-name conflicts.
pub fn hub_type_all_fields<'db>(
    db: &'db dyn super::Db,
    workspace: Workspace,
    hub_type: &HubType<'db>,
) -> Vec<HubFieldDef> {
    let all_types = all_hub_types(db, workspace);
    let mut result: Vec<HubFieldDef> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Walk extends_parents chain; child types come first (more specific overrides parent)
    for candidate in collect_type_chain(db, workspace, hub_type, &all_types) {
        for field in candidate.fields(db).iter() {
            if seen_names.insert(field.name.clone()) {
                result.push(field.clone());
            }
        }
    }

    result
}

/// Collect all roles from a type's extends_parents chain.
pub fn hub_type_all_roles<'db>(
    db: &'db dyn super::Db,
    workspace: Workspace,
    hub_type: &HubType<'db>,
) -> Vec<HubRoleDef> {
    let all_types = all_hub_types(db, workspace);
    let mut result: Vec<HubRoleDef> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for candidate in collect_type_chain(db, workspace, hub_type, &all_types) {
        for role in candidate.roles(db).iter() {
            if seen_names.insert(role.name.clone()) {
                result.push(role.clone());
            }
        }
    }

    result
}

/// Resolve a specific field by name across the extends_parents chain. Child overrides parent.
pub fn hub_type_field_by_name<'db>(
    db: &'db dyn super::Db,
    workspace: Workspace,
    hub_type: &HubType<'db>,
    name: &str,
) -> Option<HubFieldDef> {
    let all_types = all_hub_types(db, workspace);
    for candidate in collect_type_chain(db, workspace, hub_type, &all_types) {
        if let Some(field) = candidate.fields(db).iter().find(|f| f.name == name) {
            return Some(field.clone());
        }
    }
    None
}

/// Resolve a specific role by name across the extends_parents chain. Child overrides parent.
pub fn hub_type_role_by_name<'db>(
    db: &'db dyn super::Db,
    workspace: Workspace,
    hub_type: &HubType<'db>,
    name: &str,
) -> Option<HubRoleDef> {
    let all_types = all_hub_types(db, workspace);
    for candidate in collect_type_chain(db, workspace, hub_type, &all_types) {
        if let Some(role) = candidate.roles(db).iter().find(|r| r.name == name) {
            return Some(role.clone());
        }
    }
    None
}

/// Collect all types in the extends_parents chain (BFS, including self).
pub fn collect_hub_types<'db>(
    db: &'db dyn super::Db,
    workspace: Workspace,
    hub_type: &HubType<'db>,
) -> Vec<HubType<'db>> {
    let all_types = all_hub_types(db, workspace);
    collect_type_chain(db, workspace, hub_type, &all_types)
}

fn collect_type_chain<'db>(
    db: &'db dyn super::Db,
    _workspace: Workspace,
    hub_type: &HubType<'db>,
    all_types: &[HubType<'db>],
) -> Vec<HubType<'db>> {
    let mut result = Vec::new();
    let mut queue = vec![hub_type.clone()];
    let mut visited = std::collections::HashSet::new();

    while let Some(current) = queue.pop() {
        if !visited.insert(current.name(db).clone()) {
            continue;
        }
        result.push(current.clone());
        for parent_name in &current.extends_parents(db) {
            if let Some(parent_type) = all_types
                .iter()
                .find(|t| t.name(db) == *parent_name)
                .cloned()
            {
                queue.push(parent_type);
            }
        }
    }

    result
}
