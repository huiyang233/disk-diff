<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import * as d3 from 'd3-hierarchy';
import { Folder, FileText, ExternalLink, ArrowRight, Layers } from 'lucide-vue-next';
import { formatBytes, formatPercent } from '../composables/useFormat';
import { getDiffColor, getScanColor } from '../composables/useColor';
import { useI18n } from '../composables/useI18n';
import TooltipCard from './TooltipCard.vue';

const { t, isZh } = useI18n();
import type {
  ColorTheme,
  DiffDirectoryView,
  DiffItemView,
  DiffStatus,
  DirectoryView,
  FileItemView,
} from '../types';

interface TreemapItem {
  id: string;
  name: string;
  path: string;
  size: number;
  ratio: number;
  oldSize?: number | null;
  deltaSize?: number;
  deltaPercent?: number;
  status?: DiffStatus;
  isDir: boolean;
  isOtherGroup?: boolean;
  otherCount?: number;
  fileCount?: number;
  dirCount?: number;
  hasChildren?: boolean;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  color: { bg: string; border: string; text: string; badgeBg?: string };
  originalNode: FileItemView | DiffItemView | null;
}

const props = defineProps<{
  currentNode: DirectoryView | DiffDirectoryView | null;
  isDiffMode: boolean;
  colorTheme: ColorTheme;
}>();

const emit = defineEmits<{
  (e: 'drillDown', item: FileItemView | DiffItemView): void;
  (e: 'revealInFinder', path: string): void;
}>();

const containerRef = ref<HTMLDivElement | null>(null);
const containerWidth = ref(800);
const containerHeight = ref(600);

// Hover tooltip state
const tooltip = ref({
  visible: false,
  x: 0,
  y: 0,
  name: '',
  path: '',
  isDir: false,
  size: 0,
  oldSize: null as number | null | undefined,
  deltaSize: undefined as number | undefined,
  deltaPercent: undefined as number | undefined,
  status: undefined as DiffStatus | undefined,
  fileCount: undefined as number | undefined,
  dirCount: undefined as number | undefined,
});

// Context menu state
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  item: null as TreemapItem | null,
});

function updateDimensions() {
  if (containerRef.value) {
    const rect = containerRef.value.getBoundingClientRect();
    containerWidth.value = Math.max(rect.width, 300);
    containerHeight.value = Math.max(rect.height, 300);
  }
}

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateDimensions();
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => updateDimensions());
    resizeObserver.observe(containerRef.value);
  }
  window.addEventListener('click', closeContextMenu);
});

onUnmounted(() => {
  if (resizeObserver) resizeObserver.disconnect();
  window.removeEventListener('click', closeContextMenu);
});

// Maximum prominent items to display individually
const MAX_PROMINENT_CELLS = 25;
// Minimum percentage threshold (items < 0.8% of level total are aggregated into '...')
const MIN_RATIO_PERCENT = 0.8;

