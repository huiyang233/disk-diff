pub mod commands;

pub use disk_diff_core as core;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_scan,
            cancel_scan,
            get_current_snapshot_meta,
            get_directory_node,
            get_default_storage_dir,
            save_current_snapshot,
            load_snapshot,
            load_saved_snapshot,
            delete_saved_snapshot,
            list_saved_snapshots,
            diff_snapshots,
            diff_current_with_saved,
            get_diff_directory_node,
            reveal_in_finder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
