<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import {
  Settings,
  Globe,
  Folder,
  FolderOpen,
  RotateCcw,
  ExternalLink,
  Palette,
  Check,
  Sparkles,
} from 'lucide-vue-next';
import { useI18n, type Locale } from '../composables/useI18n';
import { useSettings } from '../composables/useSettings';
import type { ColorTheme } from '../types';

const { t, locale, setLocale, isZh } = useI18n();
const {
  effectiveStorageDir,
  isCustomStorage,
  colorTheme,
  setStorageDir,
  resetStorageDir,
  setColorTheme,
} = useSettings();

const toastMsg = ref('');
let toastTimer: ReturnType<typeof setTimeout> | null = null;

const showToast = (msg: string) => {
  toastMsg.value = msg;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastMsg.value = '';
  }, 2500);
};

const handleSelectLanguage = (l: Locale) => {
  setLocale(l);
  showToast(l === 'zh' ? t('settings.switchedZh') : t('settings.switchedEn'));
};

const handleChangeStorageDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: isZh.value ? '选择快照默认存储目录' : 'Select Default Snapshot Storage Directory',
    });

    if (selected && typeof selected === 'string') {
      setStorageDir(selected);
      showToast(t('settings.successChangeDir'));
    }
  } catch (e) {
    console.error('Failed to select directory:', e);
  }
};

const handleResetStorageDir = () => {
  resetStorageDir();
  showToast(t('settings.successResetDir'));
};

const handleRevealFolder = async () => {
  if (!effectiveStorageDir.value) return;
  try {
    await invoke('reveal_in_finder', { path: effectiveStorageDir.value });
  } catch (e) {
    console.error('Failed to reveal storage dir:', e);
  }
};

const handleSelectTheme = (theme: ColorTheme) => {
  setColorTheme(theme);
  showToast(theme === 'stock_cn' ? t('settings.appliedStockCn') : t('settings.appliedStockUs'));
};
</script>

<template>
  <div class="settings-container">
    <!-- Top Header Toolbar (Matching Snapshot Manager style) -->
    <header class="settings-toolbar">
      <div class="toolbar-left">
        <div class="page-title">
          <Settings :size="16" class="title-icon" />
          <h2>{{ t('settings.title') }}</h2>
        </div>
      </div>
    </header>

    <!-- Toast Notification -->
    <transition name="toast-fade">
      <div v-if="toastMsg" class="floating-toast">
        <Sparkles :size="14" />
        <span>{{ toastMsg }}</span>
      </div>
    </transition>

    <!-- Scrollable Settings Body -->
    <div class="settings-scroll-area">
      <div class="settings-content">
        <!-- Settings Cards Grid -->
        <div class="settings-grid">
          <!-- 1. Language Setting Card -->
          <div class="setting-card glass-panel">
            <div class="card-header">
              <div class="card-title-group">
                <Globe :size="16" class="section-icon" />
                <h3>{{ t('settings.language') }}</h3>
              </div>
              <span class="card-hint">{{ t('settings.languageDesc') }}</span>
            </div>

            <div class="options-row">
              <!-- Chinese -->
              <button
                class="option-pill"
                :class="{ active: locale === 'zh' }"
                @click="handleSelectLanguage('zh')"
              >
                <div class="pill-info">
                  <span class="flag-icon">🇨🇳</span>
                  <span class="pill-title">简体中文</span>
                </div>
                <Check v-if="locale === 'zh'" :size="14" class="check-icon" />
              </button>

              <!-- English -->
              <button
                class="option-pill"
                :class="{ active: locale === 'en' }"
                @click="handleSelectLanguage('en')"
              >
                <div class="pill-info">
                  <span class="flag-icon">🇺🇸</span>
                  <span class="pill-title">English</span>
                </div>
                <Check v-if="locale === 'en'" :size="14" class="check-icon" />
              </button>
            </div>
          </div>

          <!-- 2. Snapshot Storage Location Card -->
          <div class="setting-card glass-panel">
            <div class="card-header">
              <div class="card-title-group">
                <Folder :size="16" class="section-icon" />
                <h3>{{ t('settings.storage') }}</h3>
                <span
                  class="tag-badge"
                  :class="isCustomStorage ? 'tag-custom' : 'tag-default'"
                >
                  {{ isCustomStorage ? t('settings.customPath') : t('settings.defaultPath') }}
                </span>
              </div>
              <span class="card-hint">{{ t('settings.storageDesc') }}</span>
            </div>

            <div class="path-display-box">
              <Folder :size="14" class="path-icon" />
              <span class="path-string" :title="effectiveStorageDir">{{ effectiveStorageDir || '...' }}</span>
            </div>

            <div class="action-buttons-row">
              <button class="btn-primary btn-sm" @click="handleChangeStorageDir">
                <FolderOpen :size="13" />
                <span>{{ t('settings.changeDir') }}</span>
              </button>

              <button
                v-if="isCustomStorage"
                class="btn-secondary btn-sm"
                @click="handleResetStorageDir"
              >
                <RotateCcw :size="13" />
                <span>{{ t('settings.resetDefault') }}</span>
              </button>

              <button class="btn-secondary btn-sm" @click="handleRevealFolder">
                <ExternalLink :size="13" />
                <span>{{ t('settings.openInFinder') }}</span>
              </button>
            </div>
          </div>

          <!-- 3. Color Theme Card -->
          <div class="setting-card glass-panel">
            <div class="card-header">
              <div class="card-title-group">
                <Palette :size="16" class="section-icon" />
                <h3>{{ t('settings.colorTheme') }}</h3>
              </div>
              <span class="card-hint">{{ t('settings.themeDesc') }}</span>
            </div>

            <div class="options-row">
              <!-- CN Stock Theme (Red Up, Green Down) -->
              <button
                class="theme-card-option"
                :class="{ active: colorTheme === 'stock_cn' }"
                @click="handleSelectTheme('stock_cn')"
              >
                <div class="theme-card-head">
                  <span class="theme-name">{{ t('settings.themeStockCn') }}</span>
                  <Check v-if="colorTheme === 'stock_cn'" :size="14" class="check-icon" />
                </div>
                <div class="theme-legend-demo">
                  <span class="legend-chip chip-red">{{ t('settings.gainOccupied') }}</span>
                  <span class="legend-chip chip-green">{{ t('settings.lossFreed') }}</span>
                </div>
              </button>

              <!-- US Stock Theme (Green Up, Red Down) -->
              <button
                class="theme-card-option"
                :class="{ active: colorTheme === 'stock_us' }"
                @click="handleSelectTheme('stock_us')"
              >
                <div class="theme-card-head">
                  <span class="theme-name">{{ t('settings.themeStockUs') }}</span>
                  <Check v-if="colorTheme === 'stock_us'" :size="14" class="check-icon" />
                </div>
                <div class="theme-legend-demo">
                  <span class="legend-chip chip-green">{{ t('settings.gainOccupied') }}</span>
                  <span class="legend-chip chip-red">{{ t('settings.lossFreed') }}</span>
                </div>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-app);
  overflow: hidden;
}

