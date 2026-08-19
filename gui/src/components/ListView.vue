<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import {
  Folder,
  FileText,
  ArrowUpDown,
  ExternalLink,
  Search,
  ArrowRight,
  ChevronDown,
} from 'lucide-vue-next';
import { formatBytes, formatDelta, formatPercent, formatNumber } from '../composables/useFormat';
import { useI18n } from '../composables/useI18n';
import type {
  DiffDirectoryView,
  DiffItemView,
  DiffStatus,
  DirectoryView,
  FileItemView,
} from '../types';

const { t, isZh } = useI18n();

const props = defineProps<{
  currentNode: DirectoryView | DiffDirectoryView | null;
  isDiffMode: boolean;
}>();

const emit = defineEmits<{
  (e: 'drillDown', item: FileItemView | DiffItemView): void;
  (e: 'revealInFinder', path: string): void;
}>();

const searchQuery = ref('');
const sortBy = ref<'size' | 'delta' | 'percent' | 'name'>('size');
const sortAsc = ref(false);
const pageLimit = ref(100);

watch(
  () => props.currentNode,
  () => {
    pageLimit.value = 100;
  }
);

type ItemType = (FileItemView | DiffItemView) & {
  is_dir: boolean;
  name: string;
  path: string;
  size?: number;
  new_size?: number | null;
  old_size?: number | null;
  delta_size?: number;
  delta_percent?: number;
  status?: DiffStatus;
  file_count?: number;
  dir_count?: number;
  has_children?: boolean;
};

// Toggle sorting column
function handleSort(col: 'size' | 'delta' | 'percent' | 'name') {
  if (sortBy.value === col) {
    sortAsc.value = !sortAsc.value;
  } else {
    sortBy.value = col;
    sortAsc.value = false;
  }
}

const parentSize = computed(() => {
  if (!props.currentNode) return 1;
  if (props.isDiffMode) {
    const diff = props.currentNode as DiffDirectoryView;
    return Math.max(diff.new_size ?? diff.old_size ?? 1, 1);
  }
  return Math.max((props.currentNode as DirectoryView).size, 1);
});

const items = computed<ItemType[]>(() => {
  if (!props.currentNode || !props.currentNode.children) return [];

  let list = [...(props.currentNode.children as ItemType[])];

  // Search filter
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    list = list.filter((item) => item.name.toLowerCase().includes(q));
  }

  // Sort
  list.sort((a, b) => {
    let res = 0;
    if (sortBy.value === 'name') {
      res = a.name.localeCompare(b.name);
    } else if (sortBy.value === 'size') {
      const aSize = props.isDiffMode ? (a.new_size ?? a.old_size ?? 0) : (a.size ?? 0);
      const bSize = props.isDiffMode ? (b.new_size ?? b.old_size ?? 0) : (b.size ?? 0);
      res = aSize - bSize;
    } else if (sortBy.value === 'delta') {
      const aDelta = a.delta_size ?? 0;
      const bDelta = b.delta_size ?? 0;
      res = aDelta - bDelta;
    } else if (sortBy.value === 'percent') {
      const aPct = a.delta_percent ?? 0;
      const bPct = b.delta_percent ?? 0;
      res = aPct - bPct;
    }
    return sortAsc.value ? res : -res;
  });

  return list;
});

const displayedItems = computed(() => {
  return items.value.slice(0, pageLimit.value);
});

function loadMore() {
  pageLimit.value += 100;
}

function getItemSize(item: ItemType): number {
  if (props.isDiffMode) {
    return item.new_size ?? item.old_size ?? 0;
  }
  return item.size ?? 0;
}

function getItemRatio(item: ItemType): number {
  const size = getItemSize(item);
  return Math.min(Math.max((size / parentSize.value) * 100, 0), 100);
}
</script>

