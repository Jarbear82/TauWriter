use super::Multiplicity;
/// Hubgs semantic validation rules.
use crate::db::{polymorphic, resolution, HubValue, ValidationError};

pub(crate) fn validate_hubgs(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    file: crate::db::SourceFile,
    errors: &mut Vec<ValidationError>,
) {
    let result = resolution::parse_hubgs(db, file.clone());

    if !result.instances(db).is_empty()
        && result.types(db).is_empty()
        && result.imports(db).is_empty()
    {
        for instance in result.instances(db) {
            errors.push(ValidationError {
                range: instance.range(db),
                message: format!(
                    "Instance '{}' uses type '{}' but no definitions or imports are present",
                    instance.name(db),
                    instance.type_name(db)
                ),
            });
        }
    }

    let global_fields = resolution::all_global_fields(db, workspace.clone());

    for hub_type in result.types(db) {
        for field in hub_type.fields(db) {
            if !global_fields.iter().any(|gf| gf.name(db) == field.name) {
                errors.push(ValidationError {
                    range: field.range,
                    message: format!(
                        "Field '{}' used in Hub '{}' must be defined in a FIELDS block",
                        field.name,
                        hub_type.name(db)
                    ),
                });
            }
        }
    }

    for instance in result.instances(db) {
        let type_name = instance.type_name(db);
        if let Some(hub_type) =
            resolution::resolve_type(db, workspace.clone(), file.clone(), type_name.clone())
        {
            validate_instance_assignments(
                db,
                workspace.clone(),
                &instance,
                hub_type,
                &global_fields,
                errors,
            );

            check_missing_roles(
                db,
                workspace.clone(),
                &instance,
                hub_type,
                type_name.clone(),
                errors,
            );
        } else {
            errors.push(ValidationError {
                range: instance.range(db),
                message: format!("Unknown Hub type '{}'", type_name),
            });
        }
    }
}

