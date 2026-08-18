use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

use crate::diff::DiffEngine;
use crate::model::{
    DiffDirectoryView, DiffItemView, DiffNode, DiffProgress, DiffResult, DiffResultMeta,
    DiffResultView, DirectoryView, FileItemView, FileNode, ScanResultView, Snapshot, SnapshotMeta,
};
use crate::scanner::Scanner;
use crate::snapshot::SnapshotManager;

pub struct AppState {
    pub cancel_flag: Arc<AtomicBool>,
    pub current_snapshot: Mutex<Option<Snapshot>>,
    pub current_diff: Mutex<Option<DiffResult>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            current_snapshot: Mutex::new(None),
            current_diff: Mutex::new(None),
        }
    }
}

// Convert a full FileNode to a shallow DirectoryView (only 1 level of children)
fn to_directory_view(node: &FileNode, current_path: &str) -> DirectoryView {
    let clean_path = current_path.trim_end_matches(['/', '\\']);
    
    let children = node
        .children()
        .iter()
        .map(|c| {
            let child_path = if clean_path.is_empty() {
                c.name.clone()
            } else {
                format!("{}/{}", clean_path, c.name)
            };

            FileItemView {
                name: c.name.clone(),
                path: child_path,
                size: c.size,
                is_dir: c.is_dir(),
                file_count: c.file_count(),
                dir_count: c.dir_count(),
                modified: c.modified.map(|m| m as i64),
                has_children: !c.children().is_empty(),
            }
        })
        .collect();

    DirectoryView {
        name: node.name.clone(),
        path: current_path.to_string(),
        size: node.size,
        is_dir: node.is_dir(),
        file_count: node.file_count(),
        dir_count: node.dir_count(),
        children,
    }
}

// Convert a full DiffNode to a shallow DiffDirectoryView (only 1 level of children)
fn to_diff_directory_view(node: &DiffNode, current_path: &str) -> DiffDirectoryView {
    let clean_path = current_path.trim_end_matches(['/', '\\']);

    let children = node
        .children
        .iter()
        .map(|c| {
            let child_path = if clean_path.is_empty() {
                c.name.clone()
            } else {
                format!("{}/{}", clean_path, c.name)
            };

            DiffItemView {
                name: c.name.clone(),
                path: child_path,
                is_dir: c.is_dir,
                old_size: c.old_size,
                new_size: c.new_size,
                delta_size: c.delta_size,
                delta_percent: c.delta_percent,
                status: c.status,
                has_children: !c.children.is_empty(),
            }
        })
        .collect();

    DiffDirectoryView {
        name: node.name.clone(),
        path: current_path.to_string(),
        is_dir: node.is_dir,
        old_size: node.old_size,
        new_size: node.new_size,
        delta_size: node.delta_size,
        delta_percent: node.delta_percent,
        status: node.status,
        children,
    }
}

// Find a node by path in O(depth) time by splitting path segments
fn find_file_node<'a>(
    root: &'a FileNode,
    root_path: &str,
    target_path: &str,
) -> Option<&'a FileNode> {
    let clean_root = root_path.trim_end_matches(['/', '\\']);
    let clean_target = target_path.trim_end_matches(['/', '\\']);

    if clean_root == clean_target || target_path.is_empty() {
        return Some(root);
    }

    let rel_path = if clean_target.starts_with(clean_root) {
        let remainder = &clean_target[clean_root.len()..];
        remainder.trim_start_matches(['/', '\\'])
    } else {
        clean_target.trim_start_matches(['/', '\\'])
    };

    let segments: Vec<&str> = rel_path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();

    let mut curr = root;
    for seg in segments {
        if let Some(child) = curr.children().iter().find(|c| c.name == seg) {
            curr = child;
        } else {
            return None;
        }
    }

    Some(curr)
}

// Find a diff node by path in O(depth) time by splitting path segments
fn find_diff_node<'a>(
    root: &'a DiffNode,
    root_path: &str,
    target_path: &str,
) -> Option<&'a DiffNode> {
    let clean_root = root_path.trim_end_matches(['/', '\\']);
    let clean_target = target_path.trim_end_matches(['/', '\\']);

    if clean_root == clean_target || target_path.is_empty() {
        return Some(root);
    }

    let rel_path = if clean_target.starts_with(clean_root) {
        let remainder = &clean_target[clean_root.len()..];
        remainder.trim_start_matches(['/', '\\'])
    } else {
        clean_target.trim_start_matches(['/', '\\'])
    };

    let segments: Vec<&str> = rel_path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();

    let mut curr = root;
    for seg in segments {
        if let Some(child) = curr.children.iter().find(|c| c.name == seg) {
            curr = child;
        } else {
            return None;
        }
    }

    Some(curr)
}