<template>
  <div class="list-wrapper">
    <!-- Header with Search -->
    <div class="list-toolbar">
      <div class="search-box">
        <Search :size="13" class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="isZh ? '快速筛选当前目录下的文件或文件夹...' : 'Filter files or directories...'"
          class="search-input"
        />
      </div>
      <div class="count-info">
        {{ isZh ? `共 ${items.length} 项` : `${items.length} items` }}
      </div>
    </div>

    <!-- Table Container -->
    <div class="table-container">
      <table class="tree-table">
        <thead>
          <tr>
            <th class="col-name" @click="handleSort('name')">
              <div class="th-content">
                <span>{{ t('list.name') }}</span>
                <ArrowUpDown :size="11" class="sort-icon" />
              </div>
            </th>
            <th class="col-size" @click="handleSort('size')">
              <div class="th-content">
                <span>{{ isZh ? '容量 / 占比' : 'Size / Ratio' }}</span>
                <ArrowUpDown :size="11" class="sort-icon" />
              </div>
            </th>
            <th v-if="isDiffMode" class="col-delta" @click="handleSort('delta')">
              <div class="th-content">
                <span>{{ isZh ? '容量变动' : 'Delta Size' }}</span>
                <ArrowUpDown :size="11" class="sort-icon" />
              </div>
            </th>
            <th v-if="isDiffMode" class="col-percent" @click="handleSort('percent')">
              <div class="th-content">
                <span>{{ isZh ? '涨跌幅' : 'Delta %' }}</span>
                <ArrowUpDown :size="11" class="sort-icon" />
              </div>
            </th>
            <th v-if="!isDiffMode" class="col-counts">{{ t('list.filesCount') }}</th>
            <th class="col-actions">{{ t('list.actions') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in displayedItems"
            :key="item.path"
            class="table-row"
            :class="{ 'is-dir': item.is_dir }"
          >
            <!-- Name -->
            <td class="col-name">
              <div class="name-cell">
                <div class="icon-wrap" :class="{ 'is-dir-wrap': item.is_dir }">
                  <component
                    :is="item.is_dir ? Folder : FileText"
                    :size="15"
                    :class="item.is_dir ? 'icon-folder' : 'icon-file'"
                  />
                </div>
                <div class="name-text-wrap" :title="item.name">
                  <span
                    class="name-text"
                    :class="{ 'clickable-name': item.is_dir }"
                    @click="item.is_dir && emit('drillDown', item)"
                  >
                    {{ item.name }}
                  </span>
                  <span v-if="item.is_dir" class="dir-badge">{{ isZh ? '目录' : 'DIR' }}</span>
                </div>
              </div>
            </td>

            <!-- Size & Ratio bar -->
            <td class="col-size">
              <div class="size-cell">
                <div class="size-bar-bg">
                  <div
                    class="size-bar-fill"
                    :style="{ width: `${getItemRatio(item)}%` }"
                  />
                </div>
                <span class="size-val">{{ formatBytes(getItemSize(item)) }}</span>
              </div>
            </td>

            <!-- Delta Size (Diff Mode) -->
            <td v-if="isDiffMode" class="col-delta">
              <span
                class="delta-val"
                :class="{
                  'text-red': (item.delta_size || 0) > 0,
                  'text-green': (item.delta_size || 0) < 0,
                  'text-muted': (item.delta_size || 0) === 0,
                }"
              >
                {{ formatDelta(item.delta_size || 0) }}
              </span>
            </td>

            <!-- Delta Percent (Diff Mode) -->
            <td v-if="isDiffMode" class="col-percent">
              <span
                class="badge"
                :class="{
                  'badge-red': (item.delta_percent || 0) > 0,
                  'badge-green': (item.delta_percent || 0) < 0,
                  'badge-neutral': (item.delta_percent || 0) === 0,
                }"
              >
                {{ formatPercent(item.delta_percent) }}
              </span>
            </td>

            <!-- File Count (Non-diff) -->
            <td v-if="!isDiffMode" class="col-counts">
              <span v-if="item.is_dir" class="count-text">
                {{ formatNumber(item.file_count) }} {{ t('topbar.files') }}
              </span>
              <span v-else class="count-text">-</span>
            </td>

            <!-- Actions -->
            <td class="col-actions">
              <div class="action-buttons">
                <button
                  v-if="item.is_dir"
                  class="action-btn"
                  :title="isZh ? '进入此文件夹' : 'Drill down'"
                  @click="emit('drillDown', item)"
                >
                  <ArrowRight :size="13" />
                </button>
                <button
                  class="action-btn"
                  :title="t('list.reveal')"
                  @click="emit('revealInFinder', item.path)"
                >
                  <ExternalLink :size="13" />
                </button>
              </div>
            </td>
          </tr>

          <!-- Load more row -->
          <tr v-if="items.length > displayedItems.length" class="load-more-row">
            <td :colspan="isDiffMode ? 5 : 4" class="load-more-cell">
              <button class="btn-secondary load-more-btn" @click="loadMore">
                <ChevronDown :size="13" />
                {{ isZh ? `加载更多 (已显示 ${displayedItems.length} / 共 ${items.length} 项)` : `Load More (${displayedItems.length} of ${items.length})` }}
              </button>
            </td>
          </tr>

          <!-- Empty search result -->
          <tr v-if="items.length === 0">
            <td :colspan="isDiffMode ? 5 : 4" class="empty-cell">
              {{ isZh ? '无匹配文件或文件夹' : 'No matching files or directories' }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.list-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-app);
}

