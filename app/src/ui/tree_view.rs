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

/// Convert hierarchical `FileNode`s into `gpui_component::tree::TreeItem`s.
pub(crate) fn file_nodes_to_tree_items(nodes: &[FileNode]) -> Vec<gpui_component::tree::TreeItem> {
    nodes.iter().map(file_node_to_tree_item).collect()
}

/// Convert a single `FileNode` into a `gpui_component::tree::TreeItem`.
pub(crate) fn file_node_to_tree_item(node: &FileNode) -> gpui_component::tree::TreeItem {
    let id = node.path.to_string_lossy().to_string();
    let mut item = gpui_component::tree::TreeItem::new(id, node.name.clone());
    if node.is_dir {
        item = item.children(file_nodes_to_tree_items(&node.children)).expanded(true);
    }
    item
}

