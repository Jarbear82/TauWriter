use serde::{Deserialize, Serialize};

/// f64 bit-pattern wrapper enabling `Eq` + `Hash` for use in enums.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawF64(u64);

impl RawF64 {
    pub fn from_f64(v: f64) -> Self {
        Self(v.to_bits())
    }
    pub fn into_f64(self) -> f64 {
        f64::from_bits(self.0)
    }
}

impl serde::Serialize for RawF64 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RawF64 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(u64::deserialize(deserializer)?))
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HubValue {
    Identifier(String),
    Number(RawF64),
    Boolean(bool),
    Text(String),
    ColorHex(String),
    Array(Vec<HubValue>),
}

/// Error returned when coercing a [`HubValue`] to a target type fails.
#[derive(Clone, Debug, PartialEq)]
pub enum HubValueConversionError {
    NumberExpected,
    StringExpected,
}

impl std::fmt::Display for HubValueConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HubValueConversionError::NumberExpected => write!(f, "HubValue is not a number"),
            HubValueConversionError::StringExpected => {
                write!(f, "HubValue cannot be converted to string")
            }
        }
    }
}

impl std::error::Error for HubValueConversionError {}

impl TryFrom<&HubValue> for f64 {
    type Error = HubValueConversionError;

    fn try_from(v: &HubValue) -> Result<Self, Self::Error> {
        match v {
            HubValue::Number(n) => Ok(n.into_f64()),
            _ => Err(HubValueConversionError::NumberExpected),
        }
    }
}

impl TryFrom<&HubValue> for String {
    type Error = HubValueConversionError;

    fn try_from(v: &HubValue) -> Result<Self, Self::Error> {
        match v {
            HubValue::Text(s) => Ok(s.clone()),
            HubValue::Number(n) => Ok(n.into_f64().to_string()),
            HubValue::Boolean(b) => Ok(b.to_string()),
            HubValue::Identifier(i) => Ok(i.clone()),
            _ => Err(HubValueConversionError::StringExpected),
        }
    }
}

impl std::fmt::Display for HubValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HubValue::Text(s) => write!(f, "{}", s),
            HubValue::ColorHex(s) => write!(f, "color({})", s),
            HubValue::Number(n) => write!(f, "{}", n.into_f64()),
            HubValue::Identifier(i) => write!(f, "{}", i),
            HubValue::Boolean(b) => write!(f, "{}", b),
            HubValue::Array(_) => Ok(()),
        }
    }
}

impl HubValue {
    /// Extract instance reference names from this value (used for role assignments).
    pub fn extract_refs(&self) -> Vec<String> {
        match self {
            HubValue::Identifier(s) => vec![s.clone()],
            HubValue::Array(vals) => vals.iter().flat_map(|v| v.extract_refs()).collect(),
            _ => Vec::new(),
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
