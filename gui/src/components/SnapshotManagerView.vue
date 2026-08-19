<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  Layers,
  FolderOpen,
  GitCompare,
  Trash2,
  Search,
  Save,
  Calendar,
  Folder,
  Loader2,
} from 'lucide-vue-next';
import { formatBytes, formatNumber } from '../composables/useFormat';
import { useI18n } from '../composables/useI18n';
import type { SnapshotMeta } from '../types';

const { t } = useI18n();

const props = defineProps<{
  savedSnapshots: SnapshotMeta[];
  currentSnapshotMeta: SnapshotMeta | null;
  loadingSnapshotId?: string | null;
  isLoadingExternal?: boolean;
}>();

const emit = defineEmits<{
  (e: 'openSnapshot', snap: SnapshotMeta): void;
  (e: 'diffWithSnapshot', snap: SnapshotMeta): void;
  (e: 'deleteSnapshot', id: string): void;
  (e: 'saveCurrentSnapshot'): void;
  (e: 'loadExternalSnapshot'): void;
  (e: 'revealInFinder', path: string): void;
}>();

const searchQuery = ref('');

const filteredSnapshots = computed(() => {
  if (!searchQuery.value.trim()) return props.savedSnapshots;
  const q = searchQuery.value.toLowerCase().trim();
  return props.savedSnapshots.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.root_path.toLowerCase().includes(q) ||
      s.formatted_time.toLowerCase().includes(q)
  );
});

function handleDelete(snap: SnapshotMeta) {
  const confirmMsg = t('snapshots.deleteConfirmNamed', { name: snap.name });
  if (confirm(confirmMsg)) {
    emit('deleteSnapshot', snap.id);
  }
}
</script>