// Compute Single-Layer Treemap Layout
const layoutItems = computed<TreemapItem[]>(() => {
  if (!props.currentNode) return [];

  const rawChildren = props.currentNode.children || [];
  if (rawChildren.length === 0) return [];

  const w = containerWidth.value;
  const h = containerHeight.value;

  interface ProcessedChild {
    node: FileItemView | DiffItemView;
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
    hasChildren?: boolean;
  }

  const processedList: ProcessedChild[] = rawChildren.map((c) => {
    const isDir = c.is_dir;
    if (props.isDiffMode) {
      const d = c as DiffItemView;
      const size = d.new_size ?? d.old_size ?? 0;
      return {
        node: d,
        name: d.name,
        path: d.path,
        isDir,
        size,
        oldSize: d.old_size,
        deltaSize: d.delta_size,
        deltaPercent: d.delta_percent,
        status: d.status,
        hasChildren: d.has_children,
      };
    } else {
      const f = c as FileItemView;
      return {
        node: f,
        name: f.name,
        path: f.path,
        isDir,
        size: f.size,
        fileCount: f.file_count,
        dirCount: f.dir_count,
        hasChildren: f.has_children,
      };
    }
  });

  // Sort descending by size
  processedList.sort((a, b) => b.size - a.size);

  const totalCurrentLevelSize = processedList.reduce((sum, item) => sum + item.size, 0) || 1;

  // Classify items into prominent and rest
  const topItems: ProcessedChild[] = [];
  const restItems: ProcessedChild[] = [];

  for (let i = 0; i < processedList.length; i++) {
    const item = processedList[i];
    const ratio = (item.size / totalCurrentLevelSize) * 100;

    if (i < MAX_PROMINENT_CELLS && ratio >= MIN_RATIO_PERCENT) {
      topItems.push(item);
    } else {
      restItems.push(item);
    }
  }

  if (restItems.length === 1 && topItems.length < MAX_PROMINENT_CELLS) {
    topItems.push(restItems.pop()!);
  }

  const displayList: any[] = topItems.map((item, idx) => ({
    ...item,
    ratio: (item.size / totalCurrentLevelSize) * 100,
    index: idx,
    isOtherGroup: false,
  }));

  if (restItems.length > 0) {
    const restTotalSize = restItems.reduce((sum, item) => sum + item.size, 0);
    displayList.push({
      node: null,
      name: `... (其他 ${restItems.length} 项)`,
      path: props.currentNode.path,
      isDir: true,
      size: restTotalSize,
      ratio: (restTotalSize / totalCurrentLevelSize) * 100,
      index: topItems.length,
      isOtherGroup: true,
      otherCount: restItems.length,
    });
  }

  // Flat D3 Hierarchy (Depth = 1 ONLY)
  const d3Root = d3
    .hierarchy({
      children: displayList.map((item) => ({
        ...item,
        computedValue: Math.max(item.size, 1),
      })),
    })
    .sum((d: any) => d.computedValue || 0);

  const treemapLayout = d3
    .treemap<any>()
    .size([w, h])
    .paddingInner(3)
    .paddingOuter(2)
    .round(true)
    .tile(d3.treemapSquarify.ratio(1.2));

  treemapLayout(d3Root);

  const leaves = d3Root.leaves();

  return leaves.map((leaf: any) => {
    const data = leaf.data;
    const isDir = data.isDir;
    let color: { bg: string; border: string; text: string; badgeBg?: string };

    if (data.isOtherGroup) {
      color = {
        bg: 'rgba(30, 41, 59, 0.8)',
        border: 'rgba(100, 116, 139, 0.7)',
        text: '#cbd5e1',
      };
    } else if (props.isDiffMode) {
      color = getDiffColor(
        data.deltaPercent ?? 0,
        data.status ?? 'unchanged',
        props.colorTheme
      );
    } else {
      color = getScanColor(data.index, isDir);
    }

    return {
      id: `${data.path}_${data.name}_${data.index}`,
      name: data.name,
      path: data.path,
      size: data.size,
      ratio: data.ratio,
      oldSize: data.oldSize,
      deltaSize: data.deltaSize,
      deltaPercent: data.deltaPercent,
      status: data.status,
      isDir,
      isOtherGroup: data.isOtherGroup,
      otherCount: data.otherCount,
      fileCount: data.fileCount,
      dirCount: data.dirCount,
      hasChildren: data.hasChildren,
      x0: leaf.x0,
      y0: leaf.y0,
      x1: leaf.x1,
      y1: leaf.y1,
      color,
      originalNode: data.node,
    };
  });
});

function handleMouseEnter(item: TreemapItem, event: MouseEvent) {
  if (item.isOtherGroup) {
    tooltip.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      name: isZh.value ? `其他 ${item.otherCount} 个较小文件/目录` : `${item.otherCount} other smaller items`,
      path: isZh.value ? '超出展示限制的较小文件汇总' : 'Aggregated small files',
      isDir: true,
      size: item.size,
      oldSize: null,
      deltaSize: undefined,
      deltaPercent: undefined,
      status: undefined,
      fileCount: item.otherCount,
      dirCount: undefined,
    };
    return;
  }

  tooltip.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    name: item.name,
    path: item.path,
    isDir: item.isDir,
    size: item.size,
    oldSize: item.oldSize,
    deltaSize: item.deltaSize,
    deltaPercent: item.deltaPercent,
    status: item.status,
    fileCount: item.fileCount,
    dirCount: item.dirCount,
  };
}

function handleMouseMove(event: MouseEvent) {
  if (tooltip.value.visible) {
    tooltip.value.x = event.clientX;
    tooltip.value.y = event.clientY;
  }
}

function handleMouseLeave() {
  tooltip.value.visible = false;
}

function handleClick(item: TreemapItem) {
  if (item.isDir && !item.isOtherGroup && item.originalNode) {
    tooltip.value.visible = false;
    emit('drillDown', item.originalNode);
  }
}

