use serde::{Deserialize, Serialize};

pub mod resolution;
pub mod types;
pub mod validation;

// Re-export everything at the module root for backward compatibility
pub use resolution::*;
pub use types::*;
pub use validation::*;

/// Salsa-compatible position. lsp_types::Position doesn't impl Hash, so salsa tracked structs can't store it directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

impl From<lsp_types::Position> for LspPosition {
    fn from(p: lsp_types::Position) -> Self {
        Self {
            line: p.line,
            character: p.character,
        }
    }
}

impl From<LspPosition> for lsp_types::Position {
    fn from(p: LspPosition) -> Self {
        Self {
            line: p.line,
            character: p.character,
        }
    }
}

/// Salsa-compatible range. Same Hash limitation as Position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

impl From<lsp_types::Range> for LspRange {
    fn from(r: lsp_types::Range) -> Self {
        Self {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}

impl From<LspRange> for lsp_types::Range {
    fn from(r: LspRange) -> Self {
        Self {
            start: r.start.into(),
            end: r.end.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticToken {
    pub line: u32,
    pub character: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers: u32,
}
