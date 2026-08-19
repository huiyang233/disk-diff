<script setup lang="ts">
import { computed } from 'vue';
import { Folder, FileText, ArrowUpRight, ArrowDownRight, Minus, Sparkles, Trash2 } from 'lucide-vue-next';
import { formatBytes, formatDelta, formatPercent, formatNumber } from '../composables/useFormat';
import { useI18n } from '../composables/useI18n';
import type { DiffStatus } from '../types';

const { isZh } = useI18n();

const props = defineProps<{
  visible: boolean;
  x: number;
  y: number;
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  oldSize?: number | null;
  deltaSize?: number;
  deltaPercent?: number;
  status?: DiffStatus;
  fileCount?: number;
  dirCount?: number;
  isDiffMode: boolean;
}>();

const positionStyle = computed(() => {
  // Offset tooltip so it doesn't overlap mouse cursor or overflow viewport
  const offsetX = 16;
  const offsetY = 16;
  const clampedX = Math.min(props.x + offsetX, window.innerWidth - 320);
  const clampedY = Math.min(props.y + offsetY, window.innerHeight - 240);

  return {
    left: `${clampedX}px`,
    top: `${clampedY}px`,
  };
});
</script>

<template>
  <div
    v-if="visible"
    class="tooltip-card glass-panel"
    :style="positionStyle"
  >
    <div class="header">
      <component :is="isDir ? Folder : FileText" :size="16" class="type-icon" />
      <span class="title" :title="name">{{ name }}</span>
    </div>

    <div class="path-row" :title="path">
      {{ path }}
    </div>

    <div class="divider" />

    <div class="metrics-grid">
      <!-- Current Size -->
      <div class="metric-item">
        <span class="label">{{ isZh ? '当前容量:' : 'Size:' }}</span>
        <span class="value size-value">{{ formatBytes(size) }}</span>
      </div>

      <!-- Diff specific metrics -->
      <template v-if="isDiffMode">
        <div v-if="oldSize !== undefined && oldSize !== null" class="metric-item">
          <span class="label">{{ isZh ? '原容量:' : 'Old Size:' }}</span>
          <span class="value">{{ formatBytes(oldSize) }}</span>
        </div>

        <div v-if="deltaSize !== undefined" class="metric-item">
          <span class="label">{{ isZh ? '容量变动:' : 'Delta Size:' }}</span>
          <span
            class="value delta-value"
            :class="{
              'text-red': deltaSize > 0,
              'text-green': deltaSize < 0,
              'text-neutral': deltaSize === 0,
            }"
          >
            <ArrowUpRight v-if="deltaSize > 0" :size="13" />
            <ArrowDownRight v-else-if="deltaSize < 0" :size="13" />
            <Minus v-else :size="13" />
            {{ formatDelta(deltaSize) }}
          </span>
        </div>

        <div v-if="deltaPercent !== undefined" class="metric-item">
          <span class="label">{{ isZh ? '变动幅度:' : 'Delta %:' }}</span>
          <span
            class="badge"
            :class="{
              'badge-red': deltaPercent > 0,
              'badge-green': deltaPercent < 0,
              'badge-neutral': deltaPercent === 0,
            }"
          >
            {{ formatPercent(deltaPercent) }}
          </span>
        </div>

        <div v-if="status" class="metric-item">
          <span class="label">{{ isZh ? '状态:' : 'Status:' }}</span>
          <span
            class="status-tag"
            :class="`status-${status}`"
          >
            <Sparkles v-if="status === 'added'" :size="11" />
            <Trash2 v-else-if="status === 'removed'" :size="11" />
            {{
              status === 'added'
                ? (isZh ? '新增' : 'Added')
                : status === 'removed'
                ? (isZh ? '已删除' : 'Removed')
                : status === 'modified'
                ? (isZh ? '容量变更' : 'Modified')
                : (isZh ? '未变化' : 'Unchanged')
            }}
          </span>
        </div>
      </template>

      <!-- Non-diff file count -->
      <template v-else-if="isDir">
        <div v-if="fileCount !== undefined" class="metric-item">
          <span class="label">{{ isZh ? '文件总数:' : 'Files:' }}</span>
          <span class="value">{{ formatNumber(fileCount) }}</span>
        </div>
        <div v-if="dirCount !== undefined" class="metric-item">
          <span class="label">{{ isZh ? '子文件夹:' : 'Subdirectories:' }}</span>
          <span class="value">{{ formatNumber(dirCount) }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.tooltip-card {
  position: fixed;
  z-index: 9999;
  pointer-events: none;
  width: 290px;
  padding: 12px;
  background: var(--bg-tooltip);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  font-size: 12px;
  animation: fadeIn 0.1s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.96); }
  to { opacity: 1; transform: scale(1); }
}

.header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.type-icon {
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.title {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.path-row {
  font-family: var(--font-mono);
  color: var(--text-muted);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 8px;
}

.divider {
  height: 1px;
  background: var(--border-subtle);
  margin-bottom: 8px;
}

.metrics-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.metric-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.label {
  color: var(--text-secondary);
}

.value {
  font-family: var(--font-mono);
  font-weight: 500;
  color: var(--text-primary);
}

.size-value {
  font-weight: 600;
  color: #38bdf8;
}

.delta-value {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-weight: 600;
}

.text-red {
  color: #f87171;
}

.text-green {
  color: #34d399;
}

.text-neutral {
  color: var(--text-muted);
}

.status-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 4px;
}

.status-added {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
  border: 1px solid rgba(239, 68, 68, 0.4);
}

.status-removed {
  background: rgba(107, 114, 128, 0.2);
  color: #9ca3af;
  border: 1px solid rgba(107, 114, 128, 0.4);
}

.status-modified {
  background: rgba(245, 158, 11, 0.2);
  color: #fcd34d;
  border: 1px solid rgba(245, 158, 11, 0.4);
}

.status-unchanged {
  background: rgba(148, 163, 184, 0.1);
  color: #94a3b8;
}
</style>
