<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { X, Save, HardDrive, FileText, AlertCircle, Loader2 } from 'lucide-vue-next';
import { formatBytes, formatNumber } from '../composables/useFormat';
import type { SnapshotMeta } from '../types';

const props = defineProps<{
  visible: boolean;
  currentSnapshotMeta: SnapshotMeta | null;
  isSaving?: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', name: string): void;
}>();

const snapshotName = ref('');
const touched = ref(false);

watch(
  () => props.visible,
  (val) => {
    if (val && props.currentSnapshotMeta) {
      snapshotName.value = `${props.currentSnapshotMeta.name}_快照`;
      touched.value = false;
    }
  },
  { immediate: true }
);

const isNameValid = computed(() => snapshotName.value.trim().length > 0);

function handleSave() {
  touched.value = true;
  if (!isNameValid.value || props.isSaving) return;
  emit('save', snapshotName.value.trim());
}
</script>

<template>
  <div v-if="visible" class="modal-backdrop" @click.self="!isSaving && emit('close')">
    <div class="modal-window glass-panel">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title-row">
          <Save :size="18" class="header-icon" />
          <h3>保存当前磁盘快照</h3>
        </div>
        <button class="close-btn" :disabled="isSaving" @click="emit('close')">
          <X :size="16" />
        </button>
      </div>

      <!-- Body -->
      <div v-if="currentSnapshotMeta" class="modal-body">
        <!-- Target Info Preview Card -->
        <div class="snapshot-preview-card">
          <div class="preview-title">{{ currentSnapshotMeta.name }}</div>
          <div class="preview-path" :title="currentSnapshotMeta.root_path">
            {{ currentSnapshotMeta.root_path }}
          </div>
          <div class="preview-grid">
            <div class="p-item">
              <span class="p-label">
                <HardDrive :size="13" />
                扫描总容量:
              </span>
              <span class="p-val highlight">{{ formatBytes(currentSnapshotMeta.total_size) }}</span>
            </div>
            <div class="p-item">
              <span class="p-label">
                <FileText :size="13" />
                文件总数:
              </span>
              <span class="p-val">{{ formatNumber(currentSnapshotMeta.total_files) }}</span>
            </div>
          </div>
        </div>

        <!-- Name Input (Required) -->
        <div class="form-group">
          <label class="form-label">
            <span>快照名称 <span class="required-star">*</span></span>
            <span v-if="touched && !isNameValid" class="error-hint">
              <AlertCircle :size="12" />
              快照名称不能为空
            </span>
          </label>
          <input
            v-model="snapshotName"
            type="text"
            class="form-input"
            :class="{ 'input-error': touched && !isNameValid }"
            :disabled="isSaving"
            placeholder="请输入快照名称，如: 2026年8月项目备份"
            autofocus
            @blur="touched = true"
            @keydown.enter="handleSave"
          />
        </div>

        <!-- Action Buttons -->
        <div class="actions-row">
          <div v-if="isSaving" class="saving-hint-row">
            <Loader2 :size="12" class="btn-spin" />
            <span>Zstd 压缩写入中...</span>
          </div>

          <button class="btn-secondary" :disabled="isSaving" @click="emit('close')">
            取消
          </button>
          <button
            class="btn-primary save-action-btn"
            :disabled="!isNameValid || isSaving"
            @click="handleSave"
          >
            <Loader2 v-if="isSaving" :size="14" class="btn-spin" />
            <Save v-else :size="14" />
            <span>{{ isSaving ? '正在高速压缩保存...' : '立即保存快照 (.snap)' }}</span>
          </button>
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
  max-width: 500px;
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-lg);
  background: var(--bg-card);
  border: 1px solid var(--border-medium);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-sidebar);
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  color: #38bdf8;
}

.header-title-row h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: var(--radius-xs);
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover:not(:disabled) {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}

.close-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.modal-body {
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.snapshot-preview-card {
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.preview-path {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preview-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-top: 4px;
  padding-top: 8px;
  border-top: 1px solid var(--border-subtle);
}

.p-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.p-label {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  color: var(--text-muted);
}

.p-val {
  font-family: var(--font-mono);
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.p-val.highlight {
  color: #38bdf8;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12.5px;
  color: var(--text-secondary);
  font-weight: 500;
}

.required-star {
  color: #f87171;
}

.error-hint {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  color: #f87171;
  font-size: 11px;
  font-weight: 500;
}

.form-input {
  padding: 9px 12px;
  background: var(--bg-card);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  transition: all 0.15s ease;
}

.form-input:focus {
  border-color: #38bdf8;
  outline: none;
  box-shadow: 0 0 0 2px rgba(56, 189, 248, 0.2);
}

.form-input.input-error {
  border-color: #f87171;
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.2);
}

.form-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.actions-row {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
}

.saving-hint-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: #38bdf8;
  margin-right: auto;
}

.save-action-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
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
