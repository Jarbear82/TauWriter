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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubAssignment {
    pub name: String,
    pub range: LspRange,
    pub values: Vec<String>, // For roles, these are Hub references
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
pub fn get_hub_type_at_position(
    db: &dyn Db,
    file: SourceFile,
    position: LspPosition,
) -> Option<String> {
    crate::parser::get_hub_type_at_position(db, file, position)
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

#[salsa::tracked]
pub fn validate_file(db: &dyn Db, workspace: Workspace, file: SourceFile) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if file.path(db).ends_with(".twxml") {
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
    } else if file.path(db).ends_with(".hubgs") {
        let result = parse_hubgs(db, file);
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
                        // 1. Type mismatch validation
                        for ref_name in &assignment.values {
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
                        if !mult.validate(assignment.values.len()) {
                            errors.push(ValidationError {
                                range: assignment.range,
                                message: format!(
                                    "Multiplicity violation for role '{}': expected {}, found {}",
                                    name,
                                    role_def.multiplicity,
                                    assignment.values.len()
                                ),
                            });
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
        }
    }
    all_refs
}
