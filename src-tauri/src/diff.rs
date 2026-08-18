use std::collections::{BTreeMap, BTreeSet};
use rayon::prelude::*;

use crate::model::{DiffNode, DiffResult, DiffStatus, FileNode, Snapshot};

pub struct DiffEngine;

impl DiffEngine {
    pub fn diff_snapshots(old_snap: &Snapshot, new_snap: &Snapshot) -> DiffResult {
        let root_diff = Self::diff_nodes(&old_snap.root, &new_snap.root);
        let old_total_size = old_snap.meta.total_size;
        let new_total_size = new_snap.meta.total_size;
        let delta_total_size = (new_total_size as i64) - (old_total_size as i64);
        let delta_total_percent = if old_total_size > 0 {
            ((delta_total_size as f64) / (old_total_size as f64)) * 100.0
        } else if new_total_size > 0 {
            100.0
        } else {
            0.0
        };

        DiffResult {
            snapshot_a_name: old_snap.meta.name.clone(),
            snapshot_a_time: old_snap.meta.formatted_time.clone(),
            snapshot_b_name: new_snap.meta.name.clone(),
            snapshot_b_time: new_snap.meta.formatted_time.clone(),
            root_path: new_snap.meta.root_path.clone(),
            old_total_size,
            new_total_size,
            delta_total_size,
            delta_total_percent,
            root: root_diff,
        }
    }

    pub fn diff_nodes(old_node: &FileNode, new_node: &FileNode) -> DiffNode {
        // Fast Path 1: Subtree Pruning for unchanged directories
        // If metadata (size, file_count, dir_count, modified) matches exactly, prune the entire subtree!
        if old_node.is_dir() && new_node.is_dir()
            && old_node.size == new_node.size
            && old_node.file_count() == new_node.file_count()
            && old_node.dir_count() == new_node.dir_count()
            && old_node.modified == new_node.modified
        {
            return Self::convert_unchanged_node(new_node);
        }

        let delta_size = (new_node.size as i64) - (old_node.size as i64);
        let delta_percent = if old_node.size > 0 {
            ((delta_size as f64) / (old_node.size as f64)) * 100.0
        } else if new_node.size > 0 {
            100.0
        } else {
            0.0
        };

        let status = if delta_size != 0 {
            DiffStatus::Modified
        } else {
            DiffStatus::Unchanged
        };

        let mut old_children_map: BTreeMap<&str, &FileNode> = BTreeMap::new();
        for child in old_node.children() {
            old_children_map.insert(&child.name, child);
        }

        let mut new_children_map: BTreeMap<&str, &FileNode> = BTreeMap::new();
        for child in new_node.children() {
            new_children_map.insert(&child.name, child);
        }

        let mut all_names = BTreeSet::new();
        for name in old_children_map.keys() {
            all_names.insert(*name);
        }
        for name in new_children_map.keys() {
            all_names.insert(*name);
        }

        let all_names_vec: Vec<&str> = all_names.into_iter().collect();

        // Parallel Rayon diffing for wide directories (e.g. root or large folders)
        let mut diff_children: Vec<DiffNode> = if all_names_vec.len() >= 4 {
            all_names_vec
                .into_par_iter()
                .map(|name| {
                    match (old_children_map.get(name), new_children_map.get(name)) {
                        (Some(old_child), Some(new_child)) => Self::diff_nodes(old_child, new_child),
                        (None, Some(new_child)) => Self::convert_added_node(new_child),
                        (Some(old_child), None) => Self::convert_removed_node(old_child),
                        (None, None) => unreachable!(),
                    }
                })
                .collect()
        } else {
            all_names_vec
                .into_iter()
                .map(|name| {
                    match (old_children_map.get(name), new_children_map.get(name)) {
                        (Some(old_child), Some(new_child)) => Self::diff_nodes(old_child, new_child),
                        (None, Some(new_child)) => Self::convert_added_node(new_child),
                        (Some(old_child), None) => Self::convert_removed_node(old_child),
                        (None, None) => unreachable!(),
                    }
                })
                .collect()
        };

        // Sort children in parallel: biggest absolute change or new size first
        diff_children.par_sort_by(|a, b| {
            let a_val = a.new_size.unwrap_or(0).max(a.old_size.unwrap_or(0));
            let b_val = b.new_size.unwrap_or(0).max(b.old_size.unwrap_or(0));
            b_val.cmp(&a_val)
        });

        diff_children.shrink_to_fit();

        DiffNode {
            name: new_node.name.clone(),
            is_dir: new_node.is_dir(),
            old_size: Some(old_node.size),
            new_size: Some(new_node.size),
            delta_size,
            delta_percent,
            status,
            children: diff_children,
        }
    }

