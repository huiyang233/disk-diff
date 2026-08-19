<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  GitCompare,
  FolderOpen,
  ArrowRight,
  TrendingUp,
  RotateCcw,
  Calendar,
  Folder,
  Layers,
  Zap,
  CheckCircle2,
} from 'lucide-vue-next';
import { formatBytes, formatDelta, formatPercent, formatNumber } from '../composables/useFormat';
import { useI18n } from '../composables/useI18n';
import TreemapView from './TreemapView.vue';
import ListView from './ListView.vue';
import Breadcrumb from './Breadcrumb.vue';

const { t, isZh } = useI18n();
import type {
  ColorTheme,
  DiffDirectoryView,
  DiffItemView,
  DiffProgress,
  DiffResultMeta,
  SnapshotMeta,
  ViewMode,
} from '../types';

const props = defineProps<{
  savedSnapshots: SnapshotMeta[];
  currentSnapshotMeta: SnapshotMeta | null;
  isDiffMode: boolean;
  isDiffing?: boolean;
  diffProgress?: DiffProgress | null;
  diffMeta: DiffResultMeta | null;
  currentDirView: DiffDirectoryView | null;
  breadcrumbSegments: { name: string; fullPath: string }[];
  canGoBack: boolean;
  viewMode: ViewMode;
  colorTheme: ColorTheme;
  preselectedSnapshot?: SnapshotMeta | null;
}>();

const emit = defineEmits<{
  (e: 'runDiff', oldMeta: SnapshotMeta, newMeta: SnapshotMeta | null): void;
  (e: 'loadExternalSnapshot'): void;
  (e: 'exitDiff'): void;
  (e: 'drillDown', item: DiffItemView): void;
  (e: 'navigate', index: number): void;
  (e: 'back'): void;
  (e: 'revealInFinder', path: string): void;
  (e: 'update:viewMode', mode: ViewMode): void;
  (e: 'update:colorTheme', theme: ColorTheme): void;
}>();

const compareWithCurrent = ref(false);
const selectedOldSnapshot = ref<SnapshotMeta | null>(null);
const selectedNewSnapshot = ref<SnapshotMeta | null>(null);

// Directly sync active snapshot from Rust backend memory
const rustActiveSnapshot = ref<SnapshotMeta | null>(null);

async function syncRustActiveSnapshot() {
  try {
    const meta = await invoke<SnapshotMeta | null>('get_current_snapshot_meta');
    rustActiveSnapshot.value = meta;
    if (meta && !selectedOldSnapshot.value) {
      compareWithCurrent.value = true;
    }
  } catch (err) {
    console.error('Failed to get current snapshot meta from Rust:', err);
  }
}

onMounted(() => {
  syncRustActiveSnapshot();
});

// Watch for preselected snapshot passed from SnapshotManagerView
watch(
  () => props.preselectedSnapshot,
  (val) => {
    if (val) {
      selectedOldSnapshot.value = val;
      syncRustActiveSnapshot();
      if (rustActiveSnapshot.value && rustActiveSnapshot.value.id === val.id) {
        compareWithCurrent.value = false;
      }
    }
  },
  { immediate: true }
);

// Helper to normalize path (strip trailing slashes)
function normalizePath(p?: string | null): string {
  if (!p) return '';
  return p.trim().replace(/[/\\]+$/, '');
}

// Automatically filter candidate baseline snapshots
const filteredOldSnapshots = computed(() => {
  if (compareWithCurrent.value && rustActiveSnapshot.value) {
    return props.savedSnapshots.filter(
      (s) =>
        s.id !== rustActiveSnapshot.value?.id &&
        normalizePath(s.root_path) === normalizePath(rustActiveSnapshot.value?.root_path)
    );
  }
  return props.savedSnapshots;
});

// Automatically filter candidate target snapshots based on selected baseline
const filteredNewSnapshots = computed(() => {
  if (!selectedOldSnapshot.value) {
    return props.savedSnapshots;
  }
  return props.savedSnapshots.filter(
    (s) =>
      s.id !== selectedOldSnapshot.value?.id &&
      normalizePath(s.root_path) === normalizePath(selectedOldSnapshot.value?.root_path)
  );
});

