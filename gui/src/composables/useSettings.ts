import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ColorTheme } from '../types';

const STORAGE_DIR_KEY = 'diskdiff_storage_dir';
const COLOR_THEME_KEY = 'diskdiff_color_theme';

const customStorageDir = ref<string>(localStorage.getItem(STORAGE_DIR_KEY) || '');
const defaultStorageDir = ref<string>('');
const colorTheme = ref<ColorTheme>((localStorage.getItem(COLOR_THEME_KEY) as ColorTheme) || 'stock_cn');

export function useSettings() {
  const effectiveStorageDir = computed(() => {
    return customStorageDir.value || defaultStorageDir.value;
  });

  const isCustomStorage = computed(() => {
    return !!customStorageDir.value && customStorageDir.value !== defaultStorageDir.value;
  });

  const initSettings = async () => {
    try {
      const defaultDir = await invoke<string>('get_default_storage_dir');
      if (defaultDir) {
        defaultStorageDir.value = defaultDir;
      }
    } catch (e) {
      console.error('Failed to get default storage directory:', e);
    }
  };

  const setStorageDir = (dir: string) => {
    customStorageDir.value = dir;
    if (dir) {
      localStorage.setItem(STORAGE_DIR_KEY, dir);
    } else {
      localStorage.removeItem(STORAGE_DIR_KEY);
    }
  };

  const resetStorageDir = () => {
    customStorageDir.value = '';
    localStorage.removeItem(STORAGE_DIR_KEY);
  };

  const setColorTheme = (theme: ColorTheme) => {
    colorTheme.value = theme;
    localStorage.setItem(COLOR_THEME_KEY, theme);
  };

  return {
    customStorageDir,
    defaultStorageDir,
    effectiveStorageDir,
    isCustomStorage,
    colorTheme,
    initSettings,
    setStorageDir,
    resetStorageDir,
    setColorTheme,
  };
}
