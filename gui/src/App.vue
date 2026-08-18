<script setup lang="ts">
import { ref, shallowRef, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { HardDrive, FolderOpen, Play, Square } from 'lucide-vue-next';

import Sidebar from './components/Sidebar.vue';
import TopBar from './components/TopBar.vue';
import Breadcrumb from './components/Breadcrumb.vue';
import TreemapView from './components/TreemapView.vue';
import ListView from './components/ListView.vue';
import SaveSnapshotModal from './components/SaveSnapshotModal.vue';
import SnapshotManagerView from './components/SnapshotManagerView.vue';
import SnapshotDiffView from './components/SnapshotDiffView.vue';
import AboutView from './components/AboutView.vue';
import { formatBytes, formatNumber } from './composables/useFormat';

import type {
  ColorTheme,
  DiffDirectoryView,
  DiffItemView,
  DiffProgress,
  DiffResultMeta,
  DiffResultView,
  DirectoryView,
  FileItemView,
  NavTab,
  ScanProgress,
  ScanResultView,
  SnapshotMeta,
  ViewMode,
} from './types';

// Sidebar navigation
const activeNavTab = ref<NavTab>('scan');

// Scan State
const selectedPath = ref('');
const isScanning = ref(false);
const scanProgress = ref<ScanProgress | null>(null);

// Diff State
const isDiffing = ref(false);
const diffProgress = ref<DiffProgress | null>(null);

// Shallow state for current view and metadata (Rust backend retains full deep tree in memory)
const currentMeta = shallowRef<SnapshotMeta | null>(null);
const currentDirView = shallowRef<DirectoryView | DiffDirectoryView | null>(null);
const diffMeta = shallowRef<DiffResultMeta | null>(null);
const isDiffMode = ref(false);

const viewMode = ref<ViewMode>('treemap');
const colorTheme = ref<ColorTheme>('stock_cn');
const savedSnapshots = ref<SnapshotMeta[]>([]);
const saveSnapshotModalVisible = ref(false);
const isSavingSnapshot = ref(false);
const loadingSnapshotId = ref<string | null>(null);
const isLoadingExternal = ref(false);
const diffPreselectedOld = ref<SnapshotMeta | null>(null);

// Navigation trail for scan explorer
interface NavTrailItem {
  name: string;
  path: string;
}
const navTrail = shallowRef<NavTrailItem[]>([]);

const breadcrumbSegments = computed(() => {
  return navTrail.value.map((item) => ({
    name: item.name || 'Root',
    fullPath: item.path,
  }));
});

let unlistenProgress: UnlistenFn | null = null;
let unlistenDiffProgress: UnlistenFn | null = null;

onMounted(async () => {
  try {
    // Listen to streaming scan progress
    unlistenProgress = await listen<ScanProgress>('scan-progress', (event) => {
      scanProgress.value = event.payload;
      if (event.payload.is_done) {
        isScanning.value = false;
      }
    });

    // Listen to streaming diff progress
    unlistenDiffProgress = await listen<DiffProgress>('diff-progress', (event) => {
      diffProgress.value = event.payload;
      if (event.payload.is_done) {
        isDiffing.value = false;
      }
    });

    // Load saved snapshot list (uses fast binary headers)
    await refreshSavedSnapshots();
  } catch (err) {
    console.warn('Tauri event / API listener init:', err);
  }
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
  if (unlistenDiffProgress) {
    unlistenDiffProgress();
  }
});

async function refreshSavedSnapshots() {
  try {
    const list = await invoke<SnapshotMeta[]>('list_saved_snapshots');
    savedSnapshots.value = list;
  } catch (err) {
    console.error('Failed to list saved snapshots:', err);
  }
}

// Directory Picker
async function handlePickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择要扫描分析的文件夹',
    });

    if (selected && typeof selected === 'string') {
      selectedPath.value = selected;
    }
  } catch (err) {
    console.error('Dialog picker error:', err);
  }
}