// Watch for baseline changes to auto-reset incompatible target selection
watch(selectedOldSnapshot, (newVal) => {
  if (selectedNewSnapshot.value) {
    if (
      !newVal ||
      selectedNewSnapshot.value.id === newVal.id ||
      normalizePath(selectedNewSnapshot.value.root_path) !== normalizePath(newVal.root_path)
    ) {
      selectedNewSnapshot.value = null;
    }
  }
});

// Watch mode switch to auto-reset incompatible baseline
watch(compareWithCurrent, (isCurrent) => {
  if (isCurrent && selectedOldSnapshot.value && rustActiveSnapshot.value) {
    if (normalizePath(selectedOldSnapshot.value.root_path) !== normalizePath(rustActiveSnapshot.value.root_path)) {
      selectedOldSnapshot.value = null;
    }
  }
});

// Toggle selection (Clicking selected card deselects / cancels it)
function toggleSelectOld(snap: SnapshotMeta) {
  if (selectedOldSnapshot.value?.id === snap.id) {
    selectedOldSnapshot.value = null; // Unselect / Cancel
  } else {
    selectedOldSnapshot.value = snap;
  }
}

function toggleSelectNew(snap: SnapshotMeta) {
  if (selectedNewSnapshot.value?.id === snap.id) {
    selectedNewSnapshot.value = null; // Unselect / Cancel
  } else {
    selectedNewSnapshot.value = snap;
  }
}

// Validation logic: require both snapshots to exist and share the identical root path
const canStartDiff = computed(() => {
  if (!selectedOldSnapshot.value) return false;

  if (compareWithCurrent.value) {
    if (!rustActiveSnapshot.value) return false;
    if (selectedOldSnapshot.value.id === rustActiveSnapshot.value.id) return false;
    return normalizePath(selectedOldSnapshot.value.root_path) === normalizePath(rustActiveSnapshot.value.root_path);
  } else {
    if (!selectedNewSnapshot.value) return false;
    if (selectedOldSnapshot.value.id === selectedNewSnapshot.value.id) return false;
    return normalizePath(selectedOldSnapshot.value.root_path) === normalizePath(selectedNewSnapshot.value.root_path);
  }
});

function handleStartDiff() {
  if (!canStartDiff.value || !selectedOldSnapshot.value) return;

  if (compareWithCurrent.value) {
    emit('runDiff', selectedOldSnapshot.value, null);
  } else if (selectedNewSnapshot.value) {
    emit('runDiff', selectedOldSnapshot.value, selectedNewSnapshot.value);
  }
}
</script>

