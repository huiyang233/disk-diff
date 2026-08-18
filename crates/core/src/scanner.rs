use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};
use rayon::prelude::*;

use crate::model::{FileNode, ScanProgress};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[derive(Default)]
pub struct InodeTracker {
    #[cfg(unix)]
    visited: Mutex<HashSet<(u64, u64)>>, // (dev, ino)
}

impl InodeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(unix)]
    pub fn should_count(&self, meta: &fs::Metadata) -> bool {
        if meta.is_file() && meta.nlink() > 1 {
            let key = (meta.dev(), meta.ino());
            let mut set = self.visited.lock().unwrap();
            set.insert(key)
        } else {
            true
        }
    }

    #[cfg(not(unix))]
    pub fn should_count(&self, _meta: &fs::Metadata) -> bool {
        true
    }
}

pub struct Scanner {
    cancel_flag: Arc<AtomicBool>,
    scanned_files: Arc<AtomicU64>,
    scanned_dirs: Arc<AtomicU64>,
    total_size: Arc<AtomicU64>,
    inode_tracker: Arc<InodeTracker>,
}

impl Scanner {
    pub fn new(cancel_flag: Arc<AtomicBool>) -> Self {
        Self {
            cancel_flag,
            scanned_files: Arc::new(AtomicU64::new(0)),
            scanned_dirs: Arc::new(AtomicU64::new(0)),
            total_size: Arc::new(AtomicU64::new(0)),
            inode_tracker: Arc::new(InodeTracker::new()),
        }
    }

    pub fn scan<F>(
        &self,
        root_path: &Path,
        progress_callback: Option<&F>,
    ) -> Result<FileNode, String>
    where
        F: Fn(ScanProgress) + Send + Sync,
    {
        let last_emit = Arc::new(Mutex::new(Instant::now()));

        let root_node = self.scan_dir_parallel(
            root_path,
            0,
            progress_callback,
            &last_emit,
        )?;

        // Send final 100% progress event
        if let Some(cb) = progress_callback {
            cb(ScanProgress {
                scanned_files: self.scanned_files.load(Ordering::Relaxed),
                scanned_dirs: self.scanned_dirs.load(Ordering::Relaxed),
                total_size: self.total_size.load(Ordering::Relaxed),
                current_path: root_path.to_string_lossy().to_string(),
                is_done: true,
            });
        }

        Ok(root_node)
    }

    fn scan_dir_parallel<F>(
        &self,
        path: &Path,
        depth: usize,
        progress_callback: Option<&F>,
        last_emit: &Arc<Mutex<Instant>>,
    ) -> Result<FileNode, String>
    where
        F: Fn(ScanProgress) + Send + Sync,
    {
        if self.cancel_flag.load(Ordering::Relaxed) {
            return Err("Scan cancelled by user".to_string());
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let meta = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_e) => {
                // Permission denied or missing entry
                return Ok(FileNode::new_file(name, 0, None));
            }
        };

