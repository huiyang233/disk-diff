<script setup lang="ts">
import { ref, shallowRef, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { HardDrive, FolderOpen, Play, Square, Sparkles } from 'lucide-vue-next';

import Sidebar from './components/Sidebar.vue';
import TopBar from './components/TopBar.vue';
import Breadcrumb from './components/Breadcrumb.vue';
import TreemapView from './components/TreemapView.vue';
import ListView from './components/ListView.vue';
import SaveSnapshotModal from './components/SaveSnapshotModal.vue';
import SnapshotManagerView from './components/SnapshotManagerView.vue';
import SnapshotDiffView from './components/SnapshotDiffView.vue';
import SettingsView from './components/SettingsView.vue';
import AboutView from './components/AboutView.vue';
import { formatBytes, formatNumber } from './composables/useFormat';
import { useI18n } from './composables/useI18n';
import { useSettings } from './composables/useSettings';

const { t } = useI18n();
const { customStorageDir, colorTheme, initSettings } = useSettings();

const appToastMsg = ref('');
let appToastTimer: ReturnType<typeof setTimeout> | null = null;
function showAppToast(msg: string) {
  appToastMsg.value = msg;
  if (appToastTimer) clearTimeout(appToastTimer);
  appToastTimer = setTimeout(() => {
    appToastMsg.value = '';
  }, 3000);
}

import type {
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

// ==========================================
// 1. Scan Tab State (磁盘空间扫描独立状态)
// ==========================================
const selectedPath = ref('');
const isScanning = ref(false);
const scanProgress = ref<ScanProgress | null>(null);
const scanMeta = shallowRef<SnapshotMeta | null>(null);
const scanDirView = shallowRef<DirectoryView | null>(null);
const scanViewMode = ref<ViewMode>('treemap');

interface NavTrailItem {
  name: string;
  path: string;
}
const scanNavTrail = shallowRef<NavTrailItem[]>([]);

const scanBreadcrumbSegments = computed(() => {
  return scanNavTrail.value.map((item) => ({
    name: item.name || 'Root',
    fullPath: item.path,
  }));
});

// ==========================================
// 2. Diff Tab State (快照差异对比独立状态)
// ==========================================
const isDiffing = ref(false);
const diffProgress = ref<DiffProgress | null>(null);
const diffMeta = shallowRef<DiffResultMeta | null>(null);
const diffDirView = shallowRef<DiffDirectoryView | null>(null);
const diffViewMode = ref<ViewMode>('treemap');
const diffNavTrail = shallowRef<NavTrailItem[]>([]);
const diffPreselectedOld = ref<SnapshotMeta | null>(null);

const isDiffActive = computed(() => !!diffMeta.value && !!diffDirView.value);

const diffBreadcrumbSegments = computed(() => {
  return diffNavTrail.value.map((item) => ({
    name: item.name || 'Root',
    fullPath: item.path,
  }));
});

// ==========================================
// 3. Shared & Manager State
// ==========================================
const savedSnapshots = ref<SnapshotMeta[]>([]);
const saveSnapshotModalVisible = ref(false);
const isSavingSnapshot = ref(false);
const loadingSnapshotId = ref<string | null>(null);
const isLoadingExternal = ref(false);

let unlistenProgress: UnlistenFn | null = null;
let unlistenDiffProgress: UnlistenFn | null = null;

onMounted(async () => {
  try {
    await initSettings();

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
    const list = await invoke<SnapshotMeta[]>('list_saved_snapshots', {
      customDir: customStorageDir.value || undefined,
    });
    savedSnapshots.value = list;
  } catch (err) {
    console.error('Failed to list saved snapshots:', err);
  }
}

// ==========================================
// Scan Tab Handlers
// ==========================================

// Directory Picker
async function handlePickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('app.dialogTitle'),
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
  scanDirView.value = null;
  scanNavTrail.value = [];
  activeNavTab.value = 'scan';

  try {
    const res = await invoke<ScanResultView>('start_scan', {
      path: selectedPath.value,
    });
    scanMeta.value = res.meta;
    scanDirView.value = res.root_view;
    scanNavTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
    await refreshSavedSnapshots();
  } catch (err: any) {
    if (err !== 'Scan cancelled by user') {
      showAppToast(t('app.scanError', { error: String(err) }));
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

// Drill-Down in Scan Tab
async function handleScanDrillDown(item: FileItemView) {
  try {
    const nextView = await invoke<DirectoryView>('get_directory_node', {
      path: item.path,
    });
    scanDirView.value = nextView;
    scanNavTrail.value = [...scanNavTrail.value, { name: item.name, path: item.path }];
  } catch (err) {
    console.error('Failed to load subfolder:', err);
  }
}

// Breadcrumb Navigate in Scan Tab
async function handleScanNavigate(index: number) {
  const target = scanNavTrail.value[index];
  if (!target) return;

  try {
    const nextView = await invoke<DirectoryView>('get_directory_node', {
      path: target.path,
    });
    scanDirView.value = nextView;
    scanNavTrail.value = scanNavTrail.value.slice(0, index + 1);
  } catch (err) {
    console.error('Failed to navigate:', err);
  }
}

// Go Back in Scan Tab
function handleScanBack() {
  if (scanNavTrail.value.length > 1) {
    handleScanNavigate(scanNavTrail.value.length - 2);
  }
}

// Return to Welcome Screen in Scan Tab
function handleScanHome() {
  scanMeta.value = null;
  scanDirView.value = null;
  scanNavTrail.value = [];
  selectedPath.value = '';
}

// Save Snapshot
async function handleSaveSnapshot(customName: string) {
  if (!scanMeta.value || isSavingSnapshot.value) return;

  isSavingSnapshot.value = true;
  try {
    await new Promise((resolve) => setTimeout(resolve, 50));

    await invoke<string>('save_current_snapshot', {
      name: customName,
      customDir: customStorageDir.value || undefined,
    });
    await refreshSavedSnapshots();
    saveSnapshotModalVisible.value = false;
    showAppToast(t('app.saveSuccess'));
  } catch (err) {
    showAppToast(t('app.saveFailed', { error: String(err) }));
  } finally {
    isSavingSnapshot.value = false;
  }
}

// Load Saved Snapshot by ID into Scan Tab
async function handleOpenSavedSnapshot(snapMeta: SnapshotMeta) {
  if (loadingSnapshotId.value) return;
  loadingSnapshotId.value = snapMeta.id;
  try {
    await new Promise((resolve) => setTimeout(resolve, 50));

    const res = await invoke<ScanResultView>('load_saved_snapshot', {
      snapshotId: snapMeta.id,
      customDir: customStorageDir.value || undefined,
    });
    scanMeta.value = res.meta;
    scanDirView.value = res.root_view;
    selectedPath.value = res.root_view.path;
    scanNavTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
    saveSnapshotModalVisible.value = false;
    activeNavTab.value = 'scan';
    showAppToast(t('app.loadedSnapshot', { name: snapMeta.name }));
  } catch (err) {
    showAppToast(t('app.openSnapshotFailed', { error: String(err) }));
  } finally {
    loadingSnapshotId.value = null;
  }
}

// Delete Saved Snapshot
async function handleDeleteSnapshot(snapshotId: string) {
  try {
    const updatedList = await invoke<SnapshotMeta[]>('delete_saved_snapshot', {
      snapshotId,
      customDir: customStorageDir.value || undefined,
    });
    savedSnapshots.value = updatedList;
    showAppToast(t('app.snapshotDeleted'));
  } catch (err) {
    showAppToast(t('app.deleteSnapshotFailed', { error: String(err) }));
  }
}

// Load Snapshot File (.snap) into Scan Tab
async function handleLoadSnapshotFile() {
  if (isLoadingExternal.value) return;
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [{ name: t('app.snapFilterName'), extensions: ['snap'] }],
      title: t('app.openSnapDialogTitle'),
    });

    if (selected && typeof selected === 'string') {
      isLoadingExternal.value = true;
      await new Promise((resolve) => setTimeout(resolve, 50));
      const res = await invoke<ScanResultView>('load_snapshot', {
        filePath: selected,
      });
      scanMeta.value = res.meta;
      scanDirView.value = res.root_view;
      selectedPath.value = res.root_view.path;
      scanNavTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
      saveSnapshotModalVisible.value = false;
      activeNavTab.value = 'scan';
      showAppToast(t('app.loadedExternal'));
    }
  } catch (err) {
    showAppToast(t('app.loadExternalFailed', { error: String(err) }));
  } finally {
    isLoadingExternal.value = false;
  }
}

// ==========================================
// Diff Tab Handlers
// ==========================================

// Run Diff
async function handleDiffSnapshots(
  oldMeta: SnapshotMeta,
  newMeta: SnapshotMeta | null
) {
  try {
    isDiffing.value = true;
    diffProgress.value = {
      stage: t('diff.engineStarting'),
      progress_percent: 5,
      is_done: false,
    };

    await new Promise((resolve) => setTimeout(resolve, 50));

    let res: DiffResultView;

    if (!newMeta && scanMeta.value) {
      // Diff current active scan in memory with saved snapshot
      res = await invoke<DiffResultView>('diff_current_with_saved', {
        oldSnapshotId: oldMeta.id,
        customDir: customStorageDir.value || undefined,
      });
    } else if (oldMeta && newMeta) {
      // Diff two saved snapshots
      res = await invoke<DiffResultView>('diff_snapshots', {
        oldSnapshotId: oldMeta.id,
        newSnapshotId: newMeta.id,
        customDir: customStorageDir.value || undefined,
      });
    } else {
      isDiffing.value = false;
      return;
    }

    diffMeta.value = res.meta;
    diffDirView.value = res.root_view;
    diffNavTrail.value = [{ name: res.root_view.name || 'Root', path: res.root_view.path }];
    activeNavTab.value = 'diff';
  } catch (err) {
    showAppToast(t('app.diffFailed', { error: String(err) }));
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

// Drill-Down in Diff Tab
async function handleDiffDrillDown(item: DiffItemView) {
  try {
    const nextView = await invoke<DiffDirectoryView>('get_diff_directory_node', {
      path: item.path,
    });
    diffDirView.value = nextView;
    diffNavTrail.value = [...diffNavTrail.value, { name: item.name, path: item.path }];
  } catch (err) {
    console.error('Failed to load diff subfolder:', err);
  }
}

// Breadcrumb Navigate in Diff Tab
async function handleDiffNavigate(index: number) {
  const target = diffNavTrail.value[index];
  if (!target) return;

  try {
    const nextView = await invoke<DiffDirectoryView>('get_diff_directory_node', {
      path: target.path,
    });
    diffDirView.value = nextView;
    diffNavTrail.value = diffNavTrail.value.slice(0, index + 1);
  } catch (err) {
    console.error('Failed to navigate diff:', err);
  }
}

// Go Back in Diff Tab
function handleDiffBack() {
  if (diffNavTrail.value.length > 1) {
    handleDiffNavigate(diffNavTrail.value.length - 2);
  }
}

// Exit Diff Mode (Returns Diff Tab to Workbench)
function exitDiffMode() {
  diffMeta.value = null;
  diffDirView.value = null;
  diffNavTrail.value = [];
}

// Reveal in Finder
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
      <!-- 1. Disk Scan Tab (磁盘空间扫描 - 纯净扫描模式) -->
      <template v-if="activeNavTab === 'scan'">
        <!-- TopBar only visible when viewing scan results -->
        <TopBar
          v-if="scanDirView"
          :selected-path="selectedPath"
          :is-scanning="isScanning"
          :scan-progress="scanProgress"
          :has-scan-data="!!scanDirView"
          :view-mode="scanViewMode"
          @pick-directory="handlePickDirectory"
          @start-scan="handleStartScan"
          @cancel-scan="handleCancelScan"
          @save-snapshot="saveSnapshotModalVisible = true"
          @update:view-mode="scanViewMode = $event"
        />

        <!-- Breadcrumb Hierarchy Navigation -->
        <Breadcrumb
          v-if="scanBreadcrumbSegments.length > 0 && scanDirView"
          :segments="scanBreadcrumbSegments"
          :can-go-back="scanNavTrail.length > 1"
          @navigate="handleScanNavigate"
          @back="handleScanBack"
          @home="handleScanHome"
        />

        <!-- Main Scan Content Area -->
        <main class="main-content">
          <!-- Loading / Scanning Active Dashboard -->
          <div v-if="isScanning && !scanDirView" class="center-state">
            <div class="scanning-card glass-panel">
              <div class="scanning-header">
                <div class="spinner" />
                <div class="scanning-title-box">
                  <h3>{{ t('app.scanningTitle') }}</h3>
                  <div class="scan-target-path" :title="selectedPath">
                    {{ t('app.targetDir') }}: <span>{{ selectedPath }}</span>
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
                  <span class="metric-label">{{ t('app.totalSize') }}</span>
                  <span class="metric-value highlight">
                    {{ scanProgress ? formatBytes(scanProgress.total_size) : '0 B' }}
                  </span>
                </div>
                <div class="metric-card">
                  <span class="metric-label">{{ t('app.filesFound') }}</span>
                  <span class="metric-value">
                    {{ scanProgress ? formatNumber(scanProgress.scanned_files) : '0' }}
                  </span>
                </div>
                <div class="metric-card">
                  <span class="metric-label">{{ t('app.dirsTraversed') }}</span>
                  <span class="metric-value">
                    {{ scanProgress ? formatNumber(scanProgress.scanned_dirs) : '0' }}
                  </span>
                </div>
              </div>

              <!-- Live Scanning File/Subdirectory Path -->
              <div class="live-scanning-path-box">
                <span class="live-tag">{{ t('app.scanningLive') }}</span>
                <span class="live-subpath" :title="scanProgress?.current_path || selectedPath">
                  {{ scanProgress?.current_path || selectedPath }}
                </span>
              </div>

              <!-- Cancel Button -->
              <div class="scan-action-row">
                <button class="btn-danger cancel-scan-btn" @click="handleCancelScan">
                  <Square :size="13" />
                  <span>{{ t('topbar.cancelScan') }}</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Treemap View Mode (Pure Scan Mode: isDiffMode is strictly false) -->
          <TreemapView
            v-else-if="scanViewMode === 'treemap' && scanDirView"
            :current-node="scanDirView"
            :is-diff-mode="false"
            :color-theme="colorTheme"
            @drill-down="handleScanDrillDown as any"
            @reveal-in-finder="handleRevealInFinder"
          />

          <!-- List View Mode (Pure Scan Mode: isDiffMode is strictly false) -->
          <ListView
            v-else-if="scanViewMode === 'list' && scanDirView"
            :current-node="scanDirView"
            :is-diff-mode="false"
            @drill-down="handleScanDrillDown as any"
            @reveal-in-finder="handleRevealInFinder"
          />

          <!-- Initial / Empty State for Scan Tab -->
          <div v-else class="center-state initial-state">
            <div class="scan-welcome-card glass-panel">
              <div class="welcome-icon-glow">
                <HardDrive :size="36" class="welcome-icon" />
              </div>
              <h2>{{ t('app.welcomeTitle') }}</h2>
              <p class="welcome-desc">
                {{ t('app.welcomeDesc') }}
              </p>

              <!-- Central Directory Selector Zone -->
              <div class="drop-select-zone" @click="handlePickDirectory">
                <div class="zone-icon-box">
                  <FolderOpen :size="22" />
                </div>
                <div class="zone-text">
                  <span v-if="selectedPath" class="path-selected" :title="selectedPath">{{ selectedPath }}</span>
                  <span v-else class="path-placeholder">{{ t('app.welcomePlaceholder') }}</span>
                </div>
                <button class="btn-secondary btn-sm browse-btn" @click.stop="handlePickDirectory">
                  {{ t('app.browseBtn') }}
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
                  <span>{{ t('app.startFastScan') }}</span>
                </button>
              </div>
            </div>
          </div>
        </main>
      </template>

      <!-- 2. Snapshot Diff Tab (快照差异对比 - 专属对比模式) -->
      <template v-else-if="activeNavTab === 'diff'">
        <SnapshotDiffView
          :saved-snapshots="savedSnapshots"
          :current-snapshot-meta="scanMeta"
          :is-diff-mode="isDiffActive"
          :is-diffing="isDiffing"
          :diff-progress="diffProgress"
          :diff-meta="diffMeta"
          :current-dir-view="diffDirView as any"
          :breadcrumb-segments="diffBreadcrumbSegments"
          :can-go-back="diffNavTrail.length > 1"
          :view-mode="diffViewMode"
          :color-theme="colorTheme"
          :preselected-snapshot="diffPreselectedOld"
          @run-diff="handleDiffSnapshots"
          @load-external-snapshot="handleLoadSnapshotFile"
          @exit-diff="exitDiffMode"
          @drill-down="handleDiffDrillDown as any"
          @navigate="handleDiffNavigate"
          @back="handleDiffBack"
          @reveal-in-finder="handleRevealInFinder"
          @update:view-mode="diffViewMode = $event"
          @update:color-theme="colorTheme = $event"
        />
      </template>

      <!-- 3. Snapshot Manager Tab (快照历史管理) -->
      <template v-else-if="activeNavTab === 'snapshots'">
        <SnapshotManagerView
          :saved-snapshots="savedSnapshots"
          :current-snapshot-meta="scanMeta"
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

      <!-- 4. Settings Tab (偏好设置) -->
      <template v-else-if="activeNavTab === 'settings'">
        <SettingsView />
      </template>

      <!-- 5. About Tab (关于软件) -->
      <template v-else-if="activeNavTab === 'about'">
        <AboutView />
      </template>
    </div>

    <!-- Global Toast Notification -->
    <transition name="toast-fade">
      <div v-if="appToastMsg" class="global-toast">
        <Sparkles :size="14" />
        <span>{{ appToastMsg }}</span>
      </div>
    </transition>

    <!-- Dedicated Save Snapshot Modal -->
    <SaveSnapshotModal
      v-if="saveSnapshotModalVisible"
      :visible="saveSnapshotModalVisible"
      :current-snapshot-meta="scanMeta"
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

/* Global Floating Toast */
.global-toast {
  position: fixed;
  top: 20px;
  right: 24px;
  z-index: 10000;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 16px;
  background: var(--accent-emerald);
  color: #0b0e14;
  font-size: 12.5px;
  font-weight: 600;
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
}

.toast-fade-enter-active,
.toast-fade-leave-active {
  transition: all 0.2s ease;
}

.toast-fade-enter-from,
.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>