<template>
  <div class="manager-container">
    <!-- Header Toolbar -->
    <header class="manager-toolbar">
      <div class="toolbar-left">
        <div class="page-title">
          <Layers :size="16" class="title-icon" />
          <h2>{{ t('snapshots.title') }}</h2>
          <span class="count-pill">{{ t('snapshots.countPill', { count: savedSnapshots.length }) }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <!-- Search -->
        <div class="search-box">
          <Search :size="13" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('snapshots.searchPlaceholder')"
            class="search-input"
          />
        </div>

        <!-- Action Buttons -->
        <button
          v-if="currentSnapshotMeta"
          class="btn-primary btn-sm"
          :title="t('snapshots.saveActiveTooltip')"
          @click="emit('saveCurrentSnapshot')"
        >
          <Save :size="13" />
          <span>{{ t('snapshots.saveActiveBtn') }}</span>
        </button>

        <button
          class="btn-secondary btn-sm"
          :disabled="isLoadingExternal"
          :title="t('snapshots.openExternal')"
          @click="emit('loadExternalSnapshot')"
        >
          <Loader2 v-if="isLoadingExternal" :size="13" class="btn-spin" />
          <FolderOpen v-else :size="13" />
          <span>{{ isLoadingExternal ? t('snapshots.loading') : t('snapshots.openExternal') }}</span>
        </button>
      </div>
    </header>

    <!-- Snapshot Card Grid View -->
    <div class="content-scroll-area">
      <div v-if="filteredSnapshots.length > 0" class="snapshot-grid">
        <div
          v-for="snap in filteredSnapshots"
          :key="snap.id"
          class="snapshot-card"
        >
          <!-- Card Header: Title & Delete -->
          <div class="card-header">
            <div class="card-title-group">
              <div class="card-icon-box">
                <Layers :size="15" />
              </div>
              <div class="card-title-text" :title="snap.name">
                {{ snap.name }}
              </div>
            </div>
            <button
              class="btn-icon-danger"
              :title="t('snapshots.delete')"
              @click="handleDelete(snap)"
            >
              <Trash2 :size="14" />
            </button>
          </div>

          <!-- Root Path -->
          <div class="card-path-box" :title="snap.root_path">
            <Folder :size="12" class="path-icon" />
            <span class="path-text">{{ snap.root_path }}</span>
          </div>

          <!-- 3-Metrics Grid -->
          <div class="card-metrics-grid">
            <div class="metric-item">
              <span class="metric-label">{{ t('snapshots.totalSize') }}</span>
              <span class="metric-val size-val">{{ formatBytes(snap.total_size) }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">{{ t('snapshots.filesDirs') }}</span>
              <span class="metric-val counts-val">
                {{ formatNumber(snap.total_files) }} / {{ formatNumber(snap.total_dirs) }}
              </span>
            </div>
            <div class="metric-item">
              <span class="metric-label">{{ t('snapshots.snapSize') }}</span>
              <span class="metric-val snap-val">
                {{ snap.snap_file_size ? formatBytes(snap.snap_file_size) : '-' }}
              </span>
            </div>
          </div>

          <!-- Card Footer: Time & Action Buttons -->
          <div class="card-footer">
            <div class="date-info">
              <Calendar :size="12" class="date-icon" />
              <span>{{ snap.formatted_time }}</span>
            </div>

            <div class="card-actions">
              <button
                class="btn-primary btn-xs action-btn"
                :disabled="loadingSnapshotId === snap.id"
                :title="t('snapshots.browse')"
                @click="emit('openSnapshot', snap)"
              >
                <Loader2 v-if="loadingSnapshotId === snap.id" :size="12" class="btn-spin" />
                <FolderOpen v-else :size="12" />
                <span>{{ loadingSnapshotId === snap.id ? t('snapshots.loading') : t('snapshots.browse') }}</span>
              </button>
              <button
                class="btn-secondary btn-xs action-btn"
                :title="t('snapshots.diffTooltip')"
                @click="emit('diffWithSnapshot', snap)"
              >
                <GitCompare :size="12" />
                <span>{{ t('snapshots.diffBtn') }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="empty-state">
        <div class="empty-icon-wrap">
          <Layers :size="32" class="empty-icon" />
        </div>
        <h3>{{ t('snapshots.emptyTitle') }}</h3>
        <p class="empty-desc">
          {{ t('snapshots.emptyDesc') }}
        </p>
        <div class="empty-actions">
          <button class="btn-primary" @click="emit('loadExternalSnapshot')">
            <FolderOpen :size="13" />
            <span>{{ t('snapshots.openExternal') }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.manager-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-app);
  overflow: hidden;
}

.manager-toolbar {
  height: 54px;
  max-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  gap: 16px;
  box-sizing: border-box;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.page-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  color: var(--accent-cyan);
}

.page-title h2 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.count-pill {
  font-size: 11px;
  font-weight: 500;
  font-family: var(--font-mono);
  background: rgba(14, 165, 233, 0.12);
  color: #38bdf8;
  padding: 2px 7px;
  border-radius: var(--radius-full);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-box {
  position: relative;
  width: 240px;
}

.search-icon {
  position: absolute;
  left: 9px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
}

.search-input {
  width: 100%;
  padding: 5px 10px 5px 28px;
  font-size: 12px;
}

.btn-sm {
  padding: 5px 11px;
  font-size: 12px;
}

.btn-xs {
  padding: 4px 10px;
  font-size: 11.5px;
}

.content-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

/* Card Grid Layout */
.snapshot-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: 16px;
}

.snapshot-card {
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.snapshot-card:hover {
  border-color: var(--border-medium);
  background: var(--bg-card-hover);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.card-title-group {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
}

.card-icon-box {
  width: 26px;
  height: 26px;
  border-radius: var(--radius-sm);
  background: rgba(14, 165, 233, 0.12);
  color: #38bdf8;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.card-title-text {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.01em;
}

.btn-icon-danger {
  padding: 4px 6px;
  background: transparent;
  color: var(--text-muted);
  border: 1px solid transparent;
  border-radius: var(--radius-xs);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s ease;
}

.btn-icon-danger:hover {
  background: rgba(239, 68, 68, 0.12);
  color: #f87171;
}

.card-path-box {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: rgba(0, 0, 0, 0.25);
  border-radius: var(--radius-sm);
  font-size: 11px;
  overflow: hidden;
}

.path-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.path-text {
  font-family: var(--font-mono);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-metrics-grid {
  display: grid;
  grid-template-columns: 1fr 1.3fr 1fr;
  gap: 8px;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 7px 10px;
  background: rgba(0, 0, 0, 0.15);
  border-radius: var(--radius-xs);
  border: 1px solid rgba(255, 255, 255, 0.03);
  min-width: 0;
}

.metric-label {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
}

.metric-val {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.metric-val.size-val {
  color: #38bdf8;
}

.metric-val.snap-val {
  color: #34d399;
}

.metric-val.counts-val {
  font-size: 11px;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-top: 10px;
  border-top: 1px solid var(--border-subtle);
}

.date-info {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  white-space: nowrap;
}

.date-icon {
  color: var(--text-muted);
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-btn {
  gap: 4px;
  white-space: nowrap;
}

/* Empty state */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon-wrap {
  width: 54px;
  height: 54px;
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 12px;
}

.empty-icon {
  color: var(--text-muted);
}

.empty-desc {
  max-width: 360px;
  font-size: 12.5px;
  color: var(--text-muted);
  line-height: 1.5;
  margin: 6px 0 16px;
}

.btn-spin {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