.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-subtle);
  gap: 12px;
}

.search-box {
  position: relative;
  flex: 1;
  max-width: 360px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
}

.search-input {
  width: 100%;
  padding: 5px 10px 5px 30px;
  font-size: 12px;
  border-radius: var(--radius-sm);
}

.count-info {
  font-size: 11.5px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.table-container {
  flex: 1;
  overflow-y: auto;
}

.tree-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12.5px;
  text-align: left;
}

theader, thead th {
  position: sticky;
  top: 0;
  z-index: 5;
  background: #11141a;
  border-bottom: 1px solid var(--border-medium);
  padding: 8px 14px;
  color: var(--text-muted);
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  cursor: pointer;
  user-select: none;
}

thead th:hover {
  color: var(--text-secondary);
}

.th-content {
  display: flex;
  align-items: center;
  gap: 5px;
}

.sort-icon {
  opacity: 0.4;
}

tbody tr {
  border-bottom: 1px solid rgba(255, 255, 255, 0.035);
  transition: background 0.12s ease;
}

tbody tr:hover {
  background: rgba(255, 255, 255, 0.035);
}

tbody tr:hover .action-buttons {
  opacity: 1;
}

td {
  padding: 7px 14px;
  vertical-align: middle;
}

.col-name {
  min-width: 260px;
}

.name-cell {
  display: flex;
  align-items: center;
  gap: 9px;
}

.icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  color: #64748b;
}

.icon-wrap.is-dir-wrap {
  color: #38bdf8;
}

.name-text {
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 380px;
  letter-spacing: -0.01em;
}

.name-link {
  cursor: pointer;
  transition: color 0.12s ease;
}

.name-link:hover {
  color: #38bdf8;
}

.col-size {
  width: 240px;
}

.size-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.size-text {
  font-family: var(--font-mono);
  font-weight: 500;
  font-size: 12px;
  color: var(--text-primary);
  min-width: 76px;
  font-variant-numeric: tabular-nums;
}

.progress-track {
  flex: 1;
  height: 4px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #38bdf8, #3b82f6);
  border-radius: var(--radius-full);
}

.ratio-text {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  min-width: 38px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.col-delta {
  width: 140px;
}

.delta-text {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-family: var(--font-mono);
  font-weight: 500;
  font-size: 11.5px;
}

.text-red {
  color: #f87171;
}

.text-green {
  color: #34d399;
}

.col-percent {
  width: 100px;
}

.col-counts {
  width: 110px;
}

.count-text {
  font-family: var(--font-mono);
  color: var(--text-muted);
  font-size: 11.5px;
  font-variant-numeric: tabular-nums;
}

.col-actions {
  width: 80px;
  text-align: right;
}

.action-buttons {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 3px;
  opacity: 0.6;
  transition: opacity 0.12s ease;
}

.action-btn {
  padding: 3px 5px;
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-secondary);
  border: 1px solid transparent;
  border-radius: var(--radius-xs);
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
  border-color: var(--border-subtle);
}

.load-more-row {
  background: rgba(255, 255, 255, 0.015);
}

.load-more-cell {
  text-align: center;
  padding: 12px;
}

.load-more-btn {
  margin: 0 auto;
  font-size: 11.5px;
  padding: 5px 14px;
}

.empty-cell {
  text-align: center;
  padding: 48px;
  color: var(--text-muted);
  font-size: 12px;
}
</style>