<template>
  <div class="diff-page-container">
    <!-- 0. Diff Calculating / Streaming Progress Animation -->
    <div v-if="isDiffing" class="diff-loading-center">
      <div class="diff-computing-card">
        <div class="computing-header">
          <div class="computing-icon-box">
            <GitCompare :size="22" class="computing-spin-icon" />
          </div>
          <div class="computing-title-box">
            <h3>正在进行多线程深度快照对比...</h3>
            <span class="computing-stage">
              {{ diffProgress?.stage || '正在递归比对数百万节点差异与计算涨跌幅...' }}
            </span>
          </div>
        </div>

        <!-- Animated Progress Bar -->
        <div class="diff-progress-track">
          <div
            class="diff-progress-fill"
            :style="{ width: `${diffProgress?.progress_percent || 45}%` }"
          />
        </div>

        <div class="diff-hint-row">
          <span>Rust 差异引擎正在高速遍历比对节点</span>
          <span class="diff-pct-text">{{ diffProgress?.progress_percent || 45 }}%</span>
        </div>
      </div>
    </div>

    <!-- 1. Active Diff Analysis View -->
    <template v-else-if="isDiffMode && diffMeta && currentDirView">
      <!-- Diff Status Header Banner -->
      <header class="diff-header-bar">
        <div class="diff-info-left">
          <GitCompare :size="16" class="diff-icon" />
          <div class="diff-names">
            <span class="old-snap-tag" :title="diffMeta.snapshot_a_name">
              基准: {{ diffMeta.snapshot_a_name }}
            </span>
            <ArrowRight :size="12" class="arrow-icon" />
            <span class="new-snap-tag" :title="diffMeta.snapshot_b_name">
              对比: {{ diffMeta.snapshot_b_name }}
            </span>
          </div>

          <div class="delta-summary-pill">
            <span
              class="delta-size"
              :class="{
                'text-red': diffMeta.delta_total_size > 0,
                'text-green': diffMeta.delta_total_size < 0,
              }"
            >
              {{ formatDelta(diffMeta.delta_total_size) }}
            </span>
            <span class="delta-pct">
              ({{ formatPercent(diffMeta.delta_total_percent) }})
            </span>
          </div>
        </div>

        <div class="diff-controls-right">
          <!-- View mode toggle -->
          <div class="view-mode-toggle">
            <button
              class="toggle-btn"
              :class="{ active: viewMode === 'treemap' }"
              @click="emit('update:viewMode', 'treemap')"
            >
              {{ t('topbar.viewTreemap') }}
            </button>
            <button
              class="toggle-btn"
              :class="{ active: viewMode === 'list' }"
              @click="emit('update:viewMode', 'list')"
            >
              {{ t('topbar.viewList') }}
            </button>
          </div>

          <!-- Color theme switch -->
          <button
            class="btn-secondary btn-sm theme-btn"
            :title="isZh ? '切换红绿配色模式' : 'Toggle Color Theme'"
            @click="emit('update:colorTheme', colorTheme === 'stock_cn' ? 'stock_us' : 'stock_cn')"
          >
            <TrendingUp :size="13" />
            <span>{{ colorTheme === 'stock_cn' ? (isZh ? '红涨绿跌' : 'Red Up') : (isZh ? '绿涨红跌' : 'Green Up') }}</span>
          </button>

          <!-- Exit diff button -->
          <button class="btn-primary btn-sm" @click="emit('exitDiff')">
            <RotateCcw :size="13" />
            <span>{{ isZh ? '退出对比' : 'Exit Diff' }}</span>
          </button>
        </div>
      </header>

      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        v-if="breadcrumbSegments.length > 0"
        :segments="breadcrumbSegments"
        :can-go-back="canGoBack"
        @navigate="emit('navigate', $event)"
        @back="emit('back')"
        @home="emit('exitDiff')"
      />

      <!-- Content Area -->
      <div class="diff-content-wrapper">
        <TreemapView
          v-if="viewMode === 'treemap'"
          :current-node="currentDirView"
          :is-diff-mode="true"
          :color-theme="colorTheme"
          @drill-down="emit('drillDown', $event as any)"
          @reveal-in-finder="emit('revealInFinder', $event)"
        />
        <ListView
          v-else
          :current-node="currentDirView"
          :is-diff-mode="true"
          @drill-down="emit('drillDown', $event as any)"
          @reveal-in-finder="emit('revealInFinder', $event)"
        />
      </div>
    </template>

    <!-- 2. Setup / Card-Based Selection Workbench -->
    <div v-else class="diff-setup-wrapper">
      <!-- Setup Top Toolbar (Matching Snapshot Manager style) -->
      <header class="diff-setup-toolbar">
        <div class="toolbar-left">
          <div class="page-title">
            <GitCompare :size="16" class="title-icon" />
            <h2>{{ t('diff.title') }}</h2>
          </div>
        </div>

        <div class="toolbar-right">
          <!-- Mode Segmented Control -->
          <div class="mode-segmented-control">
            <button
              class="segment-pill"
              :class="{ active: compareWithCurrent }"
              :disabled="!rustActiveSnapshot"
              @click="compareWithCurrent = true"
            >
              <Zap :size="13" />
              <span>{{ isZh ? '基准快照 VS 当前活动扫描' : 'Baseline VS Active Scan' }}</span>
              <span v-if="!rustActiveSnapshot" class="status-tip">({{ isZh ? '无活动扫描' : 'No Active Scan' }})</span>
            </button>

            <button
              class="segment-pill"
              :class="{ active: !compareWithCurrent }"
              @click="compareWithCurrent = false"
            >
              <Layers :size="13" />
              <span>{{ isZh ? '对比两份已存历史快照' : 'Compare Two Saved Snapshots' }}</span>
            </button>
          </div>
        </div>
      </header>

      <div class="setup-scroll-area">
        <div class="setup-container">
          <!-- Selection Columns (Card Grid Layout) -->
        <div class="selection-columns" :class="{ 'two-columns': !compareWithCurrent }">
          <!-- Step 1: Base Snapshot Selection -->
          <div class="selection-column">
            <div class="column-header">
              <div class="column-header-title">
                <span class="step-badge base">1. {{ t('diff.baseline') }}</span>
                <span v-if="compareWithCurrent && rustActiveSnapshot" class="constraint-pill" :title="rustActiveSnapshot.root_path">
                  <Folder :size="10" />
                  <span>{{ rustActiveSnapshot.root_path }}</span>
                </span>
              </div>
              <span class="step-subtitle">
                {{ compareWithCurrent && rustActiveSnapshot ? (isZh ? '仅可选择与活动扫描相同根目录的快照' : 'Only snapshots with matching root directory') : (isZh ? '点击选择作为对比基准的原始快照' : 'Click to select or unselect baseline snapshot') }}
              </span>
            </div>

            <div class="cards-scroll-list">
              <div
                v-for="snap in filteredOldSnapshots"
                :key="`old_${snap.id}`"
                class="diff-select-card"
                :class="{
                  selected: selectedOldSnapshot?.id === snap.id,
                }"
                @click="toggleSelectOld(snap)"
              >
                <div class="card-head-row">
                  <div class="card-title-group">
                    <Layers :size="14" class="card-icon" />
                    <span class="card-name" :title="snap.name">{{ snap.name }}</span>
                  </div>
                  <div class="card-status-slot">
                    <span v-if="selectedOldSnapshot?.id === snap.id" class="status-badge selected-base">
                      <CheckCircle2 :size="11" />
                      <span>{{ isZh ? '已选基准' : 'Selected' }}</span>
                    </span>
                  </div>
                </div>

                <div class="card-path-row" :title="snap.root_path">
                  <Folder :size="11" />
                  <span>{{ snap.root_path }}</span>
                </div>

                <div class="card-stats-row">
                  <span class="stat-pill size">{{ formatBytes(snap.total_size) }}</span>
                  <span class="stat-pill count">{{ formatNumber(snap.total_files) }} {{ t('topbar.files') }}</span>
                  <span class="stat-pill date">
                    <Calendar :size="10" />
                    {{ snap.formatted_time }}
                  </span>
                </div>
              </div>

              <div v-if="filteredOldSnapshots.length === 0" class="empty-column-state">
                <Layers :size="28" class="empty-icon" />
                <p>{{ compareWithCurrent && rustActiveSnapshot ? (isZh ? '暂无与当前活动扫描同一根目录的历史快照' : 'No saved snapshots matching active scan path') : (isZh ? '暂无已保存快照，请先在「磁盘扫描」中扫描并保存快照' : 'No saved snapshots found') }}</p>
              </div>
            </div>
          </div>

          <!-- Step 2: Compare Target Snapshot -->
          <!-- A: Two Saved Snapshots Mode -->
          <div v-if="!compareWithCurrent" class="selection-column">
            <div class="column-header">
              <div class="column-header-title">
                <span class="step-badge compare">2. {{ t('diff.target') }}</span>
                <span v-if="selectedOldSnapshot" class="constraint-pill" :title="selectedOldSnapshot.root_path">
                  <Folder :size="10" />
                  <span>{{ selectedOldSnapshot.root_path }} ({{ filteredNewSnapshots.length }})</span>
                </span>
              </div>
              <span class="step-subtitle">
                {{ selectedOldSnapshot ? (isZh ? '已自动过滤为同一根目录的历史快照' : 'Filtered to matching root directory snapshots') : (isZh ? '点击选择要与基准比对的新快照' : 'Click to select target snapshot') }}
              </span>
            </div>

            <div class="cards-scroll-list">
              <div
                v-for="snap in filteredNewSnapshots"
                :key="`new_${snap.id}`"
                class="diff-select-card"
                :class="{
                  selected: selectedNewSnapshot?.id === snap.id,
                }"
                @click="toggleSelectNew(snap)"
              >
                <div class="card-head-row">
                  <div class="card-title-group">
                    <Layers :size="14" class="card-icon" />
                    <span class="card-name" :title="snap.name">{{ snap.name }}</span>
                  </div>
                  <div class="card-status-slot">
                    <span v-if="selectedNewSnapshot?.id === snap.id" class="status-badge selected-compare">
                      <CheckCircle2 :size="11" />
                      <span>{{ isZh ? '已选对比' : 'Selected' }}</span>
                    </span>
                  </div>
                </div>

                <div class="card-path-row" :title="snap.root_path">
                  <Folder :size="11" />
                  <span>{{ snap.root_path }}</span>
                </div>

                <div class="card-stats-row">
                  <span class="stat-pill size">{{ formatBytes(snap.total_size) }}</span>
                  <span class="stat-pill count">{{ formatNumber(snap.total_files) }} {{ t('topbar.files') }}</span>
                  <span class="stat-pill date">
                    <Calendar :size="10" />
                    {{ snap.formatted_time }}
                  </span>
                </div>
              </div>

              <div v-if="filteredNewSnapshots.length === 0" class="empty-column-state">
                <Layers :size="28" class="empty-icon" />
                <p>{{ selectedOldSnapshot ? (isZh ? '该根目录下暂无其他可供对比的历史快照' : 'No other saved snapshots found for this directory') : (isZh ? '暂无已保存快照' : 'No saved snapshots') }}</p>
              </div>
            </div>
          </div>

          <!-- B: Current Active Scan Preview Card -->
          <div v-else class="selection-column current-active-column">
            <div class="column-header">
              <span class="step-badge compare">2. {{ isZh ? '对比目标 (当前内存活动数据)' : 'Target (Current Scan in Memory)' }}</span>
              <span class="step-subtitle">{{ isZh ? '直接读取 Rust 后端内存中保存的扫描数据' : 'Loaded directly from Rust memory' }}</span>
            </div>

            <div v-if="rustActiveSnapshot" class="active-scan-preview-card">
              <div class="preview-badge-row">
                <span class="live-pulse-badge">
                  <span class="live-dot" />
                  {{ isZh ? 'Rust 后端活跃内存数据' : 'Rust Backend Active Data' }}
                </span>
                <span class="size-highlight">{{ formatBytes(rustActiveSnapshot.total_size) }}</span>
              </div>

              <div class="preview-name">{{ rustActiveSnapshot.name }}</div>
              <div class="preview-path" :title="rustActiveSnapshot.root_path">
                <Folder :size="12" />
                <span>{{ rustActiveSnapshot.root_path }}</span>
              </div>

              <div class="preview-metrics-grid">
                <div class="p-item">
                  <span class="p-label">{{ isZh ? '扫描文件' : 'Files' }}</span>
                  <span class="p-val">{{ formatNumber(rustActiveSnapshot.total_files) }}</span>
                </div>
                <div class="p-item">
                  <span class="p-label">{{ isZh ? '遍历目录' : 'Dirs' }}</span>
                  <span class="p-val">{{ formatNumber(rustActiveSnapshot.total_dirs) }}</span>
                </div>
              </div>
            </div>

            <div v-else class="empty-column-state">
              <Zap :size="28" class="empty-icon" />
              <p>{{ isZh ? 'Rust 引擎当前暂无常驻扫描数据，请先扫描或选择两份历史快照进行对比' : 'No active scan in memory. Please scan first or pick two snapshots.' }}</p>
            </div>
          </div>
        </div>

        <!-- Bottom Footer Action Bar -->
        <div class="setup-bottom-bar">
          <button class="btn-secondary" @click="emit('loadExternalSnapshot')">
            <FolderOpen :size="13" />
            <span>{{ t('snapshots.openExternal') }}</span>
          </button>

          <div class="submit-group">
            <button
              class="btn-primary start-diff-btn"
              :disabled="!canStartDiff"
              @click="handleStartDiff"
            >
              <GitCompare :size="14" />
              <span>{{ t('diff.compareNow') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
</template>

<style scoped>
.diff-page-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-app);
  overflow: hidden;
}

