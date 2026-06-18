use serde::{Deserialize, Serialize};

#[salsa::db]
pub trait Db: salsa::Database {
    fn find_file(&self, path: &str) -> Option<SourceFile>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

impl From<tower_lsp::lsp_types::Position> for LspPosition {
    fn from(p: tower_lsp::lsp_types::Position) -> Self {
        Self {
            line: p.line,
            character: p.character,
        }
    }
}

impl From<LspPosition> for tower_lsp::lsp_types::Position {
    fn from(p: LspPosition) -> Self {
        Self {
            line: p.line,
            character: p.character,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

impl From<tower_lsp::lsp_types::Range> for LspRange {
    fn from(r: tower_lsp::lsp_types::Range) -> Self {
        Self {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}

impl From<LspRange> for tower_lsp::lsp_types::Range {
    fn from(r: LspRange) -> Self {
        Self {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}

#[salsa::input]
pub struct SourceFile {
    pub path: String,
    pub contents: String,
}

#[salsa::input]
pub struct Workspace {
    pub files: Vec<SourceFile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubFieldDef {
    pub name: String,
    pub range: LspRange,
    pub decorator: Option<String>,  // "@computed" or "@default"
    pub expression: Option<String>, // The expression inside the decorator
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubRoleDef {
    pub name: String,
    pub direction: String,
    pub multiplicity: String,
    pub allowed_types: Vec<String>,
}

#[salsa::tracked]
pub struct HubType<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: LspRange,
    pub fields: Vec<HubFieldDef>,
    pub roles: Vec<HubRoleDef>,
}

#[salsa::tracked]
pub struct HubEnum<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: LspRange,
    pub variants: Vec<String>,
}

#[salsa::tracked]
pub struct HubStruct<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: LspRange,
    pub field_names: Vec<String>,
}

#[salsa::tracked]
pub struct GlobalField<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: LspRange,
    pub type_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HubValue {
    Identifier(String),
    Number(String),
    String(String),
    Boolean(bool),
    Array(Vec<HubValue>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubAssignment {
    pub name: String,
    pub range: LspRange,
    pub value: HubValue,
}

#[salsa::tracked]
pub struct HubInstance<'db> {
    pub name: String,
    pub type_name: String,
    pub file: SourceFile,
    pub range: LspRange,
    pub description: Option<String>,
    pub assignments: Vec<HubAssignment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubImport {
    pub types: Vec<String>,
    pub from: String, // file path
}

#[salsa::tracked]
pub struct HubgsParseResult<'db> {
    pub instances: Vec<HubInstance<'db>>,
    pub types: Vec<HubType<'db>>,
    pub enums: Vec<HubEnum<'db>>,
    pub structs: Vec<HubStruct<'db>>,
    pub global_fields: Vec<GlobalField<'db>>,
    pub imports: Vec<HubImport>,
}

#[salsa::tracked]
pub struct HubReference<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: LspRange,
}

#[salsa::tracked]
pub fn parse_hubgs(db: &dyn Db, file: SourceFile) -> HubgsParseResult<'_> {
    crate::parser::parse_hubgs_ast(db, file)
}

#[salsa::tracked]
pub fn parse_twxml(db: &dyn Db, file: SourceFile) -> Vec<HubReference<'_>> {
    crate::parser::parse_twxml_ast(db, file)
}

#[salsa::tracked]
pub fn all_twxml_tags(db: &dyn Db, file: SourceFile) -> Vec<(String, LspRange)> {
    crate::parser::get_all_twxml_tags(db, file)
}

#[salsa::tracked]
pub fn get_hub_type_at_position(
    db: &dyn Db,
    file: SourceFile,
    position: LspPosition,
) -> Option<String> {
    crate::parser::get_hub_type_at_position(db, file, position)
}

#[salsa::tracked]
pub fn is_in_hub_definition(db: &dyn Db, file: SourceFile, position: LspPosition) -> bool {
    crate::parser::is_in_hub_definition(db, file, position)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticToken {
    pub line: u32,
    pub character: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers: u32,
}

#[salsa::tracked]
pub fn get_semantic_tokens(db: &dyn Db, file: SourceFile) -> Vec<SemanticToken> {
    crate::parser::compute_semantic_tokens(db, file)
}

#[salsa::tracked]
pub fn get_folding_ranges(db: &dyn Db, file: SourceFile) -> Vec<LspRange> {
    crate::parser::compute_folding_ranges(db, file)
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidationError {
    pub range: LspRange,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Multiplicity {
    Exact(u32),
    Range(u32, Option<u32>), // min, max (None means *)
}

impl Multiplicity {
    pub fn parse(s: &str) -> Self {
        let s = s.trim_matches('(').trim_matches(')');
        if s == "*" {
            return Multiplicity::Range(1, None);
        }
        if let Ok(val) = s.parse::<u32>() {
            return Multiplicity::Exact(val);
        }
        if s.contains("..") {
            let parts: Vec<&str> = s.split("..").collect();
            let min = parts[0].parse::<u32>().unwrap_or(0);
            let max = if parts[1] == "*" {
                None
            } else {
                Some(parts[1].parse::<u32>().unwrap_or(0))
            };
            return Multiplicity::Range(min, max);
        }
        Multiplicity::Range(0, None)
    }

    pub fn validate(&self, count: usize) -> bool {
        let count = count as u32;
        match self {
            Multiplicity::Exact(val) => count == *val,
            Multiplicity::Range(min, max) => {
                if count < *min {
                    return false;
                }
                if let Some(max_val) = max {
                    if count > *max_val {
                        return false;
                    }
                }
                true
            }
        }
    }
}

const VALID_TWXML_TAGS: &[&str] = &[
    "document",
    "meta",
    "section",
    "heading",
    "paragraph",
    "aside",
    "blockquote",
    "codeblock",
    "br",
    "hr",
    "ul",
    "ol",
    "li",
    "dl",
    "dt",
    "dd",
    "details",
    "summary",
    "hubref",
    "link",
    "image",
    "audio",
    "video",
    "code",
    "fr",
    "bold",
    "italic",
    "underline",
    "strikethrough",
    "super",
    "sub",
    "table",
    "tr",
    "th",
    "td",
    "footnote",
];

#[salsa::tracked]
pub fn validate_file(db: &dyn Db, workspace: Workspace, file: SourceFile) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if file.path(db).ends_with(".twxml") {
        // 1. Validate Hub References
        let refs = parse_twxml(db, file);
        for r in refs {
            let name = r.name(db);
            if resolve_reference(db, workspace, name.clone()).is_none() {
                errors.push(ValidationError {
                    range: r.range(db),
                    message: format!("Hub reference '{}' not found", name),
                });
            }
        }

        // 2. Validate Tag Names
        let tags = all_twxml_tags(db, file);
        for (tag_name, range) in tags {
            if !VALID_TWXML_TAGS.contains(&tag_name.as_str()) {
                errors.push(ValidationError {
                    range,
                    message: format!("Unknown TWXML tag '{}'", tag_name),
                });
            }
        }
    } else if file.path(db).ends_with(".hubgs") {
        let result = parse_hubgs(db, file);
        let global_fields = all_global_fields(db, workspace);

        // 1. Validate Hub Type Definitions
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

        // 2. Validate Hub Instances
        for instance in result.instances(db) {
            let type_name = instance.type_name(db);
            if let Some(hub_type) = resolve_type(db, workspace, file, type_name.clone()) {
                for assignment in instance.assignments(db) {
                    let name = &assignment.name;
                    let fields = hub_type.fields(db);
                    let roles = hub_type.roles(db);
                    let is_field = fields.iter().any(|f| &f.name == name);
                    let role_def = roles.iter().find(|r| &r.name == name);

                    if !is_field && role_def.is_none() {
                        errors.push(ValidationError {
                            range: assignment.range,
                            message: format!(
                                "Unknown field or role '{}' for type '{}'",
                                name, type_name
                            ),
                        });
                        continue;
                    }

                    if let Some(role_def) = role_def {
                        let refs = get_refs_from_value(&assignment.value);
                        // 1. Type mismatch validation
                        for ref_name in &refs {
                            if let Some(target_inst) =
                                resolve_reference(db, workspace, ref_name.clone())
                            {
                                let target_type = target_inst.type_name(db);
                                if !role_def.allowed_types.contains(&target_type) {
                                    errors.push(ValidationError {
                                        range: assignment.range,
                                        message: format!(
                                            "Type mismatch: Role '{}' does not allow type '{}'",
                                            name, target_type
                                        ),
                                    });
                                }
                            } else {
                                errors.push(ValidationError {
                                    range: assignment.range,
                                    message: format!("Hub reference '{}' not found", ref_name),
                                });
                            }
                        }

                        // 2. Multiplicity validation
                        let mult = Multiplicity::parse(&role_def.multiplicity);
                        if !mult.validate(refs.len()) {
                            errors.push(ValidationError {
                                range: assignment.range,
                                message: format!(
                                    "Multiplicity violation for role '{}': expected {}, found {}",
                                    name,
                                    role_def.multiplicity,
                                    refs.len()
                                ),
                            });
                        }
                    } else if is_field {
                        // Type checking for primitive fields
                        if let Some(gf) = global_fields.iter().find(|gf| gf.name(db) == *name) {
                            let expected_type = gf.type_name(db);
                            if !validate_value_type(
                                db,
                                workspace,
                                &assignment.value,
                                &expected_type,
                            ) {
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

                // 3. Missing required roles (minimum multiplicity > 0)
                for role_def in hub_type.roles(db) {
                    let mult = Multiplicity::parse(&role_def.multiplicity);
                    if let Multiplicity::Range(min, _) = mult {
                        if min > 0 {
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
                    } else if let Multiplicity::Exact(val) = mult {
                        if val > 0 {
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
            } else {
                errors.push(ValidationError {
                    range: instance.range(db),
                    message: format!("Unknown Hub type '{}'", type_name),
                });
            }
        }
    }

    errors
}

fn get_refs_from_value(value: &HubValue) -> Vec<String> {
    match value {
        HubValue::Identifier(s) => vec![s.clone()],
        HubValue::Array(vals) => vals.iter().flat_map(get_refs_from_value).collect(),
        _ => Vec::new(),
    }
}

fn validate_value_type(
    db: &dyn Db,
    workspace: Workspace,
    value: &HubValue,
    type_name: &str,
) -> bool {
    match type_name {
        "Text" | "String" => matches!(value, HubValue::String(_)),
        "Number" => matches!(value, HubValue::Number(_)),
        "Boolean" => matches!(value, HubValue::Boolean(_)),
        "Array<Text>" | "Array<String>" => {
            if let HubValue::Array(vals) = value {
                vals.iter().all(|v| matches!(v, HubValue::String(_)))
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
        _ => {
            // check Enums
            if let Some(hub_enum) = all_enums(db, workspace)
                .into_iter()
                .find(|e| e.name(db) == type_name)
            {
                if let HubValue::Identifier(s) = value {
                    return hub_enum.variants(db).contains(s);
                }
                return false;
            }
            true // Default to true for now for complex types (Structs, etc)
        }
    }
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
    // Basic "evaluator" for prototype
    // Currently only supports single field references or literal strings
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
                        all_refs.push(HubReference::new(db, name.clone(), file, r_range));
                    }
                }
            }
        }
    }
    all_refs
}

fn find_ref_in_value(value: &HubValue, name: &str, range: LspRange) -> Option<LspRange> {
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