// Start Scan
async function handleStartScan() {
  if (!selectedPath.value || isScanning.value) return;

  isScanning.value = true;
  scanProgress.value = null;
  isDiffMode.value = false;
  diffMeta.value = null;
  currentDirView.value = null;
  navTrail.value = [];
  activeNavTab.value = 'scan';

  try {
    const res = await invoke<ScanResultView>('start_scan', {
      path: selectedPath.value,
    });
    currentMeta.value = res.meta;
    currentDirView.value = res.root_view;
    navTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
    await refreshSavedSnapshots();
  } catch (err: any) {
    if (err !== 'Scan cancelled by user') {
      alert(`扫描出错: ${err}`);
    }
  } finally {
    isScanning.value = false;
  }
}

// Cancel Scan
async function handleCancelScan() {
  try {
    await invoke('cancel_scan');
    isScanning.value = false;
  } catch (err) {
    console.error('Failed to cancel scan:', err);
  }
}

// Save Snapshot
async function handleSaveSnapshot(customName: string) {
  if (!currentMeta.value || isSavingSnapshot.value) return;

  isSavingSnapshot.value = true;
  try {
    // Yield 1 micro-tick so browser repaints and runs the spinner immediately
    await new Promise((resolve) => setTimeout(resolve, 50));

    const savedPath = await invoke<string>('save_current_snapshot', {
      name: customName,
    });
    await refreshSavedSnapshots();
    saveSnapshotModalVisible.value = false;
    alert(`快照保存成功！\n保存路径: ${savedPath}`);
  } catch (err) {
    alert(`保存快照失败: ${err}`);
  } finally {
    isSavingSnapshot.value = false;
  }
}

// Load Saved Snapshot by ID
async function handleOpenSavedSnapshot(snapMeta: SnapshotMeta) {
  if (loadingSnapshotId.value) return;
  loadingSnapshotId.value = snapMeta.id;
  try {
    // Yield 1 micro-tick to let browser render the spinner
    await new Promise((resolve) => setTimeout(resolve, 50));

    const res = await invoke<ScanResultView>('load_saved_snapshot', {
      snapshotId: snapMeta.id,
    });
    currentMeta.value = res.meta;
    currentDirView.value = res.root_view;
    selectedPath.value = res.root_view.path;
    isDiffMode.value = false;
    diffMeta.value = null;
    navTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
    saveSnapshotModalVisible.value = false;
    activeNavTab.value = 'scan';
  } catch (err) {
    alert(`打开快照失败: ${err}`);
  } finally {
    loadingSnapshotId.value = null;
  }
}

// Delete Saved Snapshot
async function handleDeleteSnapshot(snapshotId: string) {
  try {
    const updatedList = await invoke<SnapshotMeta[]>('delete_saved_snapshot', {
      snapshotId,
    });
    savedSnapshots.value = updatedList;
  } catch (err) {
    alert(`删除快照失败: ${err}`);
  }
}

// Load Snapshot File (.snap)
async function handleLoadSnapshotFile() {
  if (isLoadingExternal.value) return;
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [{ name: 'Snapshot File', extensions: ['snap'] }],
      title: '打开磁盘快照文件',
    });

    if (selected && typeof selected === 'string') {
      isLoadingExternal.value = true;
      await new Promise((resolve) => setTimeout(resolve, 50));
      const res = await invoke<ScanResultView>('load_snapshot', {
        filePath: selected,
      });
      currentMeta.value = res.meta;
      currentDirView.value = res.root_view;
      selectedPath.value = res.root_view.path;
      isDiffMode.value = false;
      diffMeta.value = null;
      navTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
      saveSnapshotModalVisible.value = false;
      activeNavTab.value = 'scan';
    }
  } catch (err) {
    alert(`加载快照失败: ${err}`);
  } finally {
    isLoadingExternal.value = false;
  }
}