function handleContextMenu(item: TreemapItem, event: MouseEvent) {
  event.preventDefault();
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    item,
  };
}

function closeContextMenu() {
  contextMenu.value.visible = false;
}

function openInFinder(item: TreemapItem | null) {
  if (!item || item.isOtherGroup) return;
  emit('revealInFinder', item.path);
  closeContextMenu();
}

function drillDownFromMenu(item: TreemapItem | null) {
  if (!item || item.isOtherGroup) return;
  handleClick(item);
  closeContextMenu();
}
</script>

<template>
  <div class="treemap-wrapper">
    <div
      ref="containerRef"
      class="treemap-container"
      @mousemove="handleMouseMove"
    >
      <div
        v-for="item in layoutItems"
        :key="item.id"
        class="treemap-cell"
        :class="{
          'is-clickable': item.isDir && !item.isOtherGroup,
          'is-other': item.isOtherGroup,
        }"
        :style="{
          left: `${item.x0}px`,
          top: `${item.y0}px`,
          width: `${Math.max(item.x1 - item.x0, 0)}px`,
          height: `${Math.max(item.y1 - item.y0, 0)}px`,
          backgroundColor: item.color.bg,
          borderColor: item.color.border,
        }"
        @mouseenter="handleMouseEnter(item, $event)"
        @mouseleave="handleMouseLeave"
        @click="handleClick(item)"
        @contextmenu="handleContextMenu(item, $event)"
      >
        <!-- Content visible if rectangle is large enough -->
        <div
          v-if="(item.x1 - item.x0 > 55) && (item.y1 - item.y0 > 45)"
          class="cell-content"
        >
          <div class="cell-title-row">
            <component
              :is="item.isOtherGroup ? Layers : (item.isDir ? Folder : FileText)"
              :size="13"
              class="cell-icon"
            />
            <span class="cell-name" :title="item.name">{{ item.name }}</span>
          </div>

          <div class="cell-metrics">
            <div class="cell-size-row">
              <span class="cell-size">{{ formatBytes(item.size) }}</span>
              <span class="cell-ratio">({{ item.ratio.toFixed(1) }}%)</span>
            </div>

            <!-- Diff Mode Badge -->
            <template v-if="isDiffMode && !item.isOtherGroup">
              <span
                v-if="item.deltaPercent !== undefined"
                class="cell-delta-badge"
                :style="{ backgroundColor: item.color.badgeBg || 'rgba(0,0,0,0.3)' }"
              >
                {{ formatPercent(item.deltaPercent) }}
              </span>
            </template>
          </div>
        </div>

        <!-- Micro minimal label for small boxes -->
        <div
          v-else-if="(item.x1 - item.x0 > 24) && (item.y1 - item.y0 > 18)"
          class="cell-mini-content"
        >
          <span class="mini-name">{{ item.isOtherGroup ? '...' : item.name }}</span>
        </div>
        <div
          v-else-if="item.isOtherGroup"
          class="cell-micro-content"
        >
          <span class="micro-dots">...</span>
        </div>
      </div>

      <!-- Empty state when no children -->
      <div v-if="layoutItems.length === 0" class="empty-state">
        <Folder :size="48" class="empty-icon" />
        <p class="empty-text">{{ t('treemap.empty') }}</p>
      </div>
    </div>

    <!-- Legend bar at bottom for Diff Mode -->
    <div v-if="isDiffMode" class="heatmap-legend glass-panel">
      <span class="legend-desc">
        {{ isZh ? '颜色深浅代表容量增减幅度：' : 'Color indicates capacity delta %:' }}
      </span>
      <div class="legend-scale">
        <span class="scale-step red-deep">{{ isZh ? '+100% 暴增' : '+100% Gain' }}</span>
        <span class="scale-step red-mid">{{ isZh ? '+20% 增加' : '+20% Up' }}</span>
        <span class="scale-step gray">{{ isZh ? '0% 未变' : '0% Same' }}</span>
        <span class="scale-step green-mid">{{ isZh ? '-20% 缩减' : '-20% Down' }}</span>
        <span class="scale-step green-deep">{{ isZh ? '-100% 骤减' : '-100% Gone' }}</span>
      </div>
    </div>

    <!-- Hover Detail Tooltip -->
    <TooltipCard
      :visible="tooltip.visible"
      :x="tooltip.x"
      :y="tooltip.y"
      :name="tooltip.name"
      :path="tooltip.path"
      :is-dir="tooltip.isDir"
      :size="tooltip.size"
      :old-size="tooltip.oldSize"
      :delta-size="tooltip.deltaSize"
      :delta-percent="tooltip.deltaPercent"
      :status="tooltip.status"
      :file-count="tooltip.fileCount"
      :dir-count="tooltip.dirCount"
      :is-diff-mode="isDiffMode"
    />

    <!-- Right Click Context Menu -->
    <div
      v-if="contextMenu.visible"
      class="context-menu glass-panel"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
    >
      <div
        v-if="contextMenu.item?.isDir && contextMenu.item?.originalNode"
        class="menu-item"
        @click="drillDownFromMenu(contextMenu.item)"
      >
        <ArrowRight :size="14" />
        <span>{{ isZh ? '进入该文件夹' : 'Drill down folder' }}</span>
      </div>
      <div class="menu-item" @click="openInFinder(contextMenu.item)">
        <ExternalLink :size="14" />
        <span>{{ t('list.reveal') }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.treemap-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-app);
}