#[tauri::command]
pub async fn start_scan(
    path: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<ScanResultView, String> {
    let target_path = PathBuf::from(&path);
    if !target_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    state.cancel_flag.store(false, Ordering::Relaxed);
    let cancel_flag = state.cancel_flag.clone();
    let app_clone = app.clone();
    let scan_path = path.clone();

    let (snapshot, root_view, meta) = tauri::async_runtime::spawn_blocking(move || {
        let scanner = Scanner::new(cancel_flag);
        let root_node = scanner.scan(&target_path, Some(&app_clone))?;
        let snapshot = SnapshotManager::create_snapshot(root_node, scan_path.clone(), None);
        let root_view = to_directory_view(&snapshot.root, &scan_path);
        let meta = snapshot.meta.clone();
        Ok::<_, String>((snapshot, root_view, meta))
    })
    .await
    .map_err(|e| e.to_string())??;

    // Store complete deep tree in Rust backend memory
    let mut lock = state.current_snapshot.lock().unwrap();
    *lock = Some(snapshot);

    // Return ONLY shallow root view to frontend (< 10KB of JSON!)
    Ok(ScanResultView { meta, root_view })
}

#[tauri::command]
pub fn cancel_scan(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn get_current_snapshot_meta(state: State<'_, AppState>) -> Result<Option<SnapshotMeta>, String> {
    let lock = state.current_snapshot.lock().unwrap();
    Ok(lock.as_ref().map(|s| s.meta.clone()))
}

#[tauri::command]
pub fn get_directory_node(
    path: String,
    state: State<'_, AppState>,
) -> Result<DirectoryView, String> {
    let lock = state.current_snapshot.lock().unwrap();
    let snapshot = lock.as_ref().ok_or_else(|| "No active scan loaded".to_string())?;

    let node = find_file_node(&snapshot.root, &snapshot.meta.root_path, &path)
        .ok_or_else(|| format!("Directory not found in snapshot: {}", path))?;

    Ok(to_directory_view(node, &path))
}

#[tauri::command]
pub async fn save_current_snapshot(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let snapshot_clone = {
        let mut lock = state.current_snapshot.lock().unwrap();
        let snapshot = lock.as_mut().ok_or_else(|| "No scan available to save".to_string())?;
        snapshot.meta.name = name;
        snapshot.clone()
    };

    let target_path_str = tauri::async_runtime::spawn_blocking(move || {
        let dir = SnapshotManager::get_default_storage_dir();
        let target_path = dir.join(format!("{}.snap", snapshot_clone.meta.id));
        SnapshotManager::save_to_file(&snapshot_clone, &target_path)?;
        Ok::<_, String>(target_path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(target_path_str)
}

#[tauri::command]
pub async fn load_snapshot(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ScanResultView, String> {
    let (snapshot, root_view, meta) = tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&file_path);
        let snapshot = SnapshotManager::load_from_file(path)?;
        let root_view = to_directory_view(&snapshot.root, &snapshot.meta.root_path);
        let meta = snapshot.meta.clone();
        Ok::<_, String>((snapshot, root_view, meta))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut lock = state.current_snapshot.lock().unwrap();
    *lock = Some(snapshot);

    Ok(ScanResultView { meta, root_view })
}

#[tauri::command]
pub async fn load_saved_snapshot(
    snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<ScanResultView, String> {
    let (snapshot, root_view, meta) = tauri::async_runtime::spawn_blocking(move || {
        let dir = SnapshotManager::get_default_storage_dir();
        let snap_path = dir.join(format!("{}.snap", snapshot_id));
        let snapshot = SnapshotManager::load_from_file(&snap_path)?;
        let root_view = to_directory_view(&snapshot.root, &snapshot.meta.root_path);
        let meta = snapshot.meta.clone();
        Ok::<_, String>((snapshot, root_view, meta))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut lock = state.current_snapshot.lock().unwrap();
    *lock = Some(snapshot);

    Ok(ScanResultView { meta, root_view })
}

#[tauri::command]
pub fn delete_saved_snapshot(snapshot_id: String) -> Result<Vec<SnapshotMeta>, String> {
    let dir = SnapshotManager::get_default_storage_dir();
    let snap_path = dir.join(format!("{}.snap", snapshot_id));
    if snap_path.exists() {
        let _ = std::fs::remove_file(snap_path);
    }
    Ok(SnapshotManager::list_saved_snapshots())
}

#[tauri::command]
pub fn list_saved_snapshots() -> Result<Vec<SnapshotMeta>, String> {
    Ok(SnapshotManager::list_saved_snapshots())
}

#[tauri::command]
pub async fn diff_snapshots(
    app_handle: AppHandle,
    old_snapshot_id: String,
    new_snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<DiffResultView, String> {
    let app = app_handle.clone();

    let (diff_result, root_view, meta) = tauri::async_runtime::spawn_blocking(move || {
        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "正在多线程并行解压并载入两份快照...".to_string(),
                progress_percent: 30,
                is_done: false,
            },
        );

        let dir = SnapshotManager::get_default_storage_dir();
        let old_path = dir.join(format!("{}.snap", old_snapshot_id));
        let new_path = dir.join(format!("{}.snap", new_snapshot_id));

        let (old_res, new_res) = rayon::join(
            || SnapshotManager::load_from_file(&old_path),
            || SnapshotManager::load_from_file(&new_path),
        );
        let old_snap = old_res?;
        let new_snap = new_res?;

        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "正在多线程并行比对并快速剪枝未修改子树...".to_string(),
                progress_percent: 75,
                is_done: false,
            },
        );

        let diff_result = DiffEngine::diff_snapshots(&old_snap, &new_snap);

        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "正在生成视图...".to_string(),
                progress_percent: 95,
                is_done: false,
            },
        );

        let meta = DiffResultMeta {
            snapshot_a_name: diff_result.snapshot_a_name.clone(),
            snapshot_a_time: diff_result.snapshot_a_time.clone(),
            snapshot_b_name: diff_result.snapshot_b_name.clone(),
            snapshot_b_time: diff_result.snapshot_b_time.clone(),
            root_path: diff_result.root_path.clone(),
            old_total_size: diff_result.old_total_size,
            new_total_size: diff_result.new_total_size,
            delta_total_size: diff_result.delta_total_size,
            delta_total_percent: diff_result.delta_total_percent,
        };

        let root_view = to_diff_directory_view(&diff_result.root, &diff_result.root_path);

        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "对比完成".to_string(),
                progress_percent: 100,
                is_done: true,
            },
        );

        Ok::<_, String>((diff_result, root_view, meta))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut lock = state.current_diff.lock().unwrap();
    *lock = Some(diff_result);

    Ok(DiffResultView { meta, root_view })
}

#[tauri::command]
pub async fn diff_current_with_saved(
    app_handle: AppHandle,
    old_snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<DiffResultView, String> {
    let app = app_handle.clone();
    let current_snap = {
        let snap_lock = state.current_snapshot.lock().unwrap();
        snap_lock.as_ref().cloned().ok_or_else(|| "No current scan available".to_string())?
    };

    let (diff_result, root_view, meta) = tauri::async_runtime::spawn_blocking(move || {
        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "正在读取当前活动扫描内存与基准快照...".to_string(),
                progress_percent: 30,
                is_done: false,
            },
        );

        let dir = SnapshotManager::get_default_storage_dir();
        let old_snap = SnapshotManager::load_from_file(&dir.join(format!("{}.snap", old_snapshot_id)))?;

        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "正在多线程递归比对数百万节点差异...".to_string(),
                progress_percent: 75,
                is_done: false,
            },
        );

        let diff_result = DiffEngine::diff_snapshots(&old_snap, &current_snap);

        let meta = DiffResultMeta {
            snapshot_a_name: diff_result.snapshot_a_name.clone(),
            snapshot_a_time: diff_result.snapshot_a_time.clone(),
            snapshot_b_name: diff_result.snapshot_b_name.clone(),
            snapshot_b_time: diff_result.snapshot_b_time.clone(),
            root_path: diff_result.root_path.clone(),
            old_total_size: diff_result.old_total_size,
            new_total_size: diff_result.new_total_size,
            delta_total_size: diff_result.delta_total_size,
            delta_total_percent: diff_result.delta_total_percent,
        };

        let root_view = to_diff_directory_view(&diff_result.root, &diff_result.root_path);

        let _ = app.emit(
            "diff-progress",
            &DiffProgress {
                stage: "对比完成".to_string(),
                progress_percent: 100,
                is_done: true,
            },
        );

        Ok::<_, String>((diff_result, root_view, meta))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut lock = state.current_diff.lock().unwrap();
    *lock = Some(diff_result);

    Ok(DiffResultView { meta, root_view })
}

#[tauri::command]
pub fn get_diff_directory_node(
    path: String,
    state: State<'_, AppState>,
) -> Result<DiffDirectoryView, String> {
    let lock = state.current_diff.lock().unwrap();
    let diff = lock.as_ref().ok_or_else(|| "No active diff loaded".to_string())?;

    let node = find_diff_node(&diff.root, &diff.root_path, &path)
        .ok_or_else(|| format!("Directory not found in diff: {}", path))?;

    Ok(to_diff_directory_view(node, &path))
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,\"{}\"", path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = path_buf.parent().unwrap_or(&path_buf);
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