// Run Diff
async function handleDiffSnapshots(
  oldMeta: SnapshotMeta,
  newMeta: SnapshotMeta | null
) {
  try {
    isDiffing.value = true;
    diffProgress.value = {
      stage: '正在启动深度对比引擎...',
      progress_percent: 5,
      is_done: false,
    };

    // Yield control to let Vue render the loading card immediately
    await new Promise((resolve) => setTimeout(resolve, 50));

    let res: DiffResultView;

    if (!newMeta && currentMeta.value) {
      // Diff current active scan in memory with saved snapshot
      res = await invoke<DiffResultView>('diff_current_with_saved', {
        oldSnapshotId: oldMeta.id,
      });
    } else if (oldMeta && newMeta) {
      // Diff two saved snapshots
      res = await invoke<DiffResultView>('diff_snapshots', {
        oldSnapshotId: oldMeta.id,
        newSnapshotId: newMeta.id,
      });
    } else {
      isDiffing.value = false;
      return;
    }

    diffMeta.value = res.meta;
    currentDirView.value = res.root_view;
    isDiffMode.value = true;
    navTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
    saveSnapshotModalVisible.value = false;
    activeNavTab.value = 'diff';
  } catch (err) {
    alert(`对比分析失败: ${err}`);
  } finally {
    isDiffing.value = false;
    diffProgress.value = null;
  }
}

// Quick Diff from Snapshot Manager
function handleDiffWithSnapshot(snap: SnapshotMeta) {
  diffPreselectedOld.value = snap;
  activeNavTab.value = 'diff';
}

// Exit Diff Mode
async function exitDiffMode() {
  isDiffMode.value = false;
  diffMeta.value = null;
  if (currentMeta.value) {
    try {
      const root = await invoke<DirectoryView>('get_directory_node', {
        path: currentMeta.value.root_path,
      });
      currentDirView.value = root;
      navTrail.value = [{ name: root.name || 'Root', path: root.path }];
    } catch {
      handleGoHome();
    }
  } else {
    handleGoHome();
  }
}

// Drill-Down into Folder (On-Demand load from Rust backend)
async function handleDrillDown(item: FileItemView | DiffItemView) {
  try {
    if (isDiffMode.value) {
      const nextView = await invoke<DiffDirectoryView>('get_diff_directory_node', {
        path: item.path,
      });
      currentDirView.value = nextView;
    } else {
      const nextView = await invoke<DirectoryView>('get_directory_node', {
        path: item.path,
      });
      currentDirView.value = nextView;
    }
    navTrail.value = [...navTrail.value, { name: item.name, path: item.path }];
  } catch (err) {
    console.error('Failed to load subfolder:', err);
  }
}

// Navigate via Breadcrumb
async function handleNavigate(index: number) {
  const target = navTrail.value[index];
  if (!target) return;

  try {
    if (isDiffMode.value) {
      const nextView = await invoke<DiffDirectoryView>('get_diff_directory_node', {
        path: target.path,
      });
      currentDirView.value = nextView;
    } else {
      const nextView = await invoke<DirectoryView>('get_directory_node', {
        path: target.path,
      });
      currentDirView.value = nextView;
    }
    navTrail.value = navTrail.value.slice(0, index + 1);
  } catch (err) {
    console.error('Failed to navigate:', err);
  }
}

// Go Back 1 Level
function handleBack() {
  if (navTrail.value.length > 1) {
    handleNavigate(navTrail.value.length - 2);
  }
}

// Return to Welcome / Home Screen
function handleGoHome() {
  currentMeta.value = null;
  diffMeta.value = null;
  currentDirView.value = null;
  isDiffMode.value = false;
  navTrail.value = [];
  selectedPath.value = '';
}

// Reveal in Finder / File Manager
async function handleRevealInFinder(path: string) {
  try {
    await invoke('reveal_in_finder', { path });
  } catch (err) {
    console.error('Failed to reveal path in finder:', err);
  }
}
</script>