/* Header in active diff */
.diff-header-bar {
  height: 54px;
  max-height: 54px;
  min-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  gap: 10px;
  box-sizing: border-box;
  flex-shrink: 0;
  min-width: 0;
}

/* === Setup Top Toolbar (Same as Snapshot Manager) === */
.diff-setup-toolbar {
  height: 54px;
  max-height: 54px;
  min-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  gap: 16px;
  box-sizing: border-box;
  flex-shrink: 0;
  min-width: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  min-width: 0;
}

.page-title {
  display: flex;
  align-items: center;
  gap: 8px;
  white-space: nowrap;
}

.title-icon {
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.page-title h2 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
  white-space: nowrap;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  white-space: nowrap;
}

.diff-info-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1 1 auto;
  overflow: hidden;
}

.diff-icon {
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.diff-names {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12.5px;
  font-weight: 600;
  min-width: 0;
  flex: 0 1 auto;
  overflow: hidden;
}

.old-snap-tag {
  color: var(--text-secondary);
  max-width: 360px;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
}

.new-snap-tag {
  color: var(--text-primary);
  max-width: 360px;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
}

.arrow-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.delta-summary-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-xs);
  background: rgba(255, 255, 255, 0.05);
  font-family: var(--font-mono);
  font-size: 11.5px;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
}

