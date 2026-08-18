<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  X,
  Save,
  GitCompare,
  FolderOpen,
  HardDrive,
  Calendar,
  FileText,
  Trash2,
  ExternalLink,
  Layers,
} from 'lucide-vue-next';
import { formatBytes, formatNumber } from '../composables/useFormat';
import type { SnapshotMeta } from '../types';

const props = defineProps<{
  visible: boolean;
  currentSnapshotMeta: SnapshotMeta | null;
  savedSnapshots: SnapshotMeta[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saveSnapshot', customName: string): void;
  (e: 'openSnapshot', snapshotMeta: SnapshotMeta): void;
  (e: 'deleteSnapshot', snapshotId: string): void;
  (e: 'diffSnapshots', oldSnapMeta: SnapshotMeta, newSnapMeta: SnapshotMeta | null): void;
  (e: 'loadSnapshotFile'): void;
}>();

const activeTab = ref<'open' | 'save' | 'diff'>('open');
const customSnapshotName = ref('');
const selectedOldSnapshot = ref<SnapshotMeta | null>(null);
const selectedNewSnapshot = ref<SnapshotMeta | null>(null);
const compareWithCurrent = ref(true);

onMounted(() => {
  if (props.currentSnapshotMeta) {
    customSnapshotName.value = `${props.currentSnapshotMeta.name}_快照`;
  }
});

function handleSave() {
  emit('saveSnapshot', customSnapshotName.value.trim() || '未命名快照');
}

function handleOpen(snap: SnapshotMeta) {
  emit('openSnapshot', snap);
}

function handleDelete(snapId: string, event: MouseEvent) {
  event.stopPropagation();
  if (confirm('确定要删除该历史快照吗？此操作不可恢复。')) {
    emit('deleteSnapshot', snapId);
  }
}

function handleStartDiff() {
  if (!selectedOldSnapshot.value) return;

  if (compareWithCurrent.value) {
    emit('diffSnapshots', selectedOldSnapshot.value, null);
  } else if (selectedNewSnapshot.value) {
    emit('diffSnapshots', selectedOldSnapshot.value, selectedNewSnapshot.value);
  }
}
</script>

<template>
  <div v-if="visible" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-window glass-panel">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-title-row">
          <HardDrive :size="18" class="header-icon" />
          <h3>磁盘快照管理与对比</h3>
        </div>
        <button class="close-btn" @click="emit('close')">
          <X :size="16" />
        </button>
      </div>

      <!-- Tab Bar -->
      <div class="modal-tabs">
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'open' }"
          @click="activeTab = 'open'"
        >
          <FolderOpen :size="14" />
          打开/浏览快照 ({{ savedSnapshots.length }})
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'save' }"
          @click="activeTab = 'save'"
        >
          <Save :size="14" />
          保存当前快照
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'diff' }"
          @click="activeTab = 'diff'"
        >
          <GitCompare :size="14" />
          快照对比 (Diff)
        </button>
      </div>

      <!-- Tab 1: Open / Browse Snapshots -->
      <div v-if="activeTab === 'open'" class="modal-body">
        <div class="open-section">
          <div class="section-desc-row">
            <span>选择下方已保存的历史快照直接打开查看，或载入外部 `.snap` 文件：</span>
            <button class="btn-secondary btn-sm" @click="emit('loadSnapshotFile')">
              <ExternalLink :size="13" />
              打开外部 .snap 文件
            </button>
          </div>

          <!-- Snapshot List -->
          <div v-if="savedSnapshots.length > 0" class="snapshots-list">
            <div
              v-for="snap in savedSnapshots"
              :key="snap.id"
              class="snapshot-item-card"
            >
              <div class="item-main">
                <div class="item-header-row">
                  <span class="item-title">{{ snap.name }}</span>
                  <span class="item-time">
                    <Calendar :size="12" />
                    {{ snap.formatted_time }}
                  </span>
                </div>
                <div class="item-path" :title="snap.root_path">{{ snap.root_path }}</div>
                <div class="item-stats">
                  <span class="stat-badge size-badge">
                    <HardDrive :size="11" />
                    {{ formatBytes(snap.total_size) }}
                  </span>
                  <span class="stat-badge count-badge">
                    <FileText :size="11" />
                    {{ formatNumber(snap.total_files) }} 文件
                  </span>
                </div>
              </div>

              <div class="item-actions">
                <button
                  class="btn-primary btn-sm"
                  title="直接打开并浏览此快照"
                  @click="handleOpen(snap)"
                >
                  <FolderOpen :size="13" />
                  打开浏览
                </button>
                <button
                  class="btn-icon-danger"
                  title="删除快照"
                  @click="handleDelete(snap.id, $event)"
                >
                  <Trash2 :size="14" />
                </button>
              </div>
            </div>
          </div>

          <!-- Empty state -->
          <div v-else class="empty-snapshots">
            <Layers :size="36" class="empty-icon" />
            <p>暂无已保存的磁盘快照</p>
            <p class="empty-sub">您可以在扫描目录后，切换到「保存当前快照」标签进行保存。</p>
          </div>
        </div>
      </div>

      <!-- Tab 2: Save Snapshot -->
      <div v-if="activeTab === 'save'" class="modal-body">
        <div v-if="currentSnapshotMeta" class="save-section">
          <div class="snapshot-preview-card">
            <div class="preview-title">{{ currentSnapshotMeta.name }}</div>
            <div class="preview-path">{{ currentSnapshotMeta.root_path }}</div>
            <div class="preview-grid">
              <div class="p-item">
                <span class="p-label">总容量:</span>
                <span class="p-val highlight">{{ formatBytes(currentSnapshotMeta.total_size) }}</span>
              </div>
              <div class="p-item">
                <span class="p-label">文件总数:</span>
                <span class="p-val">{{ formatNumber(currentSnapshotMeta.total_files) }}</span>
              </div>
            </div>
          </div>

          <div class="form-group">
            <label class="form-label">快照备注名称:</label>
            <input
              v-model="customSnapshotName"
              type="text"
              class="form-input"
              placeholder="例如: 2026年8月项目备份"
            />
          </div>

          <div class="actions-row">
            <button class="btn-primary" @click="handleSave">
              <Save :size="14" />
              立即保存快照 (.snap)
            </button>
          </div>
        </div>

        <div v-else class="empty-snapshots">
          <HardDrive :size="36" class="empty-icon" />
          <p>当前没有正在查看的扫描结果</p>
          <p class="empty-sub">请先在主界面扫描目录后再保存快照。</p>
        </div>
      </div>

      <!-- Tab 3: Diff Comparison -->
      <div v-if="activeTab === 'diff'" class="modal-body">
        <div class="diff-section">
          <div class="form-group-checkbox">
            <label class="checkbox-label">
              <input
                v-model="compareWithCurrent"
                type="checkbox"
                class="form-checkbox"
                :disabled="!currentSnapshotMeta"
              />
              <span>使用「当前扫描/浏览数据」与历史快照对比</span>
            </label>
          </div>

          <div class="diff-pickers-grid">
            <!-- Base Snapshot -->
            <div class="picker-col">
              <label class="picker-label">基准快照 (过去 / 旧版本):</label>
              <div class="snapshot-selector-list">
                <div
                  v-for="snap in savedSnapshots"
                  :key="snap.id"
                  class="select-item"
                  :class="{ selected: selectedOldSnapshot?.id === snap.id }"
                  @click="selectedOldSnapshot = snap"
                >
                  <div class="select-item-title">{{ snap.name }}</div>
                  <div class="select-item-meta">
                    <span>{{ snap.formatted_time }}</span>
                    <span class="meta-size">{{ formatBytes(snap.total_size) }}</span>
                  </div>
                </div>

                <div v-if="savedSnapshots.length === 0" class="no-items">
                  无历史快照
                </div>
              </div>
            </div>

            <!-- Target Snapshot -->
            <div v-if="!compareWithCurrent" class="picker-col">
              <label class="picker-label">对比快照 (当前 / 新版本):</label>
              <div class="snapshot-selector-list">
                <div
                  v-for="snap in savedSnapshots"
                  :key="snap.id"
                  class="select-item"
                  :class="{ selected: selectedNewSnapshot?.id === snap.id }"
                  @click="selectedNewSnapshot = snap"
                >
                  <div class="select-item-title">{{ snap.name }}</div>
                  <div class="select-item-meta">
                    <span>{{ snap.formatted_time }}</span>
                    <span class="meta-size">{{ formatBytes(snap.total_size) }}</span>
                  </div>
                </div>

                <div v-if="savedSnapshots.length === 0" class="no-items">
                  无历史快照
                </div>
              </div>
            </div>
          </div>

          <!-- Bottom Actions -->
          <div class="diff-actions-row">
            <button class="btn-secondary" @click="emit('loadSnapshotFile')">
              <FolderOpen :size="14" />
              打开外部 .snap 文件
            </button>
            <button
              class="btn-primary"
              :disabled="!selectedOldSnapshot || (!compareWithCurrent && !selectedNewSnapshot)"
              @click="handleStartDiff"
            >
              <GitCompare :size="14" />
              开始对比分析 (Diff)
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.modal-window {
  width: 100%;
  max-width: 680px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-sidebar);
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  color: var(--accent-cyan);
}

