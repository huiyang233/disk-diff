<script setup lang="ts">
import {
  FolderSearch,
  Play,
  Square,
  LayoutGrid,
  ListTree,
  Save,
  TrendingUp,
} from 'lucide-vue-next';
import { formatBytes, formatNumber } from '../composables/useFormat';
import type { ColorTheme, DiffResultMeta, ScanProgress, ViewMode } from '../types';

defineProps<{
  selectedPath: string;
  isScanning: boolean;
  scanProgress: ScanProgress | null;
  hasScanData: boolean;
  viewMode: ViewMode;
  isDiffMode: boolean;
  diffResult: DiffResultMeta | null;
  colorTheme: ColorTheme;
}>();

const emit = defineEmits<{
  (e: 'pickDirectory'): void;
  (e: 'startScan'): void;
  (e: 'cancelScan'): void;
  (e: 'saveSnapshot'): void;
  (e: 'update:viewMode', mode: ViewMode): void;
  (e: 'update:colorTheme', theme: ColorTheme): void;
  (e: 'exitDiffMode'): void;
}>();
</script>

<template>
  <header class="topbar-container glass-panel">
    <!-- Left: Path Picker & Scan Trigger -->
    <div class="left-section">
      <div class="path-picker-group">
        <input
          :value="selectedPath"
          type="text"
          placeholder="请选择要扫描分析的目录路径..."
          class="path-input"
          readonly
          @click="emit('pickDirectory')"
        />
        <button
          class="btn-secondary pick-btn"
          :disabled="isScanning"
          title="选择本地文件夹"
          @click="emit('pickDirectory')"
        >
          <FolderSearch :size="14" />
          <span>选择目录</span>
        </button>
      </div>

      <!-- Action Button: Start / Cancel Scan -->
      <button
        v-if="!isScanning"
        class="btn-primary scan-btn"
        :disabled="!selectedPath"
        @click="emit('startScan')"
      >
        <Play :size="14" />
        <span>开始扫描</span>
      </button>
      <button
        v-else
        class="btn-danger cancel-btn"
        @click="emit('cancelScan')"
      >
        <Square :size="13" />
        <span>取消扫描</span>
      </button>
    </div>

    <!-- Center: Live Progress Bar (when scanning) -->
    <div v-if="isScanning && scanProgress" class="center-progress">
      <div class="progress-info">
        <span class="progress-files">已扫描: {{ formatNumber(scanProgress.scanned_files) }} 文件</span>
        <span class="progress-size">{{ formatBytes(scanProgress.total_size) }}</span>
      </div>
      <div class="progress-path" :title="scanProgress.current_path">
        {{ scanProgress.current_path }}
      </div>
    </div>

    <!-- Right: Save Snapshot & View Mode Switch -->
    <div class="right-section">
      <!-- Save Snapshot Button (Enabled when scan data is loaded) -->
      <button
        v-if="hasScanData && !isDiffMode"
        class="btn-secondary save-btn"
        title="将当前扫描结果保存为本地快照文件"
        @click="emit('saveSnapshot')"
      >
        <Save :size="14" />
        <span>保存快照</span>
      </button>

      <!-- View Switcher -->
      <div v-if="hasScanData" class="view-switch-group">
        <button
          class="view-btn"
          :class="{ active: viewMode === 'treemap' }"
          title="股市风格矩形树图 (Treemap)"
          @click="emit('update:viewMode', 'treemap')"
        >
          <LayoutGrid :size="14" />
          <span>热力图</span>
        </button>
        <button
          class="view-btn"
          :class="{ active: viewMode === 'list' }"
          title="树状层级列表 (List Table)"
          @click="emit('update:viewMode', 'list')"
        >
          <ListTree :size="14" />
          <span>列表</span>
        </button>
      </div>

      <!-- Color Theme Toggle (for Diff Mode) -->
      <button
        v-if="isDiffMode"
        class="theme-toggle-btn"
        :title="colorTheme === 'stock_cn' ? '当前配色: 红涨绿跌 (点击切换为绿涨红跌)' : '当前配色: 绿涨红跌 (点击切换为红涨绿跌)'"
        @click="emit('update:colorTheme', colorTheme === 'stock_cn' ? 'stock_us' : 'stock_cn')"
      >
        <TrendingUp
          :size="13"
          :class="colorTheme === 'stock_cn' ? 'theme-icon-red' : 'theme-icon-green'"
        />
        <span>{{ colorTheme === 'stock_cn' ? '红涨绿跌' : '绿涨红跌' }}</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.topbar-container {
  height: 54px;
  max-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  gap: 16px;
  z-index: 10;
  box-sizing: border-box;
}

.left-section {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.path-picker-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.path-input {
  width: 280px;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 12px;
}

.scan-btn, .cancel-btn, .pick-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  padding: 6px 12px;
}

.center-progress {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 340px;
}

.progress-info {
  display: flex;
  gap: 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-cyan);
}

.progress-path {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 320px;
}

.right-section {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.save-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  padding: 5px 10px;
}

.view-switch-group {
  display: flex;
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 2px;
}

.view-btn {
  background: transparent;
  color: var(--text-muted);
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.view-btn:hover {
  color: var(--text-primary);
}

.view-btn.active {
  background: var(--accent-primary);
  color: #ffffff;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
}

.theme-toggle-btn {
  padding: 4px 8px;
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  font-size: 11px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.theme-toggle-btn:hover {
  border-color: var(--border-medium);
  color: var(--text-primary);
}

.theme-icon-red {
  color: var(--stock-red);
}

.theme-icon-green {
  color: var(--stock-green);
}
</style>
