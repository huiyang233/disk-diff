use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    File,
    Dir {
        file_count: u32,
        dir_count: u32,
        children: Vec<FileNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub size: u64,
    pub modified: Option<u32>,
    pub kind: NodeKind,
}

impl FileNode {
    #[inline]
    pub fn new_file(name: String, size: u64, modified: Option<u32>) -> Self {
        Self {
            name,
            size,
            modified,
            kind: NodeKind::File,
        }
    }

    #[inline]
    pub fn new_dir(
        name: String,
        size: u64,
        modified: Option<u32>,
        file_count: u32,
        dir_count: u32,
        mut children: Vec<FileNode>,
    ) -> Self {
        children.shrink_to_fit();
        Self {
            name,
            size,
            modified,
            kind: NodeKind::Dir {
                file_count,
                dir_count,
                children,
            },
        }
    }

    #[inline]
    pub fn is_dir(&self) -> bool {
        matches!(self.kind, NodeKind::Dir { .. })
    }

    #[inline]
    pub fn file_count(&self) -> u64 {
        match &self.kind {
            NodeKind::Dir { file_count, .. } => *file_count as u64,
            NodeKind::File => 1,
        }
    }

    #[inline]
    pub fn dir_count(&self) -> u64 {
        match &self.kind {
            NodeKind::Dir { dir_count, .. } => *dir_count as u64,
            NodeKind::File => 0,
        }
    }

    #[inline]
    pub fn children(&self) -> &[FileNode] {
        match &self.kind {
            NodeKind::Dir { children, .. } => children,
            NodeKind::File => &[],
        }
    }

    #[inline]
    pub fn children_mut(&mut self) -> Option<&mut Vec<FileNode>> {
        match &mut self.kind {
            NodeKind::Dir { children, .. } => Some(children),
            NodeKind::File => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub timestamp: i64,
    pub formatted_time: String,
    pub total_size: u64,
    pub total_files: u64,
    pub total_dirs: u64,
    #[serde(default)]
    pub snap_file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub meta: SnapshotMeta,
    pub root: FileNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStatus {
    Added,
    Removed,
    Modified,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffNode {
    pub name: String,
    pub is_dir: bool,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
    pub delta_size: i64,
    pub delta_percent: f64,
    pub status: DiffStatus,
    pub children: Vec<DiffNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub snapshot_a_name: String,
    pub snapshot_a_time: String,
    pub snapshot_b_name: String,
    pub snapshot_b_time: String,
    pub root_path: String,
    pub old_total_size: u64,
    pub new_total_size: u64,
    pub delta_total_size: i64,
    pub delta_total_percent: f64,
    pub root: DiffNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub scanned_files: u64,
    pub scanned_dirs: u64,
    pub total_size: u64,
    pub current_path: String,
    pub is_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffProgress {
    pub stage: String,
    pub progress_percent: u32,
    pub is_done: bool,
}

// === Lightweight Shallow Views for IPC/CLI/Frontend ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItemView {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub file_count: u64,
    pub dir_count: u64,
    pub modified: Option<i64>,
    pub has_children: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryView {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub file_count: u64,
    pub dir_count: u64,
    pub children: Vec<FileItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResultView {
    pub meta: SnapshotMeta,
    pub root_view: DirectoryView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffItemView {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
    pub delta_size: i64,
    pub delta_percent: f64,
    pub status: DiffStatus,
    pub has_children: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffDirectoryView {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
    pub delta_size: i64,
    pub delta_percent: f64,
    pub status: DiffStatus,
    pub children: Vec<DiffItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResultMeta {
    pub snapshot_a_name: String,
    pub snapshot_a_time: String,
    pub snapshot_b_name: String,
    pub snapshot_b_time: String,
    pub root_path: String,
    pub old_total_size: u64,
    pub new_total_size: u64,
    pub delta_total_size: i64,
    pub delta_total_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResultView {
    pub meta: DiffResultMeta,
    pub root_view: DiffDirectoryView,
}