fn validate_instance_assignments(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    instance: &crate::db::HubInstance<'_>,
    hub_type: crate::db::HubType<'_>,
    global_fields: &[crate::db::GlobalField<'_>],
    errors: &mut Vec<ValidationError>,
) {
    for assignment in instance.assignments(db) {
        let name = &assignment.name;
        let all_fields = polymorphic::hub_type_all_fields(db, workspace.clone(), &hub_type);
        let all_roles = polymorphic::hub_type_all_roles(db, workspace.clone(), &hub_type);
        let is_field = all_fields.iter().any(|f| f.name.as_str() == name.as_str());
        let role_def = all_roles.iter().find(|r| r.name.as_str() == name.as_str());

        if !is_field && role_def.is_none() {
            errors.push(ValidationError {
                range: assignment.range,
                message: format!(
                    "Unknown field or role '{}' for type '{}'",
                    name,
                    instance.type_name(db)
                ),
            });
            continue;
        }

        if let Some(role_def) = role_def {
            validate_role_assignment(
                db,
                workspace.clone(),
                &assignment.value,
                role_def,
                assignment.range,
                errors,
            );
        } else if is_field {
            if let Some(gf) = global_fields.iter().find(|gf| gf.name(db) == *name) {
                let expected_type = gf.type_name(db);
                if !validate_value_type(db, workspace, &assignment.value, &expected_type) {
                    errors.push(ValidationError {
                        range: assignment.range,
                        message: format!(
                            "Type mismatch for field '{}': expected '{}'",
                            name, expected_type
                        ),
                    });
                }
            }
        }
    }
}

fn validate_role_assignment(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    value: &HubValue,
    role_def: &crate::db::HubRoleDef,
    assignment_range: crate::db::LspRange,
    errors: &mut Vec<ValidationError>,
) {
    if !matches!(value, HubValue::Array(_)) {
        errors.push(ValidationError {
            range: assignment_range,
            message: format!(
                "Role assignment for '{}' must be an array of references wrapped in '[...]'",
                role_def.name
            ),
        });
    }

    let refs = value.extract_refs();

    for ref_name in &refs {
        if let Some(target_inst) =
            resolution::resolve_reference(db, workspace.clone(), ref_name.clone())
        {
            let target_type_name = target_inst.type_name(db);
            let hub_type = match resolution::resolve_type(
                db,
                workspace.clone(),
                target_inst.file(db),
                target_type_name.clone(),
            ) {
                Some(t) => t,
                None => continue,
            };

            if !polymorphic::hub_type_allows(db, workspace, &hub_type, &role_def.allowed_types) {
                errors.push(ValidationError {
                    range: assignment_range,
                    message: format!(
                        "Type mismatch: Role '{}' does not allow type '{}'",
                        role_def.name, target_type_name
                    ),
                });
            }
        } else {
            errors.push(ValidationError {
                range: assignment_range,
                message: format!("Hub reference '{}' not found", ref_name),
            });
        }
    }

    let mult = Multiplicity::parse(&role_def.multiplicity);
    if !mult.validate(refs.len()) {
        errors.push(ValidationError {
            range: assignment_range,
            message: format!(
                "Multiplicity violation for role '{}': expected {}, found {}",
                role_def.name,
                role_def.multiplicity,
                refs.len()
            ),
        });
    }
}

fn check_missing_roles(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    instance: &crate::db::HubInstance<'_>,
    hub_type: crate::db::HubType<'_>,
    type_name: String,
    errors: &mut Vec<ValidationError>,
) {
    let all_roles = polymorphic::hub_type_all_roles(db, workspace, &hub_type);
    for role_def in all_roles {
        let mult = Multiplicity::parse(&role_def.multiplicity);
        let min_required = match mult {
            Multiplicity::Range(min, _) => min > 0,
            Multiplicity::Exact(val) => val > 0,
        };

        if min_required {
            let is_assigned = instance
                .assignments(db)
                .iter()
                .any(|a| a.name == role_def.name);
            if !is_assigned {
                errors.push(ValidationError {
                    range: instance.range(db),
                    message: format!(
                        "Missing required role '{}' for type '{}'",
                        role_def.name, type_name
                    ),
                });
            }
        }
    }
}

pub(crate) fn validate_value_type(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    value: &HubValue,
    type_name: &str,
) -> bool {
    match type_name {
        "String" | "Array<String>" => {
            // Any string value is valid — no type constraint beyond the fact that it's a string literal
            true
        }
        "Text" => matches!(value, HubValue::Text(_)),
        "Number" => matches!(value, HubValue::Number(_)),
        "Boolean" => matches!(value, HubValue::Boolean(_)),
        "Array<Text>" => {
            if let HubValue::Array(vals) = value {
                vals.iter().all(|v| matches!(v, HubValue::Text(_)))
            } else {
                false
            }
        }
        "Array<Number>" => {
            if let HubValue::Array(vals) = value {
                vals.iter().all(|v| matches!(v, HubValue::Number(_)))
            } else {
                false
            }
        }
        // Color: accept any numeric color (f64 no longer supports hex representation check)
        "Color" => match value {
            HubValue::Number(_) => true,
            HubValue::Text(s) => s.starts_with('#') || s.starts_with("rgb") || s.starts_with("hsl"),
            _ => false,
        },
        "Image" => {
            if let HubValue::Text(s) = value {
                let s_lower = s.to_lowercase();
                s_lower.ends_with(".png")
                    || s_lower.ends_with(".jpg")
                    || s_lower.ends_with(".jpeg")
                    || s_lower.ends_with(".gif")
                    || s_lower.ends_with(".svg")
                    || s_lower.ends_with(".webp")
            } else {
                false
            }
        }
        _ => {
            if let Some(hub_enum) = resolution::all_enums(db, workspace)
                .into_iter()
                .find(|e| e.name(db) == type_name)
            {
                if let HubValue::Identifier(s) = value {
                    return hub_enum.variants(db).contains(s);
                }
                return false;
            }
            true
        }
    }
}