.delta-size {
  white-space: nowrap;
}

.delta-pct {
  white-space: nowrap;
}

.text-red {
  color: #f87171;
}

.text-green {
  color: #34d399;
}

.diff-controls-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  white-space: nowrap;
}

.view-mode-toggle {
  display: flex;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 2px;
  flex-shrink: 0;
  white-space: nowrap;
}

.toggle-btn {
  padding: 3px 9px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  white-space: nowrap;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.toggle-btn.active {
  background: var(--accent-primary);
  color: #ffffff;
  font-weight: 500;
}

.theme-btn {
  font-size: 11.5px;
  padding: 5px 10px;
  white-space: nowrap;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
  white-space: nowrap;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.diff-content-wrapper {
  flex: 1;
  overflow: hidden;
  position: relative;
  display: flex;
  flex-direction: column;
}

/* === Card-based Setup Layout === */
.diff-setup-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.setup-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  justify-content: center;
}

.setup-container {
  width: 100%;
  max-width: 900px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.mode-segmented-control {
  display: flex;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 3px;
  gap: 3px;
  flex-shrink: 0;
}

.segment-pill {
  padding: 5px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  border-radius: var(--radius-xs);
  background: transparent;
  border: none;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.segment-pill:hover:not(:disabled) {
  color: var(--text-primary);
}

.segment-pill.active {
  background: var(--accent-primary);
  color: #ffffff;
  font-weight: 500;
}

.status-tip {
  font-size: 10px;
  opacity: 0.7;
}

/* Selection Columns */
.selection-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.selection-column {
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 380px;
}

.column-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border-subtle);
}

.column-header-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.constraint-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 180px;
  font-family: var(--font-mono);
  font-size: 10.5px;
  color: var(--accent-cyan);
  background: rgba(14, 165, 233, 0.1);
  border: 1px solid rgba(14, 165, 233, 0.25);
  padding: 1px 7px;
  border-radius: var(--radius-full);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.step-badge {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.step-badge.base {
  color: #38bdf8;
}

.step-badge.compare {
  color: #818cf8;
}

.step-subtitle {
  font-size: 11.5px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cards-scroll-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  max-height: 400px;
  padding-right: 4px;
}

/* Select Card */
.diff-select-card {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-height: 82px;
  box-sizing: border-box;
  cursor: pointer;
  transition: all 0.12s ease;
}

.diff-select-card:hover:not(.disabled) {
  border-color: var(--border-medium);
  background: rgba(255, 255, 255, 0.03);
}

.diff-select-card.selected {
  border-color: #38bdf8;
  background: rgba(14, 165, 233, 0.08);
}

.diff-select-card.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  background: rgba(0, 0, 0, 0.35);
  border-style: dashed;
}

.card-head-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  height: 20px;
}