<template>
  <div class="app-layout">
    <!-- Left Navigation Sidebar -->
    <Sidebar
      :active-tab="activeNavTab"
      :snapshot-count="savedSnapshots.length"
      :is-scanning="isScanning"
      @update:active-tab="activeNavTab = $event"
    />

    <!-- Main Workspace Area -->
    <div class="workspace-area">
      <!-- 1. Disk Scan Tab -->
      <template v-if="activeNavTab === 'scan'">
        <!-- TopBar only visible when viewing scan results -->
        <TopBar
          v-if="currentDirView"
          :selected-path="selectedPath"
          :is-scanning="isScanning"
          :scan-progress="scanProgress"
          :has-scan-data="!!currentDirView"
          :view-mode="viewMode"
          :is-diff-mode="isDiffMode"
          :diff-result="diffMeta as any"
          :color-theme="colorTheme"
          @pick-directory="handlePickDirectory"
          @start-scan="handleStartScan"
          @cancel-scan="handleCancelScan"
          @save-snapshot="saveSnapshotModalVisible = true"
          @update:view-mode="viewMode = $event"
          @update:color-theme="colorTheme = $event"
          @exit-diff-mode="exitDiffMode"
        />

        <!-- Breadcrumb Hierarchy Navigation -->
        <Breadcrumb
          v-if="breadcrumbSegments.length > 0 && currentDirView"
          :segments="breadcrumbSegments"
          :can-go-back="navTrail.length > 1"
          @navigate="handleNavigate"
          @back="handleBack"
          @home="handleGoHome"
        />

        <!-- Main Scan Content Area -->
        <main class="main-content">
          <!-- Loading / Scanning Active Dashboard -->
          <div v-if="isScanning && !currentDirView" class="center-state">
            <div class="scanning-card glass-panel">
              <div class="scanning-header">
                <div class="spinner" />
                <div class="scanning-title-box">
                  <h3>正在高速多线程扫描目录...</h3>
                  <div class="scan-target-path" :title="selectedPath">
                    目标目录: <span>{{ selectedPath }}</span>
                  </div>
                </div>
              </div>

              <!-- Animated Active Progress Bar -->
              <div class="scan-progress-bar-track">
                <div class="scan-progress-bar-fill animated-stripe" />
              </div>

              <!-- Metrics 3-card grid -->
              <div class="scan-metrics-grid">
                <div class="metric-card">
                  <span class="metric-label">累计容量</span>
                  <span class="metric-value highlight">
                    {{ scanProgress ? formatBytes(scanProgress.total_size) : '0 B' }}
                  </span>
                </div>
                <div class="metric-card">
                  <span class="metric-label">已发现文件</span>
                  <span class="metric-value">
                    {{ scanProgress ? formatNumber(scanProgress.scanned_files) : '0' }}
                  </span>
                </div>
                <div class="metric-card">
                  <span class="metric-label">已遍历文件夹</span>
                  <span class="metric-value">
                    {{ scanProgress ? formatNumber(scanProgress.scanned_dirs) : '0' }}
                  </span>
                </div>
              </div>

              <!-- Live Scanning File/Subdirectory Path -->
              <div class="live-scanning-path-box">
                <span class="live-tag">正在扫描:</span>
                <span class="live-subpath" :title="scanProgress?.current_path || selectedPath">
                  {{ scanProgress?.current_path || selectedPath }}
                </span>
              </div>

              <!-- Cancel Button -->
              <div class="scan-action-row">
                <button class="btn-danger cancel-scan-btn" @click="handleCancelScan">
                  <Square :size="13" />
                  <span>取消扫描</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Treemap View Mode -->
          <TreemapView
            v-else-if="viewMode === 'treemap' && currentDirView"
            :current-node="currentDirView"
            :is-diff-mode="isDiffMode"
            :color-theme="colorTheme"
            @drill-down="handleDrillDown"
            @reveal-in-finder="handleRevealInFinder"
          />

          <!-- List View Mode -->
          <ListView
            v-else-if="viewMode === 'list' && currentDirView"
            :current-node="currentDirView"
            :is-diff-mode="isDiffMode"
            @drill-down="handleDrillDown"
            @reveal-in-finder="handleRevealInFinder"
          />

          <!-- Initial / Empty State for Scan Tab -->
          <div v-else class="center-state initial-state">
            <div class="scan-welcome-card glass-panel">
              <div class="welcome-icon-glow">
                <HardDrive :size="36" class="welcome-icon" />
              </div>
              <h2>选择要扫描分析的文件夹</h2>
              <p class="welcome-desc">
                多线程并发遍历，通过矩形树图可视化分析磁盘容量分布与大文件占比
              </p>

              <!-- Central Directory Selector Zone -->
              <div class="drop-select-zone" @click="handlePickDirectory">
                <div class="zone-icon-box">
                  <FolderOpen :size="22" />
                </div>
                <div class="zone-text">
                  <span v-if="selectedPath" class="path-selected" :title="selectedPath">{{ selectedPath }}</span>
                  <span v-else class="path-placeholder">点击选择电脑上的任意文件夹或磁盘路径...</span>
                </div>
                <button class="btn-secondary btn-sm browse-btn" @click.stop="handlePickDirectory">
                  浏览...
                </button>
              </div>

              <!-- Action buttons -->
              <div class="welcome-actions">
                <button
                  class="btn-primary start-scan-btn"
                  :disabled="!selectedPath"
                  @click="handleStartScan"
                >
                  <Play :size="16" />
                  <span>开始高速扫描</span>
                </button>
              </div>
            </div>
          </div>
        </main>
      </template>

      <!-- 2. Snapshot Diff Tab -->
      <template v-else-if="activeNavTab === 'diff'">
        <SnapshotDiffView
          :saved-snapshots="savedSnapshots"
          :current-snapshot-meta="currentMeta"
          :is-diff-mode="isDiffMode"
          :is-diffing="isDiffing"
          :diff-progress="diffProgress"
          :diff-meta="diffMeta"
          :current-dir-view="currentDirView as any"
          :breadcrumb-segments="breadcrumbSegments"
          :can-go-back="navTrail.length > 1"
          :view-mode="viewMode"
          :color-theme="colorTheme"
          :preselected-snapshot="diffPreselectedOld"
          @run-diff="handleDiffSnapshots"
          @load-external-snapshot="handleLoadSnapshotFile"
          @exit-diff="exitDiffMode"
          @drill-down="handleDrillDown as any"
          @navigate="handleNavigate"
          @back="handleBack"
          @reveal-in-finder="handleRevealInFinder"
          @update:view-mode="viewMode = $event"
          @update:color-theme="colorTheme = $event"
        />
      </template>

      <!-- 3. Snapshot Manager Tab -->
      <template v-else-if="activeNavTab === 'snapshots'">
        <SnapshotManagerView
          :saved-snapshots="savedSnapshots"
          :current-snapshot-meta="currentMeta"
          :loading-snapshot-id="loadingSnapshotId"
          :is-loading-external="isLoadingExternal"
          @open-snapshot="handleOpenSavedSnapshot"
          @diff-with-snapshot="handleDiffWithSnapshot"
          @delete-snapshot="handleDeleteSnapshot"
          @save-current-snapshot="saveSnapshotModalVisible = true"
          @load-external-snapshot="handleLoadSnapshotFile"
          @reveal-in-finder="handleRevealInFinder"
        />
      </template>

      <!-- 4. About Tab -->
      <template v-else-if="activeNavTab === 'about'">
        <AboutView />
      </template>
    </div>

    <!-- Dedicated Save Snapshot Modal -->
    <SaveSnapshotModal
      :visible="saveSnapshotModalVisible"
      :current-snapshot-meta="currentMeta"
      :is-saving="isSavingSnapshot"
      @close="saveSnapshotModalVisible = false"
      @save="handleSaveSnapshot"
    />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  background: var(--bg-app);
  overflow: hidden;
}

