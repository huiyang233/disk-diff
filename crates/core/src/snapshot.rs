use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::Local;

use crate::model::{FileNode, Snapshot, SnapshotMeta};

const MAGIC_HEADER: &[u8; 8] = b"DISKDIFF";
const FORMAT_VERSION: u8 = 1;

pub struct SnapshotManager;

impl SnapshotManager {
    pub fn get_default_storage_dir() -> PathBuf {
        if let Some(mut dir) = dirs_next().or_else(|| std::env::current_dir().ok()) {
            dir.push(".diskdiff");
            dir.push("snapshots");
            let _ = fs::create_dir_all(&dir);
            return dir;
        }
        PathBuf::from("snapshots")
    }

    pub fn create_snapshot(root: FileNode, root_path: String, name_opt: Option<String>) -> Snapshot {
        let now = Local::now();
        let timestamp = now.timestamp();
        let formatted_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let id = format!("{}_{}", timestamp, nanoid_simple());
        let name = name_opt.unwrap_or_else(|| {
            format!("{}_{}", root.name, now.format("%Y%m%d_%H%M%S"))
        });

        let meta = SnapshotMeta {
            id,
            name,
            root_path,
            timestamp,
            formatted_time,
            total_size: root.size,
            total_files: root.file_count(),
            total_dirs: root.dir_count(),
            snap_file_size: None,
        };

        Snapshot { meta, root }
    }

    /// Save snapshot into high-compression binary format with instant header metadata
    /// File Format:
    /// - [0..8] Magic: "DISKDIFF"
    /// - [8] Version: 1 (u8)
    /// - [9..13] Header length: u32 (little endian)
    /// - [13..13+header_len] SnapshotMeta (JSON bytes)
    /// - [13+header_len..] Zstd level 9 compressed Bincode of FileNode (Root tree)
    pub fn save_to_file(snapshot: &Snapshot, target_path: &Path) -> Result<(), String> {
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // 1. Serialize metadata header to JSON
        let meta_json = serde_json::to_vec(&snapshot.meta)
            .map_err(|e| format!("Meta serialization error: {}", e))?;
        let meta_len = meta_json.len() as u32;

        // 2. Serialize root FileNode with bincode and compress with Zstd (level 9)
        let root_encoded = bincode::serialize(&snapshot.root)
            .map_err(|e| format!("Root tree serialization error: {}", e))?;

        let compressed_root = zstd::encode_all(&root_encoded[..], 9)
            .map_err(|e| format!("Compression error: {}", e))?;

        // 3. Write binary container
        let mut file = File::create(target_path)
            .map_err(|e| format!("Failed to create snapshot file: {}", e))?;

        file.write_all(MAGIC_HEADER)
            .map_err(|e| format!("Failed to write magic header: {}", e))?;
        file.write_all(&[FORMAT_VERSION])
            .map_err(|e| format!("Failed to write version: {}", e))?;
        file.write_all(&meta_len.to_le_bytes())
            .map_err(|e| format!("Failed to write header length: {}", e))?;
        file.write_all(&meta_json)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;
        file.write_all(&compressed_root)
            .map_err(|e| format!("Failed to write compressed tree body: {}", e))?;

        Ok(())
    }

    /// Read ONLY the fast header without decompressing the multi-megabyte tree body
    pub fn read_meta_only(file_path: &Path) -> Result<SnapshotMeta, String> {
        let mut file = File::open(file_path)
            .map_err(|e| format!("Failed to open snapshot file: {}", e))?;

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .map_err(|e| format!("Failed to read magic: {}", e))?;

        if &magic != MAGIC_HEADER {
            return Err("Invalid snapshot file format".to_string());
        }

        let mut version = [0u8; 1];
        file.read_exact(&mut version)
            .map_err(|e| format!("Failed to read version: {}", e))?;

        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)
            .map_err(|e| format!("Failed to read meta length: {}", e))?;
        let meta_len = u32::from_le_bytes(len_bytes) as usize;

        let mut meta_json = vec![0u8; meta_len];
        file.read_exact(&mut meta_json)
            .map_err(|e| format!("Failed to read meta bytes: {}", e))?;

        let mut meta: SnapshotMeta = serde_json::from_slice(&meta_json)
            .map_err(|e| format!("Failed to parse metadata JSON: {}", e))?;

        if let Ok(metadata) = file.metadata() {
            meta.snap_file_size = Some(metadata.len());
        }

        Ok(meta)
    }

    /// Load full snapshot (Header + Decompressed Root Tree)
    pub fn load_from_file(file_path: &Path) -> Result<Snapshot, String> {
        let mut file = File::open(file_path)
            .map_err(|e| format!("Failed to open snapshot file: {}", e))?;

        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .map_err(|e| format!("Failed to read magic: {}", e))?;

        if &magic != MAGIC_HEADER {
            return Err("Invalid snapshot file format".to_string());
        }

        let mut version = [0u8; 1];
        file.read_exact(&mut version)
            .map_err(|e| format!("Failed to read version: {}", e))?;

        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)
            .map_err(|e| format!("Failed to read meta length: {}", e))?;
        let meta_len = u32::from_le_bytes(len_bytes) as usize;

        let mut meta_json = vec![0u8; meta_len];
        file.read_exact(&mut meta_json)
            .map_err(|e| format!("Failed to read meta: {}", e))?;

        let meta: SnapshotMeta = serde_json::from_slice(&meta_json)
            .map_err(|e| format!("Failed to parse meta: {}", e))?;

        let mut compressed_root = Vec::new();
        file.read_to_end(&mut compressed_root)
            .map_err(|e| format!("Failed to read compressed tree body: {}", e))?;

        let decompressed = zstd::decode_all(&compressed_root[..])
            .map_err(|e| format!("Decompression error: {}", e))?;

        let root: FileNode = bincode::deserialize(&decompressed)
            .map_err(|e| format!("Root tree deserialization error: {}", e))?;

        Ok(Snapshot { meta, root })
    }

    /// List all saved snapshots by reading ONLY the fast binary headers (< 1ms!)
    pub fn list_saved_snapshots() -> Vec<SnapshotMeta> {
        let dir = Self::get_default_storage_dir();
        let mut results = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("snap") {
                    if let Ok(meta) = Self::read_meta_only(&path) {
                        results.push(meta);
                    }
                }
            }
        }

        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results
    }
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn nanoid_simple() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:x}", now)
}
