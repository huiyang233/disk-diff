<script setup lang="ts">
import { ChevronRight, Home, ArrowLeft } from 'lucide-vue-next';
import { useI18n } from '../composables/useI18n';

const { t } = useI18n();

interface PathSegment {
  name: string;
  fullPath: string;
}

defineProps<{
  segments: PathSegment[];
  canGoBack: boolean;
}>();

const emit = defineEmits<{
  (e: 'navigate', index: number): void;
  (e: 'back'): void;
  (e: 'home'): void;
}>();
</script>

<template>
  <div class="breadcrumb-container">
    <!-- Left Navigation Controls -->
    <div class="left-nav">
      <button
        class="home-btn"
        :title="t('breadcrumb.homeTitle')"
        @click="emit('home')"
      >
        <Home :size="13" />
        <span>{{ t('breadcrumb.home') }}</span>
      </button>

      <button
        v-if="canGoBack"
        class="back-btn"
        :title="t('breadcrumb.upTitle')"
        @click="emit('back')"
      >
        <ArrowLeft :size="13" />
        <span>{{ t('breadcrumb.up') }}</span>
      </button>

      <div class="divider" />

      <!-- Segment path trail -->
      <div class="segments-wrapper">
        <div
          v-for="(seg, idx) in segments"
          :key="seg.fullPath"
          class="segment-item"
        >
          <span
            class="segment-text"
            :class="{ active: idx === segments.length - 1 }"
            @click="emit('navigate', idx)"
          >
            {{ seg.name || '/' }}
          </span>
          <ChevronRight
            v-if="idx < segments.length - 1"
            :size="12"
            class="divider-icon"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.breadcrumb-container {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding: 6px 16px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border-subtle);
  overflow-x: auto;
  white-space: nowrap;
  gap: 12px;
}

.left-nav {
  display: flex;
  align-items: center;
  gap: 8px;
}

.home-btn,
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.home-btn:hover,
.back-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
  border-color: var(--border-medium);
}

.home-btn {
  color: var(--accent-cyan);
}

.divider {
  width: 1px;
  height: 14px;
  background: var(--border-medium);
  margin: 0 2px;
}

.segments-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
}

.segment-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.segment-text {
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.segment-text:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.06);
}

.segment-text.active {
  color: var(--accent-cyan);
  font-weight: 600;
  cursor: default;
}

.segment-text.active:hover {
  background: transparent;
}

.divider-icon {
  color: var(--text-muted);
}
</style>
