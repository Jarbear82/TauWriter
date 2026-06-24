use serde::{Deserialize, Serialize};

#[salsa::db]
pub trait Db: salsa::Database {
    fn find_file(&self, path: &str) -> Option<SourceFile>;
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
    pub range: super::LspRange,
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
    pub range: super::LspRange,
    pub block_range: super::LspRange,
    pub fields: Vec<HubFieldDef>,
    pub roles: Vec<HubRoleDef>,
}

#[salsa::tracked]
pub struct HubEnum<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: super::LspRange,
    pub variants: Vec<String>,
}

#[salsa::tracked]
pub struct HubStruct<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: super::LspRange,
    pub field_names: Vec<String>,
}

#[salsa::tracked]
pub struct GlobalField<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: super::LspRange,
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

impl std::fmt::Display for HubValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HubValue::String(s) => write!(f, "{}", s),
            HubValue::Number(n) => write!(f, "{}", n),
            HubValue::Boolean(b) => write!(f, "{}", b),
            HubValue::Identifier(i) => write!(f, "{}", i),
            HubValue::Array(_) => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubAssignment {
    pub name: String,
    pub range: super::LspRange,
    pub value: HubValue,
}

#[salsa::tracked]
pub struct HubInstance<'db> {
    pub name: String,
    pub type_name: String,
    pub file: SourceFile,
    pub range: super::LspRange,
    pub block_range: super::LspRange,
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
    pub range: super::LspRange,
    pub field: Option<String>,
    pub text: Option<String>,
    pub tag_range: super::LspRange,
    pub is_reviewed: bool,
}

#[salsa::tracked]
pub struct TwxmlTag<'db> {
    pub name: String,
    pub file: SourceFile,
    pub range: super::LspRange,
    pub parent_name: Option<String>,
}
