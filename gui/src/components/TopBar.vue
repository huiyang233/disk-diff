<script setup lang="ts">
import {
  FolderSearch,
  Play,
  Square,
  LayoutGrid,
  ListTree,
  Save,
} from 'lucide-vue-next';
import { formatBytes, formatNumber } from '../composables/useFormat';
import { useI18n } from '../composables/useI18n';
import type { ScanProgress, ViewMode } from '../types';

const { t, isZh } = useI18n();

defineProps<{
  selectedPath: string;
  isScanning: boolean;
  scanProgress: ScanProgress | null;
  hasScanData: boolean;
  viewMode: ViewMode;
}>();

const emit = defineEmits<{
  (e: 'pickDirectory'): void;
  (e: 'startScan'): void;
  (e: 'cancelScan'): void;
  (e: 'saveSnapshot'): void;
  (e: 'update:viewMode', mode: ViewMode): void;
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
          :placeholder="t('topbar.selectFolder')"
          class="path-input"
          readonly
          @click="emit('pickDirectory')"
        />
        <button
          class="btn-secondary pick-btn"
          :disabled="isScanning"
          :title="t('topbar.browse')"
          @click="emit('pickDirectory')"
        >
          <FolderSearch :size="14" />
          <span>{{ t('topbar.browse') }}</span>
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
        <span>{{ t('topbar.startScan') }}</span>
      </button>
      <button
        v-else
        class="btn-danger cancel-btn"
        @click="emit('cancelScan')"
      >
        <Square :size="13" />
        <span>{{ t('topbar.cancelScan') }}</span>
      </button>
    </div>

    <!-- Center: Live Progress Bar (when scanning) -->
    <div v-if="isScanning && scanProgress" class="center-progress">
      <div class="progress-info">
        <span class="progress-files">{{ isZh ? '已扫描' : 'Scanned' }}: {{ formatNumber(scanProgress.scanned_files) }} {{ t('topbar.files') }}</span>
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
        v-if="hasScanData"
        class="btn-secondary save-btn"
        :title="t('topbar.saveSnapshot')"
        @click="emit('saveSnapshot')"
      >
        <Save :size="14" />
        <span>{{ t('topbar.saveSnapshot') }}</span>
      </button>

      <!-- View Switcher -->
      <div v-if="hasScanData" class="view-switch-group">
        <button
          class="view-btn"
          :class="{ active: viewMode === 'treemap' }"
          :title="t('topbar.viewTreemap')"
          @click="emit('update:viewMode', 'treemap')"
        >
          <LayoutGrid :size="14" />
          <span>{{ t('topbar.viewTreemap') }}</span>
        </button>
        <button
          class="view-btn"
          :class="{ active: viewMode === 'list' }"
          :title="t('topbar.viewList')"
          @click="emit('update:viewMode', 'list')"
        >
          <ListTree :size="14" />
          <span>{{ t('topbar.viewList') }}</span>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar-container {
  height: 54px;
  max-height: 54px;
  min-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  gap: 12px;
  z-index: 10;
  box-sizing: border-box;
  flex-shrink: 0;
  min-width: 0;
}

.left-section {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1 1 auto;
}

.path-picker-group {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1 1 auto;
  max-width: 380px;
}

.path-input {
  width: 100%;
  min-width: 100px;
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
  white-space: nowrap;
  flex-shrink: 0;
}

.center-progress {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 340px;
  min-width: 0;
}

.progress-info {
  display: flex;
  gap: 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent-cyan);
  white-space: nowrap;
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
  gap: 8px;
  flex-shrink: 0;
  white-space: nowrap;
}

.save-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  padding: 5px 10px;
  white-space: nowrap;
  flex-shrink: 0;
}

.view-switch-group {
  display: flex;
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 2px;
  flex-shrink: 0;
  white-space: nowrap;
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
  white-space: nowrap;
  flex-shrink: 0;
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
  white-space: nowrap;
  flex-shrink: 0;
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