/* Header Toolbar (Matching Snapshot Manager) */
.settings-toolbar {
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
  flex-shrink: 0;
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

/* Scroll Area */
.settings-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  justify-content: center;
}

.settings-content {
  width: 100%;
  max-width: 720px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Settings Cards Grid */
.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.setting-card {
  padding: 18px 20px;
  border-radius: var(--radius-lg);
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  gap: 14px;
  transition: border-color 0.2s ease;
}

.setting-card:hover {
  border-color: var(--border-medium);
}

.card-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.card-title-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-icon {
  color: var(--accent-cyan);
}

.card-title-group h3 {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.card-hint {
  font-size: 11.5px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.tag-badge {
  font-size: 10.5px;
  font-weight: 500;
  padding: 1px 6px;
  border-radius: var(--radius-xs);
  margin-left: auto;
}

.tag-default {
  background: rgba(100, 116, 139, 0.15);
  color: var(--text-muted);
  border: 1px solid rgba(100, 116, 139, 0.25);
}

.tag-custom {
  background: rgba(14, 165, 233, 0.15);
  color: var(--accent-cyan);
  border: 1px solid rgba(14, 165, 233, 0.3);
}

/* Option Pills (Language) */
.options-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.option-pill {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.option-pill:hover {
  background: var(--bg-panel-hover);
  border-color: var(--border-medium);
  color: var(--text-primary);
}

.option-pill.active {
  background: rgba(14, 165, 233, 0.1);
  border-color: var(--accent-cyan);
  color: var(--text-primary);
  font-weight: 600;
}

.pill-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.flag-icon {
  font-size: 16px;
}

.pill-title {
  font-size: 12.5px;
}

.pill-sub {
  font-size: 11px;
  color: var(--text-muted);
}

.check-icon {
  color: var(--accent-cyan);
}

/* Path Display Box */
.path-display-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 11.5px;
  color: var(--text-primary);
}

.path-icon {
  color: var(--accent-cyan);
  flex-shrink: 0;
}

.path-string {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  user-select: text;
}

.action-buttons-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-sm {
  padding: 5px 12px;
  font-size: 12px;
}

/* Theme Option Cards */
.theme-card-option {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: var(--bg-panel);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: left;
}

.theme-card-option:hover {
  background: var(--bg-panel-hover);
  border-color: var(--border-medium);
}

.theme-card-option.active {
  background: rgba(14, 165, 233, 0.1);
  border-color: var(--accent-cyan);
}

.theme-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.theme-name {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.theme-legend-demo {
  display: flex;
  align-items: center;
  gap: 6px;
}

.legend-chip {
  font-size: 10.5px;
  padding: 2px 7px;
  border-radius: var(--radius-xs);
  font-weight: 500;
}

.chip-red {
  background: var(--stock-red-bg);
  color: var(--stock-red);
  border: 1px solid rgba(239, 68, 68, 0.25);
}

.chip-green {
  background: var(--stock-green-bg);
  color: var(--stock-green);
  border: 1px solid rgba(16, 185, 129, 0.25);
}

/* Floating Toast */
.floating-toast {
  position: fixed;
  top: 20px;
  right: 24px;
  z-index: 9999;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--accent-emerald);
  color: #0b0e14;
  font-size: 12px;
  font-weight: 600;
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
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