.card-title-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.card-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.card-name {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-status-slot {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  flex-shrink: 0;
  height: 18px;
}

.status-badge {
  font-size: 10px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: var(--radius-xs);
  display: inline-flex;
  align-items: center;
  gap: 3px;
  white-space: nowrap;
  line-height: 1.2;
}

.status-badge.selected-base {
  background: rgba(14, 165, 233, 0.15);
  color: #38bdf8;
  border: 1px solid rgba(14, 165, 233, 0.3);
}

.status-badge.selected-compare {
  background: rgba(99, 102, 241, 0.15);
  color: #a5b4fc;
  border: 1px solid rgba(99, 102, 241, 0.3);
}

.status-badge.disabled-tag {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.status-badge.mismatch-tag {
  background: rgba(245, 158, 11, 0.12);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.28);
}

.card-path-row {
  display: flex;
  align-items: center;
  gap: 5px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-stats-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.stat-pill {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
}

.stat-pill.size {
  color: #38bdf8;
  font-weight: 600;
}

.stat-pill.date {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  color: var(--text-muted);
  margin-left: auto;
}

/* Active scan preview card */
.active-scan-preview-card {
  padding: 16px;
  background: rgba(14, 165, 233, 0.04);
  border: 1px solid rgba(14, 165, 233, 0.25);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.preview-badge-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.live-pulse-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  font-weight: 600;
  color: #38bdf8;
  background: rgba(14, 165, 233, 0.12);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #38bdf8;
  box-shadow: 0 0 6px #38bdf8;
}

.size-highlight {
  font-family: var(--font-mono);
  font-size: 14px;
  font-weight: 700;
  color: #38bdf8;
}

.preview-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.preview-path {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-mono);
  font-size: 11.5px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preview-metrics-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 4px;
}

.p-item {
  padding: 8px 10px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: var(--radius-xs);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.p-label {
  font-size: 10.5px;
  color: var(--text-muted);
}

.p-val {
  font-family: var(--font-mono);
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

/* Empty column */
.empty-column-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
  gap: 8px;
}

.empty-icon {
  opacity: 0.3;
}

/* Bottom Bar */
.setup-bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  gap: 12px;
}

.submit-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.warning-text {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11.5px;
  color: #f87171;
}

.start-diff-btn {
  padding: 7px 16px;
  font-size: 13px;
  font-weight: 500;
}

/* Diff Loading State */
.diff-loading-center {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  padding: 24px;
}

.diff-computing-card {
  padding: 24px 28px;
  width: 100%;
  max-width: 480px;
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.4);
  background: var(--bg-card);
  border: 1px solid var(--border-medium);
}

.computing-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.computing-icon-box {
  width: 42px;
  height: 42px;
  border-radius: var(--radius-md);
  background: rgba(14, 165, 233, 0.12);
  color: #38bdf8;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid rgba(14, 165, 233, 0.25);
}

.computing-spin-icon {
  animation: spin 1.8s linear infinite;
}

.computing-title-box {
  display: flex;
  flex-direction: column;
  gap: 3px;
  overflow: hidden;
}

.computing-title-box h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.computing-stage {
  font-size: 12px;
  color: #38bdf8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.diff-progress-track {
  width: 100%;
  height: 6px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.diff-progress-fill {
  height: 100%;
  border-radius: var(--radius-full);
  background: linear-gradient(90deg, #38bdf8 0%, #6366f1 50%, #38bdf8 100%);
  background-size: 200% 100%;
  animation: progressPulse 1.4s ease infinite;
  transition: width 0.2s ease;
}

.diff-hint-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11.5px;
  color: var(--text-muted);
}

.diff-pct-text {
  font-family: var(--font-mono);
  font-weight: 600;
  color: #38bdf8;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes progressPulse {
  0% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
  100% {
    background-position: 0% 50%;
  }
}
</style>