.treemap-container {
  position: relative;
  flex: 1;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.treemap-cell {
  position: absolute;
  box-sizing: border-box;
  border-width: 1px;
  border-style: solid;
  border-radius: var(--radius-sm);
  padding: 6px 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  justify-content: center;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
  transition: transform 0.12s ease, box-shadow 0.12s ease, border-color 0.12s ease;
}

.treemap-cell:hover {
  z-index: 10;
  box-shadow: 0 0 0 1.5px rgba(255, 255, 255, 0.9), 0 4px 16px rgba(0, 0, 0, 0.6);
  border-color: #ffffff !important;
  transform: translateY(-1px);
}

.treemap-cell.is-clickable {
  cursor: pointer;
}

.treemap-cell.is-other {
  border-style: dashed;
  opacity: 0.85;
}

.cell-content {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  height: 100%;
  width: 100%;
  pointer-events: none;
}

.cell-title-row {
  display: flex;
  align-items: center;
  gap: 5px;
  width: 100%;
  overflow: hidden;
}

.cell-icon {
  flex-shrink: 0;
  opacity: 0.9;
}

.cell-name {
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6);
}

.cell-metrics {
  display: flex;
  flex-direction: column;
  gap: 3px;
  margin-top: 4px;
}

.cell-size-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.cell-size {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  color: #ffffff;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6);
}

.cell-ratio {
  font-family: var(--font-mono);
  font-size: 11px;
  color: rgba(255, 255, 255, 0.8);
}

.cell-delta-badge {
  display: inline-flex;
  align-items: center;
  align-self: flex-start;
  padding: 1px 5px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  color: #ffffff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
}

.cell-mini-content {
  pointer-events: none;
  font-size: 10px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
}

.cell-micro-content {
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.micro-dots {
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.7);
  line-height: 1;
}

.empty-state {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-muted);
}

.empty-icon {
  opacity: 0.4;
}

.empty-text {
  font-size: 14px;
}

/* Legend */
.heatmap-legend {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 6px 16px;
  border-top: 1px solid var(--border-subtle);
  font-size: 11.5px;
  background: var(--bg-sidebar);
  white-space: nowrap;
  flex-shrink: 0;
  min-height: 36px;
  box-sizing: border-box;
  overflow-x: auto;
}

.legend-desc {
  color: var(--text-secondary);
  white-space: nowrap;
  flex-shrink: 0;
}

.legend-scale {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  flex-shrink: 0;
}

.scale-step {
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.scale-step.red-deep {
  background: rgba(220, 38, 38, 0.9);
  color: #ffffff;
}

.scale-step.red-mid {
  background: rgba(239, 68, 68, 0.45);
  color: #fca5a5;
}

.scale-step.gray {
  background: rgba(51, 65, 85, 0.7);
  color: #cbd5e1;
}

.scale-step.green-mid {
  background: rgba(16, 185, 129, 0.45);
  color: #6ee7b7;
}

.scale-step.green-deep {
  background: rgba(16, 185, 129, 0.9);
  color: #ffffff;
}

/* Context Menu */
.context-menu {
  position: fixed;
  z-index: 10000;
  min-width: 190px;
  padding: 6px;
  background: var(--bg-tooltip);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.menu-item:hover {
  background: var(--accent-primary);
  color: #ffffff;
}
</style>
