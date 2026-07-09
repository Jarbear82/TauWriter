mod hubgs;
mod twxml;

// Re-export types used by downstream consumers via db::*
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationError {
    pub range: crate::db::LspRange,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

/// Validate a single workspace file for structural and semantic correctness.
pub fn validate_file(
    db: &dyn crate::db::Db,
    workspace: crate::db::Workspace,
    file: crate::db::SourceFile,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if file.path(db).ends_with(".twxml") {
        twxml::validate_twxml(db, workspace, file, &mut errors);
    } else if file.path(db).ends_with(".hubgs") {
        hubgs::validate_hubgs(db, workspace, file, &mut errors);
    }

    errors
}
