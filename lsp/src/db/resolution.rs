use super::types::*;

#[salsa::tracked]
pub fn parse_hubgs(db: &dyn Db, file: SourceFile) -> HubgsParseResult<'_> {
    crate::parser::parse_hubgs_ast(db, file)
}

#[salsa::tracked]
pub fn parse_twxml(db: &dyn Db, file: SourceFile) -> Vec<HubReference<'_>> {
    crate::parser::parse_twxml_ast(db, file)
}

#[salsa::tracked]
pub fn all_twxml_tags(db: &dyn Db, file: SourceFile) -> Vec<TwxmlTag<'_>> {
    crate::parser::get_all_twxml_tags(db, file)
}

#[salsa::tracked]
pub fn get_hub_type_at_position(
    db: &dyn Db,
    file: SourceFile,
    position: super::LspPosition,
) -> Option<String> {
    crate::parser::get_hub_type_at_position(db, file, position.into())
}

#[salsa::tracked]
pub fn is_in_hub_definition(db: &dyn Db, file: SourceFile, position: super::LspPosition) -> bool {
    crate::parser::is_in_hub_definition(db, file, position.into())
}

#[salsa::tracked]
pub fn get_semantic_tokens(db: &dyn Db, file: SourceFile) -> Vec<super::SemanticToken> {
    crate::parser::compute_semantic_tokens(db, file)
}

#[salsa::tracked]
pub fn get_folding_ranges(db: &dyn Db, file: SourceFile) -> Vec<super::LspRange> {
    crate::parser::compute_folding_ranges(db, file)
        .into_iter()
        .map(|r| r.into())
        .collect()
}

#[salsa::tracked]
pub fn all_hub_instances(db: &dyn Db, workspace: Workspace) -> Vec<HubInstance<'_>> {
    let mut all = Vec::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".hubgs") {
            all.extend(parse_hubgs(db, file).instances(db).clone());
        }
    }
    all
}

#[salsa::tracked]
pub fn all_hub_types(db: &dyn Db, workspace: Workspace) -> Vec<HubType<'_>> {
    let mut all = Vec::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".hubgs") {
            all.extend(parse_hubgs(db, file).types(db).clone());
        }
    }
    all
}

#[salsa::tracked]
pub fn all_global_fields(db: &dyn Db, workspace: Workspace) -> Vec<GlobalField<'_>> {
    let mut all = Vec::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".hubgs") {
            all.extend(parse_hubgs(db, file).global_fields(db).clone());
        }
    }
    all
}

#[salsa::tracked]
pub fn all_enums(db: &dyn Db, workspace: Workspace) -> Vec<HubEnum<'_>> {
    let mut all = Vec::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".hubgs") {
            all.extend(parse_hubgs(db, file).enums(db).clone());
        }
    }
    all
}

#[salsa::tracked]
pub fn all_structs(db: &dyn Db, workspace: Workspace) -> Vec<HubStruct<'_>> {
    let mut all = Vec::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".hubgs") {
            all.extend(parse_hubgs(db, file).structs(db).clone());
        }
    }
    all
}

#[salsa::tracked]
pub fn resolve_reference(
    db: &dyn Db,
    workspace: Workspace,
    name: String,
) -> Option<HubInstance<'_>> {
    all_hub_instances(db, workspace)
        .into_iter()
        .find(|i| i.name(db) == name)
}

#[salsa::tracked]
pub fn visible_types(db: &dyn Db, workspace: Workspace, file: SourceFile) -> Vec<HubType<'_>> {
    let mut visible = Vec::new();
    let result = parse_hubgs(db, file);

    // 1. Local types
    visible.extend(result.types(db).clone());

    // 2. Imported types
    for imp in result.imports(db) {
        let from_file = workspace
            .files(db)
            .into_iter()
            .find(|f| f.path(db).ends_with(&imp.from));

        if let Some(ff) = from_file {
            let ff_result = parse_hubgs(db, ff);
            for t in ff_result.types(db) {
                if imp.types.contains(&t.name(db)) {
                    visible.push(t.clone());
                }
            }
        }
    }

    visible
}

#[salsa::tracked]
pub fn resolve_type(
    db: &dyn Db,
    workspace: Workspace,
    file: SourceFile,
    name: String,
) -> Option<HubType<'_>> {
    visible_types(db, workspace, file)
        .into_iter()
        .find(|t| t.name(db) == name)
}

#[salsa::tracked]
pub fn compute_field_value(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    field_name: String,
) -> Option<HubValue> {
    // 1. Check if assigned in instance
    if let Some(assignment) = instance
        .assignments(db)
        .iter()
        .find(|a| a.name == field_name)
    {
        return Some(assignment.value.clone());
    }

    // 2. Check if computed or default in type
    let hub_type = resolve_type(db, workspace, instance.file(db), instance.type_name(db))?;
    if let Some(field_def) = hub_type.fields(db).iter().find(|f| f.name == field_name) {
        if let Some(expr) = &field_def.expression {
            return evaluate_expression(db, workspace, instance, expr);
        }
    }
    None
}

fn evaluate_expression(
    db: &dyn Db,
    workspace: Workspace,
    instance: HubInstance<'_>,
    expr: &str,
) -> Option<HubValue> {
    let expr = expr.trim();
    if expr.starts_with('\'') || expr.starts_with('"') || expr.starts_with('`') {
        return Some(HubValue::String(
            expr.trim_matches(|c| c == '\'' || c == '"' || c == '`')
                .to_string(),
        ));
    }

    if expr.parse::<f64>().is_ok() {
        return Some(HubValue::Number(expr.to_string()));
    }

    if expr == "true" {
        return Some(HubValue::Boolean(true));
    }
    if expr == "false" {
        return Some(HubValue::Boolean(false));
    }

    // Try to resolve as another field in the same instance
    if let Some(other_val) = compute_field_value(db, workspace, instance, expr.to_string()) {
        return Some(other_val);
    }

    None
}

#[salsa::tracked]
pub fn find_all_references(
    db: &dyn Db,
    workspace: Workspace,
    name: String,
) -> Vec<HubReference<'_>> {
    let mut all_refs = Vec::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".twxml") {
            let refs = parse_twxml(db, file);
            for r in refs {
                if r.name(db) == name {
                    all_refs.push(r);
                }
            }
        } else if file.path(db).ends_with(".hubgs") {
            let result = parse_hubgs(db, file);
            for inst in result.instances(db) {
                for assignment in inst.assignments(db) {
                    if let Some(r_range) =
                        find_ref_in_value(&assignment.value, &name, assignment.range)
                    {
                        all_refs.push(HubReference::new(
                            db,
                            name.clone(),
                            file,
                            r_range,
                            None,
                            None,
                            r_range,
                            false,
                        ));
                    }
                }
            }
        }
    }
    all_refs
}

fn find_ref_in_value(
    value: &HubValue,
    name: &str,
    range: super::LspRange,
) -> Option<super::LspRange> {
    match value {
        HubValue::Identifier(s) if s == name => Some(range),
        HubValue::Array(vals) => {
            for v in vals {
                if let Some(r) = find_ref_in_value(v, name, range) {
                    return Some(r);
                }
            }
            None
        }
        _ => None,
    }
}