.workspace-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  position: relative;
}

.main-content {
  flex: 1;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.center-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
}

/* Scanning Active Card */
.scanning-card {
  padding: 24px 28px;
  width: 100%;
  max-width: 540px;
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
}

.scanning-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 2.5px solid rgba(56, 189, 248, 0.15);
  border-top-color: #38bdf8;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.scanning-title-box {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.scanning-title-box h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.scan-target-path {
  font-size: 11.5px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scan-target-path span {
  font-family: var(--font-mono);
  color: #38bdf8;
}

/* Progress bar */
.scan-progress-bar-track {
  width: 100%;
  height: 6px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.scan-progress-bar-fill {
  height: 100%;
  width: 100%;
  border-radius: var(--radius-full);
  background: linear-gradient(
    90deg,
    #38bdf8 0%,
    #3b82f6 50%,
    #38bdf8 100%
  );
  background-size: 200% 100%;
  animation: progressPulse 1.4s ease infinite;
}

@keyframes progressPulse {
  0% { background-position: 100% 0; }
  100% { background-position: -100% 0; }
}

/* Metrics grid */
.scan-metrics-grid {
  display: grid;
  grid-template-columns: 1.2fr 1fr 1fr;
  gap: 8px;
}

.metric-card {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
}

.metric-label {
  font-size: 11px;
  color: var(--text-muted);
}

.metric-value {
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.metric-value.highlight {
  color: #38bdf8;
}

/* Live path */
.live-scanning-path-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: var(--radius-xs);
  border: 1px solid var(--border-subtle);
  font-size: 11px;
  overflow: hidden;
}

.live-tag {
  color: var(--text-muted);
  flex-shrink: 0;
  font-weight: 500;
}

.live-subpath {
  font-family: var(--font-mono);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scan-action-row {
  display: flex;
  justify-content: flex-end;
}

.cancel-scan-btn {
  padding: 5px 12px;
  font-size: 12px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

/* Landing scan card */
.scan-welcome-card {
  padding: 32px 36px;
  text-align: center;
  max-width: 480px;
  width: 100%;
  border-radius: var(--radius-xl);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
}

.welcome-icon-glow {
  width: 52px;
  height: 52px;
  border-radius: var(--radius-md);
  background: linear-gradient(180deg, rgba(56, 189, 248, 0.15) 0%, rgba(59, 130, 246, 0.2) 100%);
  border: 1px solid rgba(56, 189, 248, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
  margin-bottom: 2px;
}

.welcome-icon {
  color: #38bdf8;
}

.scan-welcome-card h2 {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.welcome-desc {
  font-size: 12.5px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 6px;
}

.drop-select-zone {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px dashed var(--border-medium);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s ease;
}

.drop-select-zone:hover {
  background: rgba(56, 189, 248, 0.04);
  border-color: rgba(56, 189, 248, 0.5);
}

.zone-icon-box {
  color: #38bdf8;
  display: flex;
  align-items: center;
}

.zone-text {
  flex: 1;
  text-align: left;
  overflow: hidden;
}

.path-selected {
  font-family: var(--font-mono);
  font-size: 11.5px;
  font-weight: 500;
  color: #38bdf8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.path-placeholder {
  font-size: 12px;
  color: var(--text-muted);
}

.browse-btn {
  padding: 4px 9px;
  font-size: 11.5px;
}

.welcome-actions {
  width: 100%;
  margin-top: 6px;
}

.start-scan-btn {
  width: 100%;
  padding: 9px;
  font-size: 13px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  letter-spacing: -0.01em;
}
</style>