.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  padding: 4px;
  background: transparent;
  color: var(--text-muted);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.modal-tabs {
  display: flex;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  padding: 0 16px;
  gap: 8px;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 14px;
  background: transparent;
  color: var(--text-secondary);
  border: none;
  border-bottom: 2px solid transparent;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  color: var(--text-primary);
}

.tab-btn.active {
  color: var(--accent-cyan);
  border-bottom-color: var(--accent-cyan);
  font-weight: 600;
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

/* Open Section */
.open-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-desc-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
  gap: 8px;
}

.btn-sm {
  padding: 5px 10px;
  font-size: 12px;
}

.snapshots-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 380px;
  overflow-y: auto;
}

.snapshot-item-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  gap: 12px;
  transition: all 0.15s ease;
}

.snapshot-item-card:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: var(--border-medium);
}

.item-main {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.item-header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-primary);
}

.item-time {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
}

.item-path {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--font-mono);
}

.item-stats {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.stat-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-family: var(--font-mono);
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
}

.size-badge {
  color: #38bdf8;
  font-weight: 600;
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-icon-danger {
  padding: 6px 8px;
  background: transparent;
  color: var(--text-muted);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.btn-icon-danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border-color: rgba(239, 68, 68, 0.3);
}

/* Save Section */
.save-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.snapshot-preview-card {
  padding: 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.preview-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.preview-path {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  margin-bottom: 12px;
}

.preview-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.p-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.p-label {
  color: var(--text-secondary);
}

.p-val {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--text-primary);
}

.p-val.highlight {
  color: var(--accent-cyan);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.form-input {
  padding: 8px 12px;
  background: var(--bg-card);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
}

.form-input:focus {
  border-color: var(--accent-primary);
  outline: none;
}

.actions-row {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

/* Diff Section */
.diff-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group-checkbox {
  display: flex;
  align-items: center;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
}

.form-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--accent-primary);
}

.diff-pickers-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.picker-col {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.picker-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}

.snapshot-selector-list {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-card);
  max-height: 200px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.select-item {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-subtle);
  cursor: pointer;
  transition: all 0.1s ease;
}

.select-item:last-child {
  border-bottom: none;
}

.select-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.select-item.selected {
  background: rgba(99, 102, 241, 0.2);
  border-left: 3px solid var(--accent-primary);
}

.select-item-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.select-item-meta {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}

.meta-size {
  font-family: var(--font-mono);
  color: var(--accent-cyan);
}

.no-items {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.diff-actions-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
}

.empty-snapshots {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px 20px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.empty-icon {
  color: var(--text-muted);
  margin-bottom: 4px;
}

.empty-sub {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
