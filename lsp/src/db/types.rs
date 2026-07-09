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
    pub is_display: bool,
    pub is_background: bool,
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
    /// Parent type names from EXTENDS clause for polymorphic resolution.
    pub extends_parents: Vec<String>,
    pub constraints: Vec<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HubValue {
    Identifier(String),
    Number(f64),
    Boolean(bool),
    Text(String),
    ColorHex(String),
    Array(Vec<HubValue>),
}

impl std::fmt::Display for HubValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HubValue::Text(s) => write!(f, "{}", s),
            HubValue::ColorHex(s) => write!(f, "color({})", s),
            HubValue::Number(n) => write!(f, "{}", n),
            HubValue::Identifier(i) => write!(f, "{}", i),
            HubValue::Boolean(b) => write!(f, "{}", b),
            HubValue::Array(_) => Ok(()),
        }
    }
}

impl std::cmp::PartialEq for HubValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HubValue::Identifier(a), HubValue::Identifier(b)) => a == b,
            (HubValue::Number(a), HubValue::Number(b)) => a.to_bits() == b.to_bits(),
            (HubValue::Text(a), HubValue::Text(b)) => a == b,
            (HubValue::Boolean(a), HubValue::Boolean(b)) => a == b,
            (HubValue::Array(a), HubValue::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x.eq(y))
            }
            (HubValue::ColorHex(a), HubValue::ColorHex(b)) => a == b,
            _ => false, // Different variants are never equal
        }
    }
}

impl std::cmp::Eq for HubValue {}

impl std::hash::Hash for HubValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            HubValue::Identifier(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            HubValue::Number(n) => {
                1u8.hash(state);
                n.to_bits().hash(state);
            }
            HubValue::Boolean(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            HubValue::Text(s) => {
                5u8.hash(state);
                s.hash(state);
            }
            HubValue::ColorHex(s) => {
                6u8.hash(state);
                s.hash(state);
            }
            HubValue::Array(a) => {
                4u8.hash(state);
                a.len().hash(state);
                for item in a {
                    item.hash(state);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HubAssignment {
    pub name: String,
    pub range: super::LspRange,
    pub value: HubValue,
    pub value_range: super::LspRange,
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
