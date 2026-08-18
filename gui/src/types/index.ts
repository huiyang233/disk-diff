export interface SnapshotMeta {
  id: string;
  name: string;
  root_path: string;
  timestamp: number;
  formatted_time: string;
  total_size: number;
  total_files: number;
  total_dirs: number;
  snap_file_size?: number;
}

export type NavTab = 'scan' | 'diff' | 'snapshots' | 'about';

export type DiffStatus = 'added' | 'removed' | 'modified' | 'unchanged';

export interface FileItemView {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
  file_count: number;
  dir_count: number;
  modified: number | null;
  has_children: boolean;
}

export interface DirectoryView {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
  file_count: number;
  dir_count: number;
  children: FileItemView[];
}

export interface ScanResultView {
  meta: SnapshotMeta;
  root_view: DirectoryView;
}

export interface DiffItemView {
  name: string;
  path: string;
  is_dir: boolean;
  old_size: number | null;
  new_size: number | null;
  delta_size: number;
  delta_percent: number;
  status: DiffStatus;
  has_children: boolean;
}

export interface DiffDirectoryView {
  name: string;
  path: string;
  is_dir: boolean;
  old_size: number | null;
  new_size: number | null;
  delta_size: number;
  delta_percent: number;
  status: DiffStatus;
  children: DiffItemView[];
}

export interface DiffResultMeta {
  snapshot_a_name: string;
  snapshot_a_time: string;
  snapshot_b_name: string;
  snapshot_b_time: string;
  root_path: string;
  old_total_size: number;
  new_total_size: number;
  delta_total_size: number;
  delta_total_percent: number;
}

export interface DiffResultView {
  meta: DiffResultMeta;
  root_view: DiffDirectoryView;
}

export interface ScanProgress {
  scanned_files: number;
  scanned_dirs: number;
  total_size: number;
  current_path: string;
  is_done: boolean;
}

export interface DiffProgress {
  stage: string;
  progress_percent: number;
  is_done: boolean;
}

export type ViewMode = 'treemap' | 'list';
export type ColorTheme = 'stock_cn' | 'stock_us' | 'standard';
