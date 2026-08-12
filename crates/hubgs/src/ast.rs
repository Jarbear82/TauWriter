#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SpanPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SpanRange {
    pub start: SpanPosition,
    pub end: SpanPosition,
}

pub fn ts_range_to_span(range: tree_sitter::Range) -> SpanRange {
    SpanRange {
        start: SpanPosition {
            line: range.start_point.row as u32,
            character: range.start_point.column as u32,
        },
        end: SpanPosition {
            line: range.end_point.row as u32,
            character: range.end_point.column as u32,
        },
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum HubValue {
    Identifier(String),
    Number(f64),
    Text(String),
    Boolean(bool),
    Array(Vec<HubValue>),
}

impl HubValue {
    pub fn extract_refs(&self) -> Vec<String> {
        match self {
            HubValue::Identifier(s) => vec![s.clone()],
            HubValue::Array(vals) => vals.iter().flat_map(|v| v.extract_refs()).collect(),
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubFieldDef {
    pub name: String,
    pub decorator: Option<String>,
    pub expression: Option<String>,
    pub is_display: bool,
    pub is_background: bool,
    pub range: SpanRange,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubRoleDef {
    pub name: String,
    pub direction: String,
    pub multiplicity: String,
    pub allowed_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubTypeAst {
    pub name: String,
    pub range: SpanRange,
    pub block_range: SpanRange,
    pub fields: Vec<HubFieldDef>,
    pub roles: Vec<HubRoleDef>,
    pub extends_parents: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HubAssignmentAst {
    pub name: String,
    pub range: SpanRange,
    pub value: HubValue,
    pub value_range: SpanRange,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HubInstanceAst {
    pub id: String,
    pub type_name: String,
    pub name_range: SpanRange,
    pub block_range: SpanRange,
    pub description: Option<String>,
    pub assignments: Vec<HubAssignmentAst>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GlobalFieldAst {
    pub name: String,
    pub type_name: String,
    pub range: SpanRange,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubEnumAst {
    pub name: String,
    pub variants: Vec<String>,
    pub range: SpanRange,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubStructAst {
    pub name: String,
    pub field_names: Vec<String>,
    pub range: SpanRange,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsLink {
    pub name: String,
    pub arrow: String,
    pub target: String,
    pub multiplicity: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsDefinition {
    pub name: String,
    pub links: Vec<HubgsLink>,
    pub parents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InstanceLink {
    pub relation: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubgsInstance {
    pub id: String,
    pub type_name: String,
    pub name: String,
    pub theme_color: Option<u32>,
    pub links: Vec<InstanceLink>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubImport {
    pub types: Vec<String>,
    pub from: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubEnum {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HubStruct {
    pub name: String,
    pub field_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GlobalField {
    pub name: String,
    pub type_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HubgsParseOutput {
    pub definitions: Vec<HubgsDefinition>,
    pub instances: Vec<HubgsInstance>,
    pub imports: Vec<HubImport>,
    pub enums: Vec<HubEnum>,
    pub structs: Vec<HubStruct>,
    pub global_fields: Vec<GlobalField>,

    pub types_ast: Vec<HubTypeAst>,
    pub instances_ast: Vec<HubInstanceAst>,
    pub enums_ast: Vec<HubEnumAst>,
    pub structs_ast: Vec<HubStructAst>,
    pub global_fields_ast: Vec<GlobalFieldAst>,
}
