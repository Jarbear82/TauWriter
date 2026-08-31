//! Shared workspace utilities.

use std::path::{Path, PathBuf};

/// Resolves the project root (parent of CARGO_MANIFEST_DIR).
pub(crate) fn resolve_workspace_root() -> Option<PathBuf> {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    Some(base.to_path_buf())
}

/// Recursively search a directory tree for files matching an optional extension filter.
/// If `ext` is Some, only returns files with that extension. If None, returns all non-hidden directories/files.
pub(crate) fn find_files_by_extension(dir: &Path, ext: Option<&str>) -> Vec<PathBuf> {
    let mut results = Vec::new();
    _walk_dir(dir, ext, &mut results);
    results
}

fn _walk_dir(dir: &Path, ext: Option<&str>, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.starts_with('.') || name == "target" || name == "vendor" {
                continue;
            }
            if path.is_dir() {
                _walk_dir(&path, ext, results);
            } else if let Some(ext) = ext {
                if path.extension().map_or(false, |e| e == ext) {
                    results.push(path);
                }
            } else {
                results.push(path);
            }
        }
    }
}

/// Recursively search a directory tree for the first file with a given extension.
pub(crate) fn find_first_file(dir: &Path, ext: Option<&str>) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if name.starts_with('.') || name == "target" || name == "vendor" {
                continue;
            }
            if path.is_dir() {
                if let Some(found) = find_first_file(&path, ext) {
                    return Some(found);
                }
            } else if let Some(ext_filter) = ext {
                if path.extension().map_or(false, |e| e == ext_filter) {
                    return Some(path);
                }
            }
        }
    }
    None
}
