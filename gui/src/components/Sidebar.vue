<script setup lang="ts">
import { computed } from 'vue';
import {
  HardDrive,
  GitCompare,
  Layers,
  Settings,
  Info,
} from 'lucide-vue-next';
import type { NavTab } from '../types';
import { useI18n } from '../composables/useI18n';

const { t, isZh } = useI18n();

defineProps<{
  activeTab: NavTab;
  snapshotCount: number;
  isScanning: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:activeTab', tab: NavTab): void;
}>();

const navItems = computed(() => [
  { id: 'scan' as NavTab, label: t('nav.scan'), icon: HardDrive },
  { id: 'diff' as NavTab, label: t('nav.diff'), icon: GitCompare },
  { id: 'snapshots' as NavTab, label: t('nav.snapshots'), icon: Layers },
  { id: 'settings' as NavTab, label: t('nav.settings'), icon: Settings },
  { id: 'about' as NavTab, label: t('nav.about'), icon: Info },
]);
</script>

<template>
  <aside class="sidebar-container">
    <!-- Brand Logo Section -->
    <div class="sidebar-brand">
      <div class="brand-logo-glow">
        <img src="/app-icon.png" alt="DiskDiff" class="brand-img" />
      </div>
      <div class="brand-text">
        <span class="brand-name">DiskDiff</span>
        <span class="brand-version">v0.1.1</span>
      </div>
    </div>

    <!-- Navigation Menu -->
    <nav class="sidebar-nav">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: activeTab === item.id }"
        @click="emit('update:activeTab', item.id)"
      >
        <component :is="item.icon" :size="16" class="nav-icon" />
        <span class="nav-label">{{ item.label }}</span>

        <!-- Badge for snapshot count -->
        <span
          v-if="item.id === 'snapshots' && snapshotCount > 0"
          class="nav-badge"
        >
          {{ snapshotCount }}
        </span>

        <!-- Scanning indicator dot -->
        <span
          v-if="item.id === 'scan' && isScanning"
          class="scanning-pulse-dot"
        />
      </button>
    </nav>

    <!-- Bottom Footer -->
    <div class="sidebar-footer">
      <div class="engine-status">
        <span class="status-dot" />
        <span>Rust {{ isZh ? '引擎运行中' : 'Engine Ready' }}</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar-container {
  width: 190px;
  min-width: 190px;
  height: 100%;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  user-select: none;
  z-index: 20;
}

.sidebar-brand {
  height: 54px;
  max-height: 54px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border-subtle);
  box-sizing: border-box;
}

.brand-logo-glow {
  width: 28px;
  height: 28px;
  border-radius: 7px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  overflow: hidden;
  flex-shrink: 0;
}

.brand-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.brand-text {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.brand-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.brand-version {
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.sidebar-nav {
  flex: 1;
  padding: 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  width: 100%;
  text-align: left;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-primary);
}

.nav-item.active {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
  border-color: rgba(255, 255, 255, 0.06);
  font-weight: 600;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.nav-item.active .nav-icon {
  color: #38bdf8;
}

.nav-icon {
  color: var(--text-muted);
  flex-shrink: 0;
  transition: color 0.15s ease;
}

.nav-label {
  flex: 1;
}

.nav-badge {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-secondary);
  padding: 1px 6px;
  border-radius: var(--radius-full);
}

.nav-item.active .nav-badge {
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
}

.scanning-pulse-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #38bdf8;
  box-shadow: 0 0 6px #38bdf8;
  animation: pulse 1.2s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.85); opacity: 0.6; }
  50% { transform: scale(1.15); opacity: 1; }
  100% { transform: scale(0.85); opacity: 0.6; }
}

.sidebar-footer {
  padding: 12px 14px;
  border-top: 1px solid var(--border-subtle);
}

.engine-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #10b981;
  box-shadow: 0 0 5px rgba(16, 185, 129, 0.6);
}
</style>