    fn convert_unchanged_node(node: &FileNode) -> DiffNode {
        let mut children: Vec<DiffNode> = node
            .children()
            .iter()
            .map(Self::convert_unchanged_node)
            .collect();
        children.shrink_to_fit();

        DiffNode {
            name: node.name.clone(),
            is_dir: node.is_dir(),
            old_size: Some(node.size),
            new_size: Some(node.size),
            delta_size: 0,
            delta_percent: 0.0,
            status: DiffStatus::Unchanged,
            children,
        }
    }

    fn convert_added_node(node: &FileNode) -> DiffNode {
        let mut children: Vec<DiffNode> = node
            .children()
            .iter()
            .map(Self::convert_added_node)
            .collect();
        children.shrink_to_fit();

        DiffNode {
            name: node.name.clone(),
            is_dir: node.is_dir(),
            old_size: None,
            new_size: Some(node.size),
            delta_size: node.size as i64,
            delta_percent: 100.0,
            status: DiffStatus::Added,
            children,
        }
    }

    fn convert_removed_node(node: &FileNode) -> DiffNode {
        let mut children: Vec<DiffNode> = node
            .children()
            .iter()
            .map(Self::convert_removed_node)
            .collect();
        children.shrink_to_fit();

        DiffNode {
            name: node.name.clone(),
            is_dir: node.is_dir(),
            old_size: Some(node.size),
            new_size: None,
            delta_size: -(node.size as i64),
            delta_percent: -100.0,
            status: DiffStatus::Removed,
            children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_nodes() {
        let old_child1 = FileNode::new_file("a.txt".into(), 100, None);
        let old_child2 = FileNode::new_file("b.txt".into(), 200, None);
        let old_root = FileNode::new_dir(
            "root".into(),
            300,
            None,
            2,
            0,
            vec![old_child1, old_child2],
        );

        let new_child1 = FileNode::new_file("a.txt".into(), 150, None); // increased by 50 (+50%)
        let new_child3 = FileNode::new_file("c.txt".into(), 300, None); // newly added
        let new_root = FileNode::new_dir(
            "root".into(),
            450, // 150 + 300
            None,
            2,
            0,
            vec![new_child1, new_child3], // b.txt was removed
        );

        let diff = DiffEngine::diff_nodes(&old_root, &new_root);
        assert_eq!(diff.delta_size, 150); // 450 - 300
        assert_eq!(diff.status, DiffStatus::Modified);

        // Check children
        assert_eq!(diff.children.len(), 3);
    }

    #[test]
    fn test_subtree_pruning() {
        let child = FileNode::new_file("same.txt".into(), 500, Some(12345));
        let old_dir = FileNode::new_dir("dir".into(), 500, Some(12345), 1, 0, vec![child.clone()]);
        let new_dir = FileNode::new_dir("dir".into(), 500, Some(12345), 1, 0, vec![child]);

        let diff = DiffEngine::diff_nodes(&old_dir, &new_dir);
        assert_eq!(diff.status, DiffStatus::Unchanged);
        assert_eq!(diff.delta_size, 0);
        assert_eq!(diff.children.len(), 1);
        assert_eq!(diff.children[0].status, DiffStatus::Unchanged);
    }
}
