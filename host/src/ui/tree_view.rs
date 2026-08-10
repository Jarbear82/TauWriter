//! File tree utilities — walks the workspace directory to produce a [`FileNode`] tree.
//! Extracted from `main.rs` so it can be tested and reused independently.

use std::path::{Path, PathBuf};

/// A node in the workspace file tree (file or directory).
#[derive(Debug, Clone)]
pub(crate) struct FileNode {
    pub(crate) path: PathBuf,
    pub(crate) is_dir: bool,
    pub(crate) name: String,
    pub(crate) children: Vec<FileNode>,
}

/// A flattened node suitable for virtualized list rendering.
#[derive(Debug, Clone)]
pub(crate) struct FlatFileNode {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) is_dir: bool,
    pub(crate) depth: usize,
}

/// Build a tree of file nodes rooted at `dir`.  Hidden files (starting with '.')
/// and common build directories (`target`, `vendor`) are skipped.
pub(crate) fn build_file_tree(dir: &Path) -> Vec<FileNode> {
    let mut nodes = Vec::new();
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
            let is_dir = path.is_dir();
            let children = if is_dir {
                build_file_tree(&path)
            } else {
                Vec::new()
            };
            nodes.push(FileNode {
                path,
                is_dir,
                name,
                children,
            });
        }
    }
    nodes.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.cmp(&b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    nodes
}

/// Flatten a hierarchical FileNode tree into a flat list for virtualized rendering.
pub(crate) fn flatten_file_tree(nodes: &[FileNode]) -> Vec<FlatFileNode> {
    let empty_set = std::collections::HashSet::new();
    flatten_file_tree_with_collapse(nodes, &empty_set)
}

/// Flatten a hierarchical FileNode tree respecting a set of collapsed directory paths.
pub(crate) fn flatten_file_tree_with_collapse(
    nodes: &[FileNode],
    collapsed_paths: &std::collections::HashSet<PathBuf>,
) -> Vec<FlatFileNode> {
    let mut flat = Vec::new();
    _flatten_recursive_with_collapse(nodes, 0, collapsed_paths, &mut flat);
    flat
}

fn _flatten_recursive_with_collapse(
    nodes: &[FileNode],
    depth: usize,
    collapsed_paths: &std::collections::HashSet<PathBuf>,
    flat: &mut Vec<FlatFileNode>,
) {
    for node in nodes {
        flat.push(FlatFileNode {
            path: node.path.clone(),
            name: node.name.clone(),
            is_dir: node.is_dir,
            depth,
        });
        if node.is_dir && !node.children.is_empty() && !collapsed_paths.contains(&node.path) {
            _flatten_recursive_with_collapse(&node.children, depth + 1, collapsed_paths, flat);
        }
    }
}