        let modified = meta.modified().ok().and_then(|t| {
            t.duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as u32)
        });

        // 1. Symlink: Do not follow to prevent recursive loops
        if meta.file_type().is_symlink() {
            let size = meta.len();
            self.scanned_files.fetch_add(1, Ordering::Relaxed);
            self.total_size.fetch_add(size, Ordering::Relaxed);
            return Ok(FileNode::new_file(name, size, modified));
        }

        // 2. Regular File:
        if !meta.is_dir() {
            let should_count = self.inode_tracker.should_count(&meta);
            let size = if should_count { meta.len() } else { 0 };

            self.scanned_files.fetch_add(1, Ordering::Relaxed);
            self.total_size.fetch_add(size, Ordering::Relaxed);

            return Ok(FileNode::new_file(name, size, modified));
        }

        // 3. Directory:
        self.scanned_dirs.fetch_add(1, Ordering::Relaxed);

        // Throttle progress events to once every 50ms
        if let Some(cb) = progress_callback {
            if let Ok(mut last) = last_emit.try_lock() {
                if last.elapsed() > Duration::from_millis(50) {
                    *last = Instant::now();
                    cb(ScanProgress {
                        scanned_files: self.scanned_files.load(Ordering::Relaxed),
                        scanned_dirs: self.scanned_dirs.load(Ordering::Relaxed),
                        total_size: self.total_size.load(Ordering::Relaxed),
                        current_path: path.to_string_lossy().to_string(),
                        is_done: false,
                    });
                }
            }
        }

        let mut child_files: Vec<FileNode> = Vec::new();
        let mut child_dirs_paths: Vec<PathBuf> = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry_res in entries.flatten() {
                if self.cancel_flag.load(Ordering::Relaxed) {
                    return Err("Scan cancelled by user".to_string());
                }

                let entry_path = entry_res.path();
                let file_type = entry_res.file_type().ok();

                if let Some(ft) = file_type {
                    if ft.is_dir() {
                        child_dirs_paths.push(entry_path);
                    } else {
                        // Regular file or symlink - evaluate quickly in current thread
                        let entry_name = entry_res.file_name().to_string_lossy().to_string();
                        let entry_meta = fs::symlink_metadata(&entry_path).ok();
                        let (entry_size, entry_mod) = if let Some(ref em) = entry_meta {
                            let m = em.modified().ok().and_then(|t| {
                                t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs() as u32)
                            });
                            let s = if em.is_file() && !self.inode_tracker.should_count(em) {
                                0
                            } else {
                                em.len()
                            };
                            (s, m)
                        } else {
                            (0, None)
                        };

                        self.scanned_files.fetch_add(1, Ordering::Relaxed);
                        self.total_size.fetch_add(entry_size, Ordering::Relaxed);
                        child_files.push(FileNode::new_file(entry_name, entry_size, entry_mod));
                    }
                } else {
                    // Fallback
                    child_dirs_paths.push(entry_path);
                }
            }
        }

        // Rayon parallel branch traversal for subdirectories
        let mut child_dirs: Vec<FileNode> = if depth < 6 && child_dirs_paths.len() >= 2 {
            child_dirs_paths
                .into_par_iter()
                .map(|dir_path| self.scan_dir_parallel(&dir_path, depth + 1, progress_callback, last_emit))
                .collect::<Result<Vec<FileNode>, String>>()?
        } else {
            let mut dirs = Vec::with_capacity(child_dirs_paths.len());
            for dir_path in child_dirs_paths {
                dirs.push(self.scan_dir_parallel(&dir_path, depth + 1, progress_callback, last_emit)?);
            }
            dirs
        };

        let mut total_dir_size: u64 = 0;
        let mut total_file_count: u64 = child_files.len() as u64;
        let mut total_dir_count: u64 = child_dirs.len() as u64;

        for f in &child_files {
            total_dir_size += f.size;
        }

        for d in &child_dirs {
            total_dir_size += d.size;
            total_file_count += d.file_count();
            total_dir_count += d.dir_count();
        }

        // Combine files and directories
        let mut all_children = child_files;
        all_children.append(&mut child_dirs);

        // Sort descending by size
        if all_children.len() >= 64 {
            all_children.par_sort_by(|a, b| b.size.cmp(&a.size));
        } else {
            all_children.sort_by(|a, b| b.size.cmp(&a.size));
        }

        Ok(FileNode::new_dir(
            name,
            total_dir_size,
            modified,
            total_file_count as u32,
            total_dir_count as u32,
            all_children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_parallel_scanner() {
        let temp_dir = std::env::temp_dir().join("disk_diff_core_test_scan");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("sub1/sub2")).unwrap();

        let mut f1 = File::create(temp_dir.join("a.txt")).unwrap();
        f1.write_all(&vec![0u8; 1000]).unwrap();

        let mut f2 = File::create(temp_dir.join("sub1/b.txt")).unwrap();
        f2.write_all(&vec![0u8; 2000]).unwrap();

        let mut f3 = File::create(temp_dir.join("sub1/sub2/c.txt")).unwrap();
        f3.write_all(&vec![0u8; 3000]).unwrap();

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let scanner = Scanner::new(cancel_flag);
        let root = scanner.scan(&temp_dir, None::<&fn(ScanProgress)>).unwrap();

        assert_eq!(root.size, 6000);
        assert_eq!(root.file_count(), 3);
        assert_eq!(root.dir_count(), 2);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
